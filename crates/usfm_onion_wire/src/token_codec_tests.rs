//! Token-section round-trip tests.
//!
//! The narrow cases run on every `cargo test`. The corpus gate is `#[ignore]`d,
//! like the lint oracle, because it parses, encodes, and decodes every
//! `testData/**/*.usfm` fixture plus two full example corpora; run it with
//! `cargo test -p usfm_onion_wire -- --ignored`.

use std::fs;
use std::path::{Path, PathBuf};

use usfm_onion::parse::parse;
use usfm_onion::token::{
    BookId, OwnedToken, tokens_to_usfm_reconstruct, tokens_to_usfm_reconstruct_spanned,
};

use crate::container::{read_container, write_container};
use crate::error::DecodeError;
use crate::schema::token_field;
use crate::token_codec::{
    decode_token_section, encode_owned_token_section, encode_token_section,
    encode_token_section_with_ids, owned_to_borrowed,
};
use crate::token_section::{SidFidelity, TokenColumns};

const SNAPSHOT_ID: u64 = 7;

/// Books the corpus gates walk: every `testData/**/*.usfm` fixture plus
/// `example-corpora/en_ult` and `en_ulb`. Pinned exactly rather than as a lower
/// bound, so adding or removing a fixture is a deliberate edit here and a silently
/// shrinking corpus cannot make a gate vacuous.
const CORPUS_BOOKS: usize = 395;

fn book(code: &str) -> BookId {
    BookId::from_str(code).expect("test book code")
}

fn encoded(source: &str) -> Vec<u8> {
    let parsed = parse(source);
    let buffers =
        encode_token_section(book("GEN"), source, &parsed.tokens).expect("parsed tokens encode");
    write_container(SNAPSHOT_ID, &[buffers.payload()]).expect("section lays out")
}

/// Semantic equality against core's own parse, including marker metadata and
/// structural info — `Token`'s `PartialEq` covers every public field, so this is
/// the strongest available assertion rather than a hand-listed subset.
fn assert_round_trips(source: &str) {
    let parsed = parse(source);
    let bytes = encoded(source);
    let container = read_container(&bytes).expect("container decodes");
    let section = container
        .section(0)
        .expect("one section")
        .expect("section decodes");
    let decoded = decode_token_section(&section, source)
        .expect("token section decodes")
        .tokens;
    assert_eq!(
        decoded.len(),
        parsed.tokens.len(),
        "token count for {source:?}"
    );
    for (index, (decoded, expected)) in decoded.iter().zip(&parsed.tokens).enumerate() {
        assert_eq!(decoded, expected, "token {index} of {source:?}");
    }
    // Owned parity: the semantic type braid holds must agree too, which catches
    // any divergence that `Token`'s equality would tolerate through a derived
    // field.
    let owned_decoded: Vec<_> = decoded.iter().map(OwnedToken::from_parsed).collect();
    let owned_expected: Vec<_> = parsed.tokens.iter().map(OwnedToken::from_parsed).collect();
    assert_eq!(owned_decoded, owned_expected, "owned parity for {source:?}");
}

#[test]
fn round_trips_every_token_kind() {
    assert_round_trips("\\id GEN Some Book\n\\c 1\n\\p\n\\v 1 In the beginning.\n");
    // Newline, optbreak, milestone pair, end marker, nested marker.
    assert_round_trips("\\p text // more\n\\qt-s |x=\"y\"\\*inside\\qt-e\\*\n");
    assert_round_trips("\\p \\w word|strong=\"H1\"\\w* and \\add \\+nd Lord\\+nd*\\add*\n");
    // Number shapes: single, bridge, sequence.
    assert_round_trips("\\c 1\n\\p\n\\v 1-2 bridge\n\\v 3,5 sequence\n\\v 7a suffix\n");
    // An unknown marker keeps its name through the dictionary and comes back with
    // the same all-None metadata core produced.
    assert_round_trips("\\zzz unknown marker\n");
    // Invalid book code: the stored validity bit, not a recomputed one.
    assert_round_trips("\\id xyz lower case\n");
}

#[test]
fn round_trips_empty_and_whitespace_only_sources() {
    assert_round_trips("");
    assert_round_trips("\n");
    assert_round_trips("   ");
}

#[test]
fn round_trips_multibyte_text() {
    assert_round_trips("\\p Ἐν ἀρχῇ ἦν ὁ λόγος — 起初\n");
    assert_round_trips("\\p \\w λόγος|lemma=\"λόγος\"\\w*\n");
}

#[test]
fn round_trips_default_attribute_shorthand_and_empty_list() {
    // Shorthand stores an empty key, which the dictionary must preserve as an
    // empty string rather than collapse to absent.
    assert_round_trips("\\p \\w grace|H1234\\w*\n");
    assert_round_trips("\\p \\w grace|\\w*\n");
}

#[test]
fn wrong_source_length_rejects() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 text\n";
    let bytes = encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    assert_eq!(
        decode_token_section(&section, "\\id GEN\n").err(),
        Some(DecodeError::SourceLengthMismatch)
    );
}

#[test]
fn same_length_wrong_bytes_rejects() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 text\n";
    let mut wrong = source.to_string();
    // One byte different, same length: only the content hash can catch this.
    wrong.replace_range(source.len() - 5..source.len() - 4, "T");
    assert_eq!(wrong.len(), source.len());
    let bytes = encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    assert_eq!(
        decode_token_section(&section, &wrong).err(),
        Some(DecodeError::SourceHashMismatch)
    );
}

