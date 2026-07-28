//! Container-level codec tests: canonical round trips plus a malformed-input
//! battery. Every malformed case is a hand-built buffer, derived from a valid
//! one by changing exactly the bytes under test, so a failure names one rule.
//!
//! Two properties are asserted globally rather than case by case, because they
//! are the ones a hostile buffer attacks: no input panics, and no declared count
//! or offset is acted on before the bytes it claims are proved to exist.

use crate::container::{
    Container, ElementWidth, FieldPayload, Section, SectionPayload, SectionVariant,
    read_container as read_checked_container, read_container_unchecked as read_container,
    write_container,
};
use crate::error::{DecodeError, EncodeError, LayoutRefusal};
use crate::schema::{
    CONTAINER_HEADER_LEN, SECTION_HEADER_LEN, SectionKind, finding_field, token_field,
};
use usfm_onion::token::BookId;

// Two-token payloads for the required token columns. Values are irrelevant to
// container validation; only their widths and lengths are.
static KINDS: [u8; 2] = [8, 8];
static SPAN_STARTS: [u8; 8] = [0, 0, 0, 0, 5, 0, 0, 0];
static SPAN_ENDS: [u8; 8] = [5, 0, 0, 0, 9, 0, 0, 0];
static SID_INDEXES: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
static MARKER_INDEXES: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
static STRING_DICTIONARY: [u8; 4] = [0, 0, 0, 0];
static DESCRIPTOR_DICTIONARY: [u8; 6] = [1, 0, 0, 0, 0, 0];
static FINDING_ROW: [u8; 16] = [0; 16];

fn book(code: &str) -> BookId {
    BookId::from_str(code).expect("test book code")
}

fn token_section(code: &str, source_hash: u64) -> SectionPayload<'static> {
    SectionPayload {
        variant: SectionVariant::Token {
            positional_ids: true,
        },
        book: book(code),
        source_hash,
        record_count: 2,
        fields: vec![
            FieldPayload {
                id: token_field::KIND,
                width: ElementWidth::One,
                count: 2,
                bytes: &KINDS,
            },
            FieldPayload {
                id: token_field::SPAN_START,
                width: ElementWidth::Four,
                count: 2,
                bytes: &SPAN_STARTS,
            },
            FieldPayload {
                id: token_field::SPAN_END,
                width: ElementWidth::Four,
                count: 2,
                bytes: &SPAN_ENDS,
            },
            FieldPayload {
                id: token_field::SID_INDEX,
                width: ElementWidth::Two,
                count: 2,
                bytes: &SID_INDEXES,
            },
            FieldPayload {
                id: token_field::MARKER_DESCRIPTOR_INDEX,
                width: ElementWidth::Two,
                count: 2,
                bytes: &MARKER_INDEXES,
            },
            FieldPayload {
                id: token_field::STRING_DICTIONARY,
                width: ElementWidth::Variable,
                count: 0,
                bytes: &STRING_DICTIONARY,
            },
            FieldPayload {
                id: token_field::MARKER_DESCRIPTOR_DICTIONARY,
                width: ElementWidth::Variable,
                count: 1,
                bytes: &DESCRIPTOR_DICTIONARY,
            },
        ],
    }
}

/// A token section carrying every required column at length zero.
fn empty_token_section(code: &str, source_hash: u64) -> SectionPayload<'static> {
    let mut section = token_section(code, source_hash);
    section.record_count = 0;
    for field in &mut section.fields {
        field.count = 0;
        field.bytes = &[];
    }
    section
}

fn finding_section(code: &str, source_hash: u64) -> SectionPayload<'static> {
    SectionPayload {
        variant: SectionVariant::Finding { rules_version: 7 },
        book: book(code),
        source_hash,
        record_count: 1,
        fields: vec![FieldPayload {
            id: finding_field::COMMON_ROW,
            width: ElementWidth::Sixteen,
            count: 1,
            bytes: &FINDING_ROW,
        }],
    }
}

