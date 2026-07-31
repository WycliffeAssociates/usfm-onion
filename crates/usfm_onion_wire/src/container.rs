//! Container header, table of contents, section header, and field directory —
//! the trust boundary of the wire format.
//!
//! Layout constants live in [`crate::schema`]. Nothing here trusts a declared count, offset,
//! length, width, flag, or discriminant: each is validated against the enclosing
//! byte range before any view is constructed and before any allocation is sized
//! from it. That ordering is the point — a hostile `section_count` of `u32::MAX`
//! must cost a bounds check, not 137 GB.
//!
//! Reading is layered so each layer only sees bytes the layer above already
//! proved to exist: [`read_container`] validates the header and TOC, and only a
//! [`Container`] can hand out a [`Section`].
//!
//! Writing produces canonical bytes **by construction** rather than by post-hoc
//! checking: sections are emitted token-kind-first in caller order, fields in
//! ascending id order, gaps zero-filled, every offset computed from the emitted
//! length. No hash map, sort-by-address, or other iteration-order dependence
//! exists in the write path, so one input always yields one buffer.

use usfm_onion::token::BookId;

use crate::error::{DecodeError, EncodeError, LayoutRefusal};
use crate::primitives::{
    Cursor, Writer, checked_extent, integrity_checksum, integrity_checksum_parts, window,
};
use crate::schema::{
    CHECKSUM_OMITTED, CONTAINER_CHECKSUM_OFFSET, CONTAINER_FLAGS_KNOWN, CONTAINER_HEADER_LEN,
    CONTAINER_MAGIC, CONTAINER_RESERVED_LEN, DIRECTORY_ENTRY_LEN, ELEMENT_WIDTH_VARIABLE,
    FIELD_FLAG_REQUIRED, FIELD_FLAGS_KNOWN, FORMAT_VERSION, SECTION_ALIGN, SECTION_CHECKSUM_OFFSET,
    SECTION_FLAG_POSITIONAL_IDS, SECTION_HEADER_LEN, SECTION_MAGIC, SECTION_VERSION, SectionKind,
    TOC_ENTRY_LEN, TOC_FLAGS_KNOWN, TOKEN_SECTION_RULES_VERSION, token_field,
};

/// Element width of one field payload. A closed enum rather than a raw `u8`, so
/// neither decoded nor caller-supplied bytes can name an unsupported stride.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementWidth {
    /// Payload is not a uniform array (string dictionaries, sparse keyed
    /// records). Byte length is authoritative and no alignment is implied.
    Variable,
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

impl ElementWidth {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            ELEMENT_WIDTH_VARIABLE => Some(Self::Variable),
            1 => Some(Self::One),
            2 => Some(Self::Two),
            4 => Some(Self::Four),
            8 => Some(Self::Eight),
            16 => Some(Self::Sixteen),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::Variable => ELEMENT_WIDTH_VARIABLE,
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
            Self::Sixteen => 16,
        }
    }

    /// Alignment a payload of this width requires. Variable-width payloads are
    /// byte-aligned; every other width aligns to itself.
    const fn align(self) -> usize {
        match self {
            Self::Variable => 1,
            other => other.as_u8() as usize,
        }
    }
}

/// Validated container header (48 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainerHeader {
    pub format_version: u16,
    pub flags: u32,
    pub section_count: u32,
    pub toc_offset: u64,
    /// Zero means the producer omitted integrity checking.
    pub checksum: u64,
    /// The snapshot the corpus was encoded from. Wire stores the caller's value
    /// verbatim and never derives or re-derives it.
    pub snapshot_id: u64,
}

/// Validated TOC entry (32 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TocEntry {
    pub kind: SectionKind,
    pub book: BookId,
    pub section_version: u16,
    pub flags: u16,
    pub offset: u64,
    pub byte_len: u64,
    pub source_hash: u64,
}

/// Validated section header (64 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionHeader {
    pub format_version: u16,
    /// Rule-catalog version. Always zero for token sections.
    pub rules_version: u16,
    pub kind: SectionKind,
    pub flags: u8,
    pub book: BookId,
    pub record_count: u32,
    pub source_hash: u64,
    pub section_len: u64,
    pub checksum: u64,
    /// Exact byte length of the book source this section's spans index into.
    /// Carried per section, not per container, because one container holds
    /// several books. Binding external bytes against it is `decode_borrowed`'s
    /// job — this layer only surfaces the validated value.
    pub source_len: u64,
    /// Marker-catalog stamp. A mismatch means packed marker ordinals no longer
    /// name the same markers.
    pub catalog_stamp: u64,
}

