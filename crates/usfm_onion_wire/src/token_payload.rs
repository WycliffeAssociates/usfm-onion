//! Mixed and sparse token-section payloads: the UTF-8 string dictionary, the
//! marker descriptor dictionary, and the sparse number, book-code, and attribute
//! records.
//!
//! Byte framing is specified per record by the framing freeze the schema
//! document carries; this module is its only implementation. Each view validates
//! its whole payload at construction — extents, index ranges, discriminants,
//! reserved bytes, and ascending record order — so every accessor below is
//! infallible and no caller repeats the arithmetic.
//!
//! Sparse records are keyed by `token_idx` in strictly ascending order, which is
//! what lets lookup be a binary search with no map and no allocation.

use std::collections::BTreeMap;

use crate::container::SectionField;
use crate::error::DecodeError;
use crate::schema::{
    ATTRIBUTE_ENTRY_LEN, ATTRIBUTE_FLAG_DEFAULT, ATTRIBUTE_ROW_LEN, BOOK_CODE_FLAG_VALID,
    BOOK_CODE_RECORD_LEN, DESCRIPTOR_FLAG_NESTED, DESCRIPTOR_FLAGS_KNOWN, DESCRIPTOR_RECORD_LEN,
    MAX_MARKER_DESCRIPTORS, NUMBER_FLAG_HAS_END, NUMBER_FLAGS_KNOWN, NUMBER_RECORD_LEN,
    NumberRangeKindTag, SPAN_ABSENT,
};

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    let raw: [u8; 2] = bytes.get(at..at.checked_add(2)?)?.try_into().ok()?;
    Some(u16::from_le_bytes(raw))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<u32> {
    let raw: [u8; 4] = bytes.get(at..at.checked_add(4)?)?.try_into().ok()?;
    Some(u32::from_le_bytes(raw))
}

/// A byte span into the bound book source. Resolving it to text needs the
/// source, which only the codec has, so views hand back the span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan {
    pub(crate) start: u32,
    pub(crate) len: u32,
}

impl SourceSpan {
    fn validate(self, source_len: u64) -> Result<Self, DecodeError> {
        let end = u64::from(self.start)
            .checked_add(u64::from(self.len))
            .ok_or(DecodeError::OffsetOverflow)?;
        if end > source_len {
            return Err(DecodeError::InvalidSection);
        }
        Ok(self)
    }
}

/// Validated UTF-8 string dictionary: `[u32; count]` start offsets followed by
/// concatenated character data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct StringDictionary<'wire> {
    offsets: &'wire [u8],
    data: &'wire [u8],
    count: u32,
}

impl<'wire> StringDictionary<'wire> {
    pub(crate) fn from_field(field: &SectionField<'wire>) -> Result<Self, DecodeError> {
        // An empty dictionary is zero bytes, not a lone terminating offset:
        // storing starts rather than bounds is what buys that.
        if field.count == 0 {
            if !field.bytes.is_empty() {
                return Err(DecodeError::InvalidSection);
            }
            return Ok(Self::default());
        }
        let offsets_len =
            usize::try_from(u64::from(field.count) * 4).map_err(|_| DecodeError::OffsetOverflow)?;
        let (offsets, data) = field
            .bytes
            .split_at_checked(offsets_len)
            .ok_or(DecodeError::Truncated)?;

        let dictionary = Self {
            offsets,
            data,
            count: field.count,
        };
        // Two passes on purpose. Offsets are validated as a set first, so a
        // descending or out-of-range offset reports the shape violation it is;
        // only then are the resulting slices decoded, so `InvalidUtf8` means
        // genuinely bad bytes rather than a bad offset seen through them.
        let mut previous = 0u32;
        for index in 0..field.count {
            let start = dictionary.start_at(index).ok_or(DecodeError::Truncated)?;
            let start_index = usize::try_from(start).map_err(|_| DecodeError::OffsetOverflow)?;
            // Ascending starts anchored at zero: any other arrangement would let
            // two strings overlap or leave data unreachable.
            if start < previous || start_index > data.len() {
                return Err(DecodeError::InvalidSection);
            }
            previous = start;
        }
        if dictionary.start_at(0) != Some(0) {
            return Err(DecodeError::InvalidSection);
        }
        for index in 0..field.count {
            dictionary.get(index).ok_or(DecodeError::InvalidUtf8)?;
        }
        Ok(dictionary)
    }