/// Recomputes the section and container integrity checksums after an edit, so a
/// case can target a semantic check instead of stopping at the hash.
fn restamp(bytes: &mut [u8]) {
    let section_offset = u64::from_le_bytes(bytes[56..64].try_into().unwrap()) as usize;
    let section_len = u64::from_le_bytes(bytes[64..72].try_into().unwrap()) as usize;
    let section_end = section_offset + section_len;
    bytes[section_offset + 40..section_offset + 48].copy_from_slice(&[0u8; 8]);
    let section_checksum =
        crate::primitives::integrity_checksum(&bytes[section_offset..section_end], 40);
    bytes[section_offset + 40..section_offset + 48]
        .copy_from_slice(&section_checksum.to_le_bytes());
    bytes[24..32].copy_from_slice(&[0u8; 8]);
    let container_checksum = crate::primitives::integrity_checksum(bytes, 24);
    bytes[24..32].copy_from_slice(&container_checksum.to_le_bytes());
}

#[test]
fn catalog_stamp_mismatch_rejects() {
    let source = "\\p text\n";
    let mut bytes = encoded(source);
    let section_offset = u64::from_le_bytes(bytes[56..64].try_into().unwrap()) as usize;
    // The stamp sits at section-header offset 56. Flip it and re-stamp the
    // checksums, so the container is otherwise perfectly well formed and the only
    // thing wrong is that its marker ordinals were written against a different
    // registry than this build has.
    bytes[section_offset + 56] ^= 0xff;
    restamp(&mut bytes);

    let container = read_container(&bytes).expect("container is still well formed");
    let section = container
        .section(0)
        .unwrap()
        .expect("section is well formed");
    assert_eq!(
        decode_token_section(&section, source).err(),
        Some(DecodeError::CatalogMismatch)
    );
}

#[test]
fn truncating_the_wire_never_panics() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1-2 text \\w x|y=\"z\"\\w*\n";
    let bytes = encoded(source);
    for len in 0..bytes.len() {
        let result = read_container(&bytes[..len]).and_then(|container| {
            let section = container.section(0).ok_or(DecodeError::InvalidToc)??;
            decode_token_section(&section, source).map(|_| ())
        });
        assert!(result.is_err(), "prefix of {len} bytes must not decode");
    }
}

#[test]
fn corrupting_any_wire_byte_never_panics() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1-2 text \\w x|y=\"z\"\\w*\n";
    let base = encoded(source);
    for index in 0..base.len() {
        for value in [0x00u8, 0x01, 0x7f, 0xff] {
            let mut bytes = base.clone();
            bytes[index] = value;
            // Only that nothing panics: almost every mutation is caught by the
            // container checksum, and the ones that are not must still return a
            // typed error rather than a bad view.
            let _ = read_container(&bytes).and_then(|container| {
                let section = container.section(0).ok_or(DecodeError::InvalidToc)??;
                decode_token_section(&section, source).map(|_| ())
            });
        }
    }
}

#[test]
fn encoding_is_deterministic() {
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1 a \\w x|y=\"z\"\\w* b\n\\v 2 c\n";
    assert_eq!(encoded(source), encoded(source));
}

#[test]
fn sparse_fields_are_omitted_when_unused() {
    // Plain prose has no numbers, no book code, and no attributes, so none of
    // the three sparse columns should exist at all.
    let bytes = encoded("\\p just text\n");
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    assert!(section.field(token_field::NUMBER_RECORDS).is_none());
    assert!(section.field(token_field::BOOK_CODE_RECORDS).is_none());
    assert!(section.field(token_field::ATTRIBUTE_RECORDS).is_none());
    // The id column and its dictionary are omitted too: ids are positional.
    assert!(section.field(token_field::TOKEN_ID_INDEX).is_none());
    assert!(section.field(token_field::TOKEN_ID_DICTIONARY).is_none());
    assert!(section.positional_ids());
}

// ------------------------------------------------------- owned token path

/// The `MarkerAttrs` of a decoded token, when it has one.
fn marker_attrs<'t, 'a>(
    tokens: &'t [usfm_onion::token::Token<'a>],
) -> &'t usfm_onion::token::MarkerAttrs<'a> {
    tokens
        .iter()
        .find_map(|token| match &token.data {
            usfm_onion::token::TokenData::Marker { attrs, .. }
            | usfm_onion::token::TokenData::Milestone { attrs, .. } => attrs.as_deref(),
            _ => None,
        })
        .expect("attribute-bearing token")
}

/// Parse, drop to owned tokens, then serialize-and-encode. Returns the derived
/// source and the container, so a case can assert on both.
fn owned_encoded(source: &str) -> (String, Vec<u8>) {
    let parsed = parse(source);
    let owned: Vec<_> = parsed.tokens.iter().map(OwnedToken::from_parsed).collect();
    let (derived, buffers) =
        encode_owned_token_section(book("GEN"), &owned).expect("owned tokens encode");
    let bytes = write_container(SNAPSHOT_ID, &[buffers.payload()]).expect("section lays out");
    (derived, bytes)
}

