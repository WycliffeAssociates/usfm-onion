use crate::marker_defs::{
    BlockBehavior, NoteFamily, NoteSubkind, SpecMarkerKind, StructuralScopeKind,
    lookup_marker_def, marker_block_behavior, marker_is_note_container, marker_note_family,
    marker_note_subkind,
};
use crate::parse::parse;
use crate::token::{AttributeItem, Token, TokenData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlNoteMode {
    Extracted,
    Inline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlCallerStyle {
    Numeric,
    AlphaLower,
    AlphaUpper,
    RomanLower,
    RomanUpper,
    Source,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlCallerScope {
    DocumentSequential,
    VerseSequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HtmlOptions {
    pub wrap_root: bool,
    pub prefer_native_elements: bool,
    pub note_mode: HtmlNoteMode,
    pub caller_style: HtmlCallerStyle,
    pub caller_scope: HtmlCallerScope,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            wrap_root: false,
            prefer_native_elements: true,
            note_mode: HtmlNoteMode::Extracted,
            caller_style: HtmlCallerStyle::Numeric,
            caller_scope: HtmlCallerScope::VerseSequential,
        }
    }
}

pub fn tokens_to_html(tokens: &[Token<'_>], options: HtmlOptions) -> String {
    let mut renderer = HtmlRenderer::new(options);
    let body = renderer.render_root(tokens);

    if options.wrap_root {
        format!(r#"<div data-usfm-root="true">{body}</div>"#)
    } else {
        body
    }
}

pub fn usfm_to_html(source: &str, options: HtmlOptions) -> String {
    let parsed = parse(source);
    tokens_to_html(&parsed.tokens, options)
}

struct HtmlRenderer {
    options: HtmlOptions,
    current_verse: Option<String>,
    note_count_in_verse: usize,
    document_note_count: usize,
    footnote_id: usize,
    crossref_id: usize,
    footnotes: Vec<String>,
    crossrefs: Vec<String>,
}

#[derive(Clone)]
struct OpenElement<'a> {
    marker: Option<&'a str>,
    tag: &'static str,
    attrs: Vec<(String, String)>,
    buffer: String,
    scope_kind: StructuralScopeKind,
    note_subkind: Option<NoteSubkind>,
    note_family: Option<NoteFamily>,
    synthetic: bool,
}

impl HtmlRenderer {
    fn new(options: HtmlOptions) -> Self {
        Self {
            options,
            current_verse: None,
            note_count_in_verse: 0,
            document_note_count: 0,
            footnote_id: 0,
            crossref_id: 0,
            footnotes: Vec::new(),
            crossrefs: Vec::new(),
        }
    }

    fn render_root(&mut self, tokens: &[Token<'_>]) -> String {
        let mut out = self.render_fragment(tokens, false);
        if !self.footnotes.is_empty() {
            out.push_str(r#"<section id="linkedFootnotes" data-usfm-notes="footnotes">"#);
            for note in &self.footnotes {
                out.push_str(note);
            }
            out.push_str("</section>");
        }
        if !self.crossrefs.is_empty() {
            out.push_str(r#"<section id="linkedCrossrefs" data-usfm-notes="crossrefs">"#);
            for note in &self.crossrefs {
                out.push_str(note);
            }
            out.push_str("</section>");
        }
        out
    }

    fn render_fragment(&mut self, tokens: &[Token<'_>], in_note_body: bool) -> String {
        let mut output = String::new();
        let mut stack = Vec::<OpenElement<'_>>::new();
        let mut index = 0usize;

        while index < tokens.len() {
            match &tokens[index].data {
                TokenData::Text => {
                    push_fragment(&mut output, &mut stack, &escape_html(tokens[index].source));
                }
                TokenData::Newline => {}
                TokenData::OptBreak => {
                    push_fragment(&mut output, &mut stack, "<wbr>");
                }
                TokenData::AttributeList { entries } => {
                    if let Some(top) = stack.last_mut() {
                        push_attribute_entries(&mut top.attrs, entries);
                    }
                }
                TokenData::BookCode { code, .. } => {
                    if !stack.iter().any(|item| {
                        item.scope_kind == StructuralScopeKind::Header
                            && item
                                .attrs
                                .iter()
                                .any(|(key, value)| key == "data-usfm-type" && value == "book")
                    }) {
                        close_for_new_block(&mut output, &mut stack, false);
                        stack.push(open_book_element(
                            "id",
                            code,
                            self.options.prefer_native_elements,
                        ));
                    }
                }
                TokenData::Number { .. } => {}
                TokenData::MilestoneEnd => {
                    close_top_matching_scope(
                        &mut output,
                        &mut stack,
                        |item| item.scope_kind == StructuralScopeKind::Milestone,
                    );
                }
                TokenData::EndMarker { name, .. } => {
                    close_for_end_marker(&mut output, &mut stack, name);
                }
                TokenData::Milestone {
                    name,
                    metadata,
                    structural,
                } => {
                    if in_note_body {
                        close_for_note_structural(&mut output, &mut stack, name);
                    }
                    let mut attrs = common_marker_attrs(marker_data_type(name, metadata.kind), name);
                    if let Some(entries) = next_attribute_list(tokens, index) {
                        push_attribute_entries(&mut attrs, entries);
                    }
                    stack.push(OpenElement {
                        marker: Some(name),
                        tag: "span",
                        attrs,
                        buffer: String::new(),
                        scope_kind: structural.scope_kind,
                        note_subkind: marker_note_subkind(name),
                        note_family: marker_note_family(name),
                        synthetic: false,
                    });
                }
                TokenData::Marker {
                    name,
                    metadata,
                    structural,
                    ..
                } => {
                    if marker_is_sidebar_end(name, metadata.kind) {
                        close_top_matching_scope(
                            &mut output,
                            &mut stack,
                            |item| item.scope_kind == StructuralScopeKind::Sidebar,
                        );
                        index += 1;
                        continue;
                    }

                    if marker_is_note_container(name) {
                        let (caller, content_start, note_end) = parse_note_tokens(tokens, index);
                        let label = self.note_label(caller.as_deref().unwrap_or_default());
                        let note_kind = note_kind(name);

                        if in_note_body || matches!(self.options.note_mode, HtmlNoteMode::Inline) {
                            let mut attrs = common_marker_attrs("note", name);
                            attrs.push(("data-usfm-caller".to_string(), label.clone()));
                            attrs.push((
                                "data-usfm-source-caller".to_string(),
                                caller.clone().unwrap_or_default(),
                            ));
                            attrs.push((
                                "data-usfm-note-kind".to_string(),
                                note_kind.as_str().to_string(),
                            ));
                            let body = self.render_fragment(&tokens[content_start..note_end], true);
                            let mut html = String::from("<span");
                            push_attrs(&mut html, &attrs);
                            html.push('>');
                            html.push_str("<sup>");
                            html.push_str(&escape_html(&label));
                            html.push_str("</sup>");
                            html.push_str(&body);
                            html.push_str("</span>");
                            push_fragment(&mut output, &mut stack, &html);
                        } else {
                            let id_index = match note_kind {
                                NoteKind::Footnote => {
                                    self.footnote_id += 1;
                                    self.footnote_id
                                }
                                NoteKind::Crossref => {
                                    self.crossref_id += 1;
                                    self.crossref_id
                                }
                            };
                            let (call_id, note_id) = match note_kind {
                                NoteKind::Footnote => {
                                    (format!("fnref-{id_index}"), format!("fn-{id_index}"))
                                }
                                NoteKind::Crossref => {
                                    (format!("xrref-{id_index}"), format!("xr-{id_index}"))
                                }
                            };

                            let mut caller_html = String::from("<sup><a");
                            push_attr(&mut caller_html, "href", &format!("#{note_id}"));
                            push_attr(&mut caller_html, "id", &call_id);
                            push_attr(
                                &mut caller_html,
                                "data-usfm-note-kind",
                                note_kind.as_str(),
                            );
                            push_attr(&mut caller_html, "data-usfm-caller", &label);
                            push_attr(
                                &mut caller_html,
                                "data-usfm-source-caller",
                                caller.as_deref().unwrap_or_default(),
                            );
                            caller_html.push('>');
                            caller_html.push_str(&escape_html(&label));
                            caller_html.push_str("</a></sup>");
                            push_fragment(&mut output, &mut stack, &caller_html);

                            let body = self.render_fragment(&tokens[content_start..note_end], true);
                            let note_html = render_extracted_note(
                                name,
                                note_kind,
                                caller.as_deref().unwrap_or_default(),
                                &label,
                                &call_id,
                                &note_id,
                                &body,
                            );

                            match note_kind {
                                NoteKind::Footnote => self.footnotes.push(note_html),
                                NoteKind::Crossref => self.crossrefs.push(note_html),
                            }
                        }

                        index = note_end;
                    } else if matches!(metadata.kind, Some(SpecMarkerKind::Chapter)) {
                        close_for_new_block(&mut output, &mut stack, true);
                        self.current_verse = None;
                        self.note_count_in_verse = 0;
                        let number = next_number_text(tokens, index).unwrap_or_default();
                        push_fragment(
                            &mut output,
                            &mut stack,
                            &empty_marker_span("chapter", name, &number),
                        );
                    } else if matches!(metadata.kind, Some(SpecMarkerKind::Verse)) {
                        let number = next_number_text(tokens, index).unwrap_or_default();
                        self.current_verse = (!number.is_empty()).then_some(number.clone());
                        self.note_count_in_verse = 0;
                        push_fragment(
                            &mut output,
                            &mut stack,
                            &empty_marker_span("verse", name, &number),
                        );
                    } else {
                        open_marker_element(
                            &mut output,
                            &mut stack,
                            tokens,
                            index,
                            name,
                            metadata.kind,
                            *structural,
                            in_note_body,
                            self.options.prefer_native_elements,
                        );
                    }
                }
            }

            index += 1;
        }

        while let Some(item) = stack.pop() {
            append_closed_element(&mut output, &mut stack, item);
        }

        output
    }

    fn note_label(&mut self, source_caller: &str) -> String {
        if matches!(self.options.caller_style, HtmlCallerStyle::Source) && !source_caller.is_empty()
        {
            return source_caller.to_string();
        }

        match self.options.caller_scope {
            HtmlCallerScope::DocumentSequential => {
                self.document_note_count += 1;
                format_ordinal(self.document_note_count, self.options.caller_style)
            }
            HtmlCallerScope::VerseSequential => {
                if let Some(verse) = self.current_verse.as_deref() {
                    self.note_count_in_verse += 1;
                    format!(
                        "{}.{}",
                        verse,
                        format_ordinal(self.note_count_in_verse, self.options.caller_style)
                    )
                } else {
                    self.document_note_count += 1;
                    format_ordinal(self.document_note_count, self.options.caller_style)
                }
            }
        }
    }
}

fn open_marker_element<'a>(
    output: &mut String,
    stack: &mut Vec<OpenElement<'a>>,
    tokens: &'a [Token<'a>],
    index: usize,
    name: &'a str,
    kind: Option<SpecMarkerKind>,
    structural: crate::marker_defs::StructuralMarkerInfo,
    in_note_body: bool,
    prefer_native_elements: bool,
) {
    match structural.scope_kind {
        StructuralScopeKind::Header
        | StructuralScopeKind::Block
        | StructuralScopeKind::Sidebar
        | StructuralScopeKind::Periph
        | StructuralScopeKind::Meta => {
            close_for_new_block(output, stack, false);
        }
        StructuralScopeKind::TableRow => {
            close_inline_scopes(output, stack);
            close_top_matching_scope(output, stack, |item| {
                item.scope_kind == StructuralScopeKind::TableCell
            });
            close_top_matching_scope(output, stack, |item| {
                item.scope_kind == StructuralScopeKind::TableRow
            });
            close_non_book_table_block_scopes(output, stack);
            ensure_table_open(stack, prefer_native_elements);
        }
        StructuralScopeKind::TableCell => {
            close_inline_scopes(output, stack);
            close_top_matching_scope(output, stack, |item| {
                item.scope_kind == StructuralScopeKind::TableCell
            });
            if !stack
                .iter()
                .any(|item| item.scope_kind == StructuralScopeKind::TableRow)
            {
                ensure_table_open(stack, prefer_native_elements);
                stack.push(synthetic_table_row());
            }
        }
        StructuralScopeKind::Character | StructuralScopeKind::Milestone => {
            if in_note_body {
                close_for_note_structural(output, stack, name);
            }
        }
        StructuralScopeKind::Unknown | StructuralScopeKind::Chapter | StructuralScopeKind::Verse | StructuralScopeKind::Note => {}
    }

    let (tag, data_type) = tag_and_type_for_marker(name, kind, structural.scope_kind, prefer_native_elements);
    let mut attrs = common_marker_attrs(data_type, name);
    if structural.scope_kind == StructuralScopeKind::TableCell {
        attrs.push(("data-usfm-align".to_string(), table_cell_align(name).to_string()));
    }
    if let Some(entries) = next_attribute_list(tokens, index) {
        push_attribute_entries(&mut attrs, entries);
    }

    stack.push(OpenElement {
        marker: Some(name),
        tag,
        attrs,
        buffer: String::new(),
        scope_kind: structural.scope_kind,
        note_subkind: marker_note_subkind(name),
        note_family: marker_note_family(name),
        synthetic: false,
    });
}

fn parse_note_tokens(tokens: &[Token<'_>], start: usize) -> (Option<String>, usize, usize) {
    let mut content_start = start + 1;
    let mut caller = None;

    if let Some(token) = tokens.get(content_start)
        && matches!(token.data, TokenData::Text)
    {
        let trimmed = token.source.trim();
        if !trimmed.is_empty() {
            caller = Some(trimmed.to_string());
            content_start += 1;
        }
    }

    let mut depth = 0usize;
    let mut index = start;
    while index < tokens.len() {
        match &tokens[index].data {
            TokenData::Marker { name, .. } if marker_is_note_container(name) => {
                depth += 1;
            }
            TokenData::EndMarker { name, .. } if marker_is_note_container(name) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return (caller, content_start, index);
                }
            }
            _ => {}
        }
        index += 1;
    }

    (caller, content_start, tokens.len())
}

fn next_number_text(tokens: &[Token<'_>], marker_index: usize) -> Option<String> {
    match tokens.get(marker_index + 1).map(|token| &token.data) {
        Some(TokenData::Number { start, end, .. }) => Some(match end {
            Some(end) => format!("{start}-{end}"),
            None => start.to_string(),
        }),
        _ => None,
    }
}

fn next_attribute_list<'a>(tokens: &'a [Token<'a>], marker_index: usize) -> Option<&'a [AttributeItem<'a>]> {
    match tokens.get(marker_index + 1).map(|token| &token.data) {
        Some(TokenData::AttributeList { entries }) => Some(entries.as_slice()),
        _ => None,
    }
}

fn open_book_element<'a>(marker: &'a str, code: &'a str, prefer_native_elements: bool) -> OpenElement<'a> {
    let tag = if prefer_native_elements { "section" } else { "div" };
    let mut attrs = common_marker_attrs("book", marker);
    attrs.push(("data-usfm-code".to_string(), code.to_string()));
    OpenElement {
        marker: Some(marker),
        tag,
        attrs,
        buffer: String::new(),
        scope_kind: StructuralScopeKind::Header,
        note_subkind: None,
        note_family: None,
        synthetic: false,
    }
}

fn synthetic_table(prefer_native_elements: bool) -> OpenElement<'static> {
    let _ = prefer_native_elements;
    OpenElement {
        marker: None,
        tag: "table",
        attrs: vec![("data-usfm-type".to_string(), "table".to_string())],
        buffer: String::new(),
        scope_kind: StructuralScopeKind::Block,
        note_subkind: None,
        note_family: None,
        synthetic: true,
    }
}

fn synthetic_table_row() -> OpenElement<'static> {
    OpenElement {
        marker: None,
        tag: "tr",
        attrs: vec![("data-usfm-type".to_string(), "table:row".to_string())],
        buffer: String::new(),
        scope_kind: StructuralScopeKind::TableRow,
        note_subkind: None,
        note_family: None,
        synthetic: true,
    }
}

fn ensure_table_open(stack: &mut Vec<OpenElement<'_>>, prefer_native_elements: bool) {
    let _ = prefer_native_elements;
    if !stack
        .iter()
        .any(|item| item.attrs.iter().any(|(key, value)| key == "data-usfm-type" && value == "table"))
    {
        stack.push(synthetic_table(prefer_native_elements));
    }
}

fn tag_and_type_for_marker(
    marker: &str,
    kind: Option<SpecMarkerKind>,
    scope_kind: StructuralScopeKind,
    prefer_native_elements: bool,
) -> (&'static str, &'static str) {
    match kind {
        Some(SpecMarkerKind::Figure) => {
            if prefer_native_elements {
                ("figure", "figure")
            } else {
                ("div", "figure")
            }
        }
        Some(SpecMarkerKind::Periph) => ("div", "periph"),
        Some(SpecMarkerKind::Sidebar) => ("div", "sidebar"),
        Some(SpecMarkerKind::TableRow) => ("tr", "table:row"),
        Some(SpecMarkerKind::TableCell) => ("td", "table:cell"),
        Some(SpecMarkerKind::Character) if marker == "ref" => ("span", "ref"),
        Some(SpecMarkerKind::Character) => ("span", "char"),
        Some(SpecMarkerKind::Milestone) => ("span", "ms"),
        Some(SpecMarkerKind::Header | SpecMarkerKind::Paragraph | SpecMarkerKind::Meta) => {
            if marker == "id" {
                if prefer_native_elements {
                    ("section", "book")
                } else {
                    ("div", "book")
                }
            } else {
                ("div", "para")
            }
        }
        _ => match scope_kind {
            StructuralScopeKind::TableRow => ("tr", "table:row"),
            StructuralScopeKind::TableCell => ("td", "table:cell"),
            StructuralScopeKind::Sidebar => ("div", "sidebar"),
            StructuralScopeKind::Periph => ("div", "periph"),
            StructuralScopeKind::Character | StructuralScopeKind::Milestone => ("span", "char"),
            _ => ("div", "unknown"),
        },
    }
}

fn close_for_new_block(output: &mut String, stack: &mut Vec<OpenElement<'_>>, keep_book: bool) {
    close_inline_scopes(output, stack);
    close_non_book_table_block_scopes(output, stack);
    if !keep_book {
        while matches!(stack.last().map(|item| item.scope_kind), Some(StructuralScopeKind::Header))
            && stack
                .last()
                .is_some_and(|item| !item.synthetic && item.marker == Some("id"))
        {
            let item = stack.pop().expect("book wrapper present");
            append_closed_element(output, stack, item);
        }
    }
}

fn close_non_book_table_block_scopes(output: &mut String, stack: &mut Vec<OpenElement<'_>>) {
    while let Some(top) = stack.last() {
        if top.synthetic && top.attrs.iter().any(|(key, value)| key == "data-usfm-type" && value == "table")
        {
            let item = stack.pop().expect("table wrapper present");
            append_closed_element(output, stack, item);
            continue;
        }

        match top.scope_kind {
            StructuralScopeKind::TableCell
            | StructuralScopeKind::TableRow
            | StructuralScopeKind::Block
            | StructuralScopeKind::Sidebar
            | StructuralScopeKind::Periph
            | StructuralScopeKind::Meta => {
                let item = stack.pop().expect("block scope present");
                append_closed_element(output, stack, item);
            }
            StructuralScopeKind::Header if top.marker != Some("id") => {
                let item = stack.pop().expect("header scope present");
                append_closed_element(output, stack, item);
            }
            _ => break,
        }
    }
}

fn close_inline_scopes(output: &mut String, stack: &mut Vec<OpenElement<'_>>) {
    while matches!(
        stack.last().map(|item| item.scope_kind),
        Some(StructuralScopeKind::Character | StructuralScopeKind::Milestone)
    ) {
        let item = stack.pop().expect("inline scope present");
        append_closed_element(output, stack, item);
    }
}

fn close_for_note_structural(output: &mut String, stack: &mut Vec<OpenElement<'_>>, next_marker: &str) {
    if marker_note_subkind(next_marker).is_none() {
        return;
    }

    while let Some(top) = stack.last() {
        if !matches!(
            top.scope_kind,
            StructuralScopeKind::Character | StructuralScopeKind::Milestone
        ) {
            break;
        }
        let item = stack.pop().expect("note inline scope present");
        let should_stop = item.note_subkind.is_some() || item.note_family.is_some();
        append_closed_element(output, stack, item);
        if should_stop {
            break;
        }
    }
}

fn close_for_end_marker(output: &mut String, stack: &mut Vec<OpenElement<'_>>, name: &str) {
    let target = lookup_marker_def(name)
        .map(|def| def.marker)
        .unwrap_or(name.trim_start_matches('+'));
    close_top_matching_scope(output, stack, |item| {
        item.marker == Some(target) || item.marker == Some(name)
    });
}

fn close_top_matching_scope<F>(output: &mut String, stack: &mut Vec<OpenElement<'_>>, matches: F)
where
    F: Fn(&OpenElement<'_>) -> bool,
{
    let Some(position) = stack.iter().rposition(matches) else {
        return;
    };

    while stack.len() > position {
        let item = stack.pop().expect("matching scope should exist");
        append_closed_element(output, stack, item);
    }
}

fn append_closed_element(output: &mut String, stack: &mut [OpenElement<'_>], item: OpenElement<'_>) {
    let mut html = String::new();
    html.push('<');
    html.push_str(item.tag);
    push_attrs(&mut html, &item.attrs);
    html.push('>');
    html.push_str(&item.buffer);
    html.push_str("</");
    html.push_str(item.tag);
    html.push('>');
    if let Some(parent) = stack.last_mut() {
        parent.buffer.push_str(&html);
    } else {
        output.push_str(&html);
    }
}

fn push_fragment(output: &mut String, stack: &mut Vec<OpenElement<'_>>, html: &str) {
    if let Some(parent) = stack.last_mut() {
        parent.buffer.push_str(html);
    } else {
        output.push_str(html);
    }
}

fn common_marker_attrs(data_type: &str, marker: &str) -> Vec<(String, String)> {
    vec![
        ("data-usfm-type".to_string(), data_type.to_string()),
        ("data-usfm-marker".to_string(), marker.to_string()),
    ]
}

fn empty_marker_span(data_type: &str, marker: &str, number: &str) -> String {
    let mut out = String::from("<span");
    push_attr(&mut out, "data-usfm-type", data_type);
    push_attr(&mut out, "data-usfm-marker", marker);
    if !number.is_empty() {
        push_attr(&mut out, "data-usfm-number", number);
    }
    out.push_str("></span>");
    out
}

fn push_attribute_entries(attrs: &mut Vec<(String, String)>, entries: &[AttributeItem<'_>]) {
    for entry in entries {
        attrs.push((
            format!("data-usfm-{}", kebab_case(entry.key)),
            entry.value.to_string(),
        ));
    }
}

fn render_extracted_note(
    marker: &str,
    note_kind: NoteKind,
    source_caller: &str,
    label: &str,
    call_id: &str,
    note_id: &str,
    body: &str,
) -> String {
    let mut out = String::from("<aside");
    push_attr(&mut out, "id", note_id);
    push_attr(&mut out, "data-usfm-type", "note");
    push_attr(&mut out, "data-usfm-marker", marker);
    push_attr(&mut out, "data-usfm-caller", label);
    push_attr(&mut out, "data-usfm-source-caller", source_caller);
    push_attr(&mut out, "data-usfm-note-kind", note_kind.as_str());
    out.push('>');
    out.push_str("<a");
    push_attr(&mut out, "href", &format!("#{call_id}"));
    out.push('>');
    out.push_str(&escape_html(label));
    out.push_str("</a>");
    out.push_str(body);
    out.push_str("</aside>");
    out
}

#[derive(Clone, Copy)]
enum NoteKind {
    Footnote,
    Crossref,
}

impl NoteKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Footnote => "footnote",
            Self::Crossref => "crossref",
        }
    }
}

fn note_kind(marker: &str) -> NoteKind {
    match marker_note_family(marker) {
        Some(NoteFamily::CrossReference) => NoteKind::Crossref,
        _ => NoteKind::Footnote,
    }
}

fn marker_data_type(marker: &str, kind: Option<SpecMarkerKind>) -> &'static str {
    match kind {
        Some(SpecMarkerKind::Figure) => "figure",
        Some(SpecMarkerKind::Periph) => "periph",
        Some(SpecMarkerKind::Sidebar) => "sidebar",
        Some(SpecMarkerKind::TableRow) => "table:row",
        Some(SpecMarkerKind::TableCell) => "table:cell",
        Some(SpecMarkerKind::Milestone) => "ms",
        Some(SpecMarkerKind::Character) if marker == "ref" => "ref",
        Some(SpecMarkerKind::Character) => "char",
        Some(SpecMarkerKind::Header | SpecMarkerKind::Paragraph | SpecMarkerKind::Meta) => "para",
        _ => "unknown",
    }
}

fn marker_is_sidebar_end(marker: &str, kind: Option<SpecMarkerKind>) -> bool {
    matches!(kind, Some(SpecMarkerKind::Sidebar))
        || matches!(marker_block_behavior(marker), BlockBehavior::SidebarEnd)
}

fn table_cell_align(marker: &str) -> &'static str {
    if marker.starts_with("thr")
        || marker.starts_with("tcr")
        || marker.ends_with('r')
    {
        "end"
    } else if marker.starts_with("thc") || marker.starts_with("tcc") || marker.ends_with('c') {
        "center"
    } else {
        "start"
    }
}

fn push_attrs(out: &mut String, attrs: &[(String, String)]) {
    for (key, value) in attrs {
        push_attr(out, key, value);
    }
}

fn push_attr(out: &mut String, key: &str, value: &str) {
    out.push(' ');
    out.push_str(key);
    out.push_str("=\"");
    out.push_str(&escape_html(value));
    out.push('"');
}

fn escape_html(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn kebab_case(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' || ch == ' ' {
            out.push('-');
        } else {
            out.push(ch);
        }
    }
    out
}

fn format_ordinal(index: usize, style: HtmlCallerStyle) -> String {
    match style {
        HtmlCallerStyle::Numeric => index.to_string(),
        HtmlCallerStyle::AlphaLower => alpha_label(index, false),
        HtmlCallerStyle::AlphaUpper => alpha_label(index, true),
        HtmlCallerStyle::RomanLower => roman_label(index, false),
        HtmlCallerStyle::RomanUpper => roman_label(index, true),
        HtmlCallerStyle::Source => index.to_string(),
    }
}

fn alpha_label(mut index: usize, uppercase: bool) -> String {
    let mut out = String::new();
    while index > 0 {
        let rem = (index - 1) % 26;
        let base = if uppercase { b'A' } else { b'a' };
        out.insert(0, (base + rem as u8) as char);
        index = (index - 1) / 26;
    }
    out
}

fn roman_label(mut index: usize, uppercase: bool) -> String {
    let numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for (value, numeral) in numerals {
        while index >= value {
            out.push_str(numeral);
            index -= value;
        }
    }
    if uppercase {
        out
    } else {
        out.to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_html_extracts_footnotes_with_verse_scoped_callers() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\f + \\ft note one\\f* more\\f + \\ft note two\\f*\n",
            HtmlOptions::default(),
        );

        assert!(html.contains(r#"data-usfm-caller="1.1""#));
        assert!(html.contains(r#"data-usfm-caller="1.2""#));
        assert!(html.contains(r#"id="linkedFootnotes""#));
        assert!(html.contains(r#"data-usfm-source-caller="+""#));
    }

    #[test]
    fn crossrefs_are_extracted_into_separate_group() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\x - \\xo 1.1 \\xt cross ref\\x*\n",
            HtmlOptions::default(),
        );

        assert!(html.contains(r#"id="linkedCrossrefs""#));
        assert!(html.contains(r#"data-usfm-note-kind="crossref""#));
    }

    #[test]
    fn preverse_notes_fall_back_to_document_sequential_labels() {
        let html = usfm_to_html("\\s1 Heading\\f + \\ft note\\f*\n", HtmlOptions::default());
        assert!(html.contains(r#"data-usfm-caller="1""#));
    }

    #[test]
    fn inline_note_mode_renders_note_in_flow() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 Text\\f + \\ft note\\f*\n",
            HtmlOptions {
                note_mode: HtmlNoteMode::Inline,
                ..HtmlOptions::default()
            },
        );

        assert!(html.contains(r#"data-usfm-type="note""#));
        assert!(!html.contains(r#"id="linkedFootnotes""#));
    }

    #[test]
    fn delayed_attribute_list_attaches_to_open_char() {
        let html = usfm_to_html(
            "\\c 1\n\\p\n\\v 1 \\w gracious|lemma=\"grace\" strong=\"H1\"\\w*.\n",
            HtmlOptions::default(),
        );

        assert!(html.contains(r#"data-usfm-lemma="grace""#));
        assert!(html.contains(r#"data-usfm-strong="H1""#));
    }
}
