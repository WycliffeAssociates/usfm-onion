//! Renders the checked-in JS/TS wire-schema constants module
//! (`js/wire-schema.js` + `.d.ts`) from this crate's own `schema` module — the
//! same constants the Rust codec compiles against, never a hand-written copy.
//!
//! The checked-in files are regenerated, not hand-edited:
//!
//!   cargo run --example generate_js_schema -p usfm_onion_wire
//!
//! [`tests::wire_schema_matches_generator`] fails if the files on disk drift
//! from what [`render`] currently produces.
//!
//! This is codegen rather than a `wasm-bindgen` export because these values
//! are static schema data, not a decoder — reading them through wasm would
//! mean instantiating the module and crossing a runtime boundary just for
//! constants. The byte-layout tables (field ids/widths, magics) are
//! load-bearing production data for the official pure-JS `materialize`
//! decoder, which decodes wasm-certified packed buffers directly in the JS
//! engine (Rust/wasm remains the sole trust boundary — validation, XXH3
//! checksums, source binding; there is never a JS hash implementation). The
//! semantic catalog (lint codes, param contracts) additionally serves runtime
//! JS like message localization. Both decoders are held identical by golden
//! vectors and a serde-JSON equivalence gate. See the generated header for
//! the full rationale.
//!
//! Field names below (`TOKEN_FIELD_NAMES`/`FINDING_FIELD_NAMES`) are the one
//! thing this module supplies rather than reads: Rust constant identifiers
//! aren't reflectable at runtime, so a human-readable JS name has to be
//! chosen somewhere. Every numeric value, width, requiredness, and the
//! dense/ordered position each name is paired with all come straight from
//! [`crate::schema`]'s compiled tables — see that module's
//! `field_tables_are_dense_and_ordered` test for the invariant this relies on.

use std::fmt::Write as _;

use crate::catalog::catalog_stamp;
use crate::schema::{
    self, ATTRIBUTE_ENTRY_LEN, ATTRIBUTE_FLAG_DEFAULT, ATTRIBUTE_ROW_LEN, BOOK_CODE_FLAG_VALID,
    BOOK_CODE_RECORD_LEN, CHECKSUM_OMITTED, CONTAINER_CHECKSUM_OFFSET, CONTAINER_FLAGS_KNOWN,
    CONTAINER_HEADER_LEN, CONTAINER_MAGIC, CONTAINER_RESERVED_LEN, CONTAINER_RESERVED_OFFSET,
    DESCRIPTOR_FLAG_NESTED, DESCRIPTOR_RECORD_LEN, DIRECTORY_ENTRY_LEN, ELEMENT_WIDTH_VARIABLE,
    ELEMENT_WIDTHS, FIELD_FLAG_REQUIRED, FINDING_SECTION_FLAGS_KNOWN,
    FINDING_SECTION_RULES_VERSION, FORMAT_VERSION, INDEX_NONE_U16, INDEX_NONE_U32, LINT_CODE_TABLE,
    MAX_DISTINCT_SIDS, MAX_MARKER_DESCRIPTORS, NUMBER_FLAG_HAS_END, NUMBER_RECORD_LEN,
    PACKED_SID_LEN, PARAM_CONTRACTS, SECTION_ALIGN, SECTION_CHECKSUM_OFFSET,
    SECTION_FLAG_POSITIONAL_IDS, SECTION_HEADER_LEN, SECTION_MAGIC, SECTION_VERSION,
    SID_DELTA_MASK, SID_FIDELITY_BIT, SPAN_ABSENT, TOC_ENTRY_LEN, TOC_FLAGS_KNOWN,
    TOKEN_SECTION_FLAGS_KNOWN, TOKEN_SECTION_RULES_VERSION, finding_field, finding_flag, layout,
    token_field,
};

/// The exact command that regenerates the checked-in files — printed in their
/// header so a reader knows how to refresh them.
pub const GENERATOR_COMMAND: &str = "cargo run --example generate_js_schema -p usfm_onion_wire";

