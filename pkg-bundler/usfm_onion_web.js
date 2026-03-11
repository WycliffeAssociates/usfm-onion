/* @ts-self-types="./usfm_onion_web.d.ts" */

import * as wasm from "./usfm_onion_web_bg.wasm";
import { __wbg_set_wasm } from "./usfm_onion_web_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    applyRevertByBlockId, applyRevertsByBlockId, applyTokenFixes, buildSidBlocks, classifyTokens, convertContent, diffChapterTokenStreams, diffContent, diffSidBlocks, diffTokens, diffUsfm, diffUsfmByChapter, diffUsfmSources, diffUsfmSourcesByChapter, documentTreeToHtml, documentTreeToTokens, documentTreeToUsj, documentTreeToUsx, documentTreeToVref, flattenDiffMap, formatContent, formatContents, formatFlatTokens, formatTokenBatches, fromUsj, fromUsx, intoDocumentTree, intoHtml, intoTokens, intoTokensBatch, intoTokensFromContent, intoTokensFromContents, intoUsj, intoUsx, intoVref, lexSources, lintContent, lintContents, lintDocument, lintDocumentBatch, lintFlatTokens, lintTokenBatches, packageVersion, parseContent, parseContents, parseSources, projectContent, projectContents, projectDocument, projectUsfmBatch, pushWhitespace, replaceChapterDiffsInMap, replaceManyChapterDiffsInMap, revertDiffBlock, revertDiffBlocks, tokensToDocumentTree, tokensToHtml, tokensToUsfm, tokensToUsj, tokensToUsx, tokensToVref, usfmToDocumentTree, usfmToHtml, usfmToTokenVariants, usfmToTokens, usfmToUsj, usfmToUsx, usfmToVref, usjToDocumentTree, usjToTokens, usjToUsfm, usxToDocumentTree, usxToTokens, usxToUsfm
} from "./usfm_onion_web_bg.js";