    fn start_at(&self, index: u32) -> Option<u32> {
        u32_at(self.offsets, usize::try_from(index).ok()?.checked_mul(4)?)
    }

    pub(crate) const fn len(&self) -> u32 {
        self.count
    }

    /// The string at `index`, or `None` when the index is past the dictionary.
    /// After construction the UTF-8 decode below cannot fail.
    pub(crate) fn get(&self, index: u32) -> Option<&'wire str> {
        let start = usize::try_from(self.start_at(index)?).ok()?;
        let end = match index.checked_add(1) {
            Some(next) if next < self.count => usize::try_from(self.start_at(next)?).ok()?,
            _ => self.data.len(),
        };
        std::str::from_utf8(self.data.get(start..end)?).ok()
    }

    fn require(&self, index: u32) -> Result<&'wire str, DecodeError> {
        self.get(index).ok_or(DecodeError::InvalidSection)
    }
}

/// Interns strings, assigning indices in first-use order.
///
/// The `BTreeMap` is a lookup index only and is never iterated, so no map
/// ordering reaches the bytes.
#[derive(Debug, Default)]
pub(crate) struct StringDictionaryBuilder {
    starts: Vec<u32>,
    data: String,
    ordinals: BTreeMap<Box<str>, u32>,
}

impl StringDictionaryBuilder {
    pub(crate) fn intern(&mut self, value: &str) -> Result<u32, DecodeError> {
        if let Some(index) = self.ordinals.get(value) {
            return Ok(*index);
        }
        let start = u32::try_from(self.data.len()).map_err(|_| DecodeError::OffsetOverflow)?;
        let index = u32::try_from(self.starts.len()).map_err(|_| DecodeError::OffsetOverflow)?;
        self.starts.push(start);
        self.data.push_str(value);
        self.ordinals.insert(Box::from(value), index);
        Ok(index)
    }

    pub(crate) fn len(&self) -> u32 {
        self.starts.len() as u32
    }

    pub(crate) fn bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.starts.len() * 4 + self.data.len());
        for start in &self.starts {
            out.extend_from_slice(&start.to_le_bytes());
        }
        out.extend_from_slice(self.data.as_bytes());
        out
    }
}

/// Validated marker descriptor dictionary. A descriptor is a name plus the
/// `nested` flag; all marker metadata is recovered from core's registry by name,
/// which is why nothing else is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct MarkerDescriptors<'wire> {
    records: &'wire [u8],
    count: u16,
    strings: StringDictionary<'wire>,
}

impl<'wire> MarkerDescriptors<'wire> {
    pub(crate) fn from_field(
        field: &SectionField<'wire>,
        strings: StringDictionary<'wire>,
    ) -> Result<Self, DecodeError> {
        // The `0xffff` sentinel on the index column consumes one value, exactly
        // as it does for the SID dictionary.
        if field.count > MAX_MARKER_DESCRIPTORS {
            return Err(DecodeError::InvalidSection);
        }
        let descriptors = Self {
            records: field.bytes,
            count: field.count as u16,
            strings,
        };
        for record in field.bytes.chunks_exact(DESCRIPTOR_RECORD_LEN) {
            let name_index = u32_at(record, 0).ok_or(DecodeError::Truncated)?;
            strings.require(name_index)?;
            let flags = record[4];
            if flags & !DESCRIPTOR_FLAGS_KNOWN != 0 {
                return Err(DecodeError::UnsupportedFlags {
                    found: u32::from(flags),
                });
            }
            if record[5..8] != [0u8; 3] {
                return Err(DecodeError::InvalidSection);
            }
        }
        Ok(descriptors)
    }

    pub(crate) const fn len(&self) -> u16 {
        self.count
    }