fn write(sections: &[SectionPayload<'_>]) -> Vec<u8> {
    write_container(sections).expect("valid sections encode")
}

fn read_all(bytes: &[u8]) -> Result<(Container<'_>, Vec<Section<'_>>), DecodeError> {
    let container = read_container(bytes)?;
    let sections = container.sections().collect::<Result<Vec<_>, _>>()?;
    Ok((container, sections))
}

fn put_u8(bytes: &mut [u8], at: usize, value: u8) {
    bytes[at] = value;
}

fn put_u16(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], at: usize, value: u64) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    let mut raw = [0u8; 8];
    raw.copy_from_slice(&bytes[at..at + 8]);
    u64::from_le_bytes(raw)
}

fn toc_entry_at(index: usize) -> usize {
    CONTAINER_HEADER_LEN + index * 32
}

fn section_offset(bytes: &[u8], index: usize) -> usize {
    read_u64(bytes, toc_entry_at(index) + 8) as usize
}

fn directory_entry(bytes: &[u8], section: usize, entry: usize) -> usize {
    section_offset(bytes, section) + SECTION_HEADER_LEN + entry * 16
}

/// Zeroes every integrity checksum, which the format defines as "omitted". A
/// malformed-case buffer has to opt out of checking, or every case below would
/// stop at the checksum instead of reaching the rule it targets.
fn unchecked(mut bytes: Vec<u8>) -> Vec<u8> {
    let count = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
    put_u64(&mut bytes, 24, 0);
    for index in 0..count {
        let offset = section_offset(&bytes, index);
        put_u64(&mut bytes, offset + 40, 0);
    }
    bytes
}

// ---------------------------------------------------------------- valid input

#[test]
fn round_trip_preserves_header_toc_and_fields() {
    let sections = [
        token_section("GEN", 0x1111),
        finding_section("GEN", 0x1111),
        token_section("EXO", 0x2222),
    ];
    let bytes = write(&sections);
    let (container, decoded) = read_all(&bytes).expect("canonical container decodes");

    assert_eq!(container.header().section_count, 3);
    assert_eq!(container.header().toc_offset, CONTAINER_HEADER_LEN as u64);
    assert_ne!(container.header().checksum, 0);

    // Canonical order: token sections in corpus order, then finding sections.
    let kinds: Vec<_> = container.toc().iter().map(|entry| entry.kind).collect();
    assert_eq!(
        kinds,
        [SectionKind::Token, SectionKind::Token, SectionKind::Finding]
    );
    let books: Vec<_> = container
        .toc()
        .iter()
        .map(|entry| entry.book.as_str().to_string())
        .collect();
    assert_eq!(books, ["GEN", "EXO", "GEN"]);

    for entry in container.toc() {
        assert_eq!(entry.offset % 16, 0);
    }

    let genesis = &decoded[0];
    assert_eq!(genesis.header.book, book("GEN"));
    assert_eq!(genesis.header.record_count, 2);
    assert_eq!(genesis.header.rules_version, 0);
    assert!(genesis.positional_ids());
    assert_eq!(genesis.fields().len(), 7);
    assert_eq!(genesis.field(token_field::KIND).unwrap().bytes, &KINDS);
    assert_eq!(
        genesis.field(token_field::SPAN_START).unwrap().bytes,
        &SPAN_STARTS
    );
    // Every column this section carries is schema-required, and the directory
    // says so — the writer stamps required-ness from the schema, not the caller.
    assert!(genesis.fields().iter().all(|field| field.required));

    let findings = &decoded[2];
    assert_eq!(findings.header.kind, SectionKind::Finding);
    assert_eq!(findings.header.rules_version, 7);
    assert_eq!(
        findings.field(finding_field::COMMON_ROW).unwrap().bytes,
        &FINDING_ROW
    );
}

#[test]
fn zero_section_container_is_valid() {
    let bytes = write(&[]);
    let (container, sections) = read_all(&bytes).expect("empty container decodes");
    assert_eq!(container.header().section_count, 0);
    assert!(container.toc().is_empty());
    assert!(sections.is_empty());
}

