/* @ts-self-types="./usfm_onion_web.d.ts" */

import * as wasm from "./usfm_onion_web_bg.wasm";
import { __wbg_set_wasm } from "./usfm_onion_web_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    applyRevertByBlockId, applyRevertsByBlockId, applyTokenFixes, buildSidBlocks, convertContent, diffChapterTokenStreams, diffContent, diffFlatTokens, diffSidBlocks, diffTokens, diffUsfm, diffUsfmByChapter, diffUsfmSources, diffUsfmSourcesByChapter, flattenDiffMap, formatContent, formatContents, formatFlatTokens, formatTokenBatches, formatTokens, fromUsj, fromUsx, intoDocumentTree, intoEditorTree, intoHtml, intoTokens, intoTokensBatch, intoTokensFromContent, intoTokensFromContents, intoUsfmFromTokens, intoUsj, intoUsjFromTokens, intoUsx, intoUsxFromTokens, intoVref, intoVrefFromTokens, lexSources, lintContent, lintContents, lintDocument, lintDocumentBatch, lintFlatTokens, lintTokenBatches, lintTokens, packageVersion, parseContent, parseContents, parseSources, projectContent, projectContents, projectDocument, projectUsfmBatch, pushWhitespace, replaceChapterDiffsInMap, replaceManyChapterDiffsInMap, revertDiffBlock, revertDiffBlocks, usfmToHtml, usfmToUsj, usfmToUsx, usjToUsfm, usxToUsfm
} from "./usfm_onion_web_bg.js";
export { wasm as __wasm }
