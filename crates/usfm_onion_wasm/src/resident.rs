//! The resident corpus handle: `Braid` as a JS class.
//!
//! Every verb here operates on ingested resident state and nothing else. The
//! stateless exports next door take their input from the caller; these take theirs
//! from the corpus the handle is holding. That separation is deliberate even where
//! both reach the same core function, because "operate on what I gave you" and
//! "operate on what you are holding" are different promises.
//!
//! Failures a caller can act on are values, not exceptions: every verb that can be
//! refused returns [`ApiResult`] carrying the same typed error the Rust crate
//! returns. Exceptions are reserved for programmer errors — a minter that is not
//! callable, a value that is not the shape its type claims.

use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use braid::Braid as NativeBraid;

use crate::dto::{LintOptions, lint_options_into_native};

/// A verb's outcome: the value it produced, or the typed reason it was refused.
///
/// Tagged on a string rather than a boolean, matching the crate's existing packed
/// outcome type: a string tag is what makes this a real discriminated union in
/// TypeScript, so a consumer that checks `status` has the other field narrowed for
/// it rather than asserted by hand.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ApiResult<T, E> {
    Ok { value: T },
    Error { error: E },
}

impl<T, E> ApiResult<T, E> {
    fn of(result: Result<T, E>) -> Self {
        match result {
            Ok(value) => Self::Ok { value },
            Err(error) => Self::Error { error },
        }
    }
}

// ---------------------------------------------------------------------------
// Named outcomes.
//
// `wasm_bindgen` erases a generic's parameters in a method signature, so a verb
// returning `ApiResult<MutationEffect, IngestError>` would be declared as a bare
// `ApiResult` — no value at all to a TypeScript consumer. A transparent newtype per
// shape costs nothing at runtime and restores the full type: each one renders as
// `export type XOutcome = ApiResult<T, E>`.
// ---------------------------------------------------------------------------

macro_rules! outcome {
    ($(#[$meta:meta])* $name:ident, $value:ty, $error:ty) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
        #[tsify(into_wasm_abi, from_wasm_abi)]
        #[serde(transparent)]
        pub struct $name(pub ApiResult<$value, $error>);

        impl From<Result<$value, $error>> for $name {
            fn from(result: Result<$value, $error>) -> Self {
                Self(ApiResult::of(result))
            }
        }

        impl $name {
            // Not every instantiation constructs a refusal by hand (some, like
            // `PublishOutcome`, only ever go through the `From<Result<_, _>>`
            // impl above) -- allowed rather than instantiated conditionally,
            // since a real caller adding one later is exactly the ordinary
            // case this macro exists for.
            #[allow(dead_code)]
            fn refused(error: $error) -> Self {
                Self(ApiResult::Error { error })
            }
        }
    };
}

outcome!(
    /// A mutation, or the reason the input was refused.
    MutationOutcome,
    MutationEffect,
    IngestError
);
outcome!(
    /// A mutation addressed by scope, or the reason the scope does not resolve.
    ScopedMutationOutcome,
    MutationEffect,
    ScopeError
);
outcome!(
    /// A recorded baseline, or the reason it could not be.
    BaselineMutationOutcome,
    MutationEffect,
    SetBaselineError
);
outcome!(
    /// One patch, or the reason it is not addressable.
    PatchOutcome,
    Patch,
    PatchError
);
outcome!(
    /// A patch's projected tokens, or the reason it is not addressable.
    PatchPreviewOutcome,
    Vec<crate::Token>,
    PatchError
);
outcome!(
    /// An applied patch, or the reason it was refused.
    PatchMutationOutcome,
    MutationEffect,
    PatchError
);
outcome!(
    /// A prepared format patch, or the reason the scope does not resolve.
    FormatPreparationOutcome,
    PatchPreparation,
    FormatError
);
outcome!(
    /// An applied format patch, or the reason it was refused.
    FormatMutationOutcome,
    MutationEffect,
    FormatPatchError
);
outcome!(
    /// One book's chapter labels, or the reason the book does not resolve.
    ChapterLabelsOutcome,
    Vec<ChapterLabel>,
    ScopeError
);
outcome!(
    /// Hydrated tokens, or the reason a scope does not resolve.
    ScopeTokensOutcome,
    Vec<ScopeTokens>,
    ScopeError
);
outcome!(
    /// A scope's exact bytes, or the reason it does not resolve.
    UsfmOutcome,
    ScopedOutput<String>,
    ScopeError
);
outcome!(
    /// Whether a scope differs from its baseline, or the reason it does not resolve.
    DirtyOutcome,
    bool,
    ScopeError
);
outcome!(
    /// A warm restore's report, or the reason the bytes were refused.
    RestoreOutcome,
    RestoreReport,
    RestoreError
);
outcome!(
    /// A packed corpus, or the reason it could not be produced.
    PublishOutcome,
    PublishedCorpus,
    PublishError
);
outcome!(
    /// A scope's verse index, or the reason the scope does not resolve.
    VrefIndexOutcome,
    ScopedOutput<crate::VrefIndex>,
    ScopeError
);
outcome!(
    /// A baseline diff, or the reason it cannot be answered.
    DiffBaselineOutcome,
    ScopedOutput<crate::DiffSkeleton>,
    BaselineError
);
outcome!(
    /// A transfer-ready scoped publication, or the reason it could not be
    /// produced.
    ScopedPublishOutcome,
    ScopedPublication,
    ScopedPublishError
);
outcome!(
    /// A baseline revert, or the reason it was refused.
    RevertBaselineOutcome,
    MutationEffect,
    BaselineError
);

/// The resident configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BraidConfig {
    pub lint: LintOptions,
}

impl BraidConfig {
    fn into_native(self) -> braid::BraidConfig {
        braid::BraidConfig::new(lint_options_into_native(self.lint))
    }
}

/// The resident corpus handle.
//
// A `#[wasm_bindgen]` projection of `braid::Braid` and nothing
// else: every verb below converts its wasm-facing DTO arguments into that
// handle's native types, calls the same-named native verb, and projects the
// native `Result` back into this file's `ApiResult` shape. No verb here
// composes anything -- a second copy of composition logic on this side of the
// boundary is exactly what the native handle exists to prevent.
//
// Kept as a plain comment rather than doc prose on purpose: tsify/wasm-bindgen
// copy a doc comment on this struct verbatim into the generated `.d.ts` and
// `.js`, and an internal note about which Rust crate owns the composition is
// not something a JS consumer's published surface should move for.
#[wasm_bindgen]
pub struct Braid {
    // `pub(crate)` rather than private: the parity transcript generator
    // (`crate::parity`, a sibling test-only module) constructs this struct
    // directly via a literal, bypassing the public `new` constructor whose
    // `js_sys::Function` minter parameter has no meaningful native behavior
    // outside a real JS engine. No production code outside `resident.rs`
    // reaches into this field.
    pub(crate) inner: NativeBraid,
}

#[wasm_bindgen]
impl Braid {
    /// Creates an empty handle bound to the application's own id minter.
    ///
    /// The minter is a JS callback returning a string, held for the life of the
    /// handle: core never invents a token id, so every token a fix or format pass
    /// synthesizes gets one from here. Speed, spelling, and collision resistance
    /// are the application's trade — uniqueness is not assumed but enforced at the
    /// residency boundary, where a collision is a typed rejection rather than a
    /// corrupted book.
    ///
    /// Throws only for a programmer error: a minter that throws, or one that
    /// returns something other than a string.
    #[wasm_bindgen(constructor)]
    pub fn new(config: BraidConfig, minter: js_sys::Function) -> Braid {
        let mint = move || {
            minter
                .call0(&JsValue::NULL)
                .expect("the id minter must not throw")
                .as_string()
                .expect("the id minter must return a string")
        };
        Braid {
            inner: NativeBraid::new(config.into_native(), mint),
        }
    }

    // ---- mutation ------------------------------------------------------

    /// Replaces the whole corpus with a validated candidate.
    ///
    /// Every book is built, validated, and hashed before resident state is touched,
    /// so a rejection leaves the corpus, its stamps, and its identity exactly as
    /// they were.
    #[wasm_bindgen(js_name = replaceCorpus)]
    pub fn replace_corpus(&mut self, corpus: CorpusInput) -> MutationOutcome {
        let native = match corpus_into_native(corpus) {
            Ok(native) => native,
            Err(error) => return MutationOutcome::refused(error),
        };
        MutationOutcome::from(
            self.inner
                .replace_corpus(native)
                .map(MutationEffect::from)
                .map_err(IngestError::from),
        )
    }

    /// Seeds the whole corpus from packed bytes plus the sources they were bound to
    /// — the warm cold-open.
    ///
    /// Composed here because this is the only layer allowed to know both halves: the
    /// bytes are verified and decoded by the wire codec, and the results are handed
    /// to the resident corpus, which never sees a packed byte itself. Verification is
    /// the full trust boundary — structure, both checksums, exact source length and
    /// content hash, the catalog stamp, every discriminant and index — so a container
    /// that does not check out is refused before anything is installed.
    ///
    /// A book whose cached findings cannot be adopted still seeds: residency and
    /// lint-priming are independent facts, so that book arrives with no lex or parse
    /// and is simply awaiting recompute.
    ///
    /// `packed_all`/`sources` are two single buffers -- every record's own
    /// container concatenated into the first, every record's own source
    /// concatenated into the second -- with `records` naming each one's
    /// extent into whichever buffer it belongs to (v0.1.5, bytes-at-boundary
    /// convention: this is the exact shape [`Braid::publish_scope`]'s output
    /// already is, so it forwards here with zero reshaping -- see
    /// [`ScopedPublication`]'s own doc comment). An extent that falls
    /// outside its buffer, or whose own end overflows computing it, is
    /// refused (`RestoreError::InvalidExtent`, naming the record's own
    /// `path`) before any native call -- never clamped, never truncated.
    //
    // The composition itself (`braid::Braid::restore_packed_books`) lives in
    // braid and is unchanged; this boundary's own job is resolving each
    // record's two extents against its two buffers before building the
    // native `Vec<braid::RestoreRecord>` that call actually takes.
    #[wasm_bindgen(js_name = restoreCorpus)]
    pub fn restore_corpus(
        &mut self,
        packed_all: &[u8],
        sources: &[u8],
        records: Vec<RestoreRecord>,
    ) -> RestoreOutcome {
        let mut native = Vec::with_capacity(records.len());
        for record in records {
            let Some(packed) = crate::bytes::slice_extent(packed_all, record.packed) else {
                return RestoreOutcome::refused(RestoreError::InvalidExtent { book: record.path });
            };
            // `slice_extent_str`, not the plain byte slice: this is the
            // boundary's own chance to refuse-and-name-the-book for invalid
            // UTF-8, rather than reaching a native call whose own
            // `DecodeError::InvalidUtf8` refusal carries no book identifier
            // at all (v0.1.5, bytes-at-boundary convention).
            let Some(source) = crate::bytes::slice_extent_str(sources, record.source) else {
                return RestoreOutcome::refused(RestoreError::InvalidExtent { book: record.path });
            };
            native.push(braid::RestoreRecord {
                path: record.path,
                packed: packed.to_vec(),
                source: source.as_bytes().to_vec(),
            });
        }
        RestoreOutcome::from(
            self.inner
                .restore_packed_books(&native)
                .map(RestoreReport::from)
                .map_err(RestoreError::from),
        )
    }