#[test]
fn zero_record_section_is_valid() {
    let bytes = write(&[empty_token_section("GEN", 1)]);
    let (_, sections) = read_all(&bytes).expect("empty section decodes");
    assert_eq!(sections[0].header.record_count, 0);
    assert_eq!(sections[0].fields().len(), 7);
    assert!(
        sections[0]
            .fields()
            .iter()
            .all(|field| field.bytes.is_empty())
    );
}

#[test]
fn encoding_is_deterministic_and_order_independent_within_a_kind() {
    let interleaved = [
        token_section("GEN", 1),
        finding_section("GEN", 1),
        token_section("EXO", 2),
        finding_section("EXO", 2),
    ];
    let grouped = [
        token_section("GEN", 1),
        token_section("EXO", 2),
        finding_section("GEN", 1),
        finding_section("EXO", 2),
    ];
    // Canonical layout is a function of the logical section set, so the caller's
    // interleaving cannot change the bytes.
    assert_eq!(write(&interleaved), write(&grouped));
    assert_eq!(write(&interleaved), write(&interleaved));
}

#[test]
fn unknown_optional_field_is_skipped() {
    let mut section = token_section("GEN", 1);
    section.fields.push(FieldPayload {
        id: 4000,
        width: ElementWidth::Four,
        count: 1,
        bytes: &SPAN_STARTS[..4],
    });
    let bytes = write(&[section]);
    let (_, sections) = read_all(&bytes).expect("unknown optional field is skipped, not fatal");
    assert!(sections[0].field(4000).is_none());
    assert_eq!(sections[0].fields().len(), 7);
}

#[test]
fn unknown_optional_field_is_structurally_validated_before_skipping() {
    let mut section = token_section("GEN", 1);
    section.fields.push(FieldPayload {
        id: 4000,
        width: ElementWidth::Four,
        count: 1,
        bytes: &SPAN_STARTS[..4],
    });
    let mut bytes = unchecked(write(&[section]));
    let section_len = read_u64(&bytes, section_offset(&bytes, 0) + 32) as u32;
    let unknown = directory_entry(&bytes, 0, 7);
    put_u32(&mut bytes, unknown + 4, section_len);
    assert_eq!(read_all(&bytes), Err(DecodeError::Truncated));
}

// ------------------------------------------------------- no panic, ever

#[test]
fn every_truncation_rejects_without_panicking() {
    let bytes = write(&[
        token_section("GEN", 1),
        finding_section("GEN", 1),
        token_section("EXO", 2),
    ]);
    for len in 0..bytes.len() {
        assert!(
            read_all(&bytes[..len]).is_err(),
            "prefix of {len} bytes must not decode"
        );
    }
    assert!(read_all(&bytes).is_ok());
}

#[test]
fn every_single_byte_corruption_is_survivable() {
    // Run over a checksum-omitted buffer so the corruption reaches the
    // structural validators instead of stopping at the hash; the only assertion
    // is that nothing panics and nothing decodes to a view built from
    // unvalidated bytes.
    let base = unchecked(write(&[token_section("GEN", 1), finding_section("GEN", 1)]));
    for index in 0..base.len() {
        for value in [0x00u8, 0x01, 0x7f, 0xff] {
            let mut bytes = base.clone();
            bytes[index] = value;
            if let Ok((container, sections)) = read_all(&bytes) {
                for (entry, section) in container.toc().iter().zip(&sections) {
                    assert_eq!(entry.book, section.header.book);
                    for field in section.fields() {
                        assert!(field.bytes.len() <= entry.byte_len as usize);
                    }
                }
            }
        }
    }
}

#[test]
fn hostile_section_count_does_not_allocate_before_bounds_checking() {
    // 32 bytes of header claiming 4 billion TOC entries. The declared count is
    // 137 GB of TOC; rejecting it must cost a bounds check.
    let mut bytes = unchecked(write(&[]));
    put_u32(&mut bytes, 12, u32::MAX);
    assert_eq!(read_container(&bytes), Err(DecodeError::Truncated));
}

#[test]
fn toc_offset_arithmetic_overflow_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u64(&mut bytes, 16, u64::MAX - 15);
    put_u32(&mut bytes, 12, 1);
    assert_eq!(read_container(&bytes), Err(DecodeError::OffsetOverflow));
}

