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
 * A `\c` label repeated later in the same stream gets a per-book positional
 * `_cdup_N` suffix on every sid it produces (chapter-open pseudo-sid and
 * every verse under it), riding in the verse segment so the chapter segment
 * stays a bare integer for consumers that parse it directly (mirrors
 * `usfm_onion::diff::derive_canonical_sids`, onion's Rust twin of this
 * function). Verse-duplicate `_dup_N` counting resets for every chapter
 * occurrence, same as it already resets on every `\c`.
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
  let chapterSuffix = "";
  let currentSid = `${bookCode} 0:0`;
  const seenThisChapter = new Map();
  const seenChapters = new Map();

  return tokens.map((token, index) => {
    if (token.kind === "marker") {
      const numberInfo = tokens[index + 1]?.numberInfo;

      if (token.marker === "c" && numberInfo) {
        chapter = numberInfo.start;
        const chapterOccurrence = seenChapters.get(chapter) ?? 0;
        chapterSuffix = chapterOccurrence === 0 ? "" : `_cdup_${chapterOccurrence}`;
        seenChapters.set(chapter, chapterOccurrence + 1);
        currentSid = `${bookCode} ${chapter}:0${chapterSuffix}`;
        seenThisChapter.clear();
      } else if (token.marker === "v" && numberInfo) {
        const start = numberInfo.start;
        const end = numberInfo.end ?? start;
        const rangeBase = end === start
          ? `${bookCode} ${chapter}:${start}${chapterSuffix}`
          : `${bookCode} ${chapter}:${start}-${end}${chapterSuffix}`;
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

/**
 * Mutable twin of {@link normalizeTokenSids}: identical SID derivation, but
 * writes `sid` onto each caller-owned token in place instead of allocating a
 * cloned token and a new result array. Callers that already materialize
 * fresh token arrays before committing them (so no other code still holds a
 * reference to the pre-normalization tokens) can use this to skip a clone
 * pass over every token. `normalizeTokenSids` keeps its non-mutating
 * contract; this function does not replace it.
 *
 * @template {Record<string, unknown> & {
 *   kind: string,
 *   marker?: string,
 *   numberInfo?: { start: number, end?: number },
 *   sid?: string
 * }} T
 * @param {T[]} tokens
 * @param {string} bookCode
 * @returns {void}
 */
export function normalizeTokenSidsMut(tokens, bookCode) {
  let chapter = 0;
  let chapterSuffix = "";
  let currentSid = `${bookCode} 0:0`;
  const seenThisChapter = new Map();
  const seenChapters = new Map();

  for (let index = 0; index < tokens.length; index++) {
    const token = tokens[index];
    if (token.kind === "marker") {
      const numberInfo = tokens[index + 1]?.numberInfo;

      if (token.marker === "c" && numberInfo) {
        chapter = numberInfo.start;
        const chapterOccurrence = seenChapters.get(chapter) ?? 0;
        chapterSuffix = chapterOccurrence === 0 ? "" : `_cdup_${chapterOccurrence}`;
        seenChapters.set(chapter, chapterOccurrence + 1);
        currentSid = `${bookCode} ${chapter}:0${chapterSuffix}`;
        seenThisChapter.clear();
      } else if (token.marker === "v" && numberInfo) {
        const start = numberInfo.start;
        const end = numberInfo.end ?? start;
        const rangeBase = end === start
          ? `${bookCode} ${chapter}:${start}${chapterSuffix}`
          : `${bookCode} ${chapter}:${start}-${end}${chapterSuffix}`;
        const occurrence = seenThisChapter.get(rangeBase) ?? 0;

        currentSid = occurrence === 0
          ? rangeBase
          : `${rangeBase}_dup_${occurrence}`;
        seenThisChapter.set(rangeBase, occurrence + 1);
      }
    }

    token.sid = currentSid;
  }
}