    /// Publishes the resident corpus as one packed `corpus.bin` container.
    ///
    /// A thin projection of `PublicationCache::publish` (this handle's own
    /// cache, so a repeat publish gets the adapter's whole point -- splice-
    /// reuse of whatever did not change -- automatically): dirty books are
    /// linted first (the adapter's own rule, via the `lint()` it runs
    /// internally), every book's bytes and stamps decide reuse vs. re-encode,
    /// and the reuse-cache's own sections/bytes never cross this boundary --
    /// only the per-book bookkeeping in [`PublishedBookInfo`] does.
    //
    // The composition itself, and the `PublicationCache` it reuses across
    // calls, are `braid::Braid`'s own -- Rust-first, primary API -- not this
    // crate's. `PublishedCorpus`/`PublishedBookInfo`/`PublishError` *are*
    // braid's own types (re-exported, tsify-derived via its `wasm` feature),
    // not a second, wasm-only mirror of them, so there is nothing left to
    // convert in this one-line delegation.
    #[wasm_bindgen(js_name = publish)]
    pub fn publish(&mut self) -> PublishOutcome {
        PublishOutcome::from(self.inner.publish())
    }

    /// Publishes exactly the books a scope names, as per-book packed
    /// containers -- the exact shape `restoreCorpus` consumes, never
    /// `PublishedCorpus`-shaped. Every returned book is always freshly
    /// encoded and always carries its source; there is no splice-reuse arm,
    /// and this call never reads or invalidates the handle's own
    /// `PublicationCache` (that cache is `publish`'s alone).
    //
    // The composition itself (`braid::Braid::publish_scope`) lives in braid.
    // This crate's own `ScopedPublication`/`ScopedPublishedBook` (defined
    // below) concatenate every in-scope book's owned `packed`/`source` into
    // the two buffers `restoreCorpus` takes verbatim -- see
    // `ScopedPublication`'s own doc comment for the key symmetry.
    #[wasm_bindgen(js_name = publishScope)]
    pub fn publish_scope(&mut self, scope: CorpusScope) -> ScopedPublishOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => {
                return ScopedPublishOutcome::refused(ScopedPublishError::Scope {
                    error: ScopeError::BookNotFound { book },
                });
            }
        };
        ScopedPublishOutcome::from(
            self.inner
                .publish_scope(native)
                .map(ScopedPublication::from)
                .map_err(ScopedPublishError::from),
        )
    }

    /// Restores the whole resident corpus from one packed `corpus.bin`
    /// container -- the corpus-grain counterpart to [`Self::publish`], as
    /// [`Self::restore_corpus`] is to a per-book publication.
    ///
    /// `packed` is the one whole-corpus container (a single `Uint8Array`
    /// argument, one memcpy). `sources` is every named book's source bytes
    /// concatenated into one buffer; `records` supplies each book's own
    /// declared code, its source key (a packed container names the book but
    /// never the key a corpus was originally addressed by), and its own
    /// extent into `sources` (v0.1.5, bytes-at-boundary convention -- see
    /// [`crate::bytes`]). An extent outside `sources`, or one whose own end
    /// overflows computing it, refuses by name
    /// (`RestoreError::InvalidExtent`, naming the record's own `book`)
    /// before any native call. Verification is corpus-wide (`verify_corpus`):
    /// every book must have exactly one source supplied, and findings that
    /// carry stamps must all carry the *same* stamps, checked atomically
    /// before anything installs.
    //
    // The composition itself (`braid::Braid::restore_published_corpus`)
    // lives in braid and is unchanged; this boundary resolves every record's
    // extent against `sources` and builds the native, per-book
    // `braid::PublishedCorpusSource` values that call actually takes (that
    // native type is deliberately not wasm/tsify-derived any more -- a
    // `source: Vec<u8>` field crossing wasm directly would itself be a JS
    // `number[]`). Its `RestoreReport`/`RestoreError` stay native
    // (`braid::RestoreReport`, `braid::IngestError` verbatim) rather than
    // tsify-derived, because their String-projected shape is this boundary's
    // business and not braid's -- so this converts into the wasm-facing
    // `RestoreReport`/`RestoreError` DTOs below, through the same conversions
    // `restore_corpus` uses.
    #[wasm_bindgen(js_name = restorePublishedCorpus)]
    pub fn restore_published_corpus(
        &mut self,
        packed: &[u8],
        sources: &[u8],
        records: Vec<PublishedCorpusRecord>,
    ) -> RestoreOutcome {
        let mut native = Vec::with_capacity(records.len());
        for record in records {
            let extent = crate::ByteExtent {
                byte_offset: record.byte_offset,
                byte_length: record.byte_length,
            };
            // `slice_extent_str`, for the same reason `restore_corpus`
            // uses it on its own source extent: refuse-and-name-the-book
            // here, rather than reaching a native call whose own decode
            // refusal would carry no book identifier.
            let Some(source) = crate::bytes::slice_extent_str(sources, extent) else {
                return RestoreOutcome::refused(RestoreError::InvalidExtent { book: record.book });
            };
            native.push(braid::PublishedCorpusSource {
                book: record.book,
                source_key: record.source_key,
                source: source.as_bytes().to_vec(),
            });
        }
        RestoreOutcome::from(
            self.inner
                .restore_published_corpus(packed, &native)
                .map(RestoreReport::from)
                .map_err(RestoreError::from),
        )
    }

    /// Replaces one book, or appends it when it is not resident yet.
    ///
    /// Whole-book replacement is the structural escape hatch: chapter insertion,
    /// deletion, reordering, and duplicate resolution all go through here.
    #[wasm_bindgen(js_name = updateBook)]
    pub fn update_book(&mut self, book: BookInput) -> MutationOutcome {
        let native = match book_into_native(book) {
            Ok(native) => native,
            Err(error) => return MutationOutcome::refused(error),
        };
        MutationOutcome::from(
            self.inner
                .update_book(native)
                .map(MutationEffect::from)
                .map_err(IngestError::from),
        )
    }

    /// Replaces exactly one existing chapter run with the caller's content.
    ///
    /// The replacement must be that same one run: no matching run is not found,
    /// several is ambiguous, and content that is a different or additional chapter
    /// is a label mismatch. The book's stored line ending is inherited.
    #[wasm_bindgen(js_name = updateChapter)]
    pub fn update_chapter(
        &mut self,
        target: ChapterTarget,
        replacement: ChapterInput,
    ) -> MutationOutcome {
        let target = match target_into_native(target) {
            Ok(target) => target,
            Err(error) => return MutationOutcome::refused(error),
        };
        let ChapterInput::Tokens { tokens } = replacement;
        let tokens = match tokens_into_native(tokens) {
            Ok(tokens) => tokens,
            Err(error) => return MutationOutcome::refused(error),
        };
        MutationOutcome::from(
            self.inner
                .update_chapter(target, braid::ChapterInput::Tokens(tokens))
                .map(MutationEffect::from)
                .map_err(IngestError::from),
        )
    }

    /// Removes a book. Removing an absent book is a no-op, not an error: the
    /// requested end state already holds.
    #[wasm_bindgen(js_name = removeBook)]
    pub fn remove_book(&mut self, book: String) -> MutationOutcome {
        let book = match book_id(&book) {
            Ok(book) => book,
            Err(error) => return MutationOutcome::refused(error),
        };
        MutationOutcome::from(Ok(self.inner.remove_book(book).into()))
    }

    /// Removes one chapter run's tokens from its book. The effect is whole-book:
    /// the address the caller used no longer exists.
    #[wasm_bindgen(js_name = removeChapter)]
    pub fn remove_chapter(&mut self, target: ChapterTarget) -> ScopedMutationOutcome {
        // A book code this library cannot even read names no resident book, so the
        // caller gets the same refusal an absent book gives — carrying the code it
        // actually sent, which is the only part it can act on.
        let unreadable = target.book.clone();
        let target = match target_into_native(target) {
            Ok(target) => target,
            Err(_) => {
                return ScopedMutationOutcome::refused(ScopeError::BookNotFound {
                    book: unreadable,
                });
            }
        };
        ScopedMutationOutcome::from(
            self.inner
                .remove_chapter(target)
                .map(MutationEffect::from)
                .map_err(ScopeError::from),
        )
    }

    /// Drops every resident book. Clearing an empty corpus is a no-op.
    pub fn clear(&mut self) -> MutationEffect {
        self.inner.clear().into()
    }

    /// Replaces the resident configuration.
    ///
    /// No tokens are rewritten, so nothing needs re-pulling and the identity — which
    /// covers source bytes only — is unchanged. What changes is staleness: every
    /// book is marked for recompute, because the configuration its cached findings
    /// were produced under no longer applies.
    #[wasm_bindgen(js_name = updateConfig)]
    pub fn update_config(&mut self, config: BraidConfig) -> MutationEffect {
        self.inner.update_config(config.into_native()).into()
    }

    /// Records one book's baseline — the state later comparisons are against.
    ///
    /// Only for a book that is already resident: a baseline is what the *current*
    /// state is compared against, so installing one for a book with no current
    /// state would invent the comparison rather than record it.
    #[wasm_bindgen(js_name = setBaseline)]
    pub fn set_baseline(&mut self, book: BookInput) -> BaselineMutationOutcome {
        let native = match book_into_native(book) {
            Ok(native) => native,
            Err(error) => {
                return BaselineMutationOutcome::refused(SetBaselineError::Invalid { error });
            }
        };
        BaselineMutationOutcome::from(
            self.inner
                .set_baseline(native)
                .map(MutationEffect::from)
                .map_err(SetBaselineError::from),
        )
    }

    /// Forgets one book's baseline. Clearing an absent one is a no-op.
    #[wasm_bindgen(js_name = clearBaseline)]
    pub fn clear_baseline(&mut self, book: String) -> MutationOutcome {
        let book = match book_id(&book) {
            Ok(book) => book,
            Err(error) => return MutationOutcome::refused(error),
        };
        MutationOutcome::from(Ok(self.inner.clear_baseline(book).into()))
    }

    // ---- lint and patches ----------------------------------------------

    /// Recomputes every book awaiting it and returns the complete snapshot.
    ///
    /// The only recompute verb, and always explicit: no mutation lints implicitly
    /// and no effect carries findings. Exactly the stale books run rules — a clean
    /// corpus runs none.
    pub fn lint(&mut self) -> LintSnapshot {
        let snapshot = self.inner.lint();
        LintSnapshot {
            snapshot_id: format!("{:016x}", snapshot.id.0),
            summary: crate::dto::map_lint_summary(snapshot.summary.clone()),
            books: snapshot
                .books
                .iter()
                .map(|book| BookLintSnapshot {
                    source_key: book.source_key.as_str().to_string(),
                    book: book.book.as_str().to_string(),
                    source_hash: format!("{:016x}", book.source_hash.0),
                    token_identity: format!("{:016x}", book.token_identity.0),
                    findings: book
                        .result
                        .issues
                        .iter()
                        .cloned()
                        .map(crate::dto::map_lint_issue)
                        .collect(),
                    summary: crate::dto::map_lint_summary(book.result.summary.clone()),
                })
                .collect(),
        }
    }

    /// Every patch of the current snapshot, in corpus order and then each book's own
    /// canonical finding order — which is what assigns each one its ordinal.
    ///
    /// A book awaiting recompute contributes none: its stored positions address the
    /// token stream it held when its findings were computed.
    pub fn patches(&self) -> Vec<Patch> {
        self.inner.patches().into_iter().map(Patch::from).collect()
    }

    /// One patch by id, refusing a stale or unknown one.
    pub fn patch(&self, id: PatchId) -> PatchOutcome {
        let native = match id.clone().into_native() {
            Ok(native) => native,
            Err(()) => return PatchOutcome::refused(unknown_patch(id)),
        };
        PatchOutcome::from(
            self.inner
                .patch(native)
                .map(Patch::from)
                .map_err(PatchError::from),
        )
    }

    /// The token stream the patch would produce, without applying it.
    ///
    /// A preview is a projection and is never admitted to residency, so it mints
    /// nothing: a surviving token carries the id it already had, and a token the fix
    /// would synthesize carries none until an apply grants it one.
    #[wasm_bindgen(js_name = previewPatch)]
    pub fn preview_patch(&self, id: PatchId) -> PatchPreviewOutcome {
        let native = match id.clone().into_native() {
            Ok(native) => native,
            Err(()) => return PatchPreviewOutcome::refused(unknown_patch(id)),
        };
        PatchPreviewOutcome::from(
            self.inner
                .preview_patch(native)
                .map(|tokens| tokens.iter().map(crate::dto::map_format_token).collect())
                .map_err(PatchError::from),
        )
    }

    /// Applies a patch as an ordinary mutation, atomically.
    #[wasm_bindgen(js_name = applyPatch)]
    pub fn apply_patch(&mut self, id: PatchId) -> PatchMutationOutcome {
        let native = match id.clone().into_native() {
            Ok(native) => native,
            Err(()) => return PatchMutationOutcome::refused(unknown_patch(id)),
        };
        PatchMutationOutcome::from(
            self.inner
                .apply_patch(native)
                .map(MutationEffect::from)
                .map_err(PatchError::from),
        )
    }

    /// Prepares a formatting pass over a scope without applying it.
    #[wasm_bindgen(js_name = prepareFormatPatch)]
    pub fn prepare_format_patch(
        &mut self,
        scope: CorpusScope,
        options: Option<crate::FormatOptions>,
    ) -> FormatPreparationOutcome {
        let scope = match scope_into_native(scope) {
            Ok(scope) => scope,
            Err(book) => {
                return FormatPreparationOutcome::refused(FormatError::Scope {
                    error: ScopeError::BookNotFound { book },
                });
            }
        };
        FormatPreparationOutcome::from(
            self.inner
                .prepare_format_patch(scope, crate::dto::format_options_into_native(options))
                .map(PatchPreparation::from)
                .map_err(FormatError::from),
        )
    }

    /// Applies a prepared format patch. All-or-nothing across every book it covers.
    #[wasm_bindgen(js_name = applyFormatPatch)]
    pub fn apply_format_patch(&mut self, id: FormatPatchId) -> FormatMutationOutcome {
        let native = match id.clone().into_native() {
            Ok(native) => native,
            Err(()) => {
                return FormatMutationOutcome::refused(FormatPatchError::UnknownPatch { id });
            }
        };
        FormatMutationOutcome::from(
            self.inner
                .apply_format_patch(native)
                .map(MutationEffect::from)
                .map_err(FormatPatchError::from),
        )
    }

    // ---- reads ---------------------------------------------------------

    /// Resident books with their derived stamps, in corpus order.
    pub fn books(&self) -> Vec<BookEntry> {
        self.inner
            .books()
            .into_iter()
            .map(|entry| BookEntry {
                source_key: entry.source_key.as_str().to_string(),
                book: entry.book.as_str().to_string(),
                source_hash: format!("{:016x}", entry.source_hash.0),
                token_identity: format!("{:016x}", entry.token_identity.0),
                line_ending: entry.line_ending.into(),
            })
            .collect()
    }

    /// One book's chapter-run labels in source order, duplicates included.
    #[wasm_bindgen(js_name = chapterLabels)]
    pub fn chapter_labels(&self, book: String) -> ChapterLabelsOutcome {
        let book = match usfm_onion::token::BookId::from_str(&book) {
            Some(book) => book,
            None => {
                return ChapterLabelsOutcome::refused(ScopeError::BookNotFound { book });
            }
        };
        ChapterLabelsOutcome::from(
            self.inner
                .chapter_labels(book)
                .map(|labels| labels.iter().map(ChapterLabel::from).collect())
                .map_err(ScopeError::from),
        )
    }

    /// Current tokens for the requested scopes — the single hydration verb.
    ///
    /// Returns current truth, not state as of any earlier effect. The input is
    /// normalized first (duplicates collapse, a whole-book scope absorbs that
    /// book's chapter scopes), so concatenating several effects' `changed` lists is
    /// always correct.
    #[wasm_bindgen(js_name = toTokens)]
    pub fn to_tokens(&self, scopes: Vec<Scope>) -> ScopeTokensOutcome {
        let mut native = Vec::with_capacity(scopes.len());
        for scope in scopes {
            let book = match usfm_onion::token::BookId::from_str(&scope.book) {
                Some(book) => book,
                None => {
                    return ScopeTokensOutcome::refused(ScopeError::BookNotFound {
                        book: scope.book,
                    });
                }
            };
            native.push(match scope.chapter {
                None => braid::Scope::book(book),
                Some(label) => braid::Scope::chapter(book, label.into()),
            });
        }
        ScopeTokensOutcome::from(
            self.inner
                .to_tokens(native)
                .map(|scopes| {
                    scopes
                        .into_iter()
                        .map(|scope| ScopeTokens {
                            book: scope.book.as_str().to_string(),
                            chapter: scope.chapter.as_ref().map(ChapterLabel::from),
                            tokens: scope
                                .tokens
                                .iter()
                                .map(crate::dto::map_owned_token)
                                .collect(),
                        })
                        .collect()
                })
                .map_err(ScopeError::from),
        )
    }

    /// The exact bytes a scope would be saved as.
    #[wasm_bindgen(js_name = toUsfm)]
    pub fn to_usfm(&self, scope: CorpusScope) -> UsfmOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => return UsfmOutcome::refused(ScopeError::BookNotFound { book }),
        };
        UsfmOutcome::from(
            self.inner
                .to_usfm(native)
                .map(|output| scoped_out(output, |value| value))
                .map_err(ScopeError::from),
        )
    }

    /// Whether a scope differs from its baseline, by exact serialized equality.
    #[wasm_bindgen(js_name = isDirty)]
    pub fn is_dirty(&self, scope: CorpusScope) -> DirtyOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => return DirtyOutcome::refused(ScopeError::BookNotFound { book }),
        };
        DirtyOutcome::from(self.inner.is_dirty(native).map_err(ScopeError::from))
    }

    /// The resident diff against the baseline.
    #[wasm_bindgen(js_name = diffBaseline)]
    pub fn diff_baseline(&self, scope: CorpusScope) -> DiffBaselineOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => {
                return DiffBaselineOutcome::refused(BaselineError::Scope {
                    error: ScopeError::BookNotFound { book },
                });
            }
        };
        DiffBaselineOutcome::from(
            self.inner
                .diff_baseline(native)
                .map(|output| {
                    scoped_out(output, |skeleton| {
                        crate::dto::map_native_skeleton(
                            &skeleton,
                            crate::dto::map_owned_token,
                            usfm_onion::diff::TextDiffMode::None,
                        )
                    })
                })
                .map_err(BaselineError::from),
        )
    }

    /// Whole-book replacement from each targeted book's own declared
    /// baseline, atomic across the scope. `all`/`book` scopes only -- a
    /// chapter scope refuses via `BaselineError.chapterScopeUnsupported`
    /// rather than reverting one run in isolation (use `diffBaseline` plus
    /// `updateChapter` with the baseline run's own tokens instead).
    ///
    /// Atomicity: every targeted book must be resident and baselined before
    /// anything mutates -- any missing baseline refuses with every offender
    /// named, and resident state is left exactly as it was. A book already
    /// equal to its baseline is a no-op, absent from `changed`.
    #[wasm_bindgen(js_name = revertToBaseline)]
    pub fn revert_to_baseline(&mut self, scope: CorpusScope) -> RevertBaselineOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => {
                return RevertBaselineOutcome::refused(BaselineError::Scope {
                    error: ScopeError::BookNotFound { book },
                });
            }
        };
        RevertBaselineOutcome::from(
            self.inner
                .revert_to_baseline(native)
                .map(MutationEffect::from)
                .map_err(BaselineError::from),
        )
    }

    /// Declares each in-scope book's CURRENT resident state as its baseline
    /// -- no re-parse, no `BookInput`: the bulk, no-parse counterpart to
    /// `setBaseline`. `all`/`book` scopes only, deliberately symmetric with
    /// `revertToBaseline` (a baseline is a whole-book slot, so the set and
    /// revert halves of its lifecycle agree on what scopes can address it);
    /// a chapter scope refuses the same way. Idempotent, and there is no
    /// missing-baseline case -- this verb's whole point is to create one.
    #[wasm_bindgen(js_name = setBaselineToCurrent)]
    pub fn set_baseline_to_current(&mut self, scope: CorpusScope) -> RevertBaselineOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => {
                return RevertBaselineOutcome::refused(BaselineError::Scope {
                    error: ScopeError::BookNotFound { book },
                });
            }
        };
        RevertBaselineOutcome::from(
            self.inner
                .set_baseline_to_current(native)
                .map(MutationEffect::from)
                .map_err(BaselineError::from),
        )
    }

    /// Every verse's lossless text projection for a scope, in document order.
    ///
    /// The resident answer to what the stateless projection computes from scratch:
    /// identical entries, but a read after a one-chapter edit recomputes only that
    /// chapter and takes the rest from cache — which is what makes this callable on
    /// a keystroke instead of once a document.
    ///
    /// Entries are `[sid, projection]` pairs in first-seen token order, the same
    /// shape the stateless `vrefIndexUsfm`/`vrefIndexTokens` exports return: one
    /// authoritative sequence, since an object keyed by sid enumerates its keys
    /// sorted and would silently reorder a document that is deliberately not.
    #[wasm_bindgen(js_name = vrefIndex)]
    pub fn vref_index(&mut self, scope: CorpusScope) -> VrefIndexOutcome {
        let native = match scope_into_native(scope) {
            Ok(native) => native,
            Err(book) => return VrefIndexOutcome::refused(ScopeError::BookNotFound { book }),
        };
        VrefIndexOutcome::from(
            self.inner
                .vref_index(native)
                .map(|output| {
                    scoped_out(output, |entries| {
                        crate::VrefIndex(
                            entries
                                .into_iter()
                                .map(|entry| {
                                    (
                                        entry.sid,
                                        crate::dto::map_verse_projection(entry.projection),
                                    )
                                })
                                .collect(),
                        )
                    })
                })
                .map_err(ScopeError::from),
        )
    }

    /// Books whose findings are stale, in corpus order. Derived from authoritative
    /// stamps rather than drained from a queue, so reading it twice is safe.
    #[wasm_bindgen(js_name = booksAwaitingLint)]
    pub fn books_awaiting_lint(&self) -> Vec<String> {
        self.inner
            .books_awaiting_lint()
            .into_iter()
            .map(|book| book.as_str().to_string())
            .collect()
    }

    /// The corpus's content-derived identity, as a 16-digit hex string.
    ///
    /// Hex rather than a number because the value is 64 bits: a JS `number` cannot
    /// hold it without silently rounding, and a `bigint` does not survive every
    /// structured clone a worker boundary performs.
    #[wasm_bindgen(js_name = expectedSnapshotId)]
    pub fn expected_snapshot_id(&self) -> String {
        format!("{:016x}", self.inner.expected_snapshot_id().0)
    }
}