#[test]
fn section_extent_arithmetic_overflow_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u64(&mut bytes, toc_entry_at(0) + 16, u64::MAX);
    assert_eq!(read_container(&bytes), Err(DecodeError::OffsetOverflow));
}

// ------------------------------------------------------ container header rules

#[test]
fn container_header_shorter_than_the_fixed_length_truncates() {
    let bytes = write(&[]);
    assert_eq!(
        read_container(&bytes[..CONTAINER_HEADER_LEN - 1]),
        Err(DecodeError::Truncated)
    );
    assert_eq!(read_container(&[]), Err(DecodeError::Truncated));
}

#[test]
fn container_magic_and_version_gate_the_buffer() {
    let mut bytes = unchecked(write(&[]));
    put_u8(&mut bytes, 0, b'U');
    assert_eq!(read_container(&bytes), Err(DecodeError::BadMagic));

    // A big-endian producer's magic is `nosu`, so wrong byte order rejects here
    // rather than through any endianness heuristic.
    let mut bytes = unchecked(write(&[]));
    bytes[..4].copy_from_slice(b"nosu");
    assert_eq!(read_container(&bytes), Err(DecodeError::BadMagic));

    let mut bytes = unchecked(write(&[]));
    put_u16(&mut bytes, 4, 2);
    assert_eq!(
        read_container(&bytes),
        Err(DecodeError::UnsupportedVersion { found: 2 })
    );
}

#[test]
fn container_header_length_must_match_the_declared_version() {
    let mut bytes = unchecked(write(&[]));
    put_u16(&mut bytes, 6, 40);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn unknown_container_flag_rejects() {
    let mut bytes = unchecked(write(&[]));
    put_u32(&mut bytes, 8, 1);
    assert_eq!(
        read_container(&bytes),
        Err(DecodeError::UnsupportedFlags { found: 1 })
    );
}

#[test]
fn container_checksum_mismatch_rejects() {
    let bytes = write(&[token_section("GEN", 1)]);
    let mut corrupted = bytes.clone();
    let payload = section_offset(&bytes, 0) + SECTION_HEADER_LEN + 16 * 7;
    corrupted[payload] ^= 0xff;
    assert_eq!(
        read_container(&corrupted),
        Err(DecodeError::ChecksumMismatch)
    );

    let mut wrong_stamp = bytes;
    put_u64(&mut wrong_stamp, 24, 1);
    assert_eq!(
        read_container(&wrong_stamp),
        Err(DecodeError::ChecksumMismatch)
    );
}

#[test]
fn persistent_reader_rejects_omitted_checksums() {
    let bytes = unchecked(write(&[token_section("GEN", 1)]));
    assert_eq!(
        read_checked_container(&bytes),
        Err(DecodeError::ChecksumMismatch)
    );

    let mut bytes = write(&[token_section("GEN", 1)]);
    let section = section_offset(&bytes, 0);
    put_u64(&mut bytes, section + 40, 0);
    let container_checksum = crate::primitives::integrity_checksum(&bytes, 24);
    put_u64(&mut bytes, 24, container_checksum);
    let container = read_checked_container(&bytes).expect("outer checksum is valid");
    assert_eq!(
        container.section(0).expect("section exists"),
        Err(DecodeError::ChecksumMismatch)
    );
}

#[test]
fn misplaced_toc_rejects() {
    let mut bytes = unchecked(write(&[]));
    put_u64(&mut bytes, 16, 24);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));

    let mut bytes = unchecked(write(&[]));
    put_u64(&mut bytes, 16, 40);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

// ----------------------------------------------------------------- TOC rules

#[test]
fn unknown_section_kind_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u8(&mut bytes, toc_entry_at(0), 2);
    assert_eq!(
        read_container(&bytes),
        Err(DecodeError::InvalidDiscriminant)
    );
}

#[test]
fn malformed_book_code_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u8(&mut bytes, toc_entry_at(0) + 1, b'-');
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));

    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u8(&mut bytes, toc_entry_at(0) + 1, 0x80);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidUtf8));
}