    /// `(marker name as written, nested)`.
    pub(crate) fn get(&self, index: u16) -> Option<(&'wire str, bool)> {
        if index >= self.count {
            return None;
        }
        let at = usize::from(index) * DESCRIPTOR_RECORD_LEN;
        let record = self.records.get(at..at + DESCRIPTOR_RECORD_LEN)?;
        let name = self.strings.get(u32_at(record, 0)?)?;
        Some((name, record[4] & DESCRIPTOR_FLAG_NESTED != 0))
    }
}

/// Interns `(name index, nested)` pairs, first-use order.
#[derive(Debug, Default)]
pub(crate) struct MarkerDescriptorBuilder {
    records: Vec<u8>,
    ordinals: BTreeMap<(u32, bool), u16>,
}

impl MarkerDescriptorBuilder {
    pub(crate) fn intern(&mut self, name_index: u32, nested: bool) -> Option<u16> {
        if let Some(index) = self.ordinals.get(&(name_index, nested)) {
            return Some(*index);
        }
        let next = self.ordinals.len();
        if next >= MAX_MARKER_DESCRIPTORS as usize {
            return None;
        }
        let index = next as u16;
        self.records.extend_from_slice(&name_index.to_le_bytes());
        self.records
            .push(if nested { DESCRIPTOR_FLAG_NESTED } else { 0 });
        self.records.extend_from_slice(&[0u8; 3]);
        self.ordinals.insert((name_index, nested), index);
        Some(index)
    }

    pub(crate) fn len(&self) -> u32 {
        self.ordinals.len() as u32
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.records
    }
}

/// One decoded number payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WireNumber {
    pub(crate) start: u32,
    pub(crate) end: Option<u32>,
    pub(crate) kind: NumberRangeKindTag,
}

/// Validated sparse number records, ascending by `token_idx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct NumberRecords<'wire> {
    records: &'wire [u8],
}

impl<'wire> NumberRecords<'wire> {
    pub(crate) fn from_field(
        field: &SectionField<'wire>,
        record_count: u32,
    ) -> Result<Self, DecodeError> {
        let mut previous: Option<u32> = None;
        for record in field.bytes.chunks_exact(NUMBER_RECORD_LEN) {
            let token_idx = u32_at(record, 0).ok_or(DecodeError::Truncated)?;
            if token_idx >= record_count {
                return Err(DecodeError::InvalidSection);
            }
            if previous.is_some_and(|last| token_idx <= last) {
                return Err(DecodeError::InvalidSection);
            }
            previous = Some(token_idx);
            NumberRangeKindTag::from_u8(record[12]).ok_or(DecodeError::InvalidDiscriminant)?;
            let flags = record[13];
            if flags & !NUMBER_FLAGS_KNOWN != 0 {
                return Err(DecodeError::UnsupportedFlags {
                    found: u32::from(flags),
                });
            }
            // One encoding per value: an absent end must be zero, so two
            // encoders cannot disagree about the same number.
            if flags & NUMBER_FLAG_HAS_END == 0 && u32_at(record, 8) != Some(0) {
                return Err(DecodeError::InvalidSection);
            }
            if record[14..16] != [0u8; 2] {
                return Err(DecodeError::InvalidSection);
            }
        }
        Ok(Self {
            records: field.bytes,
        })
    }

    pub(crate) fn get(&self, token_idx: u32) -> Option<WireNumber> {
        let record = find_record(self.records, NUMBER_RECORD_LEN, token_idx)?;
        Some(WireNumber {
            start: u32_at(record, 4)?,
            end: (record[13] & NUMBER_FLAG_HAS_END != 0)
                .then(|| u32_at(record, 8))
                .flatten(),
            kind: NumberRangeKindTag::from_u8(record[12])?,
        })
    }
}

/// Validated sparse book-code records, ascending by `token_idx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct BookCodeRecords<'wire> {
    records: &'wire [u8],
    strings: StringDictionary<'wire>,
}

