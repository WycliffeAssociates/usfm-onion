pub mod convert;
pub mod diff;
pub mod format;
mod internal;
pub mod lint;
pub mod model;
pub mod parse;

pub mod advanced {
    pub use crate::diff::{
        ChapterTokenDiff, DiffStatus, DiffTokenChange, DiffUndoSide, DiffsByChapterMap, SidBlock,
        SidBlockDiff, TokenAlignment, apply_revert_by_block_id, apply_reverts_by_block_id,
        build_sid_blocks, diff_chapter_token_streams, diff_sid_blocks, diff_usfm_sources,
        diff_usfm_sources_by_chapter, flatten_diff_map, replace_chapter_diffs_in_map,
        replace_many_chapter_diffs_in_map,
    };
    #[cfg(feature = "rayon")]
    pub use crate::diff::{diff_usfm_sources_by_chapter_parallel, diff_usfm_sources_parallel};
    pub use crate::format::{
        FormatProfile, FormatRule, TokenFormatPass, format, format_mut, format_mut_with_passes,
        format_with_passes, prettify_tokens,
    };
    pub use crate::format::{apply_fixes, format_tokens_result, format_tokens_result_with_passes};
    pub use crate::internal::usj::{
        to_usj_document, to_usj_lossless_document, to_usj_lossless_string,
        to_usj_lossless_string_pretty, to_usj_lossless_value, to_usj_string, to_usj_string_pretty,
        to_usj_value,
    };
    pub use crate::internal::usj_to_usfm::{from_usj_document, from_usj_value};
    pub use crate::internal::usx::{to_usx_lossless_string, to_usx_string};
    pub use crate::internal::usx_to_usfm::from_usx_string;
    pub use crate::internal::vref::{to_vref_json_string, to_vref_map};
    pub use crate::parse::{recoveries, tokens};
}

pub use diff::{BuildSidBlocksOptions, DiffStatus};
pub use format::{FormatOptions, FormatRule};
pub use lint::{LintIssue, LintOptions};
pub use model::{DocumentFormat, FlatToken, TokenKind};
pub use parse::ParseHandle;