#[test]
fn unknown_section_version_and_toc_flags_reject() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u16(&mut bytes, toc_entry_at(0) + 4, 2);
    assert_eq!(
        read_container(&bytes),
        Err(DecodeError::UnsupportedVersion { found: 2 })
    );

    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u16(&mut bytes, toc_entry_at(0) + 6, 0x8000);
    assert_eq!(
        read_container(&bytes),
        Err(DecodeError::UnsupportedFlags { found: 0x8000 })
    );
}

#[test]
fn misaligned_or_header_overlapping_section_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&bytes, 0) as u64;
    put_u64(&mut bytes, toc_entry_at(0) + 8, offset + 8);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));

    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u64(&mut bytes, toc_entry_at(0) + 8, 16);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

#[test]
fn section_shorter_than_its_own_header_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u64(&mut bytes, toc_entry_at(0) + 16, 47);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

#[test]
fn section_overlapping_the_toc_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    // Offset 32 is 16-aligned and past the header, but it is where the TOC
    // itself lives, so the section would alias the entry describing it.
    put_u64(&mut bytes, toc_entry_at(0) + 8, CONTAINER_HEADER_LEN as u64);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

#[test]
fn overlapping_sections_reject() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1), token_section("EXO", 2)]));
    let first = read_u64(&bytes, toc_entry_at(0) + 8);
    put_u64(&mut bytes, toc_entry_at(1) + 8, first);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

#[test]
fn duplicate_book_sections_reject() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1), token_section("EXO", 1)]));
    bytes[toc_entry_at(1) + 1..toc_entry_at(1) + 4].copy_from_slice(b"GEN");
    // The section header restates the book, so fix it too: the target here is the
    // duplicate TOC key, not the header/TOC disagreement.
    let second = section_offset(&bytes, 1);
    bytes[second + 10..second + 13].copy_from_slice(b"GEN");
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

#[test]
fn finding_section_without_a_matching_token_section_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1), finding_section("GEN", 1)]));
    put_u64(&mut bytes, toc_entry_at(1) + 24, 999);
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));

    let mut bytes = unchecked(write(&[token_section("GEN", 1), finding_section("GEN", 1)]));
    bytes[toc_entry_at(1) + 1..toc_entry_at(1) + 4].copy_from_slice(b"EXO");
    assert_eq!(read_container(&bytes), Err(DecodeError::InvalidToc));
}

// -------------------------------------------------------- section header rules

#[test]
fn section_magic_and_version_gate_the_section() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&bytes, 0);
    put_u8(&mut bytes, offset, b'U');
    assert_eq!(read_all(&bytes), Err(DecodeError::BadMagic));

    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u16(&mut bytes, offset + 4, 3);
    assert_eq!(
        read_all(&bytes),
        Err(DecodeError::UnsupportedVersion { found: 3 })
    );
}

#[test]
fn section_header_must_agree_with_its_toc_entry() {
    let base = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&base, 0);

    // kind
    let mut bytes = base.clone();
    put_u8(&mut bytes, offset + 8, SectionKind::Finding.as_u8());
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));

    // book
    let mut bytes = base.clone();
    bytes[offset + 10..offset + 13].copy_from_slice(b"EXO");
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));

    // source hash
    let mut bytes = base.clone();
    put_u64(&mut bytes, offset + 24, 99);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));

    // section length
    let mut bytes = base.clone();
    let declared = read_u64(&bytes, offset + 32);
    put_u64(&mut bytes, offset + 32, declared - 16);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn section_header_reserved_bytes_and_entry_size_are_fixed() {
    let base = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&base, 0);

    let mut bytes = base.clone();
    put_u8(&mut bytes, offset + 13, 1);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));

    let mut bytes = base.clone();
    put_u16(&mut bytes, offset + 22, 20);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn token_section_may_not_declare_a_rules_version() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&bytes, 0);
    put_u16(&mut bytes, offset + 6, 4);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn unknown_section_flag_rejects_per_kind() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&bytes, 0);
    put_u8(&mut bytes, offset + 9, 0b10);
    assert_eq!(
        read_all(&bytes),
        Err(DecodeError::UnsupportedFlags { found: 0b10 })
    );

    // `positional_ids` is a token-only concept; set on a finding section it is
    // an undefined bit for that kind, not a redundant one.
    let mut bytes = unchecked(write(&[token_section("GEN", 1), finding_section("GEN", 1)]));
    let finding = section_offset(&bytes, 1);
    put_u8(&mut bytes, finding + 9, 1);
    assert_eq!(
        read_all(&bytes),
        Err(DecodeError::UnsupportedFlags { found: 1 })
    );
}

