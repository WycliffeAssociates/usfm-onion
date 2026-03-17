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
     * @returns {any}
     */
    cst() {
        const ret = wasm.parsedusfm_cst(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {ParsedUsfm} other
     * @param {any | null} [options]
     * @returns {any}
     */
    diff(other, options) {
        _assertClass(other, ParsedUsfm);
        const ret = wasm.parsedusfm_diff(this.__wbg_ptr, other.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {ParsedUsfm} other
     * @param {any | null} [options]
     * @returns {any}
     */
    diffByChapter(other, options) {
        _assertClass(other, ParsedUsfm);
        const ret = wasm.parsedusfm_diffByChapter(this.__wbg_ptr, other.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any | null} [options]
     * @returns {string}
     */
    format(options) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.parsedusfm_format(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @param {any | null} [options]
     * @returns {any}
     */
    lint(options) {
        const ret = wasm.parsedusfm_lint(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any | null} [options]
     * @returns {string}
     */
    to_html(options) {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.parsedusfm_to_html(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    to_usfm() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.parsedusfm_to_usfm(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    to_usj() {
        const ret = wasm.parsedusfm_to_usj(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {string}
     */
    to_usx() {
        let deferred2_0;
        let deferred2_1;
        try {
            const ret = wasm.parsedusfm_to_usx(this.__wbg_ptr);
            var ptr1 = ret[0];
            var len1 = ret[1];
            if (ret[3]) {
                ptr1 = 0; len1 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred2_0 = ptr1;
            deferred2_1 = len1;
            return getStringFromWasm0(ptr1, len1);
        } finally {
            wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    to_vref() {
        const ret = wasm.parsedusfm_to_vref(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    tokens() {
        const ret = wasm.parsedusfm_tokens(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) ParsedUsfm.prototype[Symbol.dispose] = ParsedUsfm.prototype.free;

export class ParsedUsfmBatch {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ParsedUsfmBatch.prototype);
        obj.__wbg_ptr = ptr;
        ParsedUsfmBatchFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ParsedUsfmBatchFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_parsedusfmbatch_free(ptr, 0);
    }
    /**
     * @param {any | null} [options]
     * @returns {any}
     */
    format(options) {
        const ret = wasm.parsedusfmbatch_format(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {Array<any>}
     */
    items() {
        const ret = wasm.parsedusfmbatch_items(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any | null} [options]
     * @returns {any}
     */
    lint(options) {
        const ret = wasm.parsedusfmbatch_lint(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {any | null} [options]
     * @returns {any}
     */
    to_html(options) {
        const ret = wasm.parsedusfmbatch_to_html(this.__wbg_ptr, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    to_usfm() {
        const ret = wasm.parsedusfmbatch_to_usfm(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    to_usj() {
        const ret = wasm.parsedusfmbatch_to_usj(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    to_usx() {
        const ret = wasm.parsedusfmbatch_to_usx(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    to_vref() {
        const ret = wasm.parsedusfmbatch_to_vref(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {any}
     */
    tokens() {
        const ret = wasm.parsedusfmbatch_tokens(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) ParsedUsfmBatch.prototype[Symbol.dispose] = ParsedUsfmBatch.prototype.free;

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
     * @returns {any}
     */
    all() {
        const ret = wasm.usfmmarkercatalog_all(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @param {string} marker
     * @returns {boolean}
     */
    contains(marker) {
        const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.usfmmarkercatalog_contains(this.__wbg_ptr, ptr0, len0);
        return ret !== 0;
    }
    /**
     * @param {string} marker
     * @returns {any}
     */
    get(marker) {
        const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.usfmmarkercatalog_get(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) UsfmMarkerCatalog.prototype[Symbol.dispose] = UsfmMarkerCatalog.prototype.free;

/**
 * @param {any} left
 * @param {any} right
 * @param {any | null} [options]
 * @returns {any}
 */
export function diffTokens(left, right, options) {
    const ret = wasm.diffTokens(left, right, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} left
 * @param {string} right
 * @param {any | null} [options]
 * @returns {any}
 */
export function diffUsfm(left, right, options) {
    const ptr0 = passStringToWasm0(left, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(right, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.diffUsfm(ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} left
 * @param {string} right
 * @param {any | null} [options]
 * @returns {any}
 */
export function diffUsfmByChapter(left, right, options) {
    const ptr0 = passStringToWasm0(left, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(right, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.diffUsfmByChapter(ptr0, len0, ptr1, len1, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @returns {any}
 */
export function formatRuleMeta() {
    const ret = wasm.formatRuleMeta();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @returns {any}
 */
export function formatRules() {
    const ret = wasm.formatRules();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} token_batches
 * @param {any | null} [options]
 * @returns {any}
 */
export function formatTokenBatch(token_batches, options) {
    const ret = wasm.formatTokenBatch(token_batches, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} tokens
 * @param {any | null} [options]
 * @returns {any}
 */
export function formatTokens(tokens, options) {
    const ret = wasm.formatTokens(tokens, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} tokens
 * @param {any | null} [options]
 * @returns {any}
 */
export function formatTokensMut(tokens, options) {
    const ret = wasm.formatTokensMut(tokens, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} source
 * @param {any | null} [options]
 * @returns {string}
 */
export function formatUsfm(source, options) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.formatUsfm(ptr0, len0, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} marker
 * @returns {boolean}
 */
export function isKnownMarker(marker) {
    const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.isKnownMarker(ptr0, len0);
    return ret !== 0;
}

/**
 * @returns {any}
 */
export function lintCodeMeta() {
    const ret = wasm.lintCodeMeta();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @returns {any}
 */
export function lintCodes() {
    const ret = wasm.lintCodes();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} token_batches
 * @param {any | null} [options]
 * @returns {any}
 */
export function lintTokenBatch(token_batches, options) {
    const ret = wasm.lintTokenBatch(token_batches, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} tokens
 * @param {any | null} [options]
 * @returns {any}
 */
export function lintTokens(tokens, options) {
    const ret = wasm.lintTokens(tokens, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} source
 * @param {any | null} [options]
 * @returns {any}
 */
export function lintUsfm(source, options) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.lintUsfm(ptr0, len0, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
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
 * @returns {any}
 */
export function markerInfo(marker) {
    const ptr0 = passStringToWasm0(marker, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.markerInfo(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} source
 * @returns {ParsedUsfm}
 */
export function parse(source) {
    const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parse(ptr0, len0);
    return ParsedUsfm.__wrap(ret);
}

/**
 * @param {any} sources
 * @returns {ParsedUsfmBatch}
 */
export function parseBatch(sources) {
    const ret = wasm.parseBatch(sources);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ParsedUsfmBatch.__wrap(ret[0]);
}

/**
 * @param {any} tokens
 * @param {any | null} [options]
 * @returns {string}
 */
export function tokensToHtml(tokens, options) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.tokensToHtml(tokens, isLikeNone(options) ? 0 : addToExternrefTable0(options));
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} tokens
 * @returns {string}
 */
export function tokensToUsfm(tokens) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.tokensToUsfm(tokens);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}
export function __wbg_Error_83742b46f01ce22d(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
}
export function __wbg_Number_a5a435bd7bbec835(arg0) {
    const ret = Number(arg0);
    return ret;
}
export function __wbg_String_8564e559799eccda(arg0, arg1) {
    const ret = String(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_boolean_get_c0f3f60bac5a78d1(arg0) {
    const v = arg0;
    const ret = typeof(v) === 'boolean' ? v : undefined;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
}
export function __wbg___wbindgen_debug_string_5398f5bb970e0daa(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_in_41dbb8413020e076(arg0, arg1) {
    const ret = arg0 in arg1;
    return ret;
}
export function __wbg___wbindgen_is_function_3c846841762788c1(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
}
export function __wbg___wbindgen_is_null_0b605fc6b167c56f(arg0) {
    const ret = arg0 === null;
    return ret;
}
export function __wbg___wbindgen_is_object_781bc9f159099513(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
}
export function __wbg___wbindgen_is_string_7ef6b97b02428fae(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
}
export function __wbg___wbindgen_is_undefined_52709e72fb9f179c(arg0) {
    const ret = arg0 === undefined;
    return ret;
}
export function __wbg___wbindgen_jsval_loose_eq_5bcc3bed3c69e72b(arg0, arg1) {
    const ret = arg0 == arg1;
    return ret;
}
export function __wbg___wbindgen_number_get_34bb9d9dcfa21373(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
}
export function __wbg___wbindgen_string_get_395e606bd0ee4427(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_throw_6ddd609b62940d55(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
}
export function __wbg_call_e133b57c9155d22c() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments); }
export function __wbg_done_08ce71ee07e3bd17(arg0) {
    const ret = arg0.done;
    return ret;
}
export function __wbg_get_326e41e095fb2575() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments); }
export function __wbg_get_unchecked_329cfe50afab7352(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return ret;
}
export function __wbg_get_with_ref_key_6412cf3094599694(arg0, arg1) {
    const ret = arg0[arg1];
    return ret;
}
export function __wbg_instanceof_ArrayBuffer_101e2bf31071a9f6(arg0) {
    let result;
    try {
        result = arg0 instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Uint8Array_740438561a5b956d(arg0) {
    let result;
    try {
        result = arg0 instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_isArray_33b91feb269ff46e(arg0) {
    const ret = Array.isArray(arg0);
    return ret;
}
export function __wbg_isSafeInteger_ecd6a7f9c3e053cd(arg0) {
    const ret = Number.isSafeInteger(arg0);
    return ret;
}
export function __wbg_iterator_d8f549ec8fb061b1() {
    const ret = Symbol.iterator;
    return ret;
}
export function __wbg_length_b3416cf66a5452c8(arg0) {
    const ret = arg0.length;
    return ret;
}
export function __wbg_length_ea16607d7b61445b(arg0) {
    const ret = arg0.length;
    return ret;
}
export function __wbg_new_49d5571bd3f0c4d4() {
    const ret = new Map();
    return ret;
}
export function __wbg_new_5f486cdf45a04d78(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
}
export function __wbg_new_a70fbab9066b301f() {
    const ret = new Array();
    return ret;
}
export function __wbg_new_ab79df5bd7c26067() {
    const ret = new Object();
    return ret;
}
export function __wbg_new_typed_bccac67128ed885a() {
    const ret = new Array();
    return ret;
}
export function __wbg_next_11b99ee6237339e3() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments); }
export function __wbg_next_e01a967809d1aa68(arg0) {
    const ret = arg0.next;
    return ret;
}
export function __wbg_parsedusfm_new(arg0) {
    const ret = ParsedUsfm.__wrap(arg0);
    return ret;
}
export function __wbg_prototypesetcall_d62e5099504357e6(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
}
export function __wbg_push_e87b0e732085a946(arg0, arg1) {
    const ret = arg0.push(arg1);
    return ret;
}
export function __wbg_set_282384002438957f(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
}
export function __wbg_set_6be42768c690e380(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
}
export function __wbg_set_bf7251625df30a02(arg0, arg1, arg2) {
    const ret = arg0.set(arg1, arg2);
    return ret;
}
export function __wbg_value_21fc78aab0322612(arg0) {
    const ret = arg0.value;
    return ret;
}
export function __wbindgen_cast_0000000000000001(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return ret;
}
export function __wbindgen_cast_0000000000000002(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
}
export function __wbindgen_cast_0000000000000003(arg0) {
    // Cast intrinsic for `U64 -> Externref`.
    const ret = BigInt.asUintN(64, arg0);
    return ret;
}
export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_externrefs;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
}
const ParsedUsfmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parsedusfm_free(ptr >>> 0, 1));
const ParsedUsfmBatchFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parsedusfmbatch_free(ptr >>> 0, 1));
const UsfmMarkerCatalogFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_usfmmarkercatalog_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
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

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
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
