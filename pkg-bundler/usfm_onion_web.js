/* @ts-self-types="./usfm_onion_web.d.ts" */

import * as wasm from "./usfm_onion_web_bg.wasm";
import { __wbg_set_wasm } from "./usfm_onion_web_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    Braid, ParsedUsfm, UsfmMarkerCatalog, applyTokenFix, diffTokens, diffUsfm, diffUsfmByChapter, formatRuleMeta, formatRules, formatTokens, formatTokensMut, formatUsfm, isKnownMarker, lintCodeMeta, lintCodes, lintTokens, lintUsfm, markerCatalog, markerInfo, mergeDiffBlocks, normalizeTokenSids, parse, revertDiffBlock, tokensToHtml, tokensToUsfm, verifyPackedBook, verifyPublishedCorpus, vrefIndexTokens, vrefIndexUsfm
} from "./usfm_onion_web_bg.js";
export { wasm as __wasm }