#[test]
fn directory_larger_than_its_section_truncates() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&bytes, 0);
    put_u16(&mut bytes, offset + 20, u16::MAX);
    assert_eq!(read_all(&bytes), Err(DecodeError::Truncated));
}

#[test]
fn section_checksum_mismatch_rejects() {
    let mut bytes = write(&[token_section("GEN", 1)]);
    let offset = section_offset(&bytes, 0);
    put_u64(&mut bytes, offset + 40, 1);
    // Omit the container checksum so the section's own checksum is the one that
    // fails, rather than the outer hash catching the edit first.
    put_u64(&mut bytes, 24, 0);
    assert_eq!(read_all(&bytes), Err(DecodeError::ChecksumMismatch));
}

// ------------------------------------------------------- field directory rules

#[test]
fn unknown_element_width_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let entry = directory_entry(&bytes, 0, 0);
    put_u8(&mut bytes, entry + 2, 3);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn unknown_field_entry_flag_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let entry = directory_entry(&bytes, 0, 0);
    put_u8(&mut bytes, entry + 3, 0b11);
    assert_eq!(
        read_all(&bytes),
        Err(DecodeError::UnsupportedFlags { found: 0b11 })
    );
}

#[test]
fn known_field_with_the_wrong_required_bit_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let entry = directory_entry(&bytes, 0, 0);
    put_u8(&mut bytes, entry + 3, 0);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn known_field_with_the_wrong_width_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let entry = directory_entry(&bytes, 0, 0);
    put_u8(&mut bytes, entry + 2, 2);
    put_u32(&mut bytes, entry + 8, 4);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn unknown_required_field_rejects() {
    let mut section = token_section("GEN", 1);
    section.fields.push(FieldPayload {
        id: 4000,
        width: ElementWidth::Four,
        count: 1,
        bytes: &SPAN_STARTS[..4],
    });
    let mut bytes = unchecked(write(&[section]));
    // The unknown field sorts last; mark it required and the decoder can no
    // longer honestly claim it read the section.
    let entry = directory_entry(&bytes, 0, 7);
    put_u8(&mut bytes, entry + 3, 1);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn duplicate_field_id_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let first = directory_entry(&bytes, 0, 0);
    let second = directory_entry(&bytes, 0, 1);
    let width = bytes[first + 2];
    put_u16(&mut bytes, second, token_field::KIND);
    put_u8(&mut bytes, second + 2, width);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn missing_required_field_rejects() {
    let mut section = token_section("GEN", 1);
    section.fields.retain(|field| field.id != token_field::KIND);
    // The writer refuses this outright, so the buffer has to be built from a
    // complete section and then have the entry removed from the directory.
    assert_eq!(
        write_container(&[section]),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::MissingRequiredField {
                field_id: token_field::KIND
            },
        })
    );

    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let offset = section_offset(&bytes, 0);
    // Drop the first directory entry by shortening the directory and shifting
    // the rest down; the payload region is untouched, which is legal because
    // offsets are absolute within the section.
    let entry = directory_entry(&bytes, 0, 0);
    let end = directory_entry(&bytes, 0, 7);
    bytes.copy_within(entry + 16..end, entry);
    put_u16(&mut bytes, offset + 20, 6);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn field_payload_inside_the_directory_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let entry = directory_entry(&bytes, 0, 0);
    put_u32(&mut bytes, entry + 4, SECTION_HEADER_LEN as u32);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn misaligned_field_payload_rejects() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    // Entry 1 is `span_start`, a four-byte column.
    let entry = directory_entry(&bytes, 0, 1);
    let offset = u32::from_le_bytes([
        bytes[entry + 4],
        bytes[entry + 5],
        bytes[entry + 6],
        bytes[entry + 7],
    ]);
    put_u32(&mut bytes, entry + 4, offset + 2);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn field_count_and_width_must_explain_the_byte_length() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let entry = directory_entry(&bytes, 0, 1);
    put_u32(&mut bytes, entry + 12, 3);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));

    // A count at the `u32` ceiling is caught by the same rule rather than by
    // wrapping into a plausible byte length.
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    put_u32(&mut bytes, entry + 12, u32::MAX);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn field_payload_past_the_section_end_truncates() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let section_len = read_u64(&bytes, section_offset(&bytes, 0) + 32) as u32;
    let entry = directory_entry(&bytes, 0, 5);
    put_u32(&mut bytes, entry + 4, section_len);
    put_u32(&mut bytes, entry + 8, 8);
    assert_eq!(read_all(&bytes), Err(DecodeError::Truncated));
}

