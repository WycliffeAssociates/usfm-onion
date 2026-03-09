#!/usr/bin/env python3
"""Fix USFM formatting by adding proper blank lines before \\b, \\p, \\ts\\*, and \\c tags.

Processes all .usfm files in the current directory (except A0-FRT.usfm) and ensures:
  - \\b always has a blank line before it.
  - \\ts\\* has a blank line before it unless preceded by \\b.
  - \\p has a blank line before it unless preceded by \\ts\\*, \\c <integer>, or \\b.
  - \\c <integer> has a blank line before it unless preceded by \\p, \\ts\\*, or \\b.
  - No blank line after \\b, \\ts\\*, \\p, or \\c (removes them).
  - Swaps \\ts\\* / \\b to \\b / \\ts\\* (\\b must come before \\ts\\*).

Usage:
    python fix_usfm_formatting.py [--dry-run] [--book BOOK ...]

Options:
    --dry-run   Report what would be changed without modifying files.
    --book      One or more book filenames (e.g. 01-GEN.usfm) to process.
                If omitted, all .usfm files except A0-FRT.usfm are processed.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


_CHAPTER_RE = re.compile(r"^\\c\s+\d+\s*$")


def fix_file(usfm_path: Path, dry_run: bool = False) -> int:
    """Fix blank-line formatting in a single USFM file.

    Returns the number of changes made (blank lines inserted or removed).
    """
    content = usfm_path.read_text(encoding="utf-8")
    lines = content.split("\n")
    changes = 0

    # ------------------------------------------------------------------
    # Pre-pass: swap  \ts\*  /  \b  →  \b  /  \ts\*
    # Handles optional blank lines between them (they get dropped).
    # ------------------------------------------------------------------
    swapped: list[str] = []
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped == "\\ts\\*":
            # Look ahead past blank lines for \b
            j = i + 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            if j < len(lines) and lines[j].strip() == "\\b":
                # Emit \b first, then \ts\* (drop any blanks between)
                swapped.append(lines[j])   # \b
                swapped.append(lines[i])   # \ts\*
                changes += 1
                i = j + 1
                continue
        swapped.append(lines[i])
        i += 1
    lines = swapped

    # ------------------------------------------------------------------
    # Main pass: add / remove blank lines
    # ------------------------------------------------------------------
    result: list[str] = []
    in_header = True

    for i, raw_line in enumerate(lines):
        stripped = raw_line.strip()

        if in_header:
            result.append(raw_line)
            if not stripped:
                in_header = False
            continue

        # Determine what the previous non-blank line is in the result so far.
        prev_non_blank = ""
        prev_line_blank = False
        if result:
            for j in range(len(result) - 1, -1, -1):
                if result[j].strip() == "":
                    prev_line_blank = True
                    continue
                prev_non_blank = result[j].strip()
                break

        is_b = stripped == "\\b"
        is_ts_star = stripped == "\\ts\\*"
        is_p = stripped == "\\p"
        is_c = bool(_CHAPTER_RE.match(stripped))

        # ---- Remove blank lines after \c, \b, \ts\*, \p ----
        if not stripped:
            prev_is_c = bool(_CHAPTER_RE.match(prev_non_blank))
            prev_is_b = prev_non_blank == "\\b"
            prev_is_ts = prev_non_blank == "\\ts\\*"
            prev_is_p = prev_non_blank == "\\p"
            if prev_is_c or prev_is_b or prev_is_ts or prev_is_p:
                changes += 1
                continue

        # ---- Add blank lines where needed ----

        if is_b:
            # \b must ALWAYS have a blank line before it.
            if not prev_line_blank:
                result.append("")
                changes += 1

        elif is_ts_star:
            # \ts\* needs blank line before UNLESS preceded by \b or \ts\*.
            # (\ts\* can't have a blank after it, so consecutive \ts\* are OK.)
            if not prev_line_blank:
                prev_is_b = prev_non_blank == "\\b"
                prev_is_ts = prev_non_blank == "\\ts\\*"
                if not prev_is_b and not prev_is_ts:
                    result.append("")
                    changes += 1

        elif is_p:
            # \p needs blank line before UNLESS preceded by \ts\*, \c, or \b.
            if not prev_line_blank:
                prev_is_ts = prev_non_blank == "\\ts\\*"
                prev_is_c = bool(_CHAPTER_RE.match(prev_non_blank))
                prev_is_b = prev_non_blank == "\\b"
                if not prev_is_ts and not prev_is_c and not prev_is_b:
                    result.append("")
                    changes += 1

        elif is_c:
            # \c needs blank line before UNLESS preceded by \p, \ts\*, or \b.
            if not prev_line_blank:
                prev_is_ts = prev_non_blank == "\\ts\\*"
                prev_is_p = prev_non_blank == "\\p"
                prev_is_b = prev_non_blank == "\\b"
                if not prev_is_ts and not prev_is_p and not prev_is_b:
                    result.append("")
                    changes += 1

        result.append(raw_line)

    if changes > 0:
        new_content = "\n".join(result)
        if not dry_run:
            usfm_path.write_text(new_content, encoding="utf-8")

    return changes


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fix blank-line formatting in USFM files."
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Report what would be changed without modifying files.",
    )
    parser.add_argument(
        "--book",
        dest="books",
        action="append",
        help="Book filename(s) to process (e.g. 01-GEN.usfm). Repeatable.",
    )
    args = parser.parse_args()

    cwd = Path(".")

    if args.books:
        usfm_files = sorted(cwd / b for b in args.books)
    else:
        usfm_files = sorted(
            p for p in cwd.glob("*.usfm")
            if p.name.upper() != "A0-FRT.USFM"
        )

    if not usfm_files:
        print("No USFM files found to process.")
        return 0

    total_changes = 0
    files_changed = 0

    for usfm_path in usfm_files:
        if not usfm_path.is_file():
            print(f"WARNING: {usfm_path.name} not found, skipping.")
            continue

        changes = fix_file(usfm_path, dry_run=args.dry_run)
        if changes > 0:
            action = "would fix" if args.dry_run else "fixed"
            print(f"  {usfm_path.name}: {action} {changes} blank-line issue(s)")
            total_changes += changes
            files_changed += 1

    if total_changes == 0:
        print("All files already have correct blank-line formatting.")
    else:
        action = "Would fix" if args.dry_run else "Fixed"
        print(
            f"\n{action} {total_changes} blank-line issue(s) "
            f"across {files_changed} file(s)."
        )

    return 0


if __name__ == "__main__":
    sys.exit(main())
