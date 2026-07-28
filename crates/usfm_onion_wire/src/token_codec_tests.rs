//! Token-section round-trip tests.
//!
//! The narrow cases run on every `cargo test`. The corpus gate is `#[ignore]`d,
//! like the lint oracle, because it parses, encodes, and decodes every
//! `testData/**/*.usfm` fixture plus two full example corpora; run it with
//! `cargo test -p usfm_onion_wire -- --ignored`.

use std::fs;
use std::path::{Path, PathBuf};

use usfm_onion::parse::parse;
use usfm_onion::token::{BookId, OwnedToken};

use crate::container::{read_container, write_container};
use crate::error::DecodeError;
use crate::schema::token_field;
use crate::token_codec::{decode_token_section, encode_token_section};

const SNAPSHOT_ID: u64 = 7;

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
    let decoded = decode_token_section(&section, source).expect("token section decodes");
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

// ------------------------------------------- malformed payload rejection

/// Absolute offset of one field's payload inside the single-section container,
/// read out of the section's own directory.
fn field_payload_offset(bytes: &[u8], field_id: u16) -> usize {
    let section = u64::from_le_bytes(bytes[56..64].try_into().unwrap()) as usize;
    let directory_count = u16::from_le_bytes(bytes[section + 20..section + 22].try_into().unwrap());
    for entry in 0..usize::from(directory_count) {
        let at = section + 64 + entry * 16;
        if u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()) == field_id {
            let relative = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().unwrap()) as usize;
            return section + relative;
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
            .unwrap_or_else(|error| panic!("{} failed to decode: {error}", path.display()));

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

    println!("books={books} tokens={tokens} wide_bridges={wide_bridges}");
    assert_eq!(
        wide_bridges, 0,
        "a bridge wider than 127 verses exercises the documented lossy sid path"
    );
    assert!(books > 300, "corpus should cover both fixture sets");
}