/// The owned loop: parse -> owned -> serialize+encode -> decode, with the decoded
/// tokens compared against a fresh parse of the derived source.
fn assert_owned_round_trips(source: &str) {
    let (derived, bytes) = owned_encoded(source);
    // Byte identity: an owned token remembers where its attribute list sat, so
    // serializing a parse-origin stream reproduces the source exactly — including
    // the malformed shapes where the list is nowhere near the marker's closer.
    assert_eq!(derived, source, "derived source must be byte-identical");

    let container = read_container(&bytes).expect("container decodes");
    let section = container.section(0).unwrap().expect("section decodes");
    let decoded = decode_token_section(&section, &derived)
        .expect("section decodes")
        .tokens;

    let fresh = parse(&derived);
    assert_eq!(
        decoded.len(),
        fresh.tokens.len(),
        "token count for {source:?}"
    );
    for (index, (decoded, expected)) in decoded.iter().zip(&fresh.tokens).enumerate() {
        assert_eq!(decoded, expected, "token {index} of {source:?}");
    }
}

#[test]
fn owned_round_trips_every_token_kind() {
    assert_owned_round_trips("\\id GEN Some Book\n\\c 1\n\\p\n\\v 1 In the beginning.\n");
    assert_owned_round_trips("\\p text // more\n\\qt-s |x=\"y\"\\*inside\\qt-e\\*\n");
    assert_owned_round_trips("\\c 1\n\\p\n\\v 1-2 bridge\n\\v 3,5 seq\n\\v 7a suffix\n");
    assert_owned_round_trips("\\zzz unknown marker\n");
    assert_owned_round_trips("\\id xyz lower case\n");
    assert_owned_round_trips("");
    assert_owned_round_trips("\\p Ἐν ἀρχῇ ἦν ὁ λόγος — 起初\n");
}

#[test]
fn owned_round_trips_tokens_whose_source_is_not_the_concatenation() {
    // The case the whole design exists for: a character marker's attribute list
    // is emitted at its closer, so no running offset over `token.source()` could
    // have produced these spans.
    let source = "\\p \\w grace|lemma=\"grace\" x-y=\"z\"\\w* and \\add more\\add*\n";
    assert_owned_round_trips(source);

    let parsed = parse(source);
    let owned: Vec<_> = parsed.tokens.iter().map(OwnedToken::from_parsed).collect();
    let (derived, spans) = tokens_to_usfm_reconstruct_spanned(&owned);
    let word = owned
        .iter()
        .position(|token| token.marker_name() == Some("w"))
        .expect("character marker");
    // Proof that the two spans are not adjacent, which is what makes the emitter
    // the only thing able to report them.
    let list = spans[word].attribute_list.expect("verbatim list");
    assert!(list.start > spans[word].token.end);
    assert_eq!(&derived[list.as_range()], "|lemma=\"grace\" x-y=\"z\"");
}

#[test]
fn owned_round_trips_milestone_attribute_lists() {
    // Milestones close on `\*` rather than a matching end marker, a different
    // drain rule in the emitter and therefore a different span path.
    assert_owned_round_trips("\\p \\zaln-s |x-strong=\"H1\"\\*aligned\\zaln-e\\*\n");
}

#[test]
fn owned_to_borrowed_reproduces_the_parse_of_the_derived_source() {
    // The intermediate is asserted directly, not only through the wire: if this
    // conversion drifted, the encoder would be fed tokens that no parse agrees
    // with.
    let source = "\\id GEN\n\\c 1\n\\p \\w a|k=\"v\"\\w* b\n\\v 1-3 text\n";
    let parsed = parse(source);
    let owned: Vec<_> = parsed.tokens.iter().map(OwnedToken::from_parsed).collect();
    let (derived, spans) = tokens_to_usfm_reconstruct_spanned(&owned);
    let borrowed =
        owned_to_borrowed(book("GEN"), &owned, &spans, &derived).expect("owned tokens convert");
    assert_eq!(borrowed, parse(&derived).tokens);
    // And, for parsed-origin input, the original parse too.
    assert_eq!(borrowed, parsed.tokens);
}

#[test]
fn owned_encoding_is_deterministic() {
    let source = "\\id GEN\n\\c 1\n\\p \\w a|k=\"v\"\\w* b\n\\v 2 c\n";
    assert_eq!(owned_encoded(source), owned_encoded(source));
}

#[test]
fn absent_attribute_source_is_distinct_from_an_empty_one() {
    // A parsed `\w` always has a verbatim list, so the present case comes from a
    // real parse; the empty-but-present case is produced by shortening the stored
    // span to zero, which is the only difference between the two on the wire.
    let source = "\\p \\w a|k=\"v\"\\w*\n";
    let bytes = encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    let decoded = decode_token_section(&section, source).unwrap().tokens;
    let list = marker_attrs(&decoded)
        .attribute_source
        .expect("present list");
    assert_eq!(list.1, "|k=\"v\"");

    let empty = decode_after(source, |bytes| {
        let at = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        bytes[at + 16..at + 20].copy_from_slice(&0u32.to_le_bytes());
    });
    assert_eq!(empty, None, "a present empty span is legal");
    let mut bytes = encoded(source);
    let at = field_payload_offset(&bytes, token_field::ATTRIBUTE_RECORDS);
    bytes[at + 16..at + 20].copy_from_slice(&0u32.to_le_bytes());
    restamp(&mut bytes);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    let decoded = decode_token_section(&section, source).unwrap().tokens;
    assert_eq!(
        marker_attrs(&decoded)
            .attribute_source
            .map(|(_, text)| text),
        Some(""),
        "an empty present span decodes to Some(\"\")"
    );

    // The sentinel offset is the absent case, and it must not read as empty.
    let mut bytes = encoded(source);
    let at = field_payload_offset(&bytes, token_field::ATTRIBUTE_RECORDS);
    bytes[at + 12..at + 16].copy_from_slice(&u32::MAX.to_le_bytes());
    bytes[at + 16..at + 20].copy_from_slice(&0u32.to_le_bytes());
    restamp(&mut bytes);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    let decoded = decode_token_section(&section, source).unwrap().tokens;
    assert_eq!(
        marker_attrs(&decoded).attribute_source,
        None,
        "the sentinel means absent"
    );
}

