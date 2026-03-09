use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use usfm_onion::{
    BuildSidBlocksOptions, DiffStatus, DocumentFormat,
    convert::{HtmlOptions, convert_content, into_editor_tree, into_html, into_vref},
    diff::{diff_content, diff_usfm_by_chapter},
    format::{FormatOptions, FormatRule},
    lint::{LintIssue, LintOptions, lint_content, lint_path, lint_paths},
    model::{BatchExecutionOptions, TokenViewOptions},
    parse::{
        DebugDumpOptions, IntoTokensOptions, ParseRecovery, debug_dump, into_tokens,
        into_usfm_from_tokens, parse_content, recoveries,
    },
};

#[derive(Parser)]
#[command(name = "usfm-onion")]
#[command(about = "Parse, lint, format, convert, diff, and inspect USFM content")]
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
    merge_whitespace: bool,
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
    baseline_format: Option<FormatArg>,
    #[arg(long)]
    current_format: Option<FormatArg>,
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
    raw: bool,
    #[arg(long)]
    projected: bool,
    #[arg(long)]
    recoveries: bool,
    #[arg(long)]
    lint: bool,
    #[arg(long)]
    document: bool,
    #[arg(long, default_value_t = 80)]
    limit: usize,
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
    EditorTree,
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
            FormatRuleArg::RemoveDuplicateVerseNumbers => FormatRule::RemoveDuplicateVerseNumbers,
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
            FormatRuleArg::RemoveBridgeVerseEnumerators => FormatRule::RemoveBridgeVerseEnumerators,
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
    book_code: Option<String>,
    source_bytes: usize,
    token_count: usize,
    recovery_count: usize,
    recoveries: Vec<ParseRecovery>,
    vref_entries: usize,
}

#[derive(Debug, Serialize)]
struct FormatResultSummary {
    path: Option<String>,
    changed: bool,
    applied_changes: usize,
    skipped_changes: usize,
    output: Option<String>,
}

fn main() -> ExitCode {
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
    let handle = parse_content(&source, format)?;
    let tokens = into_tokens(&handle, IntoTokensOptions::default());
    let recoveries = recoveries(&handle).to_vec();
    let summary = ParseSummary {
        format,
        book_code: handle.book_code().map(str::to_string),
        source_bytes: source.len(),
        token_count: tokens.len(),
        recovery_count: recoveries.len(),
        recoveries,
        vref_entries: into_vref(&handle).len(),
    };

    if command.json {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        println!("format: {}", summary.format);
        println!(
            "book code: {}",
            summary.book_code.as_deref().unwrap_or("unknown")
        );
        println!("source bytes: {}", summary.source_bytes);
        println!("tokens: {}", summary.token_count);
        println!("recoveries: {}", summary.recovery_count);
        println!("vref entries: {}", summary.vref_entries);
    }

    Ok(ExitCode::SUCCESS)
}

fn run_lint(command: LintCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    if command.inputs.is_empty() {
        let format = command.from.map(Into::into).unwrap_or(DocumentFormat::Usfm);
        let source = read_stdin()?;
        let lint_issues = lint_content(&source, format, LintOptions::default())?;
        if command.json {
            println!("{}", serde_json::to_string_pretty(&lint_issues)?);
        } else {
            print_lint_report(None, &lint_issues);
        }
        return Ok(exit_for_issues(lint_issues.is_empty()));
    }

    if let Some(format) = command.from.map(Into::into) {
        let results = lint_paths(
            &command.inputs,
            format,
            LintOptions::default(),
            BatchExecutionOptions::parallel(),
        );
        let mut any_issues = false;
        let mut json_rows = Vec::new();

        for (path, result) in command.inputs.iter().zip(results) {
            let issues = result?;
            any_issues |= !issues.is_empty();
            if command.json {
                json_rows.push(serde_json::json!({
                    "path": path.display().to_string(),
                    "issues": issues,
                }));
            } else {
                print_lint_report(Some(path), &issues);
            }
        }

        if command.json {
            println!("{}", serde_json::to_string_pretty(&json_rows)?);
        }

        return Ok(exit_for_issues(!any_issues));
    }

    let mut any_issues = false;
    let mut json_rows = Vec::new();
    for path in &command.inputs {
        let format = resolve_optional_format(None, Some(path.as_path()), DocumentFormat::Usfm)?;
        let issues = lint_path(path, format, LintOptions::default())?;
        any_issues |= !issues.is_empty();
        if command.json {
            json_rows.push(serde_json::json!({
                "path": path.display().to_string(),
                "format": format,
                "issues": issues,
            }));
        } else {
            print_lint_report(Some(path), &issues);
        }
    }

    if command.json {
        println!("{}", serde_json::to_string_pretty(&json_rows)?);
    }

    Ok(exit_for_issues(!any_issues))
}