/// Display names for `token_field::TABLE`, in the same dense id order.
const TOKEN_FIELD_NAMES: [&str; 13] = [
    "kind",
    "spanStart",
    "spanEnd",
    "tokenIdIndex",
    "sidIndex",
    "markerDescriptorIndex",
    "numberRecords",
    "bookCodeRecords",
    "attributeRecords",
    "tokenIdDictionary",
    "stringDictionary",
    "markerDescriptorDictionary",
    "packedSidDictionary",
];

/// `dto::TokenKind`'s serde wire strings, in [`schema::TokenKindTag`] order, so
/// a JS decoder can turn a packed kind tag into the exact string the wasm token
/// DTO carries. Written out rather than derived because serde's `rename_all`
/// is not reflectable at runtime;
/// [`tests::kind_wire_strings_match_the_dto_serde_contract`] pins every entry
/// against the real `Serialize` output.
const TOKEN_KIND_WIRE: [&str; 9] = [
    "newline",
    "optBreak",
    "marker",
    "endMarker",
    "milestone",
    "milestoneEnd",
    "bookCode",
    "number",
    "text",
];

/// `dto::NumberRangeKind`'s serde wire strings, in
/// [`schema::NumberRangeKindTag`] order. Same rationale and same pinning test as
/// [`TOKEN_KIND_WIRE`].
const NUMBER_RANGE_KIND_WIRE: [&str; 4] = ["single", "range", "sequence", "sequenceWithRange"];

/// Display names for `finding_field::TABLE`, in the same dense id order.
const FINDING_FIELD_NAMES: [&str; 9] = [
    "commonRow",
    "relatedTokenIdx",
    "overflowSpan",
    "messagePayloadIdx",
    "markerRef",
    "patchId",
    "patchTable",
    "stringDictionary",
    "messagePayloadTable",
];

