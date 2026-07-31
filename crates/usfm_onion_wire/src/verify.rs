//! Whole-container verification: the one trust boundary a non-Rust
//! materializer stands on.
//!
//! [`verify_book`] runs every check the format has for one book — container and
//! section structure, both integrity checksums, the exact source length and
//! content hash, the marker-catalog stamp, every discriminant, index range and
//! reserved byte — and then hands back two things a decoder cannot derive from
//! the certified bytes alone:
//!
//! - the book's findings, materialized here because `LintIssue.message` is
//!   rendered only by core's own renderer, and
//! - its marker descriptors with `metadata`/`structural` already resolved from
//!   the registry the stamp just licensed.
//!
//! Everything else about a token is in the bytes, so a caller holding the
//! receipt and the certified buffer can materialize tokens without a registry,
//! a catalog, or a hash implementation. That is the whole point of the split:
//! this function is the only thing that has to be trusted.

use usfm_onion::lint::LintIssue;
use usfm_onion::marker_defs::structural_marker_info;
use usfm_onion::token::{BookId, marker_metadata};

use crate::container::{Section, read_container};
use crate::dto::{PackedBookReceipt, PackedMarkerDescriptor};
use crate::error::DecodeError;
use crate::finding_codec::decode_finding_section;
use crate::schema::{SectionKind, token_field};
use crate::token_codec::decode_token_section;
use crate::token_payload::{MarkerDescriptors, StringDictionary};

/// One verified book: its attestation and its findings.
///
/// Tokens are deliberately absent. Materializing them is the caller's job — in
/// the JS engine for the npm path, or via the crate's own decoder for a native
/// host — and doing it here would put the boundary cost this split exists to
/// avoid back in front of every caller.
#[derive(Debug, Clone)]
pub struct VerifiedBook {
    pub receipt: PackedBookReceipt,
    pub findings: Vec<LintIssue>,
    /// The stamps the publisher recorded for these findings, when it recorded
    /// any. `None` means they may be read but never adopted as a warm lint
    /// cache: nothing in the bytes says what produced them.
    pub lint_stamps: Option<crate::schema::LintStamps>,
}

/// Verifies one packed container against the exact source it was bound to.
///
/// The container must carry exactly one token section and at most one finding
/// section, naming the same book. A finding section is optional because a
/// producer may pack tokens alone (no lint pass has run, or its findings are
/// not wanted); two sections of either kind, or a token/finding pair that
/// disagrees about its book, is refused rather than silently paired.
pub fn verify_book(packed: &[u8], source: &str) -> Result<VerifiedBook, DecodeError> {
    let container = read_container(packed)?;
    let mut token_index = None;
    let mut finding_index = None;
    for (index, entry) in container.toc().iter().enumerate() {
        let slot = match entry.kind {
            SectionKind::Token => &mut token_index,
            SectionKind::Finding => &mut finding_index,
        };
        if slot.is_some() {
            return Err(DecodeError::InvalidToc);
        }
        *slot = Some(index);
    }
    let token_index = token_index.ok_or(DecodeError::InvalidToc)?;
    let toc = container.toc();
    if let Some(finding_index) = finding_index
        && toc[finding_index].book != toc[token_index].book
    {
        return Err(DecodeError::InvalidToc);
    }
    let book = toc[token_index].book;

    let token_section = container
        .section(token_index)
        .ok_or(DecodeError::InvalidToc)??;
    // Binds the source (length, then content hash) and the catalog stamp before
    // anything below reads a span or a marker ordinal.
    let decoded_tokens = decode_token_section(&token_section, source)?;

    let (findings, lint_stamps) = match finding_index {
        Some(index) => {
            let finding_section = container.section(index).ok_or(DecodeError::InvalidToc)??;
            decode_finding_section(&finding_section, book, source, &decoded_tokens)?
        }
        None => (Vec::new(), None),
    };

    verified_book(
        &token_section,
        book,
        container.header().snapshot_id,
        findings,
        lint_stamps,
    )
}

