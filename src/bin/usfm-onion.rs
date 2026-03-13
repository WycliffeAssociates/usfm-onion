#[cfg(not(target_arch = "wasm32"))]
mod native {
    use clap::{Parser, Subcommand, ValueEnum};
    use serde::Serialize;
    use std::fs;
    use std::io::{self, Read};
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;
    use usfm_onion::{
        DocumentFormat, ast,
        convert::{self, HtmlOptions},
        diff::{self, BuildSidBlocksOptions},
        format::{self, FormatOptions, FormatRule},
        lint::{self, LintIssue, TokenLintOptions},
        tokens,
    };

    #[derive(Parser)]
    #[command(name = "usfm-onion")]
    #[command(about = "Token-first USFM tooling")]
    struct Cli {
        #[command(subcommand)]
        command: Command,
    }

    #[derive(Subcommand)]
    enum Command {
        Parse(ParseCommand),
        Lint(LintCommand),
        Format(FormatCommand),
        Convert(ConvertCommand),
        Diff(DiffCommand),
        Inspect(InspectCommand),
    }

    #[derive(clap::Args)]
    struct ParseCommand {
        input: Option<PathBuf>,
        #[arg(long)]
        from: Option<FormatArg>,
        #[arg(long)]
        json: bool,
    }

    #[derive(clap::Args)]
    struct LintCommand {
        inputs: Vec<PathBuf>,
        #[arg(long)]
        from: Option<FormatArg>,
        #[arg(long)]
        json: bool,
    }

    #[derive(clap::Args)]
    struct FormatCommand {
        inputs: Vec<PathBuf>,
        #[arg(long)]
        from: Option<FormatArg>,
        #[arg(long, value_enum, value_delimiter = ',', conflicts_with = "exclude")]
        include: Vec<FormatRuleArg>,
        #[arg(long, value_enum, value_delimiter = ',', conflicts_with = "include")]
        exclude: Vec<FormatRuleArg>,
        #[arg(long)]
        in_place: bool,
        #[arg(long)]
        check: bool,
        #[arg(long)]
        json: bool,
    }

    #[derive(clap::Args)]
    struct ConvertCommand {
        input: Option<PathBuf>,
        #[arg(long)]
        from: Option<FormatArg>,
        #[arg(long)]
        to: OutputArg,
    }

    #[derive(clap::Args)]
    struct DiffCommand {
        baseline: PathBuf,
        current: PathBuf,
        #[arg(long)]
        format: Option<FormatArg>,
        #[arg(long)]
        by_chapter: bool,
        #[arg(long)]
        json: bool,
    }

    #[derive(clap::Args)]
    struct InspectCommand {
        input: Option<PathBuf>,
        #[arg(long)]
        from: Option<FormatArg>,
        #[arg(long)]
        json: bool,
    }

    #[derive(Debug, Clone, Copy, ValueEnum)]
    enum FormatArg {
        Usfm,
        Usj,
        Usx,
    }

    impl From<FormatArg> for DocumentFormat {
        fn from(value: FormatArg) -> Self {
            match value {
                FormatArg::Usfm => DocumentFormat::Usfm,
                FormatArg::Usj => DocumentFormat::Usj,
                FormatArg::Usx => DocumentFormat::Usx,
            }
        }
    }

    #[derive(Debug, Clone, Copy, ValueEnum)]
    enum OutputArg {
        Usfm,
        Usj,
        Usx,
        Html,
        Ast,
        Vref,
    }

    #[derive(Debug, Clone, Copy, ValueEnum)]
    enum FormatRuleArg {
        RecoverMalformedMarkers,
        CollapseWhitespaceInText,
        EnsureInlineSeparators,
        RemoveDuplicateVerseNumbers,
        NormalizeSpacingAfterParagraphMarkers,
        RemoveUnwantedLinebreaks,
        BridgeConsecutiveVerseMarkers,
        RemoveOrphanEmptyVerseBeforeContentfulVerse,
        RemoveBridgeVerseEnumerators,
        MoveChapterLabelAfterChapterMarker,
        InsertDefaultParagraphAfterChapterIntro,
        InsertStructuralLinebreaks,
        CollapseConsecutiveLinebreaks,
        NormalizeMarkerWhitespaceAtLineStart,
    }

