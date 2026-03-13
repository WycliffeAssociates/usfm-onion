/* @ts-self-types="./usfm_onion_web.d.ts" */

import * as wasm from "./usfm_onion_web_bg.wasm";
import { __wbg_set_wasm } from "./usfm_onion_web_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    allMarkers, applyRevertByBlockId, applyRevertsByBlockId, applyTokenFixes, astToHtml, astToTokens, astToUsj, astToUsx, astToVref, buildSidBlocks, characterMarkers, classifyTokens, convertContent, cstToken, cstTokenText, cstTokenValue, diffChapterTokenStreams, diffContent, diffSidBlocks, diffTokens, diffUsfm, diffUsfmByChapter, diffUsfmSources, diffUsfmSourcesByChapter, flattenDiffMap, formatContent, formatContents, formatFlatTokens, formatTokenBatches, fromUsj, fromUsx, intoAst, intoHtml, intoTokens, intoTokensBatch, intoTokensFromContent, intoTokensFromContents, intoUsj, intoUsx, intoVref, isBodyParagraphMarker, isCharacterMarker, isDocumentMarker, isKnownMarker, isNoteContainer, isNoteSubmarker, isParagraphMarker, isPoetryMarker, isRegularCharacterMarker, lexSources, lintCodes, lintContent, lintContents, lintDocument, lintDocumentBatch, lintFlatTokens, lintTokenBatches, markerInfo, noteMarkerFamily, noteMarkers, noteSubmarkers, packageVersion, paragraphMarkers, parseContent, parseContents, parseSources, projectContent, projectContents, projectDocument, projectUsfmBatch, pushWhitespace, replaceChapterDiffsInMap, replaceManyChapterDiffsInMap, revertDiffBlock, revertDiffBlocks, tokenFixCodes, tokenTransformChangeCodes, tokenTransformSkipReasonCodes, tokensToAst, tokensToHtml, tokensToUsfm, tokensToUsj, tokensToUsx, tokensToVref, usfmToAst, usfmToHtml, usfmToTokenVariants, usfmToTokens, usfmToUsj, usfmToUsx, usfmToVref, usjToAst, usjToTokens, usjToUsfm, usxToAst, usxToTokens, usxToUsfm
} from "./usfm_onion_web_bg.js";
