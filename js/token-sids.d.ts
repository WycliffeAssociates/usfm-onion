import type { Token } from "../pkg-bundler/usfm_onion_web.js";

/**
 * Returns cloned tokens with canonical SIDs derived from marker/number
 * structure. The supplied book code is authoritative; carried SIDs and
 * embedded book-code tokens do not influence the result.
 */
export declare function normalizeTokenSids(
  tokens: readonly Token[],
  bookCode: string,
): Token[];
