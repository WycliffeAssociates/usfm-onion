#[cfg(feature = "rayon")]
use rayon::prelude::*;

mod convert;
mod intake;
mod ops;
#[cfg(test)]
mod tests;
mod types;

pub use convert::{
    convert_content, convert_path, from_usj, from_usx, into_editor_tree, into_html, into_usj,
    into_usj_from_tokens, into_usj_lossless, into_usj_lossless_from_tokens, into_usx,
    into_usx_from_tokens, into_usx_lossless, into_usx_lossless_from_tokens, into_vref,
    into_vref_from_tokens, usfm_content_to_html, usfm_content_to_usj, usfm_content_to_usx,
    usfm_path_to_usj, usfm_path_to_usx, usj_content_to_usfm, usj_content_to_usx, usj_path_to_usfm,
    usj_path_to_usx, usx_content_to_usfm, usx_content_to_usj, usx_path_to_usfm, usx_path_to_usj,
};
pub use intake::{
    into_tokens, into_tokens_batch, into_tokens_from_content, into_tokens_from_contents,
    into_tokens_from_path, into_tokens_from_paths, into_tokens_from_usfm_content,
    into_tokens_from_usfm_path, into_tokens_from_usj_content, into_tokens_from_usj_path,
    into_tokens_from_usx_content, into_tokens_from_usx_path, into_usfm_from_tokens, lex_sources,
    parse_content, parse_contents, parse_path, parse_paths, parse_sources, parse_usfm_content,
    parse_usfm_path, parse_usj_content, parse_usj_path, parse_usx_content, parse_usx_path,
    project_content, project_document, project_path, project_usfm, project_usfm_batch,
    project_usfm_content, project_usfm_path, project_usj_content, project_usj_path,
    project_usx_content, project_usx_path, push_whitespace, read_document,
};
pub use ops::{
    apply_token_fixes, diff_content, diff_paths, diff_tokens, diff_usfm, diff_usfm_by_chapter,
    format_content, format_content_with_options, format_content_with_passes, format_contents,
    format_contents_with_options, format_flat_token_batches,
    format_flat_token_batches_with_options, format_flat_tokens, format_flat_tokens_mut,
    format_flat_tokens_mut_with_passes, format_flat_tokens_with_options,
    format_flat_tokens_with_passes, format_path, format_path_with_options, format_paths,
    format_paths_with_options, format_usfm_content, format_usfm_content_with_passes,
    format_usfm_path, format_usfm_sources, format_usfm_sources_with_options, format_usj_content,
    format_usj_content_with_passes, format_usj_path, format_usx_content,
    format_usx_content_with_passes, format_usx_path, lint_content, lint_contents, lint_document,
    lint_document_batch, lint_flat_token_batches, lint_flat_tokens, lint_path, lint_paths,
    lint_usfm_content, lint_usfm_path, lint_usfm_sources, lint_usj_content, lint_usj_path,
    lint_usx_content, lint_usx_path,
};
pub use types::{
    BatchExecutionOptions, DocumentError, DocumentFormat, IntoTokensOptions, ProjectUsfmOptions,
    ProjectedUsfmDocument, read_usfm,
};

#[cfg(feature = "rayon")]
fn map_with_batch<T, U, F>(items: &[T], batch_options: BatchExecutionOptions, map: F) -> Vec<U>
where
    T: Sync,
    U: Send,
    F: Fn(&T) -> U + Sync + Send,
{
    if batch_options.parallel {
        items.par_iter().map(&map).collect()
    } else {
        items.iter().map(map).collect()
    }
}

#[cfg(not(feature = "rayon"))]
fn map_with_batch<T, U, F>(items: &[T], _batch_options: BatchExecutionOptions, map: F) -> Vec<U>
where
    F: Fn(&T) -> U,
{
    items.iter().map(map).collect()
}
