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
#[wasm_bindgen]
pub struct Braid {
    inner: NativeBraid,
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
    #[wasm_bindgen(js_name = restoreCorpus)]
    pub fn restore_corpus(&mut self, records: Vec<RestoreRecord>) -> RestoreOutcome {
        let mut books = Vec::with_capacity(records.len());
        for record in &records {
            let source = match std::str::from_utf8(&record.source) {
                Ok(source) => source,
                Err(_) => {
                    return RestoreOutcome::refused(RestoreError::Decode {
                        error: crate::PackedDecodeError::InvalidUtf8,
                    });
                }
            };
            let verified = match usfm_onion_wire::verify::verify_book(&record.packed, source) {
                Ok(verified) => verified,
                Err(error) => {
                    return RestoreOutcome::refused(RestoreError::Decode {
                        error: error.into(),
                    });
                }
            };
            let tokens =
                match usfm_onion_wire::verify::materialize_owned_tokens(&record.packed, source) {
                    Ok(tokens) => tokens,
                    Err(error) => {
                        return RestoreOutcome::refused(RestoreError::Decode {
                            error: error.into(),
                        });
                    }
                };
            let book = match usfm_onion::token::BookId::from_str(&verified.receipt.book) {
                Some(book) => book,
                None => {
                    return RestoreOutcome::refused(RestoreError::Decode {
                        error: crate::PackedDecodeError::InvalidSection,
                    });
                }
            };
            let source_key = match braid::SourceKey::new(record.path.clone()) {
                Some(key) => key,
                None => {
                    return RestoreOutcome::refused(RestoreError::Ingest {
                        error: IngestError::DuplicateSourceKey {
                            source: record.path.clone(),
                        },
                    });
                }
            };
            books.push(braid::BookRestoreInput {
                source_key,
                book,
                source: source.to_string(),
                tokens,
                line_ending: usfm_onion::token::LineEnding::detect(source),
                // The findings the container carried are adoptable only if its own
                // stamps say what produced them; braid re-checks them against the
                // resident configuration before it trusts any of it.
                lint: verified.lint_stamps.map(|_| braid::BookLintPrime {
                    book,
                    source_hash: braid::SourceHash(
                        u64::from_str_radix(&verified.receipt.source_hash, 16).unwrap_or_default(),
                    ),
                    result: usfm_onion::lint::LintResult {
                        issues: verified.findings.clone(),
                        summary: Default::default(),
                    },
                }),
            });
        }

        let stamps = records
            .first()
            .and_then(|record| {
                usfm_onion_wire::verify::verify_book(
                    &record.packed,
                    std::str::from_utf8(&record.source).unwrap_or_default(),
                )
                .ok()
                .and_then(|verified| verified.lint_stamps)
            })
            .unwrap_or(usfm_onion_wire::corpus_codec::LintStamps {
                config_fingerprint: 0,
                engine_stamp: 0,
            });

        RestoreOutcome::from(
            self.inner
                .restore_corpus(braid::CorpusRestoreInput::new(
                    braid::LintConfigFingerprint(stamps.config_fingerprint),
                    braid::LintEngineStamp(stamps.engine_stamp),
                    books,
                ))
                .map(RestoreReport::from)
                .map_err(|error| RestoreError::Ingest {
                    error: error.into(),
                }),
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

/// One book's packed bytes and the source they were bound to.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct RestoreRecord {
    /// The caller's own binding for where the book came from — normally a path.
    pub path: String,
    pub packed: Vec<u8>,
    /// The exact bytes the container was bound to. Bytes rather than a string so a
    /// host can hand over what it read from disk without a UTF-16 round trip.
    pub source: Vec<u8>,
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
    Decode { error: crate::PackedDecodeError },
    Ingest { error: IngestError },
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