fn unknown_patch(id: PatchId) -> PatchError {
    PatchError::UnknownPatch { id }
}

fn book_id(code: &str) -> Result<usfm_onion::token::BookId, IngestError> {
    usfm_onion::token::BookId::from_str(code).ok_or_else(|| IngestError::DuplicateBook {
        book: code.to_string(),
        sources: Vec::new(),
    })
}

/// A scope's book code, or the code that could not be read — which is not a
/// resident book by definition, so the caller gets the same refusal it would get
/// for a book that is simply absent.
fn scope_into_native(scope: CorpusScope) -> Result<braid::CorpusScope, String> {
    match scope {
        CorpusScope::All => Ok(braid::CorpusScope::All),
        CorpusScope::Book { book } => usfm_onion::token::BookId::from_str(&book)
            .map(braid::CorpusScope::Book)
            .ok_or(book),
        CorpusScope::Chapter { target } => {
            let book = target.book.clone();
            usfm_onion::token::BookId::from_str(&target.book)
                .map(|id| {
                    braid::CorpusScope::Chapter(braid::ChapterTarget::new(id, target.label.into()))
                })
                .ok_or(book)
        }
    }
}

fn target_into_native(target: ChapterTarget) -> Result<braid::ChapterTarget, IngestError> {
    Ok(braid::ChapterTarget::new(
        book_id(&target.book)?,
        target.label.into(),
    ))
}

