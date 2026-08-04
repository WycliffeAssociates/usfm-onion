/**
 * The resident corpus handle.
 */
export class Braid {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BraidFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_braid_free(ptr, 0);
    }
    /**
     * Applies a prepared format patch. All-or-nothing across every book it covers.
     * @param {FormatPatchId} id
     * @returns {FormatMutationOutcome}
     */
    applyFormatPatch(id) {
        const ret = wasm.braid_applyFormatPatch(this.__wbg_ptr, addHeapObject(id));
        return takeObject(ret);
    }
    /**
     * Applies a patch as an ordinary mutation, atomically.
     * @param {PatchId} id
     * @returns {PatchMutationOutcome}
     */
    applyPatch(id) {
        const ret = wasm.braid_applyPatch(this.__wbg_ptr, addHeapObject(id));
        return takeObject(ret);
    }
    /**
     * Resident books with their derived stamps, in corpus order.
     * @returns {BookEntry[]}
     */
    books() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.braid_books(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Books whose findings are stale, in corpus order. Derived from authoritative
     * stamps rather than drained from a queue, so reading it twice is safe.
     * @returns {string[]}
     */
    booksAwaitingLint() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.braid_booksAwaitingLint(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * One book's chapter-run labels in source order, duplicates included.
     * @param {string} book
     * @returns {ChapterLabelsOutcome}
     */
    chapterLabels(book) {
        const ptr0 = passStringToWasm0(book, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.braid_chapterLabels(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Drops every resident book. Clearing an empty corpus is a no-op.
     * @returns {MutationEffect}
     */
    clear() {
        const ret = wasm.braid_clear(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Forgets one book's baseline. Clearing an absent one is a no-op.
     * @param {string} book
     * @returns {MutationOutcome}
     */
    clearBaseline(book) {
        const ptr0 = passStringToWasm0(book, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.braid_clearBaseline(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * The resident diff against the baseline.
     * @param {CorpusScope} scope
     * @returns {DiffBaselineOutcome}
     */
    diffBaseline(scope) {
        const ret = wasm.braid_diffBaseline(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
    /**
     * The corpus's content-derived identity, as a 16-digit hex string.
     *
     * Hex rather than a number because the value is 64 bits: a JS `number` cannot
     * hold it without silently rounding, and a `bigint` does not survive every
     * structured clone a worker boundary performs.
     * @returns {string}
     */
    expectedSnapshotId() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.braid_expectedSnapshotId(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Whether a scope differs from its baseline, by exact serialized equality.
     * @param {CorpusScope} scope
     * @returns {DirtyOutcome}
     */
    isDirty(scope) {
        const ret = wasm.braid_isDirty(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
    /**
     * Recomputes every book awaiting it and returns the complete snapshot.
     *
     * The only recompute verb, and always explicit: no mutation lints implicitly
     * and no effect carries findings. Exactly the stale books run rules — a clean
     * corpus runs none.
     * @returns {LintSnapshot}
     */
    lint() {
        const ret = wasm.braid_lint(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Creates an empty handle bound to the application's own id minter.
     *
     * The minter is a JS callback returning a string, held for the life of the
     * handle: core never invents a token id, so every token a fix or format pass
     * synthesizes gets one from here. Speed, spelling, and collision resistance
     * are the application's trade — uniqueness is not assumed but enforced at the
     * residency boundary, where a collision is a typed rejection rather than a
     * corrupted book.
     *
     * Throws only for a programmer error: a minter that throws, or one that
     * returns something other than a string.
     * @param {BraidConfig} config
     * @param {Function} minter
     */
    constructor(config, minter) {
        const ret = wasm.braid_new(addHeapObject(config), addHeapObject(minter));
        this.__wbg_ptr = ret >>> 0;
        BraidFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * One patch by id, refusing a stale or unknown one.
     * @param {PatchId} id
     * @returns {PatchOutcome}
     */
    patch(id) {
        const ret = wasm.braid_patch(this.__wbg_ptr, addHeapObject(id));
        return takeObject(ret);
    }
    /**
     * Every patch of the current snapshot, in corpus order and then each book's own
     * canonical finding order — which is what assigns each one its ordinal.
     *
     * A book awaiting recompute contributes none: its stored positions address the
     * token stream it held when its findings were computed.
     * @returns {Patch[]}
     */
    patches() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.braid_patches(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Prepares a formatting pass over a scope without applying it.
     * @param {CorpusScope} scope
     * @param {FormatOptions | null} [options]
     * @returns {FormatPreparationOutcome}
     */
    prepareFormatPatch(scope, options) {
        const ret = wasm.braid_prepareFormatPatch(this.__wbg_ptr, addHeapObject(scope), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * The token stream the patch would produce, without applying it.
     *
     * A preview is a projection and is never admitted to residency, so it mints
     * nothing: a surviving token carries the id it already had, and a token the fix
     * would synthesize carries none until an apply grants it one.
     * @param {PatchId} id
     * @returns {PatchPreviewOutcome}
     */
    previewPatch(id) {
        const ret = wasm.braid_previewPatch(this.__wbg_ptr, addHeapObject(id));
        return takeObject(ret);
    }
    /**
     * Publishes the resident corpus as one packed `corpus.bin` container.
     *
     * A thin projection of `PublicationCache::publish` (this handle's own
     * cache, so a repeat publish gets the adapter's whole point -- splice-
     * reuse of whatever did not change -- automatically): dirty books are
     * linted first (the adapter's own rule, via the `lint()` it runs
     * internally), every book's bytes and stamps decide reuse vs. re-encode,
     * and the reuse-cache's own sections/bytes never cross this boundary --
     * only the per-book bookkeeping in [`PublishedBookInfo`] does.
     * @returns {PublishOutcome}
     */
    publish() {
        const ret = wasm.braid_publish(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Publishes exactly the books a scope names, as per-book packed
     * containers -- the exact shape `restoreCorpus` consumes, never
     * `PublishedCorpus`-shaped. Every returned book is always freshly
     * encoded and always carries its source; there is no splice-reuse arm,
     * and this call never reads or invalidates the handle's own
     * `PublicationCache` (that cache is `publish`'s alone).
     * @param {CorpusScope} scope
     * @returns {ScopedPublishOutcome}
     */
    publishScope(scope) {
        const ret = wasm.braid_publishScope(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
    /**
     * Removes a book. Removing an absent book is a no-op, not an error: the
     * requested end state already holds.
     * @param {string} book
     * @returns {MutationOutcome}
     */
    removeBook(book) {
        const ptr0 = passStringToWasm0(book, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.braid_removeBook(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Removes one chapter run's tokens from its book. The effect is whole-book:
     * the address the caller used no longer exists.
     * @param {ChapterTarget} target
     * @returns {ScopedMutationOutcome}
     */
    removeChapter(target) {
        const ret = wasm.braid_removeChapter(this.__wbg_ptr, addHeapObject(target));
        return takeObject(ret);
    }
    /**
     * Replaces the whole corpus with a validated candidate.
     *
     * Every book is built, validated, and hashed before resident state is touched,
     * so a rejection leaves the corpus, its stamps, and its identity exactly as
     * they were.
     * @param {CorpusInput} corpus
     * @returns {MutationOutcome}
     */
    replaceCorpus(corpus) {
        const ret = wasm.braid_replaceCorpus(this.__wbg_ptr, addHeapObject(corpus));
        return takeObject(ret);
    }
    /**
     * Seeds the whole corpus from packed bytes plus the sources they were bound to
     * — the warm cold-open.
     *
     * Composed here because this is the only layer allowed to know both halves: the
     * bytes are verified and decoded by the wire codec, and the results are handed
     * to the resident corpus, which never sees a packed byte itself. Verification is
     * the full trust boundary — structure, both checksums, exact source length and
     * content hash, the catalog stamp, every discriminant and index — so a container
     * that does not check out is refused before anything is installed.
     *
     * A book whose cached findings cannot be adopted still seeds: residency and
     * lint-priming are independent facts, so that book arrives with no lex or parse
     * and is simply awaiting recompute.
     *
     * `packed_all`/`sources` are two single buffers -- every record's own
     * container concatenated into the first, every record's own source
     * concatenated into the second -- with `records` naming each one's
     * extent into whichever buffer it belongs to (v0.1.5, bytes-at-boundary
     * convention: this is the exact shape [`Braid::publish_scope`]'s output
     * already is, so it forwards here with zero reshaping -- see
     * [`ScopedPublication`]'s own doc comment). An extent that falls
     * outside its buffer, or whose own end overflows computing it, is
     * refused (`RestoreError::InvalidExtent`, naming the record's own
     * `path`) before any native call -- never clamped, never truncated.
     * @param {Uint8Array} packed_all
     * @param {Uint8Array} sources
     * @param {RestoreRecord[]} records
     * @returns {RestoreOutcome}
     */
    restoreCorpus(packed_all, sources, records) {
        const ptr0 = passArray8ToWasm0(packed_all, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(sources, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayJsValueToWasm0(records, wasm.__wbindgen_export);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.braid_restoreCorpus(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        return takeObject(ret);
    }
    /**
     * Restores the whole resident corpus from one packed `corpus.bin`
     * container -- the corpus-grain counterpart to [`Self::publish`], as
     * [`Self::restore_corpus`] is to a per-book publication.
     *
     * `packed` is the one whole-corpus container (a single `Uint8Array`
     * argument, one memcpy). `sources` is every named book's source bytes
     * concatenated into one buffer; `records` supplies each book's own
     * declared code, its source key (a packed container names the book but
     * never the key a corpus was originally addressed by), and its own
     * extent into `sources` (v0.1.5, bytes-at-boundary convention -- see
     * [`crate::bytes`]). An extent outside `sources`, or one whose own end
     * overflows computing it, refuses by name
     * (`RestoreError::InvalidExtent`, naming the record's own `book`)
     * before any native call. Verification is corpus-wide (`verify_corpus`):
     * every book must have exactly one source supplied, and findings that
     * carry stamps must all carry the *same* stamps, checked atomically
     * before anything installs.
     * @param {Uint8Array} packed
     * @param {Uint8Array} sources
     * @param {PublishedCorpusRecord[]} records
     * @returns {RestoreOutcome}
     */
    restorePublishedCorpus(packed, sources, records) {
        const ptr0 = passArray8ToWasm0(packed, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(sources, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayJsValueToWasm0(records, wasm.__wbindgen_export);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.braid_restorePublishedCorpus(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        return takeObject(ret);
    }
    /**
     * Whole-book replacement from each targeted book's own declared
     * baseline, atomic across the scope. `all`/`book` scopes only -- a
     * chapter scope refuses via `BaselineError.chapterScopeUnsupported`
     * rather than reverting one run in isolation (use `diffBaseline` plus
     * `updateChapter` with the baseline run's own tokens instead).
     *
     * Atomicity: every targeted book must be resident and baselined before
     * anything mutates -- any missing baseline refuses with every offender
     * named, and resident state is left exactly as it was. A book already
     * equal to its baseline is a no-op, absent from `changed`.
     * @param {CorpusScope} scope
     * @returns {RevertBaselineOutcome}
     */
    revertToBaseline(scope) {
        const ret = wasm.braid_revertToBaseline(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
    /**
     * Records one book's baseline — the state later comparisons are against.
     *
     * Only for a book that is already resident: a baseline is what the *current*
     * state is compared against, so installing one for a book with no current
     * state would invent the comparison rather than record it.
     * @param {BookInput} book
     * @returns {BaselineMutationOutcome}
     */
    setBaseline(book) {
        const ret = wasm.braid_setBaseline(this.__wbg_ptr, addHeapObject(book));
        return takeObject(ret);
    }
    /**
     * Declares each in-scope book's CURRENT resident state as its baseline
     * -- no re-parse, no `BookInput`: the bulk, no-parse counterpart to
     * `setBaseline`. `all`/`book` scopes only, deliberately symmetric with
     * `revertToBaseline` (a baseline is a whole-book slot, so the set and
     * revert halves of its lifecycle agree on what scopes can address it);
     * a chapter scope refuses the same way. Idempotent, and there is no
     * missing-baseline case -- this verb's whole point is to create one.
     * @param {CorpusScope} scope
     * @returns {RevertBaselineOutcome}
     */
    setBaselineToCurrent(scope) {
        const ret = wasm.braid_setBaselineToCurrent(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
    /**
     * Current tokens for the requested scopes — the single hydration verb.
     *
     * Returns current truth, not state as of any earlier effect. The input is
     * normalized first (duplicates collapse, a whole-book scope absorbs that
     * book's chapter scopes), so concatenating several effects' `changed` lists is
     * always correct.
     * @param {Scope[]} scopes
     * @returns {ScopeTokensOutcome}
     */
    toTokens(scopes) {
        const ptr0 = passArrayJsValueToWasm0(scopes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.braid_toTokens(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * The exact bytes a scope would be saved as.
     * @param {CorpusScope} scope
     * @returns {UsfmOutcome}
     */
    toUsfm(scope) {
        const ret = wasm.braid_toUsfm(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
    /**
     * Replaces one book, or appends it when it is not resident yet.
     *
     * Whole-book replacement is the structural escape hatch: chapter insertion,
     * deletion, reordering, and duplicate resolution all go through here.
     * @param {BookInput} book
     * @returns {MutationOutcome}
     */
    updateBook(book) {
        const ret = wasm.braid_updateBook(this.__wbg_ptr, addHeapObject(book));
        return takeObject(ret);
    }
    /**
     * Replaces exactly one existing chapter run with the caller's content.
     *
     * The replacement must be that same one run: no matching run is not found,
     * several is ambiguous, and content that is a different or additional chapter
     * is a label mismatch. The book's stored line ending is inherited.
     * @param {ChapterTarget} target
     * @param {ChapterInput} replacement
     * @returns {MutationOutcome}
     */
    updateChapter(target, replacement) {
        const ret = wasm.braid_updateChapter(this.__wbg_ptr, addHeapObject(target), addHeapObject(replacement));
        return takeObject(ret);
    }
    /**
     * Replaces the resident configuration.
     *
     * No tokens are rewritten, so nothing needs re-pulling and the identity — which
     * covers source bytes only — is unchanged. What changes is staleness: every
     * book is marked for recompute, because the configuration its cached findings
     * were produced under no longer applies.
     * @param {BraidConfig} config
     * @returns {MutationEffect}
     */
    updateConfig(config) {
        const ret = wasm.braid_updateConfig(this.__wbg_ptr, addHeapObject(config));
        return takeObject(ret);
    }
    /**
     * Every verse's lossless text projection for a scope, in document order.
     *
     * The resident answer to what the stateless projection computes from scratch:
     * identical entries, but a read after a one-chapter edit recomputes only that
     * chapter and takes the rest from cache — which is what makes this callable on
     * a keystroke instead of once a document.
     *
     * Entries are `[sid, projection]` pairs in first-seen token order, the same
     * shape the stateless `vrefIndexUsfm`/`vrefIndexTokens` exports return: one
     * authoritative sequence, since an object keyed by sid enumerates its keys
     * sorted and would silently reorder a document that is deliberately not.
     * @param {CorpusScope} scope
     * @returns {VrefIndexOutcome}
     */
    vrefIndex(scope) {
        const ret = wasm.braid_vrefIndex(this.__wbg_ptr, addHeapObject(scope));
        return takeObject(ret);
    }
}
if (Symbol.dispose) Braid.prototype[Symbol.dispose] = Braid.prototype.free;

export class ParsedUsfm {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ParsedUsfm.prototype);
        obj.__wbg_ptr = ptr;
        ParsedUsfmFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ParsedUsfmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_parsedusfm_free(ptr, 0);
    }
    /**
     * @param {TokenFix} fix
     * @returns {Token[]}
     */
    applyTokenFix(fix) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_applyTokenFix(retptr, this.__wbg_ptr, addHeapObject(fix));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {CstDocument}
     */
    cst() {
        const ret = wasm.parsedusfm_cst(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @param {ParsedUsfm} other
     * @param {DiffOptions | null} [options]
     * @returns {DiffSkeleton}
     */
    diff(other, options) {
        _assertClass(other, ParsedUsfm);
        const ret = wasm.parsedusfm_diff(this.__wbg_ptr, other.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * @param {ParsedUsfm} other
     * @param {DiffOptions | null} [options]
     * @returns {DiffsByChapterMap}
     */
    diffByChapter(other, options) {
        _assertClass(other, ParsedUsfm);
        const ret = wasm.parsedusfm_diffByChapter(this.__wbg_ptr, other.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * @param {FormatOptions | null} [options]
     * @returns {string}
     */
    format(options) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_format(retptr, this.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {LintOptions} options
     * @returns {LintResult}
     */
    lint(options) {
        const ret = wasm.parsedusfm_lint(this.__wbg_ptr, addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * @param {ParsedUsfm} current
     * @param {string} block_id
     * @returns {Token[]}
     */
    revertDiffBlock(current, block_id) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(current, ParsedUsfm);
            const ptr0 = passStringToWasm0(block_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.parsedusfm_revertDiffBlock(retptr, this.__wbg_ptr, current.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            if (r3) {
                throw takeObject(r2);
            }
            var v2 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v2;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {HtmlOptions | null} [options]
     * @returns {string}
     */
    toHtml(options) {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_toHtml(retptr, this.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    toUsfm() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_toUsfm(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    toUsj() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_toUsj(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string}
     */
    toUsx() {
        let deferred2_0;
        let deferred2_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_toUsx(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr1 = r0;
            var len1 = r1;
            if (r3) {
                ptr1 = 0; len1 = 0;
                throw takeObject(r2);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {VrefOptions | null} [options]
     * @returns {VrefMap}
     */
    toVref(options) {
        const ret = wasm.parsedusfm_toVref(this.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * @returns {Token[]}
     */
    tokens() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parsedusfm_tokens(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {VrefIndex}
     */
    vrefIndex() {
        const ret = wasm.parsedusfm_vrefIndex(this.__wbg_ptr);
        return takeObject(ret);
    }
}
if (Symbol.dispose) ParsedUsfm.prototype[Symbol.dispose] = ParsedUsfm.prototype.free;

export class UsfmMarkerCatalog {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(UsfmMarkerCatalog.prototype);
        obj.__wbg_ptr = ptr;
        UsfmMarkerCatalogFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        UsfmMarkerCatalogFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_usfmmarkercatalog_free(ptr, 0);
    }
    /**
     * @returns {MarkerInfo[]}
     */
    all() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.usfmmarkercatalog_all(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @param {string} marker
     * @returns {boolean}
     */
    contains(marker) {
        const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.usfmmarkercatalog_contains(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * @param {string} marker
     * @returns {MarkerInfo | undefined}
     */
    get(marker) {
        const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.usfmmarkercatalog_get(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
}
if (Symbol.dispose) UsfmMarkerCatalog.prototype[Symbol.dispose] = UsfmMarkerCatalog.prototype.free;

/**
 * @param {Token[]} tokens
 * @param {TokenFix} fix
 * @returns {Token[]}
 */
export function applyTokenFix(tokens, fix) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.applyTokenFix(retptr, ptr0, len0, addHeapObject(fix));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v2 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Token[]} left
 * @param {Token[]} right
 * @param {DiffOptions | null} [options]
 * @returns {DiffSkeleton}
 */
export function diffTokens(left, right, options) {
    const ptr0 = passArrayJsValueToWasm0(left, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(right, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.diffTokens(ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * @param {string} left
 * @param {string} right
 * @param {DiffOptions | null} [options]
 * @returns {DiffSkeleton}
 */
export function diffUsfm(left, right, options) {
    const ptr0 = passStringToWasm0(left, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(right, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.diffUsfm(ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * @param {string} left
 * @param {string} right
 * @param {DiffOptions | null} [options]
 * @returns {DiffsByChapterMap}
 */
export function diffUsfmByChapter(left, right, options) {
    const ptr0 = passStringToWasm0(left, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(right, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.diffUsfmByChapter(ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * @returns {FormatRuleMeta[]}
 */
export function formatRuleMeta() {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.formatRuleMeta(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @returns {string[]}
 */
export function formatRules() {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.formatRules(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Token[]} tokens
 * @param {FormatOptions | null} [options]
 * @returns {FormatResult}
 */
export function formatTokens(tokens, options) {
    const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.formatTokens(ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * @param {Token[]} tokens
 * @param {FormatOptions | null} [options]
 * @returns {Token[]}
 */
export function formatTokensMut(tokens, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.formatTokensMut(retptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v2 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} source
 * @param {FormatOptions | null} [options]
 * @returns {string}
 */
export function formatUsfm(source, options) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.formatUsfm(retptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} marker
 * @returns {boolean}
 */
export function isKnownMarker(marker) {
    const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.isKnownMarker(ptr0, len0);
    return ret !== 0;
}

/**
 * @returns {LintCodeMeta[]}
 */
export function lintCodeMeta() {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.lintCodeMeta(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @returns {LintCode[]}
 */
export function lintCodes() {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.lintCodes(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Token[]} tokens
 * @param {LintOptions} options
 * @returns {LintResult}
 */
export function lintTokens(tokens, options) {
    const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.lintTokens(ptr0, len0, addHeapObject(options));
    return takeObject(ret);
}

/**
 * @param {string} source
 * @param {LintOptions} options
 * @returns {LintResult}
 */
export function lintUsfm(source, options) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.lintUsfm(ptr0, len0, addHeapObject(options));
    return takeObject(ret);
}

/**
 * @returns {UsfmMarkerCatalog}
 */
export function markerCatalog() {
    const ret = wasm.markerCatalog();
    return UsfmMarkerCatalog.__wrap(ret);
}

/**
 * @param {string} marker
 * @returns {MarkerInfo}
 */
export function markerInfo(marker) {
    const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.markerInfo(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {Token[]} baseline
 * @param {Token[]} current
 * @param {MergeRequest} request
 * @returns {Token[]}
 */
export function mergeDiffBlocks(baseline, current, request) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(baseline, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(current, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.mergeDiffBlocks(retptr, ptr0, len0, ptr1, len1, addHeapObject(request));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v3 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Token[]} tokens
 * @param {string} book_code
 * @returns {Token[]}
 */
export function normalizeTokenSids(tokens, book_code) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(book_code, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        wasm.normalizeTokenSids(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v3 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} source
 * @returns {ParsedUsfm}
 */
export function parse(source) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parse(ptr0, len0);
    return ParsedUsfm.__wrap(ret);
}

/**
 * @param {Token[]} baseline
 * @param {Token[]} current
 * @param {string} block_id
 * @returns {Token[]}
 */
export function revertDiffBlock(baseline, current, block_id) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(baseline, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(current, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(block_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len2 = WASM_VECTOR_LEN;
        wasm.revertDiffBlock(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeObject(r2);
        }
        var v4 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v4;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Token[]} tokens
 * @param {HtmlOptions | null} [options]
 * @returns {string}
 */
export function tokensToHtml(tokens, options) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.tokensToHtml(retptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {Token[]} tokens
 * @returns {string}
 */
export function tokensToUsfm(tokens) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.tokensToUsfm(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

/**
 * The packed trust boundary: verifies one book's container against its exact
 * source and returns the receipt plus that book's findings.
 *
 * This runs the whole Rust boundary — container/section structure, both
 * integrity checksums, exact source length and XXH3 content hash, the
 * marker-catalog stamp, every discriminant, index range, and reserved byte.
 * Nothing but tokens is left for the caller to materialize, and no token
 * object crosses this boundary. Findings are materialized here so
 * `LintIssue.message` keeps a single renderer (core's), in a single language.
 *
 * `source` is bytes rather than a string so the caller can hand over the same
 * buffer it read from disk without a UTF-16 round trip; non-UTF-8 source is a
 * rejection, not a panic.
 * @param {Uint8Array} packed
 * @param {Uint8Array} source
 * @returns {PackedBookOutcome}
 */
export function verifyPackedBook(packed, source) {
    const ptr0 = passArray8ToWasm0(packed, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(source, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.verifyPackedBook(ptr0, len0, ptr1, len1);
    return takeObject(ret);
}

/**
 * Verifies a whole packed corpus container against the exact sources every
 * book was bound to -- the read-only inspection counterpart to
 * [`crate::resident::Braid::restore_published_corpus`], useful to a host
 * that wants to validate a `corpus.bin` before deciding whether to restore
 * it into a resident handle at all.
 *
 * `packed` is the one whole-corpus container. `sources` is every named
 * book's source bytes concatenated into one buffer; `records` names each
 * book's own extent into it -- the same buffer-plus-extents pairing
 * [`crate::resident::Braid::restore_published_corpus`] takes.
 *
 * Runs the same corpus-wide trust boundary `restorePublishedCorpus` does
 * (container/section structure, both integrity checksums, exact source
 * length and content hash, the marker-catalog stamp, the all-or-none lint
 * stamp invariant), and nothing more: no resident state is read or
 * mutated, and no token crosses this boundary.
 * @param {Uint8Array} packed
 * @param {Uint8Array} sources
 * @param {PublishedCorpusRecord[]} records
 * @returns {PublishedCorpusOutcome}
 */
export function verifyPublishedCorpus(packed, sources, records) {
    const ptr0 = passArray8ToWasm0(packed, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArray8ToWasm0(sources, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passArrayJsValueToWasm0(records, wasm.__wbindgen_export);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.verifyPublishedCorpus(ptr0, len0, ptr1, len1, ptr2, len2);
    return takeObject(ret);
}

/**
 * Build the vref index from an existing token stream (the editor's live
 * path) — same rehydration as `lintTokens`, no reparse. Segment ids match
 * the tokens passed in, so they line up with the editor's DOM `data-id`s.
 * @param {Token[]} tokens
 * @returns {VrefIndex}
 */
export function vrefIndexTokens(tokens) {
    const ptr0 = passArrayJsValueToWasm0(tokens, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.vrefIndexTokens(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {string} source
 * @returns {VrefIndex}
 */
export function vrefIndexUsfm(source) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.vrefIndexUsfm(ptr0, len0);
    return takeObject(ret);
}
export function __wbg_Error_83742b46f01ce22d(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}
export function __wbg_Number_a5a435bd7bbec835(arg0) {
    const ret = Number(getObject(arg0));
    return ret;
}
export function __wbg_String_8564e559799eccda(arg0, arg1) {
    const ret = String(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_bigint_get_as_i64_447a76b5c6ef7bda(arg0, arg1) {
    const v = getObject(arg1);
    const ret = typeof(v) === 'bigint' ? v : undefined;
    getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
}
export function __wbg___wbindgen_boolean_get_c0f3f60bac5a78d1(arg0) {
    const v = getObject(arg0);
    const ret = typeof(v) === 'boolean' ? v : undefined;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
}
export function __wbg___wbindgen_debug_string_5398f5bb970e0daa(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_in_41dbb8413020e076(arg0, arg1) {
    const ret = getObject(arg0) in getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_is_bigint_e2141d4f045b7eda(arg0) {
    const ret = typeof(getObject(arg0)) === 'bigint';
    return ret;
}
export function __wbg___wbindgen_is_function_3c846841762788c1(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
}
export function __wbg___wbindgen_is_object_781bc9f159099513(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
}
export function __wbg___wbindgen_is_string_7ef6b97b02428fae(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
}
export function __wbg___wbindgen_is_undefined_52709e72fb9f179c(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
}
export function __wbg___wbindgen_jsval_eq_ee31bfad3e536463(arg0, arg1) {
    const ret = getObject(arg0) === getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_jsval_loose_eq_5bcc3bed3c69e72b(arg0, arg1) {
    const ret = getObject(arg0) == getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_number_get_34bb9d9dcfa21373(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
}
export function __wbg___wbindgen_string_get_395e606bd0ee4427(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_throw_6ddd609b62940d55(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
}
export function __wbg_call_e133b57c9155d22c() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_done_08ce71ee07e3bd17(arg0) {
    const ret = getObject(arg0).done;
    return ret;
}
export function __wbg_entries_e8a20ff8c9757101(arg0) {
    const ret = Object.entries(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_from_4bdf88943703fd48(arg0) {
    const ret = Array.from(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_get_326e41e095fb2575() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_get_a8ee5c45dabc1b3b(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
}
export function __wbg_get_unchecked_329cfe50afab7352(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
}
export function __wbg_get_with_ref_key_6412cf3094599694(arg0, arg1) {
    const ret = getObject(arg0)[getObject(arg1)];
    return addHeapObject(ret);
}
export function __wbg_instanceof_ArrayBuffer_101e2bf31071a9f6(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Map_f194b366846aca0c(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Map;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Uint8Array_740438561a5b956d(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_isArray_33b91feb269ff46e(arg0) {
    const ret = Array.isArray(getObject(arg0));
    return ret;
}
export function __wbg_isSafeInteger_ecd6a7f9c3e053cd(arg0) {
    const ret = Number.isSafeInteger(getObject(arg0));
    return ret;
}
export function __wbg_iterator_d8f549ec8fb061b1() {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
}
export function __wbg_length_b3416cf66a5452c8(arg0) {
    const ret = getObject(arg0).length;
    return ret;
}
export function __wbg_length_ea16607d7b61445b(arg0) {
    const ret = getObject(arg0).length;
    return ret;
}
export function __wbg_new_49d5571bd3f0c4d4() {
    const ret = new Map();
    return addHeapObject(ret);
}
export function __wbg_new_5f486cdf45a04d78(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_new_a70fbab9066b301f() {
    const ret = new Array();
    return addHeapObject(ret);
}
export function __wbg_new_ab79df5bd7c26067() {
    const ret = new Object();
    return addHeapObject(ret);
}
export function __wbg_next_11b99ee6237339e3() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments); }
export function __wbg_next_e01a967809d1aa68(arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
}
export function __wbg_prototypesetcall_d62e5099504357e6(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
}
export function __wbg_set_282384002438957f(arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
}
export function __wbg_set_6be42768c690e380(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
}
export function __wbg_set_bf7251625df30a02(arg0, arg1, arg2) {
    const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}
export function __wbg_value_21fc78aab0322612(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000001(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000002(arg0) {
    // Cast intrinsic for `I64 -> Externref`.
    const ret = arg0;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000003(arg0, arg1) {
    // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
    const ret = getArrayU8FromWasm0(arg0, arg1);
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000004(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000005(arg0) {
    // Cast intrinsic for `U64 -> Externref`.
    const ret = BigInt.asUintN(64, arg0);
    return addHeapObject(ret);
}
export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
}
export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
}
const BraidFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_braid_free(ptr >>> 0, 1));
const ParsedUsfmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parsedusfm_free(ptr >>> 0, 1));
const UsfmMarkerCatalogFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_usfmmarkercatalog_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(takeObject(mem.getUint32(i, true)));
    }
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addHeapObject(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;


let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}
