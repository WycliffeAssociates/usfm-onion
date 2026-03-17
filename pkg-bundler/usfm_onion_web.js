/* @ts-self-types="./usfm_onion_web.d.ts" */

import * as wasm from "./usfm_onion_web_bg.wasm";
import { __wbg_set_wasm } from "./usfm_onion_web_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    ParsedUsfm, ParsedUsfmBatch, UsfmMarkerCatalog, applyTokenFix, diffTokens, diffUsfm, diffUsfmByChapter, formatRuleMeta, formatRules, formatTokenBatch, formatTokens, formatTokensMut, formatUsfm, isKnownMarker, lintCodeMeta, lintCodes, lintTokenBatch, lintTokens, lintUsfm, markerCatalog, markerInfo, parse, parseBatch, revertDiffBlock, revertDiffBlocks, tokensToHtml, tokensToUsfm
} from "./usfm_onion_web_bg.js";