#[test]
fn overlapping_field_payloads_reject() {
    let mut bytes = unchecked(write(&[token_section("GEN", 1)]));
    let first = directory_entry(&bytes, 0, 1);
    let second = directory_entry(&bytes, 0, 2);
    let offset = u32::from_le_bytes([
        bytes[first + 4],
        bytes[first + 5],
        bytes[first + 6],
        bytes[first + 7],
    ]);
    put_u32(&mut bytes, second + 4, offset);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

#[test]
fn positional_ids_flag_contradicting_the_id_columns_rejects() {
    let mut section = token_section("GEN", 1);
    section.variant = SectionVariant::Token {
        positional_ids: false,
    };
    section.fields.push(FieldPayload {
        id: token_field::TOKEN_ID_INDEX,
        width: ElementWidth::Four,
        count: 2,
        bytes: &SPAN_STARTS,
    });
    let mut bytes = unchecked(write(&[section]));
    let offset = section_offset(&bytes, 0);
    put_u8(&mut bytes, offset + 9, 1);
    assert_eq!(read_all(&bytes), Err(DecodeError::InvalidSection));
}

// ------------------------------------------------------------- writer refusals

#[test]
fn writer_refuses_duplicate_sections() {
    let sections = [token_section("GEN", 1), token_section("GEN", 2)];
    assert_eq!(
        write_container(&sections),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::DuplicateSection {
                kind: SectionKind::Token
            },
        })
    );
}

#[test]
fn writer_refuses_an_unpaired_finding_section() {
    assert_eq!(
        write_container(&[finding_section("GEN", 1)]),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::OrphanFindingSection,
        })
    );
    // Same book, different source hash: the pairing key is both, so this is not
    // a match either.
    assert_eq!(
        write_container(&[token_section("GEN", 1), finding_section("GEN", 2)]),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::OrphanFindingSection,
        })
    );
}

#[test]
fn writer_refuses_a_field_whose_extent_disagrees_with_its_count() {
    let mut section = token_section("GEN", 1);
    section.fields[1].count = 3;
    assert_eq!(
        write_container(&[section]),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::FieldExtentMismatch {
                field_id: token_field::SPAN_START
            },
        })
    );
}

#[test]
fn writer_refuses_duplicate_fields() {
    let mut section = token_section("GEN", 1);
    let duplicate = section.fields[0];
    section.fields.push(duplicate);
    assert_eq!(
        write_container(&[section]),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::DuplicateField {
                field_id: token_field::KIND
            },
        })
    );
}

#[test]
fn writer_refuses_id_columns_alongside_positional_ids() {
    let mut section = token_section("GEN", 1);
    section.fields.push(FieldPayload {
        id: token_field::TOKEN_ID_INDEX,
        width: ElementWidth::Four,
        count: 2,
        bytes: &SPAN_STARTS,
    });
    assert_eq!(
        write_container(&[section]),
        Err(EncodeError::InvalidSectionLayout {
            book: book("GEN"),
            reason: LayoutRefusal::PositionalIdConflict {
                field_id: token_field::TOKEN_ID_INDEX
            },
        })
    );
}