/// The caller's tokens as resident ones, refusing the whole array if any single
/// token cannot be one — a half-converted book is not a book.
fn tokens_into_native(
    tokens: Vec<crate::Token>,
) -> Result<Vec<usfm_onion::token::OwnedToken>, IngestError> {
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| {
            usfm_onion_wire::dto::owned_token_from_dto(token, index as u32).map_err(|error| {
                IngestError::InvalidToken {
                    message: error.to_string(),
                }
            })
        })
        .collect()
}

fn source_key(value: String) -> Result<braid::SourceKey, IngestError> {
    braid::SourceKey::new(value.clone()).ok_or(IngestError::DuplicateSourceKey { source: value })
}

fn book_into_native(book: BookInput) -> Result<braid::BookInput, IngestError> {
    match book {
        BookInput::Usfm {
            source_key: key,
            book,
            source,
        } => Ok(braid::BookInput::Usfm {
            source_key: source_key(key)?,
            book: book_id(&book)?,
            source,
        }),
        BookInput::Tokens {
            source_key: key,
            book,
            tokens,
            line_ending,
        } => Ok(braid::BookInput::Tokens(braid::BookTokensInput {
            source_key: source_key(key)?,
            book: book_id(&book)?,
            tokens: tokens_into_native(tokens)?,
            line_ending: line_ending.into(),
        })),
    }
}

fn corpus_into_native(corpus: CorpusInput) -> Result<braid::CorpusInput, IngestError> {
    Ok(braid::CorpusInput::new(
        corpus
            .books
            .into_iter()
            .map(book_into_native)
            .collect::<Result<Vec<_>, _>>()?,
    ))
}

// ---------------------------------------------------------------------------
// Inputs. Discriminated unions, not optional bags: a book arriving as bytes and
// a book arriving as tokens are different inputs with different obligations, and
// a shape that could be neither is one a caller can build by accident.
// ---------------------------------------------------------------------------

/// How a book's exact bytes end their lines when they have to be re-emitted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl From<LineEnding> for braid::LineEnding {
    fn from(value: LineEnding) -> Self {
        match value {
            LineEnding::Lf => Self::Lf,
            LineEnding::Crlf => Self::CrLf,
        }
    }
}

impl From<braid::LineEnding> for LineEnding {
    fn from(value: braid::LineEnding) -> Self {
        match value {
            braid::LineEnding::Lf => Self::Lf,
            braid::LineEnding::CrLf => Self::Crlf,
        }
    }
}

/// One book's worth of resident input.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
// `rename_all` on an enum renames variant tags only, never a struct variant's
// own field names (a genuine cross-boundary bug this crate's `TokenFix` DTO
// already had to learn: found here by the RFC parity generator, which uses
// this same `Serialize` impl to build its argument fixtures — the generated
// `.d.ts` had `source_key`/`line_ending` while every sibling multi-word field
// elsewhere in this file was already camelCase). `rename_all_fields` is the
// separate attribute that actually covers per-variant field names.
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum BookInput {
    /// Cold load: exact USFM bytes, kept verbatim.
    Usfm {
        source_key: String,
        book: String,
        source: String,
    },
    /// Live push: the caller's own token array, which is the only moment the
    /// caller knows something the corpus does not.
    Tokens {
        source_key: String,
        book: String,
        tokens: Vec<crate::Token>,
        line_ending: LineEnding,
    },
}

/// A complete ordered corpus. Caller order is preserved exactly.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct CorpusInput {
    pub books: Vec<BookInput>,
}

/// A chapter run's label, exactly as the source spells it.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChapterLabel {
    FrontMatter,
    /// The label token verbatim: `1`, `01`, and `1a` are three distinct labels,
    /// because nothing here parses or normalizes one.
    Number {
        label: String,
    },
}

impl From<ChapterLabel> for braid::ChapterLabel {
    fn from(value: ChapterLabel) -> Self {
        match value {
            ChapterLabel::FrontMatter => Self::FrontMatter,
            ChapterLabel::Number { label } => Self::Number(label.into()),
        }
    }
}

impl From<&braid::ChapterLabel> for ChapterLabel {
    fn from(value: &braid::ChapterLabel) -> Self {
        match value {
            braid::ChapterLabel::FrontMatter => Self::FrontMatter,
            braid::ChapterLabel::Number(label) => Self::Number {
                label: label.to_string(),
            },
        }
    }
}

/// One chapter run's address.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ChapterTarget {
    pub book: String,
    pub label: ChapterLabel,
}

/// One chapter run's replacement content.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ChapterInput {
    Tokens { tokens: Vec<crate::Token> },
}

/// A read or projection selector over resident data.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CorpusScope {
    All,
    Book { book: String },
    Chapter { target: ChapterTarget },
}

// ---------------------------------------------------------------------------
// Outputs.
// ---------------------------------------------------------------------------

/// What one mutation rewrote. `chapter` absent means the whole book.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    pub book: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<ChapterLabel>,
}

/// The value every mutating verb returns, after it has already applied.
///
/// `changed` is exact — what was rewritten, not what was inspected — so an empty
/// one means nothing needs re-pulling. Findings are absent by design: lint is an
/// explicit separate call.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct MutationEffect {
    /// The corpus identity *after* the mutation, as 16 hex digits.
    pub snapshot_id: String,
    pub changed: Vec<Scope>,
    pub removed: Vec<String>,
    /// The full new book order, when the relative order of the books present both
    /// before and after actually changed. A pure reorder rewrites no tokens, so it
    /// appears here and nowhere else.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reordered: Option<Vec<String>>,
}

impl From<braid::MutationEffect> for MutationEffect {
    fn from(effect: braid::MutationEffect) -> Self {
        Self {
            snapshot_id: format!("{:016x}", effect.snapshot_id.0),
            changed: effect.changed.iter().map(scope_out).collect(),
            removed: effect
                .removed
                .iter()
                .map(|book| book.as_str().to_string())
                .collect(),
            reordered: effect
                .reordered
                .as_ref()
                .map(|order| order.iter().map(|book| book.as_str().to_string()).collect()),
        }
    }
}

fn scope_out(scope: &braid::Scope) -> Scope {
    Scope {
        book: scope.book.as_str().to_string(),
        chapter: scope.chapter.as_ref().map(ChapterLabel::from),
    }
}

/// One resident book's identity and derived stamps, in corpus order.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BookEntry {
    pub source_key: String,
    pub book: String,
    /// 16 hex digits over the book's exact bytes.
    pub source_hash: String,
    /// 16 hex digits over everything the book's tokens carry that its bytes do
    /// not — the fact a consumer caching anything token-derived keys on.
    pub token_identity: String,
    pub line_ending: LineEnding,
}

/// One pulled scope's current tokens.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ScopeTokens {
    pub book: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chapter: Option<ChapterLabel>,
    pub tokens: Vec<crate::Token>,
}