/// Renders `(wire-schema.js, wire-schema.d.ts)` from the compiled schema
/// constants. Kept side-effect-free (no filesystem access) so both the
/// generator binary and the drift-check test call the exact same code.
pub fn render() -> (String, String) {
    let mut js = String::new();
    let mut dts = String::new();

    let header = format!(
        "// GENERATED FILE — DO NOT EDIT.\n\
         // Regenerate with: {GENERATOR_COMMAND}\n\
         // Source of truth: crates/usfm_onion_wire/src/schema.rs\n\
         //\n\
         // Why codegen: single-source from the compiled Rust schema constants so\n\
         // JS never hand-mirrors the contract; a drift test fails if this file\n\
         // diverges from schema.rs.\n\
         //\n\
         // Why not wasm-bindgen: this is static schema data, not a decoder.\n\
         // Reading it through wasm would mean instantiating the module and\n\
         // crossing a runtime boundary just for constants.\n\
         //\n\
         // Consumers: the byte-layout tables (field ids/widths, magics) are\n\
         // load-bearing production data for the official pure-JS `materialize`\n\
         // decoder, which decodes wasm-certified packed buffers directly in the\n\
         // JS engine (Rust/wasm remains the sole trust boundary — validation,\n\
         // XXH3 checksums, source binding; there is never a JS hash\n\
         // implementation). The semantic catalog (LINT_CODES, PARAM_CONTRACTS,\n\
         // rules version) additionally serves runtime JS like message\n\
         // localization. Both decoders are held identical by golden vectors and\n\
         // a serde-JSON equivalence gate.\n\n"
    );
    js.push_str(&header);
    dts.push_str(&header);

    js.push_str(
        "/**\n\
         * Recursively freezes an object/array and everything reachable from it\n\
         * via its own enumerable-or-not property names (`Object.freeze` alone\n\
         * is shallow). Every exported table below is immutable schema data the\n\
         * decoder indexes into; this makes that true at runtime, matching the\n\
         * `Readonly<...>`/`readonly` types already declared for it in the\n\
         * `.d.ts`, rather than only documenting it there.\n\
         */\n\
         function deepFreeze(value) {\n  \
           if (\n    \
             value !== null &&\n    \
             (typeof value === \"object\" || typeof value === \"function\") &&\n    \
             !Object.isFrozen(value)\n  \
           ) {\n    \
             Object.freeze(value);\n    \
             for (const key of Object.getOwnPropertyNames(value)) {\n      \
               deepFreeze(value[key]);\n    \
             }\n  \
           }\n  \
           return value;\n\
         }\n\n",
    );

    macro_rules! konst {
        ($name:ident : number = $value:expr) => {
            writeln!(js, "export const {} = {};", stringify!($name), $value).unwrap();
            writeln!(dts, "export declare const {}: number;", stringify!($name)).unwrap();
        };
    }
    macro_rules! konst_str {
        ($name:ident = $value:expr) => {
            writeln!(js, "export const {} = {:?};", stringify!($name), $value).unwrap();
            writeln!(dts, "export declare const {}: string;", stringify!($name)).unwrap();
        };
    }
    macro_rules! konst_bigint {
        ($name:ident = $value:expr) => {
            writeln!(js, "export const {} = {}n;", stringify!($name), $value).unwrap();
            writeln!(dts, "export declare const {}: bigint;", stringify!($name)).unwrap();
        };
    }

    konst_str!(CONTAINER_MAGIC = std::str::from_utf8(&CONTAINER_MAGIC).unwrap());
    konst_str!(SECTION_MAGIC = std::str::from_utf8(&SECTION_MAGIC).unwrap());
    konst!(FORMAT_VERSION: number = FORMAT_VERSION);
    konst!(SECTION_VERSION: number = SECTION_VERSION);
    konst!(TOKEN_SECTION_RULES_VERSION: number = TOKEN_SECTION_RULES_VERSION);
    konst!(FINDING_SECTION_RULES_VERSION: number = FINDING_SECTION_RULES_VERSION);
    konst_bigint!(MARKER_CATALOG_STAMP = catalog_stamp());
    konst!(CONTAINER_HEADER_LEN: number = CONTAINER_HEADER_LEN);
    konst!(TOC_ENTRY_LEN: number = TOC_ENTRY_LEN);
    konst!(SECTION_HEADER_LEN: number = SECTION_HEADER_LEN);
    konst!(DIRECTORY_ENTRY_LEN: number = DIRECTORY_ENTRY_LEN);
    konst!(SECTION_ALIGN: number = SECTION_ALIGN);
    konst!(CONTAINER_CHECKSUM_OFFSET: number = CONTAINER_CHECKSUM_OFFSET);
    konst!(SECTION_CHECKSUM_OFFSET: number = SECTION_CHECKSUM_OFFSET);
    konst!(CHECKSUM_OMITTED: number = CHECKSUM_OMITTED);
    konst!(CONTAINER_RESERVED_OFFSET: number = CONTAINER_RESERVED_OFFSET);
    konst!(CONTAINER_RESERVED_LEN: number = CONTAINER_RESERVED_LEN);
    konst!(CONTAINER_FLAGS_KNOWN: number = CONTAINER_FLAGS_KNOWN);
    konst!(TOC_FLAGS_KNOWN: number = TOC_FLAGS_KNOWN);
    konst!(SECTION_FLAG_POSITIONAL_IDS: number = SECTION_FLAG_POSITIONAL_IDS);
    konst!(TOKEN_SECTION_FLAGS_KNOWN: number = TOKEN_SECTION_FLAGS_KNOWN);
    konst!(FINDING_SECTION_FLAGS_KNOWN: number = FINDING_SECTION_FLAGS_KNOWN);
    konst!(FIELD_FLAG_REQUIRED: number = FIELD_FLAG_REQUIRED);
    konst!(ELEMENT_WIDTH_VARIABLE: number = ELEMENT_WIDTH_VARIABLE);
    konst!(INDEX_NONE_U16: number = INDEX_NONE_U16);
    konst!(INDEX_NONE_U32: number = INDEX_NONE_U32);
    konst!(MAX_DISTINCT_SIDS: number = MAX_DISTINCT_SIDS);
    konst!(DESCRIPTOR_RECORD_LEN: number = DESCRIPTOR_RECORD_LEN);
    konst!(NUMBER_RECORD_LEN: number = NUMBER_RECORD_LEN);
    konst!(BOOK_CODE_RECORD_LEN: number = BOOK_CODE_RECORD_LEN);
    konst!(ATTRIBUTE_ROW_LEN: number = ATTRIBUTE_ROW_LEN);
    konst!(ATTRIBUTE_ENTRY_LEN: number = ATTRIBUTE_ENTRY_LEN);
    konst!(DESCRIPTOR_FLAG_NESTED: number = DESCRIPTOR_FLAG_NESTED);
    konst!(NUMBER_FLAG_HAS_END: number = NUMBER_FLAG_HAS_END);
    konst!(BOOK_CODE_FLAG_VALID: number = BOOK_CODE_FLAG_VALID);
    konst!(ATTRIBUTE_FLAG_DEFAULT: number = ATTRIBUTE_FLAG_DEFAULT);
    konst!(SPAN_ABSENT: number = SPAN_ABSENT);
    konst!(MAX_MARKER_DESCRIPTORS: number = MAX_MARKER_DESCRIPTORS);
    konst!(PACKED_SID_LEN: number = PACKED_SID_LEN);
    konst!(SID_FIDELITY_BIT: number = SID_FIDELITY_BIT);
    konst!(SID_DELTA_MASK: number = SID_DELTA_MASK);
    konst!(STRING_DICTIONARY_ENTRY_LEN: number = layout::STRING_DICTIONARY_ENTRY_LEN);

    writeln!(
        js,
        "export const ELEMENT_WIDTHS = deepFreeze([{}]);",
        ELEMENT_WIDTHS
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(
        dts,
        "export declare const ELEMENT_WIDTHS: readonly number[];"
    )
    .unwrap();

    writeln!(
        js,
        "\nexport const FINDING_FLAG = deepFreeze({{\n  anchorOnly: {},\n  noAnchor: {},\n  range: {},\n  related: {},\n  payload: {},\n  fix: {},\n  overflow: {},\n}});",
        finding_flag::ANCHOR_ONLY,
        finding_flag::NO_ANCHOR,
        finding_flag::RANGE,
        finding_flag::RELATED,
        finding_flag::PAYLOAD,
        finding_flag::FIX,
        finding_flag::OVERFLOW,
    )
    .unwrap();
    writeln!(
        dts,
        "\nexport declare const FINDING_FLAG: Readonly<{{\n  anchorOnly: number;\n  noAnchor: number;\n  range: number;\n  related: number;\n  payload: number;\n  fix: number;\n  overflow: number;\n}}>;"
    )
    .unwrap();

    writeln!(
        js,
        "\nexport const TOKEN_KIND = deepFreeze({{\n  Newline: 0,\n  OptBreak: 1,\n  Marker: 2,\n  EndMarker: 3,\n  Milestone: 4,\n  MilestoneEnd: 5,\n  BookCode: 6,\n  Number: 7,\n  Text: 8,\n}});"
    )
    .unwrap();
    writeln!(
        dts,
        "\nexport declare const TOKEN_KIND: Readonly<{{\n  Newline: 0;\n  OptBreak: 1;\n  Marker: 2;\n  EndMarker: 3;\n  Milestone: 4;\n  MilestoneEnd: 5;\n  BookCode: 6;\n  Number: 7;\n  Text: 8;\n}}>;"
    )
    .unwrap();

    writeln!(
        js,
        "\nexport const NUMBER_RANGE_KIND = deepFreeze({{\n  Single: 0,\n  Range: 1,\n  Sequence: 2,\n  SequenceWithRange: 3,\n}});"
    )
    .unwrap();
    writeln!(
        dts,
        "\nexport declare const NUMBER_RANGE_KIND: Readonly<{{\n  Single: 0;\n  Range: 1;\n  Sequence: 2;\n  SequenceWithRange: 3;\n}}>;"
    )
    .unwrap();

    // Field offsets, so a random-access decoder never hand-writes one. The Rust
    // codec reads these structures with a sequential cursor and therefore has no
    // offsets of its own to export — `schema::layout` holds them, pinned to the
    // real byte layout by `schema::tests::layout_offsets_name_the_fields_the_codec_writes`.
    render_offsets(
        &mut js,
        &mut dts,
        "CONTAINER_HEADER_OFFSET",
        &[
            ("magic", layout::container_header::MAGIC),
            ("formatVersion", layout::container_header::FORMAT_VERSION),
            ("headerLen", layout::container_header::HEADER_LEN),
            ("flags", layout::container_header::FLAGS),
            ("sectionCount", layout::container_header::SECTION_COUNT),
            ("tocOffset", layout::container_header::TOC_OFFSET),
            ("checksum", layout::container_header::CHECKSUM),
            ("snapshotId", layout::container_header::SNAPSHOT_ID),
            ("reserved", layout::container_header::RESERVED),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "TOC_ENTRY_OFFSET",
        &[
            ("kind", layout::toc_entry::KIND),
            ("book", layout::toc_entry::BOOK),
            ("sectionVersion", layout::toc_entry::SECTION_VERSION),
            ("flags", layout::toc_entry::FLAGS),
            ("offset", layout::toc_entry::OFFSET),
            ("byteLen", layout::toc_entry::BYTE_LEN),
            ("sourceHash", layout::toc_entry::SOURCE_HASH),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "SECTION_HEADER_OFFSET",
        &[
            ("magic", layout::section_header::MAGIC),
            ("formatVersion", layout::section_header::FORMAT_VERSION),
            ("rulesVersion", layout::section_header::RULES_VERSION),
            ("kind", layout::section_header::KIND),
            ("flags", layout::section_header::FLAGS),
            ("book", layout::section_header::BOOK),
            ("reserved", layout::section_header::RESERVED),
            ("recordCount", layout::section_header::RECORD_COUNT),
            ("directoryCount", layout::section_header::DIRECTORY_COUNT),
            (
                "directoryEntrySize",
                layout::section_header::DIRECTORY_ENTRY_SIZE,
            ),
            ("sourceHash", layout::section_header::SOURCE_HASH),
            ("sectionLen", layout::section_header::SECTION_LEN),
            ("checksum", layout::section_header::CHECKSUM),
            ("sourceLen", layout::section_header::SOURCE_LEN),
            ("catalogStamp", layout::section_header::CATALOG_STAMP),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "DIRECTORY_ENTRY_OFFSET",
        &[
            ("fieldId", layout::directory_entry::FIELD_ID),
            ("elementWidth", layout::directory_entry::ELEMENT_WIDTH),
            ("flags", layout::directory_entry::FLAGS),
            ("offset", layout::directory_entry::OFFSET),
            ("byteLen", layout::directory_entry::BYTE_LEN),
            ("count", layout::directory_entry::COUNT),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "PACKED_SID_OFFSET",
        &[
            ("book", layout::packed_sid::BOOK),
            ("chapter", layout::packed_sid::CHAPTER),
            ("verse", layout::packed_sid::VERSE),
            ("delta", layout::packed_sid::DELTA),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "DESCRIPTOR_RECORD_OFFSET",
        &[
            ("nameIndex", layout::descriptor_record::NAME_INDEX),
            ("flags", layout::descriptor_record::FLAGS),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "NUMBER_RECORD_OFFSET",
        &[
            ("tokenIdx", layout::number_record::TOKEN_IDX),
            ("start", layout::number_record::START),
            ("end", layout::number_record::END),
            ("kind", layout::number_record::KIND),
            ("flags", layout::number_record::FLAGS),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "BOOK_CODE_RECORD_OFFSET",
        &[
            ("tokenIdx", layout::book_code_record::TOKEN_IDX),
            ("codeIndex", layout::book_code_record::CODE_INDEX),
            ("flags", layout::book_code_record::FLAGS),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "ATTRIBUTE_ROW_OFFSET",
        &[
            ("tokenIdx", layout::attribute_row::TOKEN_IDX),
            ("firstEntry", layout::attribute_row::FIRST_ENTRY),
            ("entryCount", layout::attribute_row::ENTRY_COUNT),
            ("listStart", layout::attribute_row::LIST_START),
            ("listLen", layout::attribute_row::LIST_LEN),
        ],
    );
    render_offsets(
        &mut js,
        &mut dts,
        "ATTRIBUTE_ENTRY_OFFSET",
        &[
            ("keyIndex", layout::attribute_entry::KEY_INDEX),
            ("valueIndex", layout::attribute_entry::VALUE_INDEX),
            ("spanStart", layout::attribute_entry::SPAN_START),
            ("spanLen", layout::attribute_entry::SPAN_LEN),
            ("flags", layout::attribute_entry::FLAGS),
        ],
    );

    render_wire_strings(&mut js, &mut dts, "TOKEN_KIND_WIRE", &TOKEN_KIND_WIRE);
    render_wire_strings(
        &mut js,
        &mut dts,
        "NUMBER_RANGE_KIND_WIRE",
        &NUMBER_RANGE_KIND_WIRE,
    );

    writeln!(
        js,
        "\nexport const SECTION_KIND = deepFreeze({{\n  Token: 0,\n  Finding: 1,\n}});"
    )
    .unwrap();
    writeln!(
        dts,
        "\nexport declare const SECTION_KIND: Readonly<{{ Token: 0; Finding: 1 }}>;"
    )
    .unwrap();

    render_field_table(
        &mut js,
        &mut dts,
        "TOKEN_FIELD",
        token_field::TABLE,
        &TOKEN_FIELD_NAMES,
    );
    render_field_table(
        &mut js,
        &mut dts,
        "FINDING_FIELD",
        finding_field::TABLE,
        &FINDING_FIELD_NAMES,
    );

    writeln!(js, "\nexport const LINT_CODES = deepFreeze([").unwrap();
    for tag in LINT_CODE_TABLE {
        writeln!(
            js,
            "  {{ code: {}, kebab: {:?} }},",
            tag.as_u8(),
            tag.kebab()
        )
        .unwrap();
    }
    writeln!(js, "]);").unwrap();
    writeln!(
        dts,
        "\nexport declare const LINT_CODES: readonly Readonly<{{ code: number; kebab: string }}>[];"
    )
    .unwrap();

    writeln!(js, "\nexport const PARAM_CONTRACTS = deepFreeze([").unwrap();
    for contract in PARAM_CONTRACTS {
        writeln!(js, "  {{ code: {}, variants: [", contract.code.as_u8()).unwrap();
        for variant in contract.variants {
            writeln!(js, "    {{ params: [").unwrap();
            for param in variant.params {
                writeln!(
                    js,
                    "      {{ key: {:?}, allowedValues: {:?} }},",
                    param.key, param.allowed_values,
                )
                .unwrap();
            }
            writeln!(js, "    ] }},").unwrap();
        }
        writeln!(js, "  ] }},").unwrap();
    }
    writeln!(js, "]);").unwrap();
    writeln!(
        dts,
        "\nexport declare const PARAM_CONTRACTS: readonly Readonly<{{ code: number; variants: readonly Readonly<{{ params: readonly Readonly<{{ key: string; allowedValues: readonly string[] }}>[] }}>[] }}>[];"
    )
    .unwrap();

    (js, dts)
}

/// Emits one structure's field offsets as a frozen object of byte positions.
fn render_offsets(js: &mut String, dts: &mut String, export_name: &str, fields: &[(&str, usize)]) {
    writeln!(js, "\nexport const {export_name} = deepFreeze({{").unwrap();
    for (name, offset) in fields {
        writeln!(js, "  {name}: {offset},").unwrap();
    }
    writeln!(js, "}});").unwrap();
    writeln!(dts, "\nexport declare const {export_name}: Readonly<{{").unwrap();
    for (name, _) in fields {
        writeln!(dts, "  {name}: number;").unwrap();
    }
    writeln!(dts, "}}>;").unwrap();
}

/// Emits a tag-indexed array of serde wire strings: `array[tag]` is the string
/// the DTO serializes that discriminant as.
fn render_wire_strings(js: &mut String, dts: &mut String, export_name: &str, values: &[&str]) {
    writeln!(
        js,
        "\nexport const {export_name} = deepFreeze([{}]);",
        values
            .iter()
            .map(|value| format!("{value:?}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(
        dts,
        "\nexport declare const {export_name}: readonly string[];"
    )
    .unwrap();
}

fn render_field_table(
    js: &mut String,
    dts: &mut String,
    export_name: &str,
    table: &[schema::FieldSpec],
    names: &[&str],
) {
    assert_eq!(
        table.len(),
        names.len(),
        "{export_name}: field table/name-list length mismatch"
    );
    writeln!(js, "\nexport const {export_name} = deepFreeze([").unwrap();
    for (field, name) in table.iter().zip(names) {
        let width = match field.element_width {
            Some(w) => w.to_string(),
            None => "null".to_string(),
        };
        writeln!(
            js,
            "  {{ id: {}, name: {:?}, elementWidth: {}, required: {} }},",
            field.id, name, width, field.required
        )
        .unwrap();
    }
    writeln!(js, "]);").unwrap();
    writeln!(
        dts,
        "\nexport declare const {export_name}: readonly Readonly<{{ id: number; name: string; elementWidth: number | null; required: boolean }}>[];"
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::catalog::catalog_stamp;

    use super::render;

    /// Fails if `js/wire-schema.{js,d.ts}` on disk differ from what
    /// [`render`] currently produces — the same drift-check shape as the
    /// wasm golden suite, but for the generated schema module instead of
    /// runtime output. Never blessed automatically: regenerate by rerunning
    /// the generator example named in [`super::GENERATOR_COMMAND`].
    #[test]
    fn wire_schema_matches_generator() {
        let (js, dts) = render();

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .expect("crates/usfm_onion_wire is two levels under the repo root");

        let committed_js = std::fs::read_to_string(repo_root.join("js/wire-schema.js"))
            .expect("js/wire-schema.js must be checked in");
        let committed_dts = std::fs::read_to_string(repo_root.join("js/wire-schema.d.ts"))
            .expect("js/wire-schema.d.ts must be checked in");

        assert_eq!(
            js,
            committed_js,
            "js/wire-schema.js is stale — regenerate with `{}`",
            super::GENERATOR_COMMAND
        );
        assert_eq!(
            dts,
            committed_dts,
            "js/wire-schema.d.ts is stale — regenerate with `{}`",
            super::GENERATOR_COMMAND
        );
    }

    /// The generated kind strings are what a JS materializer stamps onto every
    /// token, so they must be the DTO's own serde output and not a plausible
    /// guess at serde's casing rule.
    #[test]
    fn kind_wire_strings_match_the_dto_serde_contract() {
        use crate::dto::{NumberRangeKind, TokenKind};
        use crate::schema::{NumberRangeKindTag, TokenKindTag};
        use usfm_onion::token::{
            NumberRangeKind as NativeNumberRangeKind, TokenKind as NativeTokenKind,
        };

        for (tag, expected) in super::TOKEN_KIND_WIRE.iter().enumerate() {
            let native: NativeTokenKind = TokenKindTag::from_u8(tag as u8)
                .expect("every index is a frozen tag")
                .into();
            let wire: TokenKind = native.into();
            assert_eq!(
                serde_json::to_value(wire).unwrap(),
                serde_json::Value::String((*expected).to_string()),
                "TOKEN_KIND_WIRE[{tag}]"
            );
        }
        for (tag, expected) in super::NUMBER_RANGE_KIND_WIRE.iter().enumerate() {
            let native: NativeNumberRangeKind = NumberRangeKindTag::from_u8(tag as u8)
                .expect("every index is a frozen tag")
                .into();
            let wire: NumberRangeKind = native.into();
            assert_eq!(
                serde_json::to_value(wire).unwrap(),
                serde_json::Value::String((*expected).to_string()),
                "NUMBER_RANGE_KIND_WIRE[{tag}]"
            );
        }
    }

    #[test]
    fn wire_schema_emits_the_runtime_marker_catalog_stamp() {
        let (js, _) = render();
        assert!(js.contains(&format!(
            "export const MARKER_CATALOG_STAMP = {}n;",
            catalog_stamp()
        )));
    }
}
