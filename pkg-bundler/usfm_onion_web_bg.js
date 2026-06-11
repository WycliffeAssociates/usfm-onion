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
     * @param {BuildSidBlocksOptions | null} [options]
     * @returns {ChapterTokenDiff[]}
     */
    diff(other, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(other, ParsedUsfm);
            wasm.parsedusfm_diff(retptr, this.__wbg_ptr, other.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
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
     * @param {ParsedUsfm} other
     * @param {BuildSidBlocksOptions | null} [options]
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
     * @param {BuildSidBlocksOptions | null} [options]
     * @returns {Token[]}
     */
    revertDiffBlock(current, block_id, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            _assertClass(current, ParsedUsfm);
            const ptr0 = passStringToWasm0(block_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.parsedusfm_revertDiffBlock(retptr, this.__wbg_ptr, current.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
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
 * @param {BuildSidBlocksOptions | null} [options]
 * @returns {ChapterTokenDiff[]}
 */
export function diffTokens(left, right, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(left, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(right, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        wasm.diffTokens(retptr, ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
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
 * @param {string} left
 * @param {string} right
 * @param {BuildSidBlocksOptions | null} [options]
 * @returns {ChapterTokenDiff[]}
 */
export function diffUsfm(left, right, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(left, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(right, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        wasm.diffUsfm(retptr, ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addHeapObject(options));
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
 * @param {string} left
 * @param {string} right
 * @param {BuildSidBlocksOptions | null} [options]
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
 * @param {BuildSidBlocksOptions | null} [options]
 * @returns {Token[]}
 */
export function revertDiffBlock(baseline, current, block_id, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(baseline, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(current, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(block_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len2 = WASM_VECTOR_LEN;
        wasm.revertDiffBlock(retptr, ptr0, len0, ptr1, len1, ptr2, len2, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v4 = getArrayJsValueFromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v4;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Token[]} baseline
 * @param {Token[]} current
 * @param {string[]} block_ids
 * @param {BuildSidBlocksOptions | null} [options]
 * @returns {Token[]}
 */
export function revertDiffBlocks(baseline, current, block_ids, options) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArrayJsValueToWasm0(baseline, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayJsValueToWasm0(current, wasm.__wbindgen_export);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayJsValueToWasm0(block_ids, wasm.__wbindgen_export);
        const len2 = WASM_VECTOR_LEN;
        wasm.revertDiffBlocks(retptr, ptr0, len0, ptr1, len1, ptr2, len2, isLikeNone(options) ? 0 : addHeapObject(options));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
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
export function __wbg_String_8564e559799eccda(arg0, arg1) {
    const ret = String(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_is_string_7ef6b97b02428fae(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
}
export function __wbg___wbindgen_is_undefined_52709e72fb9f179c(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
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
export function __wbg_new_49d5571bd3f0c4d4() {
    const ret = new Map();
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
export function __wbg_parse_e9eddd2a82c706eb() { return handleError(function (arg0, arg1) {
    const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}, arguments); }
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
export function __wbg_stringify_5ae93966a84901ac() { return handleError(function (arg0) {
    const ret = JSON.stringify(getObject(arg0));
    return addHeapObject(ret);
}, arguments); }
export function __wbindgen_cast_0000000000000001(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
}
export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
}
export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
}
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