/// A projection over one scope, or over every resident book in corpus order.
///
/// Ordered pairs for the `all` case rather than an object keyed by source key:
/// corpus order is a contract, and an object's key enumeration is not.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScopedOutput<T> {
    Single { value: T },
    All { books: Vec<SourceOutput<T>> },
}

/// One book's value in an `all`-scoped projection.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SourceOutput<T> {
    pub source_key: String,
    pub book: String,
    pub value: T,
}

fn scoped_out<N, T>(value: braid::ScopedOutput<N>, map: impl Fn(N) -> T) -> ScopedOutput<T> {
    match value {
        braid::ScopedOutput::Single(value) => ScopedOutput::Single { value: map(value) },
        braid::ScopedOutput::All(books) => ScopedOutput::All {
            books: books
                .into_iter()
                .map(|entry| SourceOutput {
                    source_key: entry.source_key.as_str().to_string(),
                    book: entry.book.as_str().to_string(),
                    value: map(entry.value),
                })
                .collect(),
        },
    }
}

/// One book's lint contribution.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct BookLintSnapshot {
    pub source_key: String,
    pub book: String,
    pub source_hash: String,
    pub token_identity: String,
    pub findings: Vec<crate::LintIssue>,
    pub summary: crate::LintSummary,
}

/// The complete resident lint snapshot, in corpus order.
///
/// Findings, not packed bytes: a finding's message is rendered by exactly one
/// renderer in one language, so findings cross this boundary already materialized.
/// Packed bytes are a separate question with a separate verb.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LintSnapshot {
    pub snapshot_id: String,
    pub books: Vec<BookLintSnapshot>,
    pub summary: crate::LintSummary,
}

// ---------------------------------------------------------------------------
// Typed refusals. Each carries the same information the Rust crate's error does,
// as a discriminated union a consumer can switch on.
// ---------------------------------------------------------------------------

/// A mutation refused before it touched resident state.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IngestError {
    DuplicateBook {
        book: String,
        sources: Vec<String>,
    },
    DuplicateSourceKey {
        source: String,
    },
    DuplicateTokenId {
        book: String,
        id: String,
    },
    ChapterNotFound {
        target: ChapterTarget,
    },
    AmbiguousChapter {
        target: ChapterTarget,
        matches: usize,
    },
    ReplacementLabelMismatch {
        target: ChapterTarget,
        found: ChapterLabel,
    },
    /// A token the caller supplied cannot become resident. `message` is core's own
    /// verdict, which names the token and the fact that made it illegal.
    InvalidToken {
        message: String,
    },
}

fn target_out(target: &braid::ChapterTarget) -> ChapterTarget {
    ChapterTarget {
        book: target.book.as_str().to_string(),
        label: ChapterLabel::from(&target.label),
    }
}

impl From<braid::IngestError> for IngestError {
    fn from(error: braid::IngestError) -> Self {
        match error {
            braid::IngestError::DuplicateBook { book, sources } => Self::DuplicateBook {
                book: book.as_str().to_string(),
                sources: sources.iter().map(|key| key.as_str().to_string()).collect(),
            },
            braid::IngestError::DuplicateSourceKey { source } => Self::DuplicateSourceKey {
                source: source.as_str().to_string(),
            },
            braid::IngestError::DuplicateTokenId { book, id } => Self::DuplicateTokenId {
                book: book.as_str().to_string(),
                id: id.as_str().to_string(),
            },
            braid::IngestError::ChapterNotFound(target) => Self::ChapterNotFound {
                target: target_out(&target),
            },
            braid::IngestError::AmbiguousChapter { target, matches } => Self::AmbiguousChapter {
                target: target_out(&target),
                matches,
            },
            braid::IngestError::ReplacementLabelMismatch { target, found } => {
                Self::ReplacementLabelMismatch {
                    target: target_out(&target),
                    found: ChapterLabel::from(&found),
                }
            }
            ref error @ braid::IngestError::InvalidToken(_) => Self::InvalidToken {
                message: error.to_string(),
            },
        }
    }
}

/// A scope that does not resolve against the resident corpus.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScopeError {
    BookNotFound {
        book: String,
    },
    ChapterNotFound {
        target: ChapterTarget,
    },
    AmbiguousChapter {
        target: ChapterTarget,
        matches: usize,
    },
}

impl From<braid::ScopeError> for ScopeError {
    fn from(error: braid::ScopeError) -> Self {
        match error {
            braid::ScopeError::BookNotFound(book) => Self::BookNotFound {
                book: book.as_str().to_string(),
            },
            braid::ScopeError::ChapterNotFound(target) => Self::ChapterNotFound {
                target: target_out(&target),
            },
            braid::ScopeError::AmbiguousChapter { target, matches } => Self::AmbiguousChapter {
                target: target_out(&target),
                matches,
            },
        }
    }
}

/// A patch that could not be looked up or applied.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PatchError {
    /// The patch was resolved against a different corpus than the resident one —
    /// either the identity moved, or the target book was rewritten since.
    StaleSnapshot {
        expected: String,
        found: String,
    },
    UnknownPatch {
        id: PatchId,
    },
    /// Applying it produced a token stream that cannot become resident.
    InvalidResult {
        error: IngestError,
    },
}

impl From<braid::PatchError> for PatchError {
    fn from(error: braid::PatchError) -> Self {
        match error {
            braid::PatchError::StaleSnapshot { expected, found } => Self::StaleSnapshot {
                expected: format!("{:016x}", expected.0),
                found: format!("{:016x}", found.0),
            },
            braid::PatchError::UnknownPatch(id) => Self::UnknownPatch { id: id.into() },
            braid::PatchError::InvalidResult(error) => Self::InvalidResult {
                error: error.into(),
            },
        }
    }
}

/// A prepared format patch that could not be looked up or applied.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FormatPatchError {
    StaleSnapshot {
        expected: String,
        found: String,
    },
    UnknownPatch {
        id: FormatPatchId,
    },
    /// A book the preparation targeted is no longer resident. Applying is
    /// all-or-nothing across the books it covered, so one missing book refuses the
    /// whole application rather than formatting the rest.
    BookNotResident {
        book: String,
    },
    InvalidResult {
        error: IngestError,
    },
}

impl From<braid::FormatPatchError> for FormatPatchError {
    fn from(error: braid::FormatPatchError) -> Self {
        match error {
            braid::FormatPatchError::StaleSnapshot { expected, found } => Self::StaleSnapshot {
                expected: format!("{:016x}", expected.0),
                found: format!("{:016x}", found.0),
            },
            braid::FormatPatchError::UnknownPatch(id) => Self::UnknownPatch { id: id.into() },
            braid::FormatPatchError::BookNotResident(book) => Self::BookNotResident {
                book: book.as_str().to_string(),
            },
            braid::FormatPatchError::InvalidResult(error) => Self::InvalidResult {
                error: error.into(),
            },
        }
    }
}

/// A scope that does not resolve, on the way to preparing a format patch.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FormatError {
    Scope { error: ScopeError },
}

impl From<braid::FormatError> for FormatError {
    fn from(error: braid::FormatError) -> Self {
        match error {
            braid::FormatError::Scope(error) => Self::Scope {
                error: error.into(),
            },
        }
    }
}

/// One book's packed-container extent plus the source extent it was bound
/// to -- both into the two buffers [`Braid::restore_corpus`] takes
/// alongside `records` (`packedAll`/`sources`), never an owned byte payload
/// of its own (v0.1.5, bytes-at-boundary convention). This is deliberately
/// the exact shape [`ScopedPublication::books`] emits, so a
/// [`Braid::publish_scope`] result forwards into a [`Braid::restore_corpus`]
/// call with no reshaping -- see [`ScopedPublication`]'s own doc comment for
/// the symmetry.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecord {
    /// The caller's own binding for where the book came from — normally a path.
    pub path: String,
    pub packed: crate::ByteExtent,
    pub source: crate::ByteExtent,
}

/// One book's source extent for [`Braid::restore_published_corpus`] -- into
/// the `sources` buffer that call takes alongside the one whole-corpus
/// `packed` container (which needs no extent of its own: it already crosses
/// as a single `Uint8Array`, the same reasoning
/// [`PublishedCorpus::bytes`]'s own doc comment gives).
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PublishedCorpusRecord {
    pub book: String,
    pub source_key: String,
    pub byte_offset: u32,
    pub byte_length: u32,
}

/// Why a warm restore was refused outright.
///
/// A refusal here is about the *call*: bytes that do not verify, or a corpus that
/// cannot be installed. A single book whose cached findings are not adoptable is not
/// a refusal — it seeds anyway and appears in the report's rejections.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RestoreError {
    Decode {
        error: crate::PackedDecodeError,
    },
    Ingest {
        error: IngestError,
    },
    /// A record's own `packed`/`source` extent falls outside the buffer it
    /// names, or overflows computing its own end -- refused by name rather
    /// than by clamping or truncating (v0.1.5, bytes-at-boundary
    /// convention; see [`crate::bytes`]). `book` is the record's own
    /// caller-supplied identifier: for [`Braid::restore_corpus`] that is its
    /// `path` (the book code is not known until the packed extent decodes),
    /// and for [`Braid::restore_published_corpus`] it is the record's own
    /// declared `book` code.
    InvalidExtent {
        book: String,
    },
}

impl From<braid::RestoreError> for RestoreError {
    fn from(error: braid::RestoreError) -> Self {
        match error {
            braid::RestoreError::Decode(error) => Self::Decode {
                error: error.into(),
            },
            braid::RestoreError::Ingest(error) => Self::Ingest {
                error: error.into(),
            },
            // Reproduces the pre-extraction classification exactly (an empty
            // source key was always reported the same way a caller-declared
            // duplicate one is, and the key it names is empty by definition
            // of the variant): see
            // `braid::RestoreError::EmptySourceKey`'s own doc
            // comment for why the native side can't express this as
            // `braid::IngestError::DuplicateSourceKey` itself.
            braid::RestoreError::EmptySourceKey => Self::Ingest {
                error: IngestError::DuplicateSourceKey {
                    source: String::new(),
                },
            },
        }
    }
}

/// Why one book's cached lint contribution was not adopted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum PrimeRejectReason {
    BookNotResident,
    SourceHashMismatch,
    ConfigFingerprintMismatch,
    EngineStampMismatch,
    InvalidPatch,
    /// The one reason that refuses residency too: the supplied tokens do not spell
    /// the supplied bytes, so there is nothing safe to install.
    SourceTokenMismatch,
}