impl<'wire> BookCodeRecords<'wire> {
    pub(crate) fn from_field(
        field: &SectionField<'wire>,
        record_count: u32,
        strings: StringDictionary<'wire>,
    ) -> Result<Self, DecodeError> {
        let mut previous: Option<u32> = None;
        for record in field.bytes.chunks_exact(BOOK_CODE_RECORD_LEN) {
            let token_idx = u32_at(record, 0).ok_or(DecodeError::Truncated)?;
            if token_idx >= record_count || previous.is_some_and(|last| token_idx <= last) {
                return Err(DecodeError::InvalidSection);
            }
            previous = Some(token_idx);
            strings.require(u32_at(record, 4).ok_or(DecodeError::Truncated)?)?;
            if record[8] & !BOOK_CODE_FLAG_VALID != 0 {
                return Err(DecodeError::UnsupportedFlags {
                    found: u32::from(record[8]),
                });
            }
            if record[9..16] != [0u8; 7] {
                return Err(DecodeError::InvalidSection);
            }
        }
        Ok(Self {
            records: field.bytes,
            strings,
        })
    }

    /// `(code as written, is_valid)`.
    pub(crate) fn get(&self, token_idx: u32) -> Option<(&'wire str, bool)> {
        let record = find_record(self.records, BOOK_CODE_RECORD_LEN, token_idx)?;
        Some((
            self.strings.get(u32_at(record, 4)?)?,
            record[8] & BOOK_CODE_FLAG_VALID != 0,
        ))
    }
}

/// One decoded attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WireAttribute<'wire> {
    pub(crate) key: &'wire str,
    pub(crate) value: &'wire str,
    pub(crate) source: SourceSpan,
    pub(crate) is_default: bool,
}

/// One decoded attribute list. `list_source` is `None` when the token had no
/// verbatim list slice at all, which is a different value from a present empty
/// one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WireAttributeList<'wire> {
    pub(crate) list_source: Option<SourceSpan>,
    pub(crate) attributes: Vec<WireAttribute<'wire>>,
}

/// Validated sparse attribute records: ascending row entries followed by the
/// attribute entries they partition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct AttributeRecords<'wire> {
    rows: &'wire [u8],
    entries: &'wire [u8],
    strings: StringDictionary<'wire>,
}

impl<'wire> AttributeRecords<'wire> {
    pub(crate) fn from_field(
        field: &SectionField<'wire>,
        record_count: u32,
        source_len: u64,
        strings: StringDictionary<'wire>,
    ) -> Result<Self, DecodeError> {
        let rows_len = usize::try_from(u64::from(field.count) * ATTRIBUTE_ROW_LEN as u64)
            .map_err(|_| DecodeError::OffsetOverflow)?;
        let (rows, entries) = field
            .bytes
            .split_at_checked(rows_len)
            .ok_or(DecodeError::Truncated)?;
        // The entry count is derived, never stored, so the two counts cannot
        // disagree.
        if !entries.len().is_multiple_of(ATTRIBUTE_ENTRY_LEN) {
            return Err(DecodeError::InvalidSection);
        }
        let entry_count = (entries.len() / ATTRIBUTE_ENTRY_LEN) as u64;

        let records = Self {
            rows,
            entries,
            strings,
        };
        let mut previous: Option<u32> = None;
        let mut expected_first = 0u64;
        for row in rows.chunks_exact(ATTRIBUTE_ROW_LEN) {
            let token_idx = u32_at(row, 0).ok_or(DecodeError::Truncated)?;
            if token_idx >= record_count || previous.is_some_and(|last| token_idx <= last) {
                return Err(DecodeError::InvalidSection);
            }
            previous = Some(token_idx);
            let first = u64::from(u32_at(row, 4).ok_or(DecodeError::Truncated)?);
            let count = u64::from(u32_at(row, 8).ok_or(DecodeError::Truncated)?);
            // Contiguous and ascending, together covering exactly the entry
            // array: that is what lets a row's attributes be sliced without a
            // search and guarantees no entry is orphaned or shared.
            if first != expected_first {
                return Err(DecodeError::InvalidSection);
            }
            expected_first = first
                .checked_add(count)
                .ok_or(DecodeError::OffsetOverflow)?;
            if expected_first > entry_count {
                return Err(DecodeError::InvalidSection);
            }
            let list_start = u32_at(row, 12).ok_or(DecodeError::Truncated)?;
            let list_len = u32_at(row, 16).ok_or(DecodeError::Truncated)?;
            if list_start == SPAN_ABSENT {
                if list_len != 0 {
                    return Err(DecodeError::InvalidSection);
                }
            } else {
                SourceSpan {
                    start: list_start,
                    len: list_len,
                }
                .validate(source_len)?;
            }
            if row[20] != 0 || row[21..24] != [0u8; 3] {
                return Err(DecodeError::InvalidSection);
            }
        }
        if expected_first != entry_count {
            return Err(DecodeError::InvalidSection);
        }

        for entry in entries.chunks_exact(ATTRIBUTE_ENTRY_LEN) {
            let key = strings.require(u32_at(entry, 0).ok_or(DecodeError::Truncated)?)?;
            strings.require(u32_at(entry, 4).ok_or(DecodeError::Truncated)?)?;
            SourceSpan {
                start: u32_at(entry, 8).ok_or(DecodeError::Truncated)?,
                len: u32_at(entry, 12).ok_or(DecodeError::Truncated)?,
            }
            .validate(source_len)?;
            let flags = entry[16];
            if flags & !ATTRIBUTE_FLAG_DEFAULT != 0 {
                return Err(DecodeError::UnsupportedFlags {
                    found: u32::from(flags),
                });
            }
            // The default-attribute shorthand has no key; a keyed entry
            // claiming to be default would round-trip as a different token.
            if flags & ATTRIBUTE_FLAG_DEFAULT != 0 && !key.is_empty() {
                return Err(DecodeError::InvalidSection);
            }
            if entry[17..20] != [0u8; 3] {
                return Err(DecodeError::InvalidSection);
            }
        }
        Ok(records)
    }