    impl From<FormatRuleArg> for FormatRule {
        fn from(value: FormatRuleArg) -> Self {
            match value {
                FormatRuleArg::RecoverMalformedMarkers => FormatRule::RecoverMalformedMarkers,
                FormatRuleArg::CollapseWhitespaceInText => FormatRule::CollapseWhitespaceInText,
                FormatRuleArg::EnsureInlineSeparators => FormatRule::EnsureInlineSeparators,
                FormatRuleArg::RemoveDuplicateVerseNumbers => {
                    FormatRule::RemoveDuplicateVerseNumbers
                }
                FormatRuleArg::NormalizeSpacingAfterParagraphMarkers => {
                    FormatRule::NormalizeSpacingAfterParagraphMarkers
                }
                FormatRuleArg::RemoveUnwantedLinebreaks => FormatRule::RemoveUnwantedLinebreaks,
                FormatRuleArg::BridgeConsecutiveVerseMarkers => {
                    FormatRule::BridgeConsecutiveVerseMarkers
                }
                FormatRuleArg::RemoveOrphanEmptyVerseBeforeContentfulVerse => {
                    FormatRule::RemoveOrphanEmptyVerseBeforeContentfulVerse
                }
                FormatRuleArg::RemoveBridgeVerseEnumerators => {
                    FormatRule::RemoveBridgeVerseEnumerators
                }
                FormatRuleArg::MoveChapterLabelAfterChapterMarker => {
                    FormatRule::MoveChapterLabelAfterChapterMarker
                }
                FormatRuleArg::InsertDefaultParagraphAfterChapterIntro => {
                    FormatRule::InsertDefaultParagraphAfterChapterIntro
                }
                FormatRuleArg::InsertStructuralLinebreaks => FormatRule::InsertStructuralLinebreaks,
                FormatRuleArg::CollapseConsecutiveLinebreaks => {
                    FormatRule::CollapseConsecutiveLinebreaks
                }
                FormatRuleArg::NormalizeMarkerWhitespaceAtLineStart => {
                    FormatRule::NormalizeMarkerWhitespaceAtLineStart
                }
            }
        }
    }

    #[derive(Debug, Serialize)]
    struct ParseSummary {
        format: DocumentFormat,
        token_count: usize,
        ast_nodes: usize,
    }