/// One validated field payload. The slice is already proved to lie inside the
/// section, to satisfy its width's alignment, and not to overlap another field,
/// so the column readers built on top never repeat that arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionField<'a> {
    pub id: u16,
    pub width: ElementWidth,
    /// Set when the producer marked the field required for a correct reading of
    /// the section. This is what makes an *unknown* id decidable: unknown and
    /// required rejects, unknown and optional is skipped.
    pub required: bool,
    pub count: u32,
    pub bytes: &'a [u8],
}

/// A validated section: header plus the field payloads its directory names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section<'a> {
    pub header: SectionHeader,
    fields: Vec<SectionField<'a>>,
}

impl<'a> Section<'a> {
    pub fn fields(&self) -> &[SectionField<'a>] {
        &self.fields
    }

    pub fn field(&self, id: u16) -> Option<&SectionField<'a>> {
        self.fields.iter().find(|field| field.id == id)
    }

    /// True when token ids are `{book}-{index}` and therefore neither the id
    /// column nor its dictionary is present.
    pub fn positional_ids(&self) -> bool {
        self.header.flags & SECTION_FLAG_POSITIONAL_IDS != 0
    }
}

/// A container whose header and TOC are validated. Sections are validated on
/// demand, so a caller reading one book does not pay for the rest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container<'a> {
    bytes: &'a [u8],
    header: ContainerHeader,
    toc: Vec<TocEntry>,
    allow_omitted_checksums: bool,
}

impl<'a> Container<'a> {
    pub fn header(&self) -> &ContainerHeader {
        &self.header
    }

    pub fn toc(&self) -> &[TocEntry] {
        &self.toc
    }

    /// `None` only when `index` is past the TOC; a wire problem is the inner
    /// `Err`. Caller mistakes and producer mistakes stay distinguishable.
    pub fn section(&self, index: usize) -> Option<Result<Section<'a>, DecodeError>> {
        self.toc
            .get(index)
            .map(|entry| read_section(self.bytes, entry, self.allow_omitted_checksums))
    }

    pub fn sections(&self) -> impl Iterator<Item = Result<Section<'a>, DecodeError>> + '_ {
        self.toc
            .iter()
            .map(|entry| read_section(self.bytes, entry, self.allow_omitted_checksums))
    }
}

/// Validates a container header, its integrity checksum, and its whole TOC.
pub fn read_container(bytes: &[u8]) -> Result<Container<'_>, DecodeError> {
    read_container_with_policy(bytes, false)
}

/// Explicit transient-data entry point. Omitted checksums are accepted, while
/// present checksums and all structural invariants are still validated.
pub fn read_container_unchecked(bytes: &[u8]) -> Result<Container<'_>, DecodeError> {
    read_container_with_policy(bytes, true)
}

fn read_container_with_policy(
    bytes: &[u8],
    allow_omitted_checksums: bool,
) -> Result<Container<'_>, DecodeError> {
    let header = read_container_header(bytes)?;
    // Checked before the TOC is parsed: if the buffer is corrupt there is no
    // reason to act on any count it declares.
    validate_checksum(
        bytes,
        CONTAINER_CHECKSUM_OFFSET,
        header.checksum,
        allow_omitted_checksums,
    )?;
    let toc = read_toc(bytes, &header)?;
    Ok(Container {
        bytes,
        header,
        toc,
        allow_omitted_checksums,
    })
}

fn validate_checksum(
    bytes: &[u8],
    checksum_offset: usize,
    expected: u64,
    allow_omitted: bool,
) -> Result<(), DecodeError> {
    if expected == CHECKSUM_OMITTED {
        return if allow_omitted {
            Ok(())
        } else {
            Err(DecodeError::ChecksumMismatch)
        };
    }
    if integrity_checksum(bytes, checksum_offset) != expected {
        return Err(DecodeError::ChecksumMismatch);
    }
    Ok(())
}

fn read_container_header(bytes: &[u8]) -> Result<ContainerHeader, DecodeError> {
    let mut cursor = Cursor::new(
        bytes
            .get(..CONTAINER_HEADER_LEN)
            .ok_or(DecodeError::Truncated)?,
    );
    if cursor.array::<4>()? != CONTAINER_MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let format_version = cursor.u16()?;
    if format_version != FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            found: format_version,
        });
    }
    // The header declares its own length so a longer future header stays
    // self-describing. A v1 container claiming any other length contradicts the
    // version it just declared.
    if usize::from(cursor.u16()?) != CONTAINER_HEADER_LEN {
        return Err(DecodeError::InvalidSection);
    }
    let flags = cursor.u32()?;
    if flags & !CONTAINER_FLAGS_KNOWN != 0 {
        return Err(DecodeError::UnsupportedFlags { found: flags });
    }
    let section_count = cursor.u32()?;
    let toc_offset = cursor.u64()?;
    let checksum = cursor.u64()?;
    let snapshot_id = cursor.u64()?;
    // Reserved bytes reject when set for the same reason unknown flag bits do:
    // a later version's field must not slip past a build that cannot honour it.
    if cursor.array::<CONTAINER_RESERVED_LEN>()? != [0u8; CONTAINER_RESERVED_LEN] {
        return Err(DecodeError::InvalidSection);
    }
    Ok(ContainerHeader {
        format_version,
        flags,
        section_count,
        toc_offset,
        checksum,
        snapshot_id,
    })
}