/// Builds one book's receipt from its already-validated token section.
///
/// Shared with the corpus verifier so a per-book receipt means exactly the same
/// thing however many books were in the container it came out of.
pub(crate) fn verified_book(
    token_section: &Section<'_>,
    book: BookId,
    snapshot_id: u64,
    findings: Vec<LintIssue>,
    lint_stamps: Option<crate::schema::LintStamps>,
) -> Result<VerifiedBook, DecodeError> {
    let descriptors = resolve_descriptors(token_section)?;
    let header = &token_section.header;
    let receipt = PackedBookReceipt {
        book: book.as_str().to_string(),
        // The encoder refuses a source it cannot describe in `u32` spans, so a
        // section claiming a wider one contradicts its own span columns.
        source_len: u32::try_from(header.source_len).map_err(|_| DecodeError::OffsetOverflow)?,
        token_count: header.record_count,
        finding_count: u32::try_from(findings.len()).map_err(|_| DecodeError::OffsetOverflow)?,
        positional_ids: token_section.positional_ids(),
        source_hash: format!("{:016x}", header.source_hash),
        catalog_stamp: format!("{:016x}", header.catalog_stamp),
        snapshot_id: format!("{:016x}", snapshot_id),
        descriptors,
    };
    Ok(VerifiedBook {
        receipt,
        findings,
        lint_stamps,
    })
}

/// One materialized token section: the boundary DTOs, plus the opaque stable ids
/// when the section carried them.
///
/// `stable_ids` is separate from the tokens for the same reason
/// [`crate::token_codec::DecodedTokens`] separates them: core's `TokenId` is a
/// structured positional label and cannot hold an opaque caller id, so
/// `Token::id` carries the positional label in both modes and the opaque ids
/// travel alongside.
#[derive(Debug, Clone)]
pub struct MaterializedTokens {
    pub tokens: Vec<crate::dto::Token>,
    pub stable_ids: Option<Vec<String>>,
}

/// Materializes one book's tokens as boundary DTOs.
///
/// This is the Rust half of the cross-language equivalence gate, and the token
/// materializer a native host uses (the npm path materializes tokens in JS
/// instead, which is the whole point of [`verify_book`]). It repeats the same
/// verification: a caller holding only bytes is never asked to have verified them
/// first.
pub fn materialize_tokens(packed: &[u8], source: &str) -> Result<MaterializedTokens, DecodeError> {
    let container = read_container(packed)?;
    let mut token_index = None;
    for (index, entry) in container.toc().iter().enumerate() {
        if entry.kind == SectionKind::Token {
            if token_index.is_some() {
                return Err(DecodeError::InvalidToc);
            }
            token_index = Some(index);
        }
    }
    let section = container
        .section(token_index.ok_or(DecodeError::InvalidToc)?)
        .ok_or(DecodeError::InvalidToc)??;
    let decoded = decode_token_section(&section, source)?;
    Ok(MaterializedTokens {
        tokens: decoded.tokens.iter().map(crate::dto::Token::from).collect(),
        stable_ids: decoded
            .stable_ids
            .map(|ids| ids.into_iter().map(str::to_owned).collect()),
    })
}