    pub(super) fn main() -> ExitCode {
        match run() {
            Ok(code) => code,
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::FAILURE
            }
        }
    }

    fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
        let cli = Cli::parse();
        match cli.command {
            Command::Parse(command) => run_parse(command),
            Command::Lint(command) => run_lint(command),
            Command::Format(command) => run_format(command),
            Command::Convert(command) => run_convert(command),
            Command::Diff(command) => run_diff(command),
            Command::Inspect(command) => run_inspect(command),
        }
    }

    fn run_parse(command: ParseCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let format =
            resolve_optional_format(command.from, command.input.as_deref(), DocumentFormat::Usfm)?;
        let source = read_named_or_stdin(command.input.as_deref())?;
        let token_list = tokens_from_source(&source, format)?;
        let tree = ast_from_source(&source, format)?;
        let summary = ParseSummary {
            format,
            token_count: token_list.len(),
            ast_nodes: tree.content.len(),
        };
        if command.json {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else {
            println!("format: {}", summary.format);
            println!("tokens: {}", summary.token_count);
            println!("ast nodes: {}", summary.ast_nodes);
        }
        Ok(ExitCode::SUCCESS)
    }

    fn run_lint(command: LintCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let format = resolve_optional_format(
            command.from,
            command.inputs.first().map(PathBuf::as_path),
            DocumentFormat::Usfm,
        )?;
        if command.inputs.is_empty() {
            let source = read_named_or_stdin(None)?;
            let issues = lint::lint_content(&source, format, TokenLintOptions::default().into())?;
            return print_lint_result(None, &issues, command.json);
        }

        let mut all_ok = true;
        for path in &command.inputs {
            let issues = lint::lint_path(path, format, TokenLintOptions::default().into())?;
            if !issues.is_empty() {
                all_ok = false;
            }
            print_lint_result(Some(path), &issues, command.json)?;
        }
        Ok(if all_ok {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        })
    }

    fn run_format(command: FormatCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let format = resolve_optional_format(
            command.from,
            command.inputs.first().map(PathBuf::as_path),
            DocumentFormat::Usfm,
        )?;
        let options = if !command.include.is_empty() {
            let rules = command
                .include
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();
            FormatOptions::only(&rules)
        } else if !command.exclude.is_empty() {
            let rules = command
                .exclude
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>();
            FormatOptions::excluding(&rules)
        } else {
            FormatOptions::default()
        };

        if command.inputs.is_empty() {
            let source = read_named_or_stdin(None)?;
            let result =
                format::format_content_with_options(&source, format, Default::default(), options)?;
            let output = tokens::tokens_to_usfm(&result.tokens);
            println!("{output}");
            return Ok(ExitCode::SUCCESS);
        }

        let mut any_changed = false;
        for path in &command.inputs {
            let source = fs::read_to_string(path)?;
            let token_list = tokens_from_source(&source, format)?;
            let result = format::format_tokens_result(&token_list, options.clone());
            let output = tokens::tokens_to_usfm(&result.tokens);
            let changed = output != source;
            any_changed |= changed;

            if command.in_place && changed {
                fs::write(path, &output)?;
            } else if command.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "path": path.display().to_string(),
                        "changed": changed,
                        "applied_changes": result.applied_changes.len(),
                        "skipped_changes": result.skipped_changes.len(),
                    }))?
                );
            } else if !command.check {
                println!("{output}");
            }
        }

        if command.check && any_changed {
            return Ok(ExitCode::FAILURE);
        }
        Ok(ExitCode::SUCCESS)
    }

    fn run_convert(command: ConvertCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let format =
            resolve_optional_format(command.from, command.input.as_deref(), DocumentFormat::Usfm)?;
        let source = read_named_or_stdin(command.input.as_deref())?;
        let output = match command.to {
            OutputArg::Usfm => normalize_to_usfm(&source, format)?,
            OutputArg::Usj => serde_json::to_string_pretty(&convert::usfm_to_usj(
                &normalize_to_usfm(&source, format)?,
            )?)?,
            OutputArg::Usx => convert::usfm_to_usx(&normalize_to_usfm(&source, format)?)?,
            OutputArg::Html => {
                convert::usfm_to_html(&normalize_to_usfm(&source, format)?, HtmlOptions::default())?
            }
            OutputArg::Ast => serde_json::to_string_pretty(&ast_from_source(&source, format)?)?,
            OutputArg::Vref => serde_json::to_string_pretty(&convert::usfm_to_vref(
                &normalize_to_usfm(&source, format)?,
            )?)?,
        };
        println!("{output}");
        Ok(ExitCode::SUCCESS)
    }

    fn run_diff(command: DiffCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let format = resolve_optional_format(
            command.format,
            Some(command.baseline.as_path()),
            DocumentFormat::Usfm,
        )?;
        let baseline = normalize_to_usfm(&fs::read_to_string(&command.baseline)?, format)?;
        let current = normalize_to_usfm(&fs::read_to_string(&command.current)?, format)?;
        if command.by_chapter {
            let diffs = diff::diff_usfm_by_chapter(
                &baseline,
                &current,
                &tokens::TokenViewOptions::default(),
                &BuildSidBlocksOptions::default(),
            );
            if command.json {
                println!("{}", serde_json::to_string_pretty(&diffs)?);
            } else {
                println!("{:#?}", diffs);
            }
        } else {
            let diffs = diff::diff_content(
                &baseline,
                DocumentFormat::Usfm,
                &current,
                DocumentFormat::Usfm,
                &tokens::TokenViewOptions::default(),
                &BuildSidBlocksOptions::default(),
            )?;
            if command.json {
                println!("{}", serde_json::to_string_pretty(&diffs)?);
            } else {
                println!("{:#?}", diffs);
            }
        }
        Ok(ExitCode::SUCCESS)
    }

    fn run_inspect(command: InspectCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
        let format =
            resolve_optional_format(command.from, command.input.as_deref(), DocumentFormat::Usfm)?;
        let source = read_named_or_stdin(command.input.as_deref())?;
        let token_list = tokens_from_source(&source, format)?;
        let tree = ast_from_source(&source, format)?;
        let issues = lint::lint_content(&source, format, TokenLintOptions::default().into())?;
        let payload = serde_json::json!({
            "tokens": token_list,
            "ast": tree,
            "lint_issues": issues,
        });
        if command.json {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        } else {
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
        Ok(ExitCode::SUCCESS)
    }

    fn normalize_to_usfm(
        source: &str,
        format: DocumentFormat,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(match format {
            DocumentFormat::Usfm => source.to_string(),
            DocumentFormat::Usj => convert::from_usj_str(source)?,
            DocumentFormat::Usx => convert::from_usx_str(source)?,
        })
    }

    fn tokens_from_source(
        source: &str,
        format: DocumentFormat,
    ) -> Result<Vec<tokens::Token>, Box<dyn std::error::Error>> {
        Ok(match format {
            DocumentFormat::Usfm => tokens::usfm_to_tokens(source),
            DocumentFormat::Usj => tokens::usj_to_tokens(source)?,
            DocumentFormat::Usx => tokens::usx_to_tokens(source)?,
        })
    }

    fn ast_from_source(
        source: &str,
        format: DocumentFormat,
    ) -> Result<ast::AstDocument, Box<dyn std::error::Error>> {
        Ok(match format {
            DocumentFormat::Usfm => ast::usfm_to_ast(source),
            DocumentFormat::Usj => ast::usj_to_ast(source)?,
            DocumentFormat::Usx => ast::usx_to_ast(source)?,
        })
    }

    fn print_lint_result(
        path: Option<&PathBuf>,
        issues: &[LintIssue],
        json: bool,
    ) -> Result<ExitCode, Box<dyn std::error::Error>> {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "path": path.map(|p| p.display().to_string()),
                    "issues": issues,
                }))?
            );
        } else if let Some(path) = path {
            println!("{}: {} issues", path.display(), issues.len());
        } else {
            println!("issues: {}", issues.len());
        }
        Ok(if issues.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        })
    }

    fn resolve_optional_format(
        explicit: Option<FormatArg>,
        path: Option<&Path>,
        default: DocumentFormat,
    ) -> Result<DocumentFormat, Box<dyn std::error::Error>> {
        if let Some(explicit) = explicit {
            return Ok(explicit.into());
        }
        if let Some(path) = path
            && let Some(format) = DocumentFormat::from_path(path)
        {
            return Ok(format);
        }
        Ok(default)
    }

    fn read_named_or_stdin(path: Option<&Path>) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(path) = path {
            return Ok(fs::read_to_string(path)?);
        }
        let mut source = String::new();
        io::stdin().read_to_string(&mut source)?;
        Ok(source)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::process::ExitCode {
    native::main()
}