fn read_toc(bytes: &[u8], header: &ContainerHeader) -> Result<Vec<TocEntry>, DecodeError> {
    if header.toc_offset < CONTAINER_HEADER_LEN as u64
        || !header.toc_offset.is_multiple_of(SECTION_ALIGN)
    {
        return Err(DecodeError::InvalidToc);
    }
    let toc_len = checked_extent(u64::from(header.section_count), TOC_ENTRY_LEN as u64)?;
    // Bounds first, capacity second: a declared count becomes an allocation size
    // only after the bytes it claims are proved to exist.
    let toc_bytes = window(bytes, header.toc_offset, toc_len)?;
    let toc_end = header.toc_offset + toc_len;

    let mut entries = Vec::with_capacity(header.section_count as usize);
    let mut cursor = Cursor::new(toc_bytes);
    for _ in 0..header.section_count {
        entries.push(read_toc_entry(
            bytes,
            &mut cursor,
            header.toc_offset,
            toc_end,
        )?);
    }
    validate_toc_consistency(&entries)?;
    Ok(entries)
}

fn read_toc_entry(
    bytes: &[u8],
    cursor: &mut Cursor<'_>,
    toc_offset: u64,
    toc_end: u64,
) -> Result<TocEntry, DecodeError> {
    let kind = SectionKind::from_u8(cursor.u8()?).ok_or(DecodeError::InvalidDiscriminant)?;
    let book = read_book(cursor.array::<3>()?)?;
    let section_version = cursor.u16()?;
    if section_version != SECTION_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            found: section_version,
        });
    }
    let flags = cursor.u16()?;
    if flags & !TOC_FLAGS_KNOWN != 0 {
        return Err(DecodeError::UnsupportedFlags {
            found: u32::from(flags),
        });
    }
    let offset = cursor.u64()?;
    let byte_len = cursor.u64()?;
    let source_hash = cursor.u64()?;

    if !offset.is_multiple_of(SECTION_ALIGN) || offset < CONTAINER_HEADER_LEN as u64 {
        return Err(DecodeError::InvalidToc);
    }
    // Every section carries a full section header, so a shorter declared length
    // can only come from a producer that never wrote one.
    if byte_len < SECTION_HEADER_LEN as u64 {
        return Err(DecodeError::InvalidToc);
    }
    let end = offset
        .checked_add(byte_len)
        .ok_or(DecodeError::OffsetOverflow)?;
    // Proves the range exists before anything downstream reads it.
    window(bytes, offset, byte_len)?;
    if offset < toc_end && end > toc_offset {
        return Err(DecodeError::InvalidToc);
    }
    Ok(TocEntry {
        kind,
        book,
        section_version,
        flags,
        offset,
        byte_len,
        source_hash,
    })
}

/// Cross-entry TOC rules: no two sections overlap, no `(kind, book)` repeats,
/// and every finding section has a token section with the same book and source
/// hash. All three go through sorted key vectors rather than pairwise scans,
/// because `section_count` is producer-chosen and a quadratic check would be a
/// denial-of-service channel.
fn validate_toc_consistency(entries: &[TocEntry]) -> Result<(), DecodeError> {
    let mut ranges: Vec<(u64, u64)> = entries
        .iter()
        .map(|entry| (entry.offset, entry.offset + entry.byte_len))
        .collect();
    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[1].0 < pair[0].1) {
        return Err(DecodeError::InvalidToc);
    }

    let mut keys: Vec<(SectionKind, BookId)> = entries
        .iter()
        .map(|entry| (entry.kind, entry.book))
        .collect();
    keys.sort_unstable();
    if keys.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(DecodeError::InvalidToc);
    }

    let mut token_keys: Vec<(BookId, u64)> = entries
        .iter()
        .filter(|entry| entry.kind == SectionKind::Token)
        .map(|entry| (entry.book, entry.source_hash))
        .collect();
    token_keys.sort_unstable();
    if entries
        .iter()
        .filter(|entry| entry.kind == SectionKind::Finding)
        .any(|entry| {
            token_keys
                .binary_search(&(entry.book, entry.source_hash))
                .is_err()
        })
    {
        return Err(DecodeError::InvalidToc);
    }
    Ok(())
}