fn run_format(command: FormatCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let format_options = resolve_format_options(&command);

    if command.inputs.is_empty() {
        if command.in_place {
            return Err("`format --in-place` requires at least one path".into());
        }
        let format = command.from.map(Into::into).unwrap_or(DocumentFormat::Usfm);
        let source = read_stdin()?;
        let result = usfm_onion::format::format_content_with_options(
            &source,
            format,
            IntoTokensOptions::default().with_merge_horizontal_whitespace(command.merge_whitespace),
            format_options,
        )?;
        let output = into_usfm_from_tokens(&result.tokens);
        let changed = output != source;
        if command.json {
            let summary = FormatResultSummary {
                path: None,
                changed,
                applied_changes: result.applied_changes.len(),
                skipped_changes: result.skipped_changes.len(),
                output: if command.check { None } else { Some(output) },
            };
            println!("{}", serde_json::to_string_pretty(&summary)?);
        } else if !command.check {
            print!("{output}");
        }
        return Ok(if command.check && changed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        });
    }

    if command.inputs.len() > 1 && !command.in_place && !command.check && !command.json {
        return Err(
            "formatting multiple paths to stdout is ambiguous; use `--in-place`, `--check`, or `--json`"
                .into(),
        );
    }

    let mut changed_any = false;
    let mut summaries = Vec::new();

    for path in &command.inputs {
        let format =
            resolve_optional_format(command.from, Some(path.as_path()), DocumentFormat::Usfm)?;
        let source = std::fs::read_to_string(path)?;
        let result = usfm_onion::format::format_path_with_options(
            path,
            format,
            IntoTokensOptions::default().with_merge_horizontal_whitespace(command.merge_whitespace),
            format_options,
        )?;
        let output = into_usfm_from_tokens(&result.tokens);
        let changed = output != source;
        changed_any |= changed;

        if command.in_place && changed {
            std::fs::write(path, &output)?;
        }

        if !command.in_place && !command.check && !command.json {
            print!("{output}");
        }

        summaries.push(FormatResultSummary {
            path: Some(path.display().to_string()),
            changed,
            applied_changes: result.applied_changes.len(),
            skipped_changes: result.skipped_changes.len(),
            output: if command.in_place || command.check {
                None
            } else {
                Some(output)
            },
        });
    }

    if command.json {
        println!("{}", serde_json::to_string_pretty(&summaries)?);
    } else if command.check || command.in_place {
        for summary in &summaries {
            let path = summary.path.as_deref().unwrap_or("<stdin>");
            if command.in_place {
                println!(
                    "{path}: {}",
                    if summary.changed {
                        "updated"
                    } else {
                        "unchanged"
                    }
                );
            } else {
                println!(
                    "{path}: {}",
                    if summary.changed {
                        "needs formatting"
                    } else {
                        "ok"
                    }
                );
            }
        }
    }

    Ok(if command.check && changed_any {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn run_convert(command: ConvertCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let format =
        resolve_optional_format(command.from, command.input.as_deref(), DocumentFormat::Usfm)?;
    let source = read_named_or_stdin(command.input.as_deref())?;

    let output = match command.to {
        OutputArg::Usfm => convert_content(&source, format, DocumentFormat::Usfm)?,
        OutputArg::Usj => {
            pretty_json_string(&convert_content(&source, format, DocumentFormat::Usj)?)?
        }
        OutputArg::Usx => convert_content(&source, format, DocumentFormat::Usx)?,
        OutputArg::Html => {
            let handle = parse_content(&source, format)?;
            into_html(&handle, HtmlOptions::default())
        }
        OutputArg::EditorTree => {
            let handle = parse_content(&source, format)?;
            serde_json::to_string_pretty(&into_editor_tree(&handle))?
        }
        OutputArg::Vref => {
            let handle = parse_content(&source, format)?;
            serde_json::to_string_pretty(&into_vref(&handle))?
        }
    };

    print!("{output}");
    Ok(ExitCode::SUCCESS)
}

fn run_diff(command: DiffCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let baseline_format = command
        .baseline_format
        .or(command.format)
        .map(Into::into)
        .or_else(|| DocumentFormat::from_path(&command.baseline))
        .unwrap_or(DocumentFormat::Usfm);
    let current_format = command
        .current_format
        .or(command.format)
        .map(Into::into)
        .or_else(|| DocumentFormat::from_path(&command.current))
        .unwrap_or(DocumentFormat::Usfm);

    let baseline = std::fs::read_to_string(&command.baseline)?;
    let current = std::fs::read_to_string(&command.current)?;

    if command.by_chapter {
        let chapter_map = diff_usfm_by_chapter(
            &baseline,
            &current,
            &TokenViewOptions::default(),
            &BuildSidBlocksOptions::default(),
        );
        if command.json {
            println!("{}", serde_json::to_string_pretty(&chapter_map)?);
        } else if chapter_map.is_empty() {
            println!("no chapter differences");
        } else {
            for (book, chapters) in chapter_map {
                for (chapter, diffs) in chapters {
                    println!("{book} {chapter}: {} blocks changed", diffs.len());
                }
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    let diffs = diff_content(
        &baseline,
        baseline_format,
        &current,
        current_format,
        &TokenViewOptions::default(),
        &BuildSidBlocksOptions::default(),
    )?;
    if command.json {
        println!("{}", serde_json::to_string_pretty(&diffs)?);
    } else if diffs.is_empty() {
        println!("no differences");
    } else {
        for diff in &diffs {
            println!(
                "{} {} {}",
                format_diff_status(diff.status),
                diff.semantic_sid,
                diff.block_id
            );
        }
        println!("total blocks changed: {}", diffs.len());
    }

    Ok(ExitCode::SUCCESS)
}

fn run_inspect(command: InspectCommand) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let format =
        resolve_optional_format(command.from, command.input.as_deref(), DocumentFormat::Usfm)?;
    let source = read_named_or_stdin(command.input.as_deref())?;

    let focused =
        command.raw || command.projected || command.recoveries || command.lint || command.document;
    let options = if focused {
        DebugDumpOptions {
            include_raw: command.raw,
            include_projected: command.projected,
            include_recoveries: command.recoveries,
            include_lint: command.lint,
            include_document: command.document,
            limit: command.limit,
        }
    } else {
        DebugDumpOptions {
            limit: command.limit,
            ..DebugDumpOptions::default()
        }
    };

    let handle = parse_content(&source, format)?;
    let output = debug_dump(&handle, options);
    print!("{output}");
    Ok(ExitCode::SUCCESS)
}

fn resolve_optional_format(
    requested: Option<FormatArg>,
    path: Option<&Path>,
    fallback: DocumentFormat,
) -> Result<DocumentFormat, Box<dyn std::error::Error>> {
    if let Some(requested) = requested {
        return Ok(requested.into());
    }
    if let Some(path) = path
        && let Some(inferred) = DocumentFormat::from_path(path)
    {
        return Ok(inferred);
    }
    Ok(fallback)
}

fn read_named_or_stdin(path: Option<&Path>) -> Result<String, Box<dyn std::error::Error>> {
    match path {
        Some(path) => Ok(std::fs::read_to_string(path)?),
        None => read_stdin(),
    }
}

fn read_stdin() -> Result<String, Box<dyn std::error::Error>> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source)?;
    Ok(source)
}

fn print_lint_report(path: Option<&Path>, issues: &[LintIssue]) {
    if let Some(path) = path {
        println!("{}", path.display());
    }
    if issues.is_empty() {
        println!("  ok");
        return;
    }
    for issue in issues {
        println!(
            "  {} {} {:?}: {}",
            issue.severity.as_str(),
            issue.code.as_str(),
            issue.span,
            issue.message
        );
    }
}

fn exit_for_issues(clean: bool) -> ExitCode {
    if clean {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn pretty_json_string(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let value: serde_json::Value = serde_json::from_str(input)?;
    Ok(serde_json::to_string_pretty(&value)?)
}

fn resolve_format_options(command: &FormatCommand) -> FormatOptions {
    if !command.include.is_empty() {
        let rules = command
            .include
            .iter()
            .copied()
            .map(FormatRule::from)
            .collect::<Vec<_>>();
        return FormatOptions::only(&rules);
    }
    if !command.exclude.is_empty() {
        let rules = command
            .exclude
            .iter()
            .copied()
            .map(FormatRule::from)
            .collect::<Vec<_>>();
        return FormatOptions::excluding(&rules);
    }
    FormatOptions::default()
}

fn format_diff_status(status: DiffStatus) -> &'static str {
    match status {
        DiffStatus::Added => "added",
        DiffStatus::Deleted => "deleted",
        DiffStatus::Modified => "modified",
        DiffStatus::Unchanged => "unchanged",
    }
}