impl From<braid::PrimeRejectReason> for PrimeRejectReason {
    fn from(reason: braid::PrimeRejectReason) -> Self {
        match reason {
            braid::PrimeRejectReason::BookNotResident => Self::BookNotResident,
            braid::PrimeRejectReason::SourceHashMismatch => Self::SourceHashMismatch,
            braid::PrimeRejectReason::ConfigFingerprintMismatch => Self::ConfigFingerprintMismatch,
            braid::PrimeRejectReason::EngineStampMismatch => Self::EngineStampMismatch,
            braid::PrimeRejectReason::InvalidPatch => Self::InvalidPatch,
            braid::PrimeRejectReason::SourceTokenMismatch => Self::SourceTokenMismatch,
        }
    }
}

/// One book a warm seed would not fully accept.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PrimeRejection {
    pub book: String,
    pub reason: PrimeRejectReason,
}

/// What one warm restore installed, and what it would not take.
///
/// A book can appear in both lists: residency and lint-priming are independent, so a
/// book whose cached findings were refused still seeds — with no lex or parse — and
/// is simply awaiting recompute.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub seeded: Vec<String>,
    pub rejected: Vec<PrimeRejection>,
}

impl From<braid::RestoreReport> for RestoreReport {
    fn from(report: braid::RestoreReport) -> Self {
        Self {
            seeded: report
                .seeded
                .iter()
                .map(|book| book.as_str().to_string())
                .collect(),
            rejected: report
                .rejected
                .into_iter()
                .map(|rejection| PrimeRejection {
                    book: rejection.book.as_str().to_string(),
                    reason: rejection.reason.into(),
                })
                .collect(),
        }
    }
}

// `PublishedBookInfo`/`PublishedCorpus`/`PublishError` are braid's own types
// (re-exported below), not a second, wasm-only mirror of them: braid's
// `wasm` feature already derives `Tsify` on them directly, so there is
// nothing for this crate to redefine. `PublishedCorpus::bytes` crosses as a
// plain `number[]`, same as ever -- see its own doc comment in braid for why
// `serde_bytes` was tried and reverted here (a `tsify` infrastructure
// limitation, not a design choice).
//
// `PublishedCorpusSource`/`ScopedPublication`/`ScopedPublishedBook` are NOT
// re-exported here (v0.1.5, bytes-at-boundary convention): each one embeds a
// raw `Vec<u8>`/`String` byte payload, so this crate now defines its own
// boundary DTOs for the scoped-publish and restore-record shapes below, and
// builds braid's native per-book types only as an internal step immediately
// before calling into it.
pub use braid::{PublishError, PublishedBookInfo, PublishedCorpus};

/// One buffer's extent, plus the book it belongs to -- what
/// [`ScopedPublication::books`] carries per book, boundary-only (see
/// [`crate::bytes`]'s own doc comment).
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ScopedPublishedBook {
    pub book: String,
    pub source_hash: String,
    pub packed: crate::ByteExtent,
    pub source: crate::ByteExtent,
}

/// What one [`Braid::publish_scope`] call produced, transfer-ready: every
/// in-scope book's packed container concatenated into `packed`, every
/// in-scope book's source concatenated (UTF-8) into `sources`, and
/// `books[].packed`/`books[].source` naming each book's own extent into
/// whichever of those two buffers it belongs to.
///
/// **Key symmetry, by construction:** this shape is byte-for-byte
/// [`Braid::restore_corpus`]'s input. `packed`/`sources` forward verbatim as
/// `restoreCorpus`'s first two arguments, and `books` forwards verbatim
/// (after trivial per-book field renaming, `sourceHash`/`packed`/`source` ->
/// `sourceKey`/`packed`/`source`) as its `records` -- zero reshaping, and
/// (see below) at most one wrap per buffer, never one per book.
///
/// `packed`/`sources` cross as plain `number[]`, not `Uint8Array`: this
/// crate's `tsify` dependency resolves its default `json` feature
/// (`JsValue::from_serde`), not its `js` feature (`serde-wasm-bindgen`,
/// which is what would honor a `#[serde(with = "serde_bytes")]` field
/// annotation) -- confirmed by an isolated repro, not assumed. Switching
/// features crate-wide would also flip every existing map-shaped DTO field
/// (`LintSummary`, `message_params`, `VrefMap`, ...) from a plain JS object
/// to an ES `Map`, a far larger and riskier migration than this verb alone
/// justifies. The win this convention still delivers without that
/// migration: ONE array per buffer regardless of how many books are in
/// scope (`new Uint8Array(scoped.packed)`, `new Uint8Array(scoped.sources)`
/// -- the same single wrap `PublishedCorpus::bytes` has always required),
/// never the O(books) `Array.from` a per-book byte array would have needed.
///
/// Native `braid::ScopedPublication` keeps its own per-book owned shape
/// (a native caller wants one owned value per book, not buffer
/// bookkeeping); this concatenation is the one honest transformation this
/// boundary performs, the same class of conversion [`MutationEffect::from`]
/// already does for every other verb.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ScopedPublication {
    pub snapshot_id: String,
    pub packed: Vec<u8>,
    pub sources: Vec<u8>,
    pub books: Vec<ScopedPublishedBook>,
}

impl From<braid::ScopedPublication> for ScopedPublication {
    fn from(value: braid::ScopedPublication) -> Self {
        let mut packed = Vec::new();
        let mut sources = Vec::new();
        let mut books = Vec::with_capacity(value.books.len());
        for book in value.books {
            let packed_start = packed.len() as u32;
            packed.extend_from_slice(&book.packed);
            let source_start = sources.len() as u32;
            sources.extend_from_slice(book.source.as_bytes());
            books.push(ScopedPublishedBook {
                book: book.book,
                source_hash: book.source_hash,
                packed: crate::ByteExtent {
                    byte_offset: packed_start,
                    byte_length: book.packed.len() as u32,
                },
                source: crate::ByteExtent {
                    byte_offset: source_start,
                    byte_length: book.source.len() as u32,
                },
            });
        }
        Self {
            snapshot_id: value.snapshot_id,
            packed,
            sources,
            books,
        }
    }
}

/// Why [`Braid::publish_scope`] could not produce a scoped publication.
///
/// Mirrors `braid::ScopedPublishError` rather than reusing it verbatim (unlike
/// [`PublishError`], which has no [`ScopeError`]-shaped variant to convert):
/// its `Scope` arm wraps braid's own native `ScopeError`, which -- like every
/// other scope-shaped error at this boundary -- projects to this crate's own
/// String-based [`ScopeError`] DTO rather than crossing with braid's `BookId`.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScopedPublishError {
    Scope {
        error: ScopeError,
    },
    Encode {
        error: usfm_onion_wire::dto::PackedEncodeError,
    },
}

impl From<braid::ScopedPublishError> for ScopedPublishError {
    fn from(error: braid::ScopedPublishError) -> Self {
        match error {
            braid::ScopedPublishError::Scope(error) => Self::Scope {
                error: error.into(),
            },
            braid::ScopedPublishError::Encode { error } => Self::Encode { error },
        }
    }
}

/// A baseline that could not be recorded.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SetBaselineError {
    /// The book is not resident, so there is no current state for a baseline to be
    /// the counterpart of.
    BookNotResident { book: String },
    /// The supplied book is not valid input in the first place.
    Invalid { error: IngestError },
}

impl From<braid::SetBaselineError> for SetBaselineError {
    fn from(error: braid::SetBaselineError) -> Self {
        match error {
            braid::SetBaselineError::BookNotResident(book) => Self::BookNotResident {
                book: book.as_str().to_string(),
            },
            braid::SetBaselineError::Invalid(error) => Self::Invalid {
                error: error.into(),
            },
        }
    }
}

/// A baseline comparison that cannot be answered.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BaselineError {
    Scope {
        error: ScopeError,
    },
    /// These books have no baseline, so there is nothing to compare against.
    MissingBaseline {
        books: Vec<String>,
    },
    /// [`Braid::revert_to_baseline`] supports only `all`/`book` scopes; a
    /// chapter scope refuses instead of reverting one run in isolation. Use
    /// `diffBaseline` plus `updateChapter` with the baseline run's own
    /// tokens instead.
    ChapterScopeUnsupported {
        target: ChapterTarget,
    },
}

impl From<braid::BaselineError> for BaselineError {
    fn from(error: braid::BaselineError) -> Self {
        match error {
            braid::BaselineError::Scope(error) => Self::Scope {
                error: error.into(),
            },
            braid::BaselineError::MissingBaseline { books } => Self::MissingBaseline {
                books: books.iter().map(|book| book.as_str().to_string()).collect(),
            },
            braid::BaselineError::ChapterScopeUnsupported(target) => {
                Self::ChapterScopeUnsupported {
                    target: target_out(&target),
                }
            }
        }
    }
}

/// A patch's snapshot-bound identity.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PatchId {
    /// The corpus identity the patch was resolved against, as 16 hex digits.
    pub snapshot: String,
    pub ordinal: u32,
}

impl From<braid::PatchId> for PatchId {
    fn from(id: braid::PatchId) -> Self {
        Self {
            snapshot: format!("{:016x}", id.snapshot.0),
            ordinal: id.ordinal,
        }
    }
}

impl PatchId {
    fn into_native(self) -> Result<braid::PatchId, ()> {
        Ok(braid::PatchId {
            snapshot: braid::SnapshotId(u64::from_str_radix(&self.snapshot, 16).map_err(|_| ())?),
            ordinal: self.ordinal,
        })
    }
}

/// A prepared format patch's snapshot-bound identity.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FormatPatchId {
    pub snapshot: String,
    pub ordinal: u32,
}

impl From<braid::FormatPatchId> for FormatPatchId {
    fn from(id: braid::FormatPatchId) -> Self {
        Self {
            snapshot: format!("{:016x}", id.snapshot.0),
            ordinal: id.ordinal,
        }
    }
}

impl FormatPatchId {
    fn into_native(self) -> Result<braid::FormatPatchId, ()> {
        Ok(braid::FormatPatchId {
            snapshot: braid::SnapshotId(u64::from_str_radix(&self.snapshot, 16).map_err(|_| ())?),
            ordinal: self.ordinal,
        })
    }
}

/// What one patch row does at its own position.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub enum PatchOp {
    /// Place the template immediately after the row's position; several inserts at
    /// one position place in row order.
    Insert,
    Replace,
    Delete,
}

/// One token operation. `position` addresses the token stream of the snapshot the
/// owning patch is bound to, never the post-patch stream.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PatchRow {
    pub op: PatchOp,
    pub position: u32,
    /// Absent exactly for a delete, which places nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<crate::TokenTemplate>,
}

/// One resolved fix, addressable and inspectable without applying it.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub id: PatchId,
    pub book: String,
    /// The target book's hash at resolution time — the second half of the
    /// staleness check, since a book can be rewritten and restored inside a corpus
    /// that hashes the same overall.
    pub source_hash: String,
    /// The fix's own remedy code, which is not the finding's lint code.
    pub code: String,
    pub label: String,
    pub label_params: std::collections::BTreeMap<String, String>,
    pub rows: Vec<PatchRow>,
}