    pub(crate) fn get(&self, token_idx: u32) -> Option<WireAttributeList<'wire>> {
        let row = find_record(self.rows, ATTRIBUTE_ROW_LEN, token_idx)?;
        let first = usize::try_from(u32_at(row, 4)?).ok()?;
        let count = usize::try_from(u32_at(row, 8)?).ok()?;
        let list_start = u32_at(row, 12)?;
        let list_source = (list_start != SPAN_ABSENT).then(|| SourceSpan {
            start: list_start,
            len: u32_at(row, 16).unwrap_or(0),
        });

        let mut attributes = Vec::with_capacity(count);
        for index in first..first.checked_add(count)? {
            let at = index.checked_mul(ATTRIBUTE_ENTRY_LEN)?;
            let entry = self.entries.get(at..at.checked_add(ATTRIBUTE_ENTRY_LEN)?)?;
            attributes.push(WireAttribute {
                key: self.strings.get(u32_at(entry, 0)?)?,
                value: self.strings.get(u32_at(entry, 4)?)?,
                source: SourceSpan {
                    start: u32_at(entry, 8)?,
                    len: u32_at(entry, 12)?,
                },
                is_default: entry[16] & ATTRIBUTE_FLAG_DEFAULT != 0,
            });
        }
        Some(WireAttributeList {
            list_source,
            attributes,
        })
    }
}

/// Binary search over fixed-width records whose first `u32` is an ascending
/// `token_idx`. Ascending order is validated at construction, so this is a
/// search over sorted data rather than a hopeful probe.
fn find_record(records: &[u8], width: usize, token_idx: u32) -> Option<&[u8]> {
    let count = records.len() / width;
    let mut low = 0usize;
    let mut high = count;
    while low < high {
        let mid = low + (high - low) / 2;
        let record = records.get(mid * width..mid * width + width)?;
        match u32_at(record, 0)?.cmp(&token_idx) {
            std::cmp::Ordering::Less => low = mid + 1,
            std::cmp::Ordering::Greater => high = mid,
            std::cmp::Ordering::Equal => return Some(record),
        }
    }
    None
}

/// Builds the sparse and mixed payload buffers. Records are pushed in ascending
/// `token_idx` because the caller walks rows in order, so canonical ordering is a
/// property of the walk rather than a sort.
#[derive(Debug, Default)]
pub(crate) struct SparseBuilders {
    pub(crate) numbers: Vec<u8>,
    pub(crate) book_codes: Vec<u8>,
    attribute_rows: Vec<u8>,
    attribute_entries: Vec<u8>,
    /// Rows followed by entries, assembled once the walk is done. The field
    /// payload is one buffer, so the two halves are concatenated rather than
    /// written separately.
    pub(crate) attributes: Vec<u8>,
}