/// Every marker form the section references, in descriptor-ordinal order, with
/// the two name-derived token fields resolved from the registry.
///
/// Resolved here rather than per token: the descriptor dictionary is already
/// deduplicated by `(name, nested)`, so this is one registry lookup per distinct
/// marker form in the book (25-31 for a whole scripture book) instead of one per
/// token.
fn resolve_descriptors(section: &Section<'_>) -> Result<Vec<PackedMarkerDescriptor>, DecodeError> {
    let strings = StringDictionary::from_field(
        section
            .field(token_field::STRING_DICTIONARY)
            .ok_or(DecodeError::InvalidSection)?,
    )?;
    let descriptors = MarkerDescriptors::from_field(
        section
            .field(token_field::MARKER_DESCRIPTOR_DICTIONARY)
            .ok_or(DecodeError::InvalidSection)?,
        strings,
    )?;
    let mut out = Vec::with_capacity(usize::from(descriptors.len()));
    for index in 0..descriptors.len() {
        let (name, nested) = descriptors.get(index).ok_or(DecodeError::InvalidSection)?;
        let metadata = marker_metadata(name);
        let structural = structural_marker_info(name, metadata.kind);
        out.push(PackedMarkerDescriptor {
            name: name.to_string(),
            nested,
            marker_metadata: metadata.into(),
            structural: structural.into(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::write_container;
    use crate::finding_codec::{decode_book, encode_book, encode_findings};
    use crate::token_codec::encode_token_section;
    use usfm_onion::lint::{LintOptions, LintScope, lint_tokens};
    use usfm_onion::parse::parse;
    use usfm_onion::token::BookId;

    const SOURCE: &str = "\\id gen Genesis\n\\c 1\n\\p\n\\v 1 In the beginning\n\\v 1 duplicate\n";

    fn book(code: &str) -> BookId {
        BookId::from_str(code).unwrap()
    }

    fn encoded_pair() -> Vec<u8> {
        let parsed = parse(SOURCE);
        let issues = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book)).issues;
        encode_book(book("GEN"), SOURCE, &parsed.tokens, &issues).unwrap()
    }

    /// The receipt describes the section, and the findings are the *same* values
    /// `decode_book` produces — the one guarantee that lets findings have a
    /// single decoder while tokens have two.
    #[test]
    fn verify_matches_decode_book_and_describes_the_section() {
        let bytes = encoded_pair();
        let verified = verify_book(&bytes, SOURCE).unwrap();
        assert_eq!(verified.findings, decode_book(&bytes, SOURCE).unwrap());

        let parsed = parse(SOURCE);
        assert_eq!(verified.receipt.book, "GEN");
        assert_eq!(verified.receipt.source_len, SOURCE.len() as u32);
        assert_eq!(verified.receipt.token_count, parsed.tokens.len() as u32);
        assert_eq!(
            verified.receipt.finding_count,
            verified.findings.len() as u32
        );
        assert!(verified.receipt.positional_ids);
        assert_eq!(verified.receipt.source_hash.len(), 16);
        assert_eq!(
            verified.receipt.catalog_stamp,
            format!("{:016x}", crate::catalog::catalog_stamp())
        );
        // `\id`, `\c`, `\p`, `\v` — one descriptor per distinct marker form.
        let names: Vec<&str> = verified
            .receipt
            .descriptors
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert_eq!(names, ["id", "c", "p", "v"]);
        // Resolved from the registry, which is what a JS decoder cannot do.
        let verse = &verified.receipt.descriptors[3];
        assert!(!verse.nested);
        assert!(verse.marker_metadata.kind.is_some());
    }

    /// A token-only container is legal input: findings are optional, so the
    /// receipt reports zero rather than the whole verification failing.
    #[test]
    fn a_token_only_container_verifies_with_no_findings() {
        let parsed = parse(SOURCE);
        let buffers = encode_token_section(book("GEN"), SOURCE, &parsed.tokens).unwrap();
        let bytes = write_container(0, &[buffers.payload()]).unwrap();
        let verified = verify_book(&bytes, SOURCE).unwrap();
        assert!(verified.findings.is_empty());
        assert_eq!(verified.receipt.finding_count, 0);
        assert_eq!(verified.receipt.token_count, parsed.tokens.len() as u32);
    }

    #[test]
    fn a_container_with_no_token_section_is_refused() {
        let parsed = parse(SOURCE);
        let issues = lint_tokens(&parsed.tokens, LintOptions::scoped(LintScope::Book)).issues;
        // A lone finding section cannot even be written (it has no token section
        // to pair with), so the refusal is proved at the writer.
        let findings = encode_findings(book("GEN"), SOURCE, &parsed.tokens, &issues).unwrap();
        assert!(write_container(0, &[findings.payload()]).is_err());
    }

    /// Two books in one container is legal *corpus* wire, but `verify_book`
    /// names one book, so it must refuse rather than pair sections across books.
    #[test]
    fn a_two_book_container_is_refused() {
        let source_b = "\\id exo Exodus\n\\c 1\n\\p\n\\v 1 b\n";
        let parsed_a = parse(SOURCE);
        let parsed_b = parse(source_b);
        let token_a = encode_token_section(book("GEN"), SOURCE, &parsed_a.tokens).unwrap();
        let token_b = encode_token_section(book("EXO"), source_b, &parsed_b.tokens).unwrap();
        let bytes = write_container(0, &[token_a.payload(), token_b.payload()]).unwrap();
        assert_eq!(
            verify_book(&bytes, SOURCE).unwrap_err(),
            DecodeError::InvalidToc
        );
    }

    #[test]
    fn a_mismatched_source_is_refused_before_any_span_is_read() {
        let bytes = encoded_pair();
        let mut wrong = SOURCE.to_string();
        wrong.push('x');
        assert_eq!(
            verify_book(&bytes, &wrong).unwrap_err(),
            DecodeError::SourceLengthMismatch
        );
        // Same length, different bytes: only the content hash catches this.
        let same_length = SOURCE.replacen("beginning", "beginninG", 1);
        assert_eq!(same_length.len(), SOURCE.len());
        assert_eq!(
            verify_book(&bytes, &same_length).unwrap_err(),
            DecodeError::SourceHashMismatch
        );
    }
}