impl From<braid::Patch> for Patch {
    fn from(patch: braid::Patch) -> Self {
        Self {
            id: patch.id.into(),
            book: patch.book.as_str().to_string(),
            source_hash: format!("{:016x}", patch.source_hash.0),
            code: patch.code,
            label: patch.label,
            label_params: patch.label_params,
            rows: patch
                .rows
                .into_iter()
                .map(|row| PatchRow {
                    op: match row.op {
                        braid::PatchOp::Insert => PatchOp::Insert,
                        braid::PatchOp::Replace => PatchOp::Replace,
                        braid::PatchOp::Delete => PatchOp::Delete,
                    },
                    position: row.position,
                    template: row.template.map(crate::dto::map_token_template),
                })
                .collect(),
        }
    }
}

/// Whether preparing a format patch found anything to change.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PatchPreparation {
    /// The scope is already formatted; nothing was stored and there is nothing to
    /// apply.
    Unchanged,
    Ready {
        id: FormatPatchId,
    },
}

impl From<braid::PatchPreparation> for PatchPreparation {
    fn from(preparation: braid::PatchPreparation) -> Self {
        match preparation {
            braid::PatchPreparation::Unchanged => Self::Unchanged,
            braid::PatchPreparation::Ready(id) => Self::Ready { id: id.into() },
        }
    }
}

#[cfg(test)]
mod restore_tests {
    use super::*;
    use braid::{
        BookInput as NativeBookInput, BraidConfig, CorpusInput as NativeCorpusInput, SourceKey,
    };
    use usfm_onion::lint::{LintOptions, LintScope};
    use usfm_onion_wire::corpus_codec::{
        CorpusSection, CorpusSectionInput, CorpusSectionTokens, EncodedCorpus, LintStamps,
        encode_corpus,
    };

    fn empty_resident() -> Braid {
        let mut next = 0u32;
        Braid {
            inner: NativeBraid::new(
                BraidConfig::new(LintOptions::scoped(LintScope::Book)),
                move || {
                    next += 1;
                    format!("minted-{next}")
                },
            ),
        }
    }

    fn book_id(code: &str) -> usfm_onion::token::BookId {
        usfm_onion::token::BookId::from_str(code).expect("three-character code")
    }

    /// One book's own packed bytes and the exact source they are bound to,
    /// stamped with `stamps` — a container carrying exactly one token
    /// section and one finding section, which is exactly what
    /// `verify_book`/`restoreCorpus`'s per-record shape expects (never a
    /// whole multi-book publication like `PublicationCache::publish`
    /// produces).
    fn encode_one_book(
        resident: &mut NativeBraid,
        book: usfm_onion::token::BookId,
        stamps: LintStamps,
    ) -> (Vec<u8>, String) {
        let snapshot = resident.lint();
        let found = snapshot
            .books
            .iter()
            .find(|entry| entry.book == book)
            .expect("book is resident");
        let EncodedCorpus { bytes, sources, .. } = encode_corpus(
            snapshot.id.0,
            Some(stamps),
            &[CorpusSection::Fresh(CorpusSectionInput {
                book,
                tokens: CorpusSectionTokens::Owned {
                    tokens: found.tokens,
                },
                findings: Some(found.result),
            })],
        )
        .expect("one book encodes");
        let source = sources
            .into_iter()
            .find(|(candidate, _)| *candidate == book)
            .expect("the book we just encoded has a source")
            .1;
        (bytes, source)
    }

    fn current_stamps(resident: &NativeBraid) -> LintStamps {
        LintStamps {
            config_fingerprint: braid::LintConfigFingerprint::of(&resident.config().lint).0,
            engine_stamp: braid::LintEngineStamp::current().0,
        }
    }

    /// Concatenates each entry's `packed`/`source` into the two buffers
    /// `restore_corpus` now takes, returning `(packed_all, sources, records)`
    /// with each record's own extents into them -- the same shape a real
    /// caller builds by hand from several `RestoreRecord`s.
    fn records_with_buffers(
        entries: &[(&str, Vec<u8>, &str)],
    ) -> (Vec<u8>, Vec<u8>, Vec<RestoreRecord>) {
        let mut packed_all = Vec::new();
        let mut sources = Vec::new();
        let mut records = Vec::with_capacity(entries.len());
        for (path, packed, source) in entries {
            let packed_start = packed_all.len() as u32;
            packed_all.extend_from_slice(packed);
            let source_start = sources.len() as u32;
            sources.extend_from_slice(source.as_bytes());
            records.push(RestoreRecord {
                path: path.to_string(),
                packed: crate::ByteExtent {
                    byte_offset: packed_start,
                    byte_length: packed.len() as u32,
                },
                source: crate::ByteExtent {
                    byte_offset: source_start,
                    byte_length: source.len() as u32,
                },
            });
        }
        (packed_all, sources, records)
    }

    /// Field-by-field, via exhaustive destructuring (no `..`) of both sides:
    /// `dto::LintSummary` derives no `PartialEq` (a boundary DTO, not a value
    /// type production code compares), but a plain per-field `assert_eq!`
    /// list is exactly the shape that let `suppressed_count` silently drop
    /// out of a prior version of this comparison. Destructuring means a new
    /// summary field fails to *compile* here until this helper accounts for
    /// it -- the same drift-proof discipline `hash_wire_identity` uses.
    fn assert_summaries_match(actual: crate::dto::LintSummary, expected: crate::dto::LintSummary) {
        let crate::dto::LintSummary {
            by_category: actual_by_category,
            by_severity: actual_by_severity,
            by_issue_type: actual_by_issue_type,
            total_count: actual_total_count,
            suppressed_count: actual_suppressed_count,
        } = actual;
        let crate::dto::LintSummary {
            by_category: expected_by_category,
            by_severity: expected_by_severity,
            by_issue_type: expected_by_issue_type,
            total_count: expected_total_count,
            suppressed_count: expected_suppressed_count,
        } = expected;
        assert_eq!(
            actual_total_count, expected_total_count,
            "total_count must match"
        );
        assert_eq!(
            actual_by_category, expected_by_category,
            "by_category must match"
        );
        assert_eq!(
            actual_by_severity, expected_by_severity,
            "by_severity must match"
        );
        assert_eq!(
            actual_by_issue_type, expected_by_issue_type,
            "by_issue_type must match"
        );
        assert_eq!(
            actual_suppressed_count, expected_suppressed_count,
            "suppressed_count must match -- packed bytes cannot carry it, so a \
             restore must either recompute it honestly or decline to prime a \
             summary at all, never claim a stale 0"
        );
    }

    /// P1.2: a warm reopen must report the same summary a live publish-then-
    /// lint of the same content would — never a zeroed placeholder with
    /// findings plainly present beside it.
    #[test]
    fn restore_corpus_recomputes_the_summary_from_the_restored_findings() {
        let mut original = empty_resident();
        original
            .inner
            .replace_corpus(NativeCorpusInput::new(vec![NativeBookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book_id("GEN"),
                source: "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n".to_string(),
            }]))
            .expect("one book");
        // The wasm-facing summary a live publish would show, for a like-for-
        // like comparison against the restored one below (both dto::LintSummary).
        let expected_summary = original.lint().summary;
        assert!(
            expected_summary.total_count > 0,
            "the fixture must actually carry a warning"
        );

        let stamps = current_stamps(&original.inner);
        let (packed, source) = encode_one_book(&mut original.inner, book_id("GEN"), stamps);