fn read_book(raw: [u8; 3]) -> Result<BookId, DecodeError> {
    let text = std::str::from_utf8(&raw).map_err(|_| DecodeError::InvalidUtf8)?;
    BookId::from_str(text).ok_or(DecodeError::InvalidToc)
}

fn read_section<'a>(
    bytes: &'a [u8],
    entry: &TocEntry,
    allow_omitted_checksums: bool,
) -> Result<Section<'a>, DecodeError> {
    let section_bytes = window(bytes, entry.offset, entry.byte_len)?;
    let (header, directory_count) = read_section_header(section_bytes, entry)?;
    validate_checksum(
        section_bytes,
        SECTION_CHECKSUM_OFFSET,
        header.checksum,
        allow_omitted_checksums,
    )?;
    let fields = read_directory(section_bytes, &header, directory_count)?;
    Ok(Section { header, fields })
}

/// Validates standalone section bytes — structure, discriminants, reserved
/// bytes, its own integrity checksum, and its whole field directory — and returns
/// what the bytes say they are.
///
/// This is what makes reusing a previously encoded section safe: a caller offering
/// bytes back gets them read by the same reader a decoder would use, and the
/// facts it *claims* about them are then checked against the header this returns.
/// A section validated here is one a reader will accept in the container it is
/// spliced into, because a section's bytes mean the same thing wherever they sit
/// on the 16-byte grid.
pub(crate) fn inspect_section(bytes: &[u8]) -> Result<SectionHeader, DecodeError> {
    let (header, directory_count) = read_section_header_inner(bytes, None)?;
    let section_bytes = bytes
        .get(..usize::try_from(header.section_len).map_err(|_| DecodeError::OffsetOverflow)?)
        .ok_or(DecodeError::Truncated)?;
    if section_bytes.len() != bytes.len() {
        // Trailing bytes are not a harmless tail: the checksum covers exactly the
        // section, so extra bytes mean this slice is not one section.
        return Err(DecodeError::InvalidSection);
    }
    validate_checksum(
        section_bytes,
        SECTION_CHECKSUM_OFFSET,
        header.checksum,
        false,
    )?;
    read_directory(section_bytes, &header, directory_count)?;
    Ok(header)
}

/// Returns the validated header plus the raw directory entry count, which is a
/// layout detail the public header deliberately does not carry.
fn read_section_header(
    section_bytes: &[u8],
    entry: &TocEntry,
) -> Result<(SectionHeader, u16), DecodeError> {
    read_section_header_inner(section_bytes, Some(entry))
}

/// The header reader, with the TOC cross-check optional.
///
/// `entry` is `None` for standalone bytes ([`inspect_section`]), which have no TOC
/// to agree with; every other check is identical, and in the same order, so a
/// section validated without a TOC is held to the same standard as one inside a
/// container.
fn read_section_header_inner(
    section_bytes: &[u8],
    entry: Option<&TocEntry>,
) -> Result<(SectionHeader, u16), DecodeError> {
    let mut cursor = Cursor::new(
        section_bytes
            .get(..SECTION_HEADER_LEN)
            .ok_or(DecodeError::Truncated)?,
    );
    if cursor.array::<4>()? != SECTION_MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let format_version = cursor.u16()?;
    if format_version != FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            found: format_version,
        });
    }
    let rules_version = cursor.u16()?;
    let kind = SectionKind::from_u8(cursor.u8()?).ok_or(DecodeError::InvalidDiscriminant)?;
    let flags = cursor.u8()?;
    let book = read_book(cursor.array::<3>()?)?;
    if cursor.array::<3>()? != [0u8; 3] {
        return Err(DecodeError::InvalidSection);
    }
    let record_count = cursor.u32()?;
    let directory_count = cursor.u16()?;
    if usize::from(cursor.u16()?) != DIRECTORY_ENTRY_LEN {
        return Err(DecodeError::InvalidSection);
    }
    let source_hash = cursor.u64()?;
    let section_len = cursor.u64()?;
    let checksum = cursor.u64()?;
    let source_len = cursor.u64()?;
    let catalog_stamp = cursor.u64()?;

    // The TOC entry and the section header restate kind, book, source hash, and
    // length. They must agree: a decoder trusting one and indexing with the
    // other would hand back a different book's bytes than the caller asked for.
    if let Some(entry) = entry
        && (kind != entry.kind
            || book != entry.book
            || source_hash != entry.source_hash
            || section_len != entry.byte_len)
    {
        return Err(DecodeError::InvalidSection);
    }
    // Flag bits are kind-specific; the token-only `positional_ids` bit is not a
    // meaningful statement about a finding section.
    if flags & !kind.known_section_flags() != 0 {
        return Err(DecodeError::UnsupportedFlags {
            found: u32::from(flags),
        });
    }
    if kind == SectionKind::Token && rules_version != TOKEN_SECTION_RULES_VERSION {
        return Err(DecodeError::InvalidSection);
    }

    let directory_len = checked_extent(u64::from(directory_count), DIRECTORY_ENTRY_LEN as u64)?;
    if (SECTION_HEADER_LEN as u64)
        .checked_add(directory_len)
        .ok_or(DecodeError::OffsetOverflow)?
        > section_len
    {
        return Err(DecodeError::Truncated);
    }
    Ok((
        SectionHeader {
            format_version,
            rules_version,
            kind,
            flags,
            book,
            record_count,
            source_hash,
            section_len,
            checksum,
            source_len,
            catalog_stamp,
        },
        directory_count,
    ))
}