// ----------------------------------------------------- explicit stable ids

/// GUID-ish ids, the shape a live editor supplies: opaque, non-positional, and the
/// key identity-based reconciliation depends on.
fn guid_ids(count: usize) -> Vec<String> {
    (0..count)
        .map(|index| format!("7f3c9a{index:04x}-b21e-4d0f-9c8a-{index:012x}"))
        .collect()
}

/// Encodes with explicit ids and lays out the container.
fn encoded_with_ids(source: &str, ids: &[String]) -> Vec<u8> {
    let parsed = parse(source);
    let borrowed: Vec<&str> = ids.iter().map(String::as_str).collect();
    let buffers =
        encode_token_section_with_ids(book("GEN"), source, &parsed.tokens, Some(&borrowed))
            .expect("explicit ids encode");
    write_container(SNAPSHOT_ID, &[buffers.payload()]).expect("section lays out")
}

#[test]
fn explicit_stable_ids_round_trip_byte_for_byte() {
    let source = "\\id GEN\n\\c 1\n\\p \\w a|k=\"v\"\\w* b\n\\v 1-2 text\n";
    let ids = guid_ids(parse(source).tokens.len());
    let bytes = encoded_with_ids(source, &ids);

    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    // Opaque ids mean the flag is clear and both id fields are present — one
    // decision, not two independent ones.
    assert!(!section.positional_ids());
    assert!(section.field(token_field::TOKEN_ID_INDEX).is_some());
    assert!(section.field(token_field::TOKEN_ID_DICTIONARY).is_some());

    let decoded = decode_token_section(&section, source).expect("section decodes");
    let stable = decoded.stable_ids.expect("explicit ids are returned");
    assert_eq!(stable, ids.iter().map(String::as_str).collect::<Vec<_>>());
    // And the tokens themselves still round-trip.
    assert_eq!(decoded.tokens, parse(source).tokens);
}

#[test]
fn positional_sections_report_no_stable_ids() {
    let source = "\\id GEN\n\\c 1\n\\p text\n";
    let bytes = encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    assert!(section.positional_ids());
    let decoded = decode_token_section(&section, source).unwrap();
    assert_eq!(decoded.stable_ids, None);
}

#[test]
fn owned_tokens_with_positional_ids_omit_the_id_columns() {
    // Parse-origin owned tokens carry exactly `{book}-{index}`, so storing them
    // would be a fully redundant dictionary. The encoder proves it rather than
    // trusting a flag.
    let source = "\\id GEN\n\\c 1\n\\p text\n";
    let (_, bytes) = owned_encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    assert!(section.positional_ids());
    assert!(section.field(token_field::TOKEN_ID_INDEX).is_none());
    assert!(section.field(token_field::TOKEN_ID_DICTIONARY).is_none());
}

/// Encode with explicit ids, edit the bytes, re-stamp, decode.
fn decode_ids_after(
    source: &str,
    ids: &[String],
    edit: impl FnOnce(&mut Vec<u8>),
) -> Option<DecodeError> {
    let mut bytes = encoded_with_ids(source, ids);
    edit(&mut bytes);
    restamp(&mut bytes);
    let container = read_container(&bytes).expect("container stays well formed");
    let section = container.section(0).unwrap().expect("section stays valid");
    decode_token_section(&section, source).err()
}