impl SparseBuilders {
    pub(crate) fn push_number(&mut self, token_idx: u32, number: WireNumber) {
        self.numbers.extend_from_slice(&token_idx.to_le_bytes());
        self.numbers.extend_from_slice(&number.start.to_le_bytes());
        self.numbers
            .extend_from_slice(&number.end.unwrap_or(0).to_le_bytes());
        self.numbers.push(number.kind.as_u8());
        self.numbers.push(if number.end.is_some() {
            NUMBER_FLAG_HAS_END
        } else {
            0
        });
        self.numbers.extend_from_slice(&[0u8; 2]);
    }

    pub(crate) fn push_book_code(&mut self, token_idx: u32, code_index: u32, is_valid: bool) {
        self.book_codes.extend_from_slice(&token_idx.to_le_bytes());
        self.book_codes.extend_from_slice(&code_index.to_le_bytes());
        self.book_codes.push(u8::from(is_valid));
        self.book_codes.extend_from_slice(&[0u8; 7]);
    }

    pub(crate) fn push_attribute_row(
        &mut self,
        token_idx: u32,
        list_source: Option<SourceSpan>,
    ) -> u32 {
        let first = (self.attribute_entries.len() / ATTRIBUTE_ENTRY_LEN) as u32;
        self.attribute_rows
            .extend_from_slice(&token_idx.to_le_bytes());
        self.attribute_rows.extend_from_slice(&first.to_le_bytes());
        // Count is backfilled by `finish_attribute_row` once the entries are in.
        self.attribute_rows.extend_from_slice(&0u32.to_le_bytes());
        let span = list_source.unwrap_or(SourceSpan {
            start: SPAN_ABSENT,
            len: 0,
        });
        self.attribute_rows
            .extend_from_slice(&span.start.to_le_bytes());
        self.attribute_rows
            .extend_from_slice(&span.len.to_le_bytes());
        self.attribute_rows.extend_from_slice(&[0u8; 4]);
        first
    }

    pub(crate) fn push_attribute_entry(
        &mut self,
        key_index: u32,
        value_index: u32,
        source: SourceSpan,
        is_default: bool,
    ) {
        self.attribute_entries
            .extend_from_slice(&key_index.to_le_bytes());
        self.attribute_entries
            .extend_from_slice(&value_index.to_le_bytes());
        self.attribute_entries
            .extend_from_slice(&source.start.to_le_bytes());
        self.attribute_entries
            .extend_from_slice(&source.len.to_le_bytes());
        self.attribute_entries.push(u8::from(is_default));
        self.attribute_entries.extend_from_slice(&[0u8; 3]);
    }

    /// Writes the attribute count into the row started most recently.
    pub(crate) fn finish_attribute_row(&mut self, first: u32) {
        let row_start = self.attribute_rows.len() - ATTRIBUTE_ROW_LEN;
        let now = (self.attribute_entries.len() / ATTRIBUTE_ENTRY_LEN) as u32;
        self.attribute_rows[row_start + 8..row_start + 12]
            .copy_from_slice(&(now - first).to_le_bytes());
    }

    /// Concatenates rows and entries into the field payload. Called once, after
    /// the last row, because a row's entry count is only known then.
    pub(crate) fn seal_attributes(&mut self) {
        self.attributes =
            Vec::with_capacity(self.attribute_rows.len() + self.attribute_entries.len());
        self.attributes.extend_from_slice(&self.attribute_rows);
        self.attributes.extend_from_slice(&self.attribute_entries);
    }

    pub(crate) fn attribute_row_count(&self) -> u32 {
        (self.attribute_rows.len() / ATTRIBUTE_ROW_LEN) as u32
    }

    pub(crate) fn number_count(&self) -> u32 {
        (self.numbers.len() / NUMBER_RECORD_LEN) as u32
    }

    pub(crate) fn book_code_count(&self) -> u32 {
        (self.book_codes.len() / BOOK_CODE_RECORD_LEN) as u32
    }
}