fn read_directory<'a>(
    section_bytes: &'a [u8],
    header: &SectionHeader,
    directory_count: u16,
) -> Result<Vec<SectionField<'a>>, DecodeError> {
    let directory_len = checked_extent(u64::from(directory_count), DIRECTORY_ENTRY_LEN as u64)?;
    let payload_start = SECTION_HEADER_LEN as u64 + directory_len;
    let directory_bytes = window(section_bytes, SECTION_HEADER_LEN as u64, directory_len)?;

    let mut fields: Vec<SectionField<'a>> = Vec::with_capacity(directory_count as usize);
    let mut seen_ids: Vec<u16> = Vec::with_capacity(directory_count as usize);
    let mut ranges: Vec<(u64, u64)> = Vec::with_capacity(directory_count as usize);
    let mut cursor = Cursor::new(directory_bytes);
    for _ in 0..directory_count {
        let id = cursor.u16()?;
        let width = ElementWidth::from_u8(cursor.u8()?).ok_or(DecodeError::InvalidSection)?;
        let flags = cursor.u8()?;
        if flags & !FIELD_FLAGS_KNOWN != 0 {
            return Err(DecodeError::UnsupportedFlags {
                found: u32::from(flags),
            });
        }
        let required = flags & FIELD_FLAG_REQUIRED != 0;
        let offset = u64::from(cursor.u32()?);
        let byte_len = u64::from(cursor.u32()?);
        let count = cursor.u32()?;

        let spec = header.kind.field_table().iter().find(|spec| spec.id == id);
        match spec {
            // A known id's required-ness is fixed by the schema; a producer that
            // disagrees is describing a different format.
            Some(spec) if spec.required != required => {
                return Err(DecodeError::InvalidSection);
            }
            Some(spec)
                if spec
                    .element_width
                    .is_some_and(|expected| expected != width.as_u8()) =>
            {
                return Err(DecodeError::InvalidSection);
            }
            Some(_) => {}
            // Unknown ids are the forward-compatibility hinge: skip an optional
            // one, refuse a required one rather than silently dropping data the
            // producer says is load-bearing.
            None if required => return Err(DecodeError::InvalidSection),
            None => {}
        }
        if seen_ids.contains(&id) {
            return Err(DecodeError::InvalidSection);
        }
        seen_ids.push(id);
        // A payload inside the header/directory region would alias the very
        // bytes that describe it.
        if offset < payload_start {
            return Err(DecodeError::InvalidSection);
        }
        if !offset.is_multiple_of(width.align() as u64) {
            return Err(DecodeError::InvalidSection);
        }
        if width != ElementWidth::Variable
            && checked_extent(u64::from(count), u64::from(width.as_u8()))? != byte_len
        {
            return Err(DecodeError::InvalidSection);
        }
        let payload = window(section_bytes, offset, byte_len)?;
        ranges.push((offset, offset + byte_len));
        if spec.is_some() {
            fields.push(SectionField {
                id,
                width,
                required,
                count,
                bytes: payload,
            });
        }
    }

    ranges.sort_unstable();
    if ranges.windows(2).any(|pair| pair[1].0 < pair[0].1) {
        return Err(DecodeError::InvalidSection);
    }
    validate_field_set(header, &fields)?;
    Ok(fields)
}