#[test]
fn explicit_id_index_out_of_range_rejects() {
    let source = "\\p text\n";
    let ids = guid_ids(parse(source).tokens.len());
    let error = decode_ids_after(source, &ids, |bytes| {
        let at = field_payload_offset(bytes, token_field::TOKEN_ID_INDEX);
        bytes[at..at + 4].copy_from_slice(&9999u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn empty_explicit_id_rejects() {
    // An empty id is not identity: it cannot be distinguished from a missing one,
    // and core's `StableTokenId` refuses to hold it.
    let source = "\\p text\n";
    let ids = guid_ids(parse(source).tokens.len());
    let error = decode_ids_after(source, &ids, |bytes| {
        // Collapse the first dictionary string to zero length by moving the second
        // offset back onto the first.
        let at = field_payload_offset(bytes, token_field::TOKEN_ID_DICTIONARY);
        bytes[at + 4..at + 8].copy_from_slice(&0u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn malformed_explicit_id_dictionary_rejects() {
    let source = "\\p text\n";
    let ids = guid_ids(parse(source).tokens.len());
    // Descending offsets are a shape violation, not a UTF-8 one.
    let error = decode_ids_after(source, &ids, |bytes| {
        let at = field_payload_offset(bytes, token_field::TOKEN_ID_DICTIONARY);
        bytes[at..at + 4].copy_from_slice(&64u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));

    // A start past the data region is likewise a shape violation.
    let error = decode_ids_after(source, &ids, |bytes| {
        let at = field_payload_offset(bytes, token_field::TOKEN_ID_DICTIONARY);
        bytes[at + 4..at + 8].copy_from_slice(&u32::MAX.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn explicit_id_dictionary_splitting_a_code_point_rejects() {
    let source = "\\p text\n";
    let count = parse(source).tokens.len();
    // Ids ending in a two-byte character, so nudging the next start back by one
    // lands between that character's bytes.
    let ids: Vec<String> = (0..count).map(|index| format!("id{index}-λ")).collect();
    let error = decode_ids_after(source, &ids, |bytes| {
        let at = field_payload_offset(bytes, token_field::TOKEN_ID_DICTIONARY);
        let second = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap());
        bytes[at + 4..at + 8].copy_from_slice(&(second - 1).to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidUtf8));
}

#[test]
fn explicit_id_column_without_its_dictionary_rejects() {
    let source = "\\p text\n";
    let ids = guid_ids(parse(source).tokens.len());
    // Relabel the dictionary as an unknown optional field: it is then skipped, and
    // the index column is left naming a dictionary that is not there.
    let error = decode_ids_after(source, &ids, |bytes| {
        let at = field_entry_offset(bytes, token_field::TOKEN_ID_DICTIONARY);
        bytes[at..at + 2].copy_from_slice(&4000u16.to_le_bytes());
        // An unknown field cannot claim to be required.
        bytes[at + 3] = 0;
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn positional_flag_with_present_id_fields_rejects() {
    // The flag asserts both id fields are absent, so a section carrying either one
    // is self-contradictory. Rejected at the container layer, before any column is
    // interpreted.
    let source = "\\p text\n";
    let ids = guid_ids(parse(source).tokens.len());
    let mut bytes = encoded_with_ids(source, &ids);
    let section = section_start(&bytes);
    bytes[section + 9] = 1;
    restamp(&mut bytes);
    let container = read_container(&bytes).expect("container stays well formed");
    assert_eq!(
        container.section(0).unwrap().err(),
        Some(DecodeError::InvalidSection)
    );
}

// ------------------------------------------------- sparse record row kinds

#[test]
fn a_number_record_on_a_non_number_row_rejects() {
    // Accepted-and-ignored was the bug: the decoder only reads a sparse column for
    // rows whose kind calls for it, so a misaimed record silently vanished.
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::NUMBER_RECORDS);
        // Row 0 is the `\id` marker, not a number.
        bytes[at..at + 4].copy_from_slice(&0u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn a_book_code_record_on_a_non_book_code_row_rejects() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::BOOK_CODE_RECORDS);
        bytes[at..at + 4].copy_from_slice(&0u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn an_attribute_row_on_a_row_that_cannot_carry_one_rejects() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        // The book-code row has no attribute shape at all.
        bytes[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

// -------------------------------------------------------- packed sid fidelity

/// Fidelity of the anchor on the row whose source is `needle`.
fn fidelity_of(source: &str, needle: &str) -> Option<SidFidelity> {
    let bytes = encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    let columns = TokenColumns::from_section(&section).unwrap();
    let row = parse(source)
        .tokens
        .iter()
        .position(|token| token.source == needle)?;
    columns.sid(row as u32).map(|(_, fidelity)| fidelity)
}

#[test]
fn fidelity_comes_from_the_designator_not_the_anchor() {
    // A bare verse and a simple bridge spell their whole designator.
    assert_eq!(
        fidelity_of("\\id GEN\n\\c 1\n\\p\n\\v 1 a\n", "a"),
        Some(SidFidelity::Exact)
    );
    assert_eq!(
        fidelity_of("\\id GEN\n\\c 1\n\\p\n\\v 1-2 a\n", "a"),
        Some(SidFidelity::Exact)
    );
    // A sequence does not: its anchor is verse 1, but the designator names 1 and 3.
    // Indistinguishable from a single verse in `Sid` alone, which is why the rule
    // reads the source.
    assert_eq!(
        fidelity_of("\\id GEN\n\\c 1\n\\p\n\\v 1,3 a\n", "a"),
        Some(SidFidelity::AnchorOnly)
    );
    // A suffixed verse is identical to an unsuffixed one in both `Sid` *and*
    // `NumberRangeKind`.
    assert_eq!(
        fidelity_of("\\id GEN\n\\c 1\n\\p\n\\v 1a a\n", "a"),
        Some(SidFidelity::AnchorOnly)
    );
    // Suffix on the far side of a bridge.
    assert_eq!(
        fidelity_of("\\id GEN\n\\c 1\n\\p\n\\v 6b-11 a\n", "a"),
        Some(SidFidelity::AnchorOnly)
    );
    // A bridge wider than the seven delta bits degrades in the codec itself.
    assert_eq!(
        fidelity_of("\\id GEN\n\\c 1\n\\p\n\\v 1-200 a\n", "a"),
        Some(SidFidelity::AnchorOnly)
    );
}

#[test]
fn an_inexact_designator_marks_every_token_sharing_its_anchor() {
    // The bit lives on the dictionary entry, so it must be right for the whole run
    // of tokens the anchor covers, not just the number token.
    let source = "\\id GEN\n\\c 1\n\\p\n\\v 1a first\n\\v 2 second\n";
    let bytes = encoded(source);
    let container = read_container(&bytes).unwrap();
    let section = container.section(0).unwrap().unwrap();
    let columns = TokenColumns::from_section(&section).unwrap();
    let tokens = parse(source).tokens;
    for (row, token) in tokens.iter().enumerate() {
        let Some((_, fidelity)) = columns.sid(row as u32) else {
            continue;
        };
        let expected = match token.sid.map(|sid| sid.verse) {
            Some(1) => SidFidelity::AnchorOnly,
            _ => SidFidelity::Exact,
        };
        assert_eq!(fidelity, expected, "row {row} ({:?})", token.source);
    }
}

// ------------------------------------------- malformed payload rejection

/// Absolute offset of one field's payload inside the single-section container,
/// read out of the section's own directory.
fn field_payload_offset(bytes: &[u8], field_id: u16) -> usize {
    let section = section_start(bytes);
    let at = field_entry_offset(bytes, field_id);
    section + u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize
}

/// Absolute offset of the single section in a one-section container.
fn section_start(bytes: &[u8]) -> usize {
    u64::from_le_bytes(bytes[56..64].try_into().unwrap()) as usize
}

/// Absolute offset of one field's 16-byte directory entry.
fn field_entry_offset(bytes: &[u8], field_id: u16) -> usize {
    let section = section_start(bytes);
    let directory_count = u16::from_le_bytes(bytes[section + 20..section + 22].try_into().unwrap());
    for entry in 0..usize::from(directory_count) {
        let at = section + 64 + entry * 16;
        if u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()) == field_id {
            return at;
        }
    }
    panic!("field {field_id} is not in the directory");
}

/// Directory `count` of the string dictionary field.
fn string_dictionary_count(bytes: &[u8]) -> u32 {
    let section = u64::from_le_bytes(bytes[56..64].try_into().unwrap()) as usize;
    let directory_count = u16::from_le_bytes(bytes[section + 20..section + 22].try_into().unwrap());
    for entry in 0..usize::from(directory_count) {
        let at = section + 64 + entry * 16;
        if u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
            == token_field::STRING_DICTIONARY
        {
            return u32::from_le_bytes(bytes[at + 12..at + 16].try_into().unwrap());
        }
    }
    panic!("string dictionary is required");
}

/// Encode, edit the payload bytes, re-stamp, and decode — so each case reaches
/// the payload rule it targets instead of stopping at a checksum.
fn decode_after(source: &str, edit: impl FnOnce(&mut Vec<u8>)) -> Option<DecodeError> {
    let mut bytes = encoded(source);
    edit(&mut bytes);
    restamp(&mut bytes);
    let container = read_container(&bytes).expect("container stays well formed");
    let section = container.section(0).unwrap().expect("section stays valid");
    decode_token_section(&section, source).err()
}

const ATTRIBUTED: &str = "\\id GEN\n\\c 1\n\\p \\w word|strong=\"H1\"\\w*\n";

#[test]
fn string_dictionary_offsets_must_ascend_within_the_data() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::STRING_DICTIONARY);
        // Second string now starts before the first.
        bytes[at + 4..at + 8].copy_from_slice(&0u32.to_le_bytes());
        bytes[at..at + 4].copy_from_slice(&4u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));

    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::STRING_DICTIONARY);
        bytes[at..at + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn string_dictionary_offset_splitting_a_code_point_rejects() {
    // The last interned string is the two-byte value "λ"; nudging its start one
    // byte forward lands mid-character, which a whole-region UTF-8 check would
    // have accepted.
    let error = decode_after("\\p \\w x|k=\"λ\"\\w*\n", |bytes| {
        let at = field_payload_offset(bytes, token_field::STRING_DICTIONARY);
        let count = string_dictionary_count(bytes);
        let last = at + (count as usize - 1) * 4;
        let start = u32::from_le_bytes(bytes[last..last + 4].try_into().unwrap());
        bytes[last..last + 4].copy_from_slice(&(start + 1).to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidUtf8));
}

#[test]
fn marker_descriptor_must_name_an_existing_string() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::MARKER_DESCRIPTOR_DICTIONARY);
        bytes[at..at + 4].copy_from_slice(&9999u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn marker_descriptor_reserved_bytes_and_flags_are_checked() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::MARKER_DESCRIPTOR_DICTIONARY);
        bytes[at + 5] = 1;
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));

    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::MARKER_DESCRIPTOR_DICTIONARY);
        bytes[at + 4] = 0b10;
    });
    assert_eq!(error, Some(DecodeError::UnsupportedFlags { found: 0b10 }));
}

#[test]
fn number_record_discriminant_and_end_encoding_are_checked() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::NUMBER_RECORDS);
        bytes[at + 12] = 9;
    });
    assert_eq!(error, Some(DecodeError::InvalidDiscriminant));

    // `has_end` clear with a nonzero end would give an absent range two
    // encodings, so it is rejected rather than ignored.
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::NUMBER_RECORDS);
        bytes[at + 8..at + 12].copy_from_slice(&5u32.to_le_bytes());
        bytes[at + 13] = 0;
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn sparse_record_token_index_must_be_in_range() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::NUMBER_RECORDS);
        bytes[at..at + 4].copy_from_slice(&9999u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));

    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::BOOK_CODE_RECORDS);
        bytes[at..at + 4].copy_from_slice(&9999u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn book_code_reserved_bytes_are_checked() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::BOOK_CODE_RECORDS);
        bytes[at + 9] = 1;
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn attribute_rows_must_partition_the_entry_array() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        // A first-entry index that is not where the previous row ended leaves an
        // entry orphaned or shared.
        bytes[at + 4..at + 8].copy_from_slice(&1u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));

    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        bytes[at + 8..at + 12].copy_from_slice(&7u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn attribute_source_span_must_lie_within_the_bound_source() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let at = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        // Row entry, list source offset at +12.
        bytes[at + 12..at + 16].copy_from_slice(&9000u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn default_attribute_shorthand_requires_an_empty_key() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        let rows = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        // One row, so the entries start 24 bytes in; set is_default on an entry
        // whose key is the non-empty "strong".
        bytes[rows + 24 + 16] = 1;
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn a_number_row_without_its_record_rejects() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        // Point the only number record at a different row, leaving the real
        // Number row unexplained.
        let at = field_payload_offset(bytes, token_field::NUMBER_RECORDS);
        bytes[at..at + 4].copy_from_slice(&0u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

#[test]
fn attributes_on_a_row_that_cannot_carry_them_reject() {
    let error = decode_after(ATTRIBUTED, |bytes| {
        // Row 0 is the `\id` marker... point the attribute row at the book-code
        // row instead, which has no attribute shape at all.
        let at = field_payload_offset(bytes, token_field::ATTRIBUTE_RECORDS);
        bytes[at..at + 4].copy_from_slice(&1u32.to_le_bytes());
    });
    assert_eq!(error, Some(DecodeError::InvalidSection));
}

// ------------------------------------------------------------- corpus gate

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("crate sits two levels below the repo root")
}

fn collect_usfm(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_usfm(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("usfm") {
            out.push(path);
        }
    }
}

fn corpus_paths() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    collect_usfm(&root.join("testData"), &mut out);
    collect_usfm(&root.join("example-corpora/en_ult"), &mut out);
    collect_usfm(&root.join("example-corpora/en_ulb"), &mut out);
    out.sort();
    assert!(!out.is_empty(), "corpus must resolve from the crate dir");
    out
}

/// The Phase A gate: every corpus book survives encode → decode with every
/// public semantic token field intact.
#[test]
#[ignore = "walks the full corpus"]
fn corpus_token_sections_round_trip() {
    let mut books = 0usize;
    let mut tokens = 0usize;
    let mut wide_bridges = 0usize;
    let mut anchor_only = 0usize;

    for path in corpus_paths() {
        let source = fs::read_to_string(&path).expect("fixture reads");
        let parsed = parse(&source);

        // The packed sid spends seven bits on the bridge delta, so a bridge wider
        // than 127 is documented as degrading to an anchor-only first anchor.
        // Counted rather than assumed absent: if a corpus ever grows one, the
        // count makes the lossy path visible instead of a silent inequality.
        wide_bridges += parsed
            .tokens
            .iter()
            .filter(|token| {
                token
                    .sid
                    .is_some_and(|sid| sid.verse_end().saturating_sub(sid.verse) > 127)
            })
            .count();

        let buffers = encode_token_section(book("GEN"), &source, &parsed.tokens)
            .unwrap_or_else(|error| panic!("{} failed to encode: {error}", path.display()));
        let bytes = write_container(SNAPSHOT_ID, &[buffers.payload()])
            .unwrap_or_else(|error| panic!("{} failed to lay out: {error}", path.display()));
        let container = read_container(&bytes)
            .unwrap_or_else(|error| panic!("{} failed to read: {error}", path.display()));
        let section = container
            .section(0)
            .expect("one section")
            .unwrap_or_else(|error| panic!("{} section invalid: {error}", path.display()));
        let decoded = decode_token_section(&section, &source)
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()))
            .tokens;

        // The fidelity bit never reaches a decoded `Token`, so the round-trip
        // assertions below are blind to it. Read it off the dictionary directly and
        // count it, or an encoder that marked everything `Exact` would pass.
        let columns = TokenColumns::from_section(&section)
            .unwrap_or_else(|error| panic!("{} columns invalid: {error}", path.display()));
        anchor_only += (0..columns.len())
            .filter(|row| {
                columns
                    .sid(*row)
                    .is_some_and(|(_, fidelity)| fidelity == SidFidelity::AnchorOnly)
            })
            .count();

        assert_eq!(
            decoded.len(),
            parsed.tokens.len(),
            "{} token count",
            path.display()
        );
        for (index, (decoded, expected)) in decoded.iter().zip(&parsed.tokens).enumerate() {
            assert_eq!(decoded, expected, "{} token {index}", path.display());
        }
        books += 1;
        tokens += decoded.len();
    }

    println!("books={books} tokens={tokens} wide_bridges={wide_bridges} anchor_only={anchor_only}");
    assert_eq!(
        wide_bridges, 0,
        "a bridge wider than 127 verses exercises the documented lossy sid path"
    );
    // The corpus contains a suffixed bridge (`\v 6b-11`), so a nonzero count is the
    // proof that source-derived fidelity is actually reaching the wire. The narrow
    // tests cover each inexact class; this proves the rule fires on real text.
    assert!(
        anchor_only > 0,
        "corpus must exercise an anchor-only designator"
    );
    assert_eq!(books, CORPUS_BOOKS, "corpus book count changed");
}

