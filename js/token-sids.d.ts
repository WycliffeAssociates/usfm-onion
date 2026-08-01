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

/**
 * Mutable twin of {@link normalizeTokenSids}: writes `sid` onto each
 * caller-owned token in place instead of cloning. Use only when no other
 * code still holds a reference to the pre-normalization tokens.
 */
export declare function normalizeTokenSidsMut(
  tokens: Token[],
  bookCode: string,
): void;