/// Whole-section field rules: every schema-required field is present, and the
/// `positional_ids` flag agrees with the absence of the columns it claims are
/// omitted.
fn validate_field_set(
    header: &SectionHeader,
    fields: &[SectionField<'_>],
) -> Result<(), DecodeError> {
    for spec in header.kind.field_table() {
        if spec.required && !fields.iter().any(|field| field.id == spec.id) {
            return Err(DecodeError::InvalidSection);
        }
    }
    if header.kind == SectionKind::Token && header.flags & SECTION_FLAG_POSITIONAL_IDS != 0 {
        for id in token_field::POSITIONAL_ID_EXCLUSIVE {
            if fields.iter().any(|field| field.id == id) {
                return Err(DecodeError::InvalidSection);
            }
        }
    }
    Ok(())
}

/// Kind-specific header content for a section being written. Shaped so a token
/// section cannot claim a rule-catalog version and a finding section cannot
/// claim a token-only flag — the two illegal headers a raw
/// `flags`/`rules_version` pair would let a caller build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionVariant {
    Token { positional_ids: bool },
    Finding { rules_version: u16 },
}

impl SectionVariant {
    pub const fn kind(self) -> SectionKind {
        match self {
            Self::Token { .. } => SectionKind::Token,
            Self::Finding { .. } => SectionKind::Finding,
        }
    }

    const fn flags(self) -> u8 {
        match self {
            Self::Token {
                positional_ids: true,
            } => SECTION_FLAG_POSITIONAL_IDS,
            _ => 0,
        }
    }

    const fn rules_version(self) -> u16 {
        match self {
            Self::Token { .. } => TOKEN_SECTION_RULES_VERSION,
            Self::Finding { rules_version } => rules_version,
        }
    }
}

/// One field payload to write. Required-ness is not a caller input: it comes
/// from the schema table, so a writer cannot mislabel a known field. An id
/// outside the table is written as optional, which is the only claim a producer
/// that does not know the field can honestly make.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldPayload<'a> {
    pub id: u16,
    pub width: ElementWidth,
    /// Element count. For a uniform width it must agree with `bytes`; for a
    /// variable-width payload it is the record count only this field's own
    /// reader can interpret.
    pub count: u32,
    pub bytes: &'a [u8],
}

/// One section to write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionPayload<'a> {
    pub variant: SectionVariant,
    pub book: BookId,
    pub source_hash: u64,
    /// Exact byte length of the book source the spans in this section index
    /// into. Paired with `source_hash`: length alone is cheap to check first,
    /// the hash catches same-length different bytes.
    pub source_len: u64,
    pub catalog_stamp: u64,
    pub record_count: u32,
    pub fields: Vec<FieldPayload<'a>>,
}

/// One section of a container: semantics to encode now, or bytes a previous
/// publication already encoded.
///
/// A finished section is position-independent — every directory offset is
/// relative to the section's own start and its checksum covers only its own bytes
/// — which is what makes reuse a splice rather than a re-encode. The header here
/// is one a reader already validated ([`inspect_section`]); this writer trusts no
/// caller-supplied description of opaque bytes.
pub(crate) enum ContainerSection<'a> {
    Fresh(&'a SectionPayload<'a>),
    Encoded {
        header: SectionHeader,
        bytes: &'a [u8],
    },
}

impl ContainerSection<'_> {
    fn kind(&self) -> SectionKind {
        match self {
            Self::Fresh(payload) => payload.variant.kind(),
            Self::Encoded { header, .. } => header.kind,
        }
    }

    fn book(&self) -> BookId {
        match self {
            Self::Fresh(payload) => payload.book,
            Self::Encoded { header, .. } => header.book,
        }
    }

    fn source_hash(&self) -> u64 {
        match self {
            Self::Fresh(payload) => payload.source_hash,
            Self::Encoded { header, .. } => header.source_hash,
        }
    }
}

