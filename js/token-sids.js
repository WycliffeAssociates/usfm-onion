// Canonical SID normalization for callers that already own token DTOs.
//
// This module intentionally has no wasm dependency. Token diff APIs trust
// caller-supplied SIDs, so applications can normalize a complete structural
// token stream explicitly without giving up support for granular fragments.

/**
 * Returns cloned tokens with canonical SIDs derived from marker/number
 * structure. The supplied book code is authoritative; carried SIDs and
 * embedded book-code tokens do not influence the result.
 *
 * @template {Record<string, unknown> & {
 *   kind: string,
 *   marker?: string,
 *   numberInfo?: { start: number, end?: number }
 * }} T
 * @param {readonly T[]} tokens
 * @param {string} bookCode
 * @returns {(T & { sid: string })[]}
 */
export function normalizeTokenSids(tokens, bookCode) {
  let chapter = 0;
  let currentSid = `${bookCode} 0:0`;
  const seenThisChapter = new Map();

  return tokens.map((token, index) => {
    if (token.kind === "marker") {
      const numberInfo = tokens[index + 1]?.numberInfo;

      if (token.marker === "c" && numberInfo) {
        chapter = numberInfo.start;
        currentSid = `${bookCode} ${chapter}:0`;
        seenThisChapter.clear();
      } else if (token.marker === "v" && numberInfo) {
        const start = numberInfo.start;
        const end = numberInfo.end ?? start;
        const rangeBase = end === start
          ? `${bookCode} ${chapter}:${start}`
          : `${bookCode} ${chapter}:${start}-${end}`;
        const occurrence = seenThisChapter.get(rangeBase) ?? 0;

        currentSid = occurrence === 0
          ? rangeBase
          : `${rangeBase}_dup_${occurrence}`;
        seenThisChapter.set(rangeBase, occurrence + 1);
      }
    }

    return { ...token, sid: currentSid };
  });
}