/// Fixtures the reconstruct emitter does not reproduce byte for byte.
///
/// **Empty, and asserted empty.** It was nineteen: an unclosed `\fig`, a list
/// owned by a nested `\+pn` and sitting past that marker's closer, a list split by
/// a newline, and fifteen `oldformat` alignment fixtures whose `\k-s | x-tw="…"`
/// sits lines above its `\k-e\*`. All were the same cause — the emitter had only
/// one placement rule, "at the marker's closer", because the position of an
/// attribute list was dropped when a token became owned while its text survived.
///
/// Owned tokens now remember the distance from their own end to their list, and
/// the emitter honours it, so serializing a parse-origin stream reproduces the
/// file byte for byte. Kept as a list rather than deleted so a regression names
/// the fixture that broke instead of just failing a count.
const EMITTER_DIVERGENCES: [&str; 0] = [];

/// The owned-path gate: the same corpus through spanless owned tokens, where the
/// source and every span come from the reconstruct emitter rather than off a parse.
#[test]
#[ignore = "walks the full corpus"]
fn corpus_owned_token_sections_round_trip() {
    let mut books = 0usize;
    let mut tokens = 0usize;
    let mut attributed = 0usize;
    let mut diverged = Vec::new();

    for path in corpus_paths() {
        let source = fs::read_to_string(&path).expect("fixture reads");
        let owned: Vec<_> = parse(&source)
            .tokens
            .iter()
            .map(OwnedToken::from_parsed)
            .collect();
        attributed += owned
            .iter()
            .filter(|token| token.attribute_list().is_some())
            .count();

        let (derived, spans) = tokens_to_usfm_reconstruct_spanned(&owned);
        // Recording spans must not change a single byte of the emission. This is
        // what lets every divergence below be attributed to the pre-existing
        // emitter rather than to the span capture.
        assert_eq!(
            derived,
            tokens_to_usfm_reconstruct(&owned),
            "{} spanned emitter changed the bytes",
            path.display()
        );

        let borrowed = owned_to_borrowed(book("GEN"), &owned, &spans, &derived)
            .unwrap_or_else(|error| panic!("{} failed to convert: {error}", path.display()));
        let (encoded_source, buffers) = encode_owned_token_section(book("GEN"), &owned)
            .unwrap_or_else(|error| panic!("{} failed to encode: {error}", path.display()));
        assert_eq!(encoded_source, derived, "{} derived source", path.display());

        let bytes = write_container(SNAPSHOT_ID, &[buffers.payload()])
            .unwrap_or_else(|error| panic!("{} failed to lay out: {error}", path.display()));
        let container = read_container(&bytes)
            .unwrap_or_else(|error| panic!("{} failed to read: {error}", path.display()));
        let section = container
            .section(0)
            .expect("one section")
            .unwrap_or_else(|error| panic!("{} section invalid: {error}", path.display()));
        let decoded = decode_token_section(&section, &derived)
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()))
            .tokens;

        // Universal: the codec hands back exactly the tokens it was given. True
        // whether or not the serialization reproduced the file, because the
        // section describes the derived source, not the file.
        assert_eq!(
            decoded.len(),
            borrowed.len(),
            "{} token count",
            path.display()
        );
        for (index, (decoded, expected)) in decoded.iter().zip(&borrowed).enumerate() {
            assert_eq!(decoded, expected, "{} token {index}", path.display());
        }

        if derived == source {
            // Now every book: the wire agrees with a fresh parse of the source it
            // describes. The branch remains so that a regression still reports the
            // diverging fixture by name below instead of failing here.
            let fresh = parse(&derived);
            assert_eq!(
                decoded.len(),
                fresh.tokens.len(),
                "{} reparsed token count",
                path.display()
            );
            for (index, (decoded, expected)) in decoded.iter().zip(&fresh.tokens).enumerate() {
                assert_eq!(
                    decoded,
                    expected,
                    "{} reparsed token {index}",
                    path.display()
                );
            }
        } else {
            diverged.push(
                path.strip_prefix(repo_root())
                    .expect("corpus paths are under the repo root")
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
        books += 1;
        tokens += decoded.len();
    }

    diverged.sort();
    println!(
        "owned books={books} tokens={tokens} attributed_tokens={attributed} byte_exact={} diverged={}",
        books - diverged.len(),
        diverged.len()
    );
    // Serializing owned tokens reproduces every corpus file byte for byte. The
    // list, not a count, so a regression names the fixture.
    assert_eq!(
        diverged, EMITTER_DIVERGENCES,
        "a fixture stopped round-tripping byte-for-byte through the reconstruct emitter"
    );
    assert_eq!(diverged.len(), 0, "byte identity must hold for every book");
    assert_eq!(books, CORPUS_BOOKS, "corpus book count changed");
    // The attribute path is the one that cannot use a running offset, so the gate
    // is only meaningful if the corpus actually exercises it.
    assert!(
        attributed > 0,
        "corpus must contain attribute-bearing tokens"
    );
}