/// Writes a canonical container: token sections in caller (corpus) order, then
/// the corresponding finding sections in caller order, each 16-byte aligned,
/// non-overlapping, and integrity-checksummed.
pub fn write_container(
    snapshot_id: u64,
    sections: &[SectionPayload<'_>],
) -> Result<Vec<u8>, EncodeError> {
    let sections: Vec<ContainerSection<'_>> =
        sections.iter().map(ContainerSection::Fresh).collect();
    write_container_sections(snapshot_id, &sections)
}

/// [`write_container`], accepting already-encoded sections alongside fresh ones.
pub(crate) fn write_container_sections(
    snapshot_id: u64,
    sections: &[ContainerSection<'_>],
) -> Result<Vec<u8>, EncodeError> {
    let ordered = canonical_order(sections)?;
    let section_count =
        u32::try_from(ordered.len()).map_err(|_| EncodeError::InvalidSectionLayout {
            book: ordered
                .last()
                .map_or(BookId::UNKNOWN, |section| section.book()),
            reason: LayoutRefusal::TooManySections,
        })?;

    let toc_len = TOC_ENTRY_LEN * ordered.len();
    // Sections start after the TOC on the container-wide 16-byte grid. Because
    // every section starts 16-aligned and no field needs more than 16, a field's
    // section-relative alignment is also its absolute alignment.
    let sections_base = (CONTAINER_HEADER_LEN + toc_len).next_multiple_of(SECTION_ALIGN as usize);
    let toc_padding = sections_base - CONTAINER_HEADER_LEN - toc_len;

    let mut toc = Writer::with_capacity(toc_len);
    let mut body = Writer::default();
    for section in &ordered {
        let offset = (sections_base + body.len()) as u64;
        let encoded = match section {
            ContainerSection::Fresh(payload) => std::borrow::Cow::Owned(write_section(payload)?),
            ContainerSection::Encoded { bytes, .. } => std::borrow::Cow::Borrowed(*bytes),
        };
        let byte_len = encoded.len() as u64;
        body.bytes(&encoded);
        body.pad_to_from(sections_base, SECTION_ALIGN as usize);

        toc.u8(section.kind().as_u8());
        toc.bytes(section.book().as_str().as_bytes());
        toc.u16(SECTION_VERSION);
        toc.u16(0);
        toc.u64(offset);
        toc.u64(byte_len);
        toc.u64(section.source_hash());
    }

    let mut header = [0u8; CONTAINER_HEADER_LEN];
    let mut header_fields = Writer::with_capacity(CONTAINER_HEADER_LEN);
    header_fields.bytes(&CONTAINER_MAGIC);
    header_fields.u16(FORMAT_VERSION);
    header_fields.u16(CONTAINER_HEADER_LEN as u16);
    header_fields.u32(CONTAINER_FLAGS_KNOWN);
    header_fields.u32(section_count);
    header_fields.u64(CONTAINER_HEADER_LEN as u64);
    header_fields.u64(CHECKSUM_OMITTED);
    header_fields.u64(snapshot_id);
    header_fields.bytes(&[0u8; CONTAINER_RESERVED_LEN]);
    header.copy_from_slice(&header_fields.finish());

    let toc = toc.finish();
    let body = body.finish();
    let padding = &[0u8; SECTION_ALIGN as usize][..toc_padding];
    // Hashed with the checksum field still zero, exactly as a reader re-derives
    // it over the finished buffer.
    let checksum = integrity_checksum_parts(&[&header, &toc, padding, &body]);
    // Explicit end bound: the checksum is no longer the last header field, so an
    // open-ended range would try to fill the trailing fields too.
    header[CONTAINER_CHECKSUM_OFFSET..CONTAINER_CHECKSUM_OFFSET + 8]
        .copy_from_slice(&checksum.to_le_bytes());

    let mut out = Vec::with_capacity(header.len() + toc.len() + toc_padding + body.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&toc);
    out.extend_from_slice(padding);
    out.extend_from_slice(&body);
    Ok(out)
}

/// Token sections first in caller order, then finding sections in caller order.
/// The order comes from the input sequence alone, so it reproduces without
/// depending on comparison stability or key collisions.
fn canonical_order<'p, 'a>(
    sections: &'p [ContainerSection<'a>],
) -> Result<Vec<&'p ContainerSection<'a>>, EncodeError> {
    let mut keys: Vec<(SectionKind, BookId)> = sections
        .iter()
        .map(|section| (section.kind(), section.book()))
        .collect();
    keys.sort_unstable();
    if let Some(pair) = keys.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(EncodeError::InvalidSectionLayout {
            book: pair[0].1,
            reason: LayoutRefusal::DuplicateSection { kind: pair[0].0 },
        });
    }

    let mut token_keys: Vec<(BookId, u64)> = sections
        .iter()
        .filter(|section| section.kind() == SectionKind::Token)
        .map(|section| (section.book(), section.source_hash()))
        .collect();
    token_keys.sort_unstable();

    let mut ordered = Vec::with_capacity(sections.len());
    for kind in [SectionKind::Token, SectionKind::Finding] {
        for section in sections.iter().filter(|section| section.kind() == kind) {
            // A finding section pairs to its token section by book *and* source
            // hash; a differing hash means the pair a reader looks for does not
            // exist.
            if kind == SectionKind::Finding
                && token_keys
                    .binary_search(&(section.book(), section.source_hash()))
                    .is_err()
            {
                return Err(EncodeError::InvalidSectionLayout {
                    book: section.book(),
                    reason: LayoutRefusal::OrphanFindingSection,
                });
            }
            ordered.push(section);
        }
    }
    Ok(ordered)
}