        let mut reopened = empty_resident();
        let (packed_all, sources, records) = records_with_buffers(&[("GEN.usfm", packed, &source)]);
        let outcome = reopened.restore_corpus(&packed_all, &sources, records);
        let ApiResult::Ok { value: report } = outcome.0 else {
            panic!("a fresh, matching-stamp restore must succeed: {outcome:?}");
        };
        assert_eq!(report.seeded, vec!["GEN".to_string()]);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        let snapshot = reopened.lint();
        assert_summaries_match(snapshot.summary, expected_summary);
    }

    /// P1.3: two records individually verify fine but carry different
    /// stamps (as if produced by two different rule-engine builds). The
    /// whole restore must refuse atomically — never adopt the first
    /// record's stamps for the second's findings — and leave the resident
    /// corpus exactly as it was before the call.
    #[test]
    fn restore_corpus_refuses_the_whole_batch_when_records_disagree_on_stamps() {
        let mut source_a = empty_resident();
        source_a
            .inner
            .replace_corpus(NativeCorpusInput::new(vec![NativeBookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book_id("GEN"),
                source: "\\id GEN\n\\c 1\n\\p\n\\v 1 In the beginning.\\p\n".to_string(),
            }]))
            .expect("one book");
        let stamps_a = current_stamps(&source_a.inner);
        let (packed_gen, source_gen) =
            encode_one_book(&mut source_a.inner, book_id("GEN"), stamps_a);

        let mut source_b = empty_resident();
        source_b
            .inner
            .replace_corpus(NativeCorpusInput::new(vec![NativeBookInput::Usfm {
                source_key: SourceKey::new("EXO.usfm").unwrap(),
                book: book_id("EXO"),
                source: "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n".to_string(),
            }]))
            .expect("one book");
        // A stamp that cannot possibly match `stamps_a`: the real engine
        // stamp perturbed by one bit is still "some other build produced
        // this", which is all the test needs — two individually-valid
        // records that disagree with each other.
        let stamps_b = LintStamps {
            config_fingerprint: stamps_a.config_fingerprint,
            engine_stamp: stamps_a.engine_stamp ^ 1,
        };
        let (packed_exo, source_exo) =
            encode_one_book(&mut source_b.inner, book_id("EXO"), stamps_b);

        let mut reopened = empty_resident();
        let before_books: Vec<String> = reopened
            .books()
            .into_iter()
            .map(|entry| entry.book)
            .collect();
        let (packed_all, sources, records) = records_with_buffers(&[
            ("GEN.usfm", packed_gen, &source_gen),
            ("EXO.usfm", packed_exo, &source_exo),
        ]);
        let outcome = reopened.restore_corpus(&packed_all, &sources, records);
        assert!(
            matches!(
                outcome.0,
                ApiResult::Error {
                    error: RestoreError::Decode {
                        error: crate::PackedDecodeError::InvalidSection
                    }
                }
            ),
            "disagreeing stamps must refuse the whole batch typed, got {outcome:?}"
        );
        let after_books: Vec<String> = reopened
            .books()
            .into_iter()
            .map(|entry| entry.book)
            .collect();
        assert_eq!(
            after_books, before_books,
            "a refused restore must leave resident state exactly as it was"
        );
        assert!(before_books.is_empty(), "the fresh handle started empty");
    }

    fn suppressing_resident() -> Braid {
        let mut next = 0u32;
        let mut options = LintOptions::scoped(LintScope::Book);
        options.suppressed = vec![usfm_onion::lint::LintSuppression {
            code: usfm_onion::lint::LintCode::DuplicateVerseNumber,
            sid: "GEN 1:1".to_string(),
        }];
        Braid {
            inner: NativeBraid::new(BraidConfig::new(options), move || {
                next += 1;
                format!("minted-{next}")
            }),
        }
    }

    /// P1-B: packed bytes carry the post-suppression `Vec<LintIssue>` but no
    /// `suppressed_count` at all, so a config with any suppression configured
    /// makes that count unknowable from the bytes alone. A restore must not
    /// prime a cached summary that quietly claims `0` for it -- it must
    /// decline to prime the cache for the affected book, let the book seed
    /// with no lint result, and let the next `lint()` recompute the whole
    /// thing (findings and summary alike) honestly.
    #[test]
    fn restore_corpus_declines_to_prime_a_summary_a_suppressing_config_cannot_recompute_from_bytes()
    {
        let mut original = suppressing_resident();
        original
            .inner
            .replace_corpus(NativeCorpusInput::new(vec![NativeBookInput::Usfm {
                source_key: SourceKey::new("GEN.usfm").unwrap(),
                book: book_id("GEN"),
                source: "\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n".to_string(),
            }]))
            .expect("one book");
        let expected_summary = original.lint().summary;
        assert!(
            expected_summary.suppressed_count >= 1,
            "the fixture must actually suppress a finding"
        );

        let stamps = current_stamps(&original.inner);
        let (packed, source) = encode_one_book(&mut original.inner, book_id("GEN"), stamps);

        let mut reopened = suppressing_resident();
        let (packed_all, sources, records) = records_with_buffers(&[("GEN.usfm", packed, &source)]);
        let outcome = reopened.restore_corpus(&packed_all, &sources, records);
        let ApiResult::Ok { value: report } = outcome.0 else {
            panic!("a fresh, matching-stamp restore must still seed the book: {outcome:?}");
        };
        // The book still seeds -- residency and lint-priming are independent
        // facts -- and this is not a *rejection* either: priming was never
        // attempted for it, so there is nothing to report as refused.
        assert_eq!(report.seeded, vec!["GEN".to_string()]);
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        // The post-restore recompute must still return findings, and the
        // summary it recomputes -- including the suppressed count a cached
        // summary could never have supplied -- must match the original.
        let snapshot = reopened.lint();
        let restored_book = snapshot
            .books
            .iter()
            .find(|entry| entry.book == "GEN")
            .expect("GEN is resident");
        assert!(
            !restored_book.findings.is_empty(),
            "findings must still be returned after the honest recompute"
        );
        assert_summaries_match(snapshot.summary, expected_summary);
    }

    /// The public-API round trip `publish()` exists to close: publish, then
    /// restore *from that exact publication* into a fresh handle, and the
    /// restored corpus must be indistinguishable from the one that published
    /// it -- books, findings, summary, and snapshot id alike. Two books, one
    /// of which carries a real finding, so an empty-vs-nonempty findings
    /// section is exercised in the same call.
    #[test]
    fn publish_then_restore_published_corpus_reproduces_the_resident_state() {
        let mut original = empty_resident();
        original
            .inner
            .replace_corpus(NativeCorpusInput::new(vec![
                NativeBookInput::Usfm {
                    source_key: SourceKey::new("GEN.usfm").unwrap(),
                    book: book_id("GEN"),
                    source: "\\id GEN\n\\c 1\n\\v 1 text\n\\v 1 text\n".to_string(),
                },
                NativeBookInput::Usfm {
                    source_key: SourceKey::new("EXO.usfm").unwrap(),
                    book: book_id("EXO"),
                    source: "\\id EXO\n\\c 1\n\\p\n\\v 1 These are the names.\n".to_string(),
                },
            ]))
            .expect("two books");
        let expected_snapshot = original.lint();
        assert!(
            expected_snapshot
                .books
                .iter()
                .any(|book| !book.findings.is_empty()),
            "the fixture must carry a real finding"
        );

        let PublishOutcome(ApiResult::Ok { value: published }) = original.publish() else {
            panic!("a clean corpus must publish");
        };
        assert_eq!(published.books.len(), 2);
        assert!(
            published.books.iter().all(|book| book.encoded),
            "a first publish encodes every book"
        );

        let mut sources = Vec::new();
        let records: Vec<PublishedCorpusRecord> = published
            .books
            .iter()
            .map(|book| {
                let source = book
                    .source
                    .clone()
                    .expect("a freshly encoded book carries its bound source");
                let offset = sources.len() as u32;
                sources.extend_from_slice(source.as_bytes());
                PublishedCorpusRecord {
                    book: book.book.clone(),
                    source_key: format!("{}.usfm", book.book),
                    byte_offset: offset,
                    byte_length: source.len() as u32,
                }
            })
            .collect();

        let mut reopened = empty_resident();
        let outcome = reopened.restore_published_corpus(&published.bytes, &sources, records);
        let ApiResult::Ok { value: report } = outcome.0 else {
            panic!("a fresh, matching-stamp restore must succeed: {outcome:?}");
        };
        assert_eq!(
            report.seeded.len(),
            2,
            "both books seed: {:?}",
            report.rejected
        );
        assert!(report.rejected.is_empty(), "{:?}", report.rejected);

        let restored_snapshot = reopened.lint();
        assert_eq!(restored_snapshot.snapshot_id, expected_snapshot.snapshot_id);
        assert_eq!(restored_snapshot.books.len(), expected_snapshot.books.len());
        for (restored, expected) in restored_snapshot
            .books
            .iter()
            .zip(expected_snapshot.books.iter())
        {
            assert_eq!(restored.book, expected.book);
            assert_eq!(restored.findings.len(), expected.findings.len());
            for (restored, expected) in restored.findings.iter().zip(&expected.findings) {
                assert_eq!(restored.code, expected.code);
                assert_eq!(restored.sid, expected.sid);
                assert_eq!(restored.message, expected.message);
            }
        }
    }

    // ---- bytes-at-boundary: extent bounds-checking glue (v0.1.5) ----------
    //
    // `bytes.rs` itself unit-tests `slice_extent`/`slice_extent_str` in
    // isolation; these drive the actual `restore_corpus`/
    // `restore_published_corpus` methods natively (no JS engine, no
    // `wasm_bindgen` ABI involved -- these are plain Rust calls) to prove the
    // glue built on top of that slicing refuses cleanly, naming the
    // offending record, rather than panicking or reaching a native call with
    // a bad slice.

    /// `restore_corpus`: a `packed` extent that runs off the end of
    /// `packed_all` refuses `InvalidExtent`, naming the record's own `path`
    /// (the only identifier available before a container is ever decoded).
    #[test]
    fn restore_corpus_refuses_an_out_of_bounds_packed_extent_naming_the_path() {
        let mut reopened = empty_resident();
        let packed_all = vec![0u8; 4];
        let sources = b"abc".to_vec();
        let records = vec![RestoreRecord {
            path: "01-GEN.usfm".to_string(),
            packed: crate::ByteExtent {
                byte_offset: 0,
                byte_length: 10, // past the end of a 4-byte buffer
            },
            source: crate::ByteExtent {
                byte_offset: 0,
                byte_length: 3,
            },
        }];
        let outcome = reopened.restore_corpus(&packed_all, &sources, records);
        assert!(matches!(
            outcome.0,
            ApiResult::Error {
                error: RestoreError::InvalidExtent { ref book }
            } if book == "01-GEN.usfm"
        ));
    }

    /// `restore_corpus`: an extent whose `byteOffset + byteLength` would
    /// overflow a narrower accumulator still refuses cleanly -- never
    /// panics, never wraps into an in-bounds-looking value.
    #[test]
    fn restore_corpus_refuses_an_overflowing_source_extent_without_panicking() {
        let mut reopened = empty_resident();
        let packed_all = b"abcd".to_vec();
        let sources = b"abc".to_vec();
        let records = vec![RestoreRecord {
            path: "01-GEN.usfm".to_string(),
            packed: crate::ByteExtent {
                byte_offset: 0,
                byte_length: 4,
            },
            source: crate::ByteExtent {
                byte_offset: u32::MAX,
                byte_length: u32::MAX,
            },
        }];
        let outcome = reopened.restore_corpus(&packed_all, &sources, records);
        assert!(matches!(
            outcome.0,
            ApiResult::Error {
                error: RestoreError::InvalidExtent { ref book }
            } if book == "01-GEN.usfm"
        ));
    }

    /// `restorePublishedCorpus`: a record's own extent past the end of
    /// `sources` refuses `InvalidExtent`, naming the record's own declared
    /// `book` (known up front here, unlike `restore_corpus`'s `path`).
    #[test]
    fn restore_published_corpus_refuses_an_out_of_bounds_source_extent_naming_the_book() {
        let mut reopened = empty_resident();
        let packed = b"whatever".to_vec();
        let sources = b"abc".to_vec();
        let records = vec![PublishedCorpusRecord {
            book: "GEN".to_string(),
            source_key: "GEN.usfm".to_string(),
            byte_offset: 0,
            byte_length: 10,
        }];
        let outcome = reopened.restore_published_corpus(&packed, &sources, records);
        assert!(matches!(
            outcome.0,
            ApiResult::Error {
                error: RestoreError::InvalidExtent { ref book }
            } if book == "GEN"
        ));
    }

    /// A zero-length extent passes through rather than being pre-emptively
    /// refused by this layer -- an empty source's own classification (a
    /// decode/ingest defect, not an extent defect) still applies downstream.
    #[test]
    fn restore_corpus_zero_length_source_extent_passes_through_to_native_classification() {
        let mut reopened = empty_resident();
        let packed_all = b"not a real container".to_vec();
        let sources = Vec::new();
        let records = vec![RestoreRecord {
            path: "01-GEN.usfm".to_string(),
            packed: crate::ByteExtent {
                byte_offset: 0,
                byte_length: packed_all.len() as u32,
            },
            source: crate::ByteExtent {
                byte_offset: 0,
                byte_length: 0,
            },
        }];
        let outcome = reopened.restore_corpus(&packed_all, &sources, records);
        // Not refused by the extent layer: it reaches native, which refuses
        // it for its own reason (not a valid container) -- proving the empty
        // extent itself was never the problem.
        assert!(!matches!(
            outcome.0,
            ApiResult::Error {
                error: RestoreError::InvalidExtent { .. }
            }
        ));
    }
}