fn write_section(section: &SectionPayload<'_>) -> Result<Vec<u8>, EncodeError> {
    let refuse = |reason| EncodeError::InvalidSectionLayout {
        book: section.book,
        reason,
    };
    let kind = section.variant.kind();

    // Ascending field id is the canonical directory order.
    let mut fields: Vec<&FieldPayload<'_>> = section.fields.iter().collect();
    fields.sort_by_key(|field| field.id);
    if let Some(pair) = fields.windows(2).find(|pair| pair[0].id == pair[1].id) {
        return Err(refuse(LayoutRefusal::DuplicateField {
            field_id: pair[0].id,
        }));
    }
    let directory_count =
        u16::try_from(fields.len()).map_err(|_| refuse(LayoutRefusal::TooManyFields))?;

    for field in &fields {
        if field.width != ElementWidth::Variable
            && u64::from(field.count) * u64::from(field.width.as_u8()) != field.bytes.len() as u64
        {
            return Err(refuse(LayoutRefusal::FieldExtentMismatch {
                field_id: field.id,
            }));
        }
        if kind
            .field_table()
            .iter()
            .find(|spec| spec.id == field.id)
            .and_then(|spec| spec.element_width)
            .is_some_and(|expected| expected != field.width.as_u8())
        {
            return Err(refuse(LayoutRefusal::FieldExtentMismatch {
                field_id: field.id,
            }));
        }
    }
    for spec in kind.field_table() {
        if spec.required && !fields.iter().any(|field| field.id == spec.id) {
            return Err(refuse(LayoutRefusal::MissingRequiredField {
                field_id: spec.id,
            }));
        }
    }
    if matches!(
        section.variant,
        SectionVariant::Token {
            positional_ids: true
        }
    ) {
        for id in token_field::POSITIONAL_ID_EXCLUSIVE {
            if fields.iter().any(|field| field.id == id) {
                return Err(refuse(LayoutRefusal::PositionalIdConflict { field_id: id }));
            }
        }
    }

    let mut offsets = Vec::with_capacity(fields.len());
    let mut position = SECTION_HEADER_LEN + DIRECTORY_ENTRY_LEN * fields.len();
    for field in &fields {
        position = position.next_multiple_of(field.width.align());
        offsets.push(position);
        position += field.bytes.len();
    }
    let section_len = position;
    // Every directory offset, byte length, and count is a `u32`, so a section
    // that outgrows 32 bits cannot be described by its own directory.
    if u32::try_from(section_len).is_err() {
        return Err(refuse(LayoutRefusal::SectionTooLarge));
    }

    let mut header = [0u8; SECTION_HEADER_LEN];
    let mut header_fields = Writer::with_capacity(SECTION_HEADER_LEN);
    header_fields.bytes(&SECTION_MAGIC);
    header_fields.u16(FORMAT_VERSION);
    header_fields.u16(section.variant.rules_version());
    header_fields.u8(kind.as_u8());
    header_fields.u8(section.variant.flags());
    header_fields.bytes(section.book.as_str().as_bytes());
    header_fields.bytes(&[0u8; 3]);
    header_fields.u32(section.record_count);
    header_fields.u16(directory_count);
    header_fields.u16(DIRECTORY_ENTRY_LEN as u16);
    header_fields.u64(section.source_hash);
    header_fields.u64(section_len as u64);
    header_fields.u64(CHECKSUM_OMITTED);
    header_fields.u64(section.source_len);
    header_fields.u64(section.catalog_stamp);
    header.copy_from_slice(&header_fields.finish());

    let required_ids = kind.field_table();
    let mut body = Writer::with_capacity(section_len - SECTION_HEADER_LEN);
    for (field, offset) in fields.iter().zip(&offsets) {
        body.u16(field.id);
        body.u8(field.width.as_u8());
        let required = required_ids
            .iter()
            .find(|spec| spec.id == field.id)
            .is_some_and(|spec| spec.required);
        body.u8(if required { FIELD_FLAG_REQUIRED } else { 0 });
        body.u32(*offset as u32);
        body.u32(field.bytes.len() as u32);
        body.u32(field.count);
    }
    for field in &fields {
        body.pad_to_from(SECTION_HEADER_LEN, field.width.align());
        body.bytes(field.bytes);
    }
    let body = body.finish();

    let checksum = integrity_checksum_parts(&[&header, &body]);
    header[SECTION_CHECKSUM_OFFSET..SECTION_CHECKSUM_OFFSET + 8]
        .copy_from_slice(&checksum.to_le_bytes());

    let mut out = Vec::with_capacity(header.len() + body.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(&body);
    Ok(out)
}
