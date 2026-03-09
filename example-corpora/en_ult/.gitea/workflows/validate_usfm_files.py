#!/usr/bin/env python3
"""Validate USFM files and manifest consistency."""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml


# ---------------------------------------------------------------------------
# Check names
# ---------------------------------------------------------------------------

CHECK_VALID_MANIFEST = "1. Valid Manifest File Check"
CHECK_PROJECT_USFM_FILES_EXIST = "2. Project USFM Files Exist Check"
CHECK_USFM_HEADER = "3. USFM Header Check"
CHECK_CHAPTER_ORDER_AND_COUNT = "4. Chapter Order and Count Check"
CHECK_VERSE_ORDER_AND_COVERAGE = "5. Verse Order and Coverage Check"
CHECK_FOOTNOTE_SYNTAX = "6. Footnote Syntax Check"
CHECK_CONSECUTIVE_PARAGRAPH_MARKERS = "7. Consecutive Paragraph Markers Check"
CHECK_USFM_FORMATTING = "8. USFM Formatting Check"

CHECK_NUMBER_TO_NAME = {
    1: CHECK_VALID_MANIFEST,
    2: CHECK_PROJECT_USFM_FILES_EXIST,
    3: CHECK_USFM_HEADER,
    4: CHECK_CHAPTER_ORDER_AND_COUNT,
    5: CHECK_VERSE_ORDER_AND_COVERAGE,
    6: CHECK_FOOTNOTE_SYNTAX,
    7: CHECK_CONSECUTIVE_PARAGRAPH_MARKERS,
    8: CHECK_USFM_FORMATTING,
}
CHECK_DISPLAY_ORDER = [CHECK_NUMBER_TO_NAME[i] for i in sorted(CHECK_NUMBER_TO_NAME.keys())]

# ---------------------------------------------------------------------------
# Book chapter/verse data  (index 0 = chapter 1)
# ---------------------------------------------------------------------------

# fmt: off
BOOK_CHAPTER_VERSES: dict[str, list[int]] = {
    "gen": [31,25,24,26,32,22,24,22,29,32,32,20,18,24,21,16,27,33,38,18,34,24,20,67,34,35,46,22,35,43,55,32,20,31,29,43,36,30,23,23,57,38,34,34,28,34,31,22,33,26],
    "exo": [22,25,22,31,23,30,25,32,35,29,10,51,22,31,27,36,16,27,25,26,36,31,33,18,40,37,21,43,46,38,18,35,23,35,35,38,29,31,43,38],
    "lev": [17,16,17,35,19,30,38,36,24,20,47,8,59,57,33,34,16,30,37,27,24,33,44,23,55,46,34],
    "num": [54,34,51,49,31,27,89,26,23,36,35,16,33,45,41,50,13,32,22,29,35,41,30,25,18,65,23,31,40,16,54,42,56,29,34,13],
    "deu": [46,37,29,49,33,25,26,20,29,22,32,32,18,29,23,22,20,22,21,20,23,30,25,22,19,19,26,68,29,20,30,52,29,12],
    "jos": [18,24,17,24,15,27,26,35,27,43,23,24,33,15,63,10,18,28,51,9,45,34,16,33],
    "jdg": [36,23,31,24,31,40,25,35,57,18,40,15,25,20,20,31,13,31,30,48,25],
    "rut": [22,23,18,22],
    "1sa": [28,36,21,22,12,21,17,22,27,27,15,25,23,52,35,23,58,30,24,42,15,23,29,22,44,25,12,25,11,31,13],
    "2sa": [27,32,39,12,25,23,29,18,13,19,27,31,39,33,37,23,29,33,43,26,22,51,39,25],
    "1ki": [53,46,28,34,18,38,51,66,28,29,43,33,34,31,34,34,24,46,21,43,29,53],
    "2ki": [18,25,27,44,27,33,20,29,37,36,21,21,25,29,38,20,41,37,37,21,26,20,37,20,30],
    "1ch": [54,55,24,43,26,81,40,40,44,14,47,40,14,17,29,43,27,17,19,8,30,19,32,31,31,32,34,21,30],
    "2ch": [17,18,17,22,14,42,22,18,31,19,23,16,22,15,19,14,19,34,11,37,20,12,21,27,28,23,9,27,36,27,21,33,25,33,27,23],
    "ezr": [11,70,13,24,17,22,28,36,15,44],
    "neh": [11,20,32,23,19,19,73,18,38,39,36,47,31],
    "est": [22,23,15,17,14,14,10,17,32,3],
    "job": [22,13,26,21,27,30,21,22,35,22,20,25,28,22,35,22,16,21,29,29,34,30,17,25,6,14,23,28,25,31,40,22,33,37,16,33,24,41,30,24,34,17],
    "psa": [6,12,8,8,12,10,17,9,20,18,7,8,6,7,5,11,15,50,14,9,13,31,6,10,22,12,14,9,11,12,24,11,22,22,28,12,40,22,13,17,13,11,5,26,17,11,9,14,20,23,19,9,6,7,23,13,11,11,17,12,8,12,11,10,13,20,7,35,36,5,24,20,28,23,10,12,20,72,13,19,16,8,18,12,13,17,7,18,52,17,16,15,5,23,11,13,12,9,9,5,8,28,22,35,45,48,43,13,31,7,10,10,9,8,18,19,2,29,176,7,8,9,4,8,5,6,5,6,8,8,3,18,3,3,21,26,9,8,24,13,10,7,12,15,21,10,20,14,9,6],
    "pro": [33,22,35,27,23,35,27,36,18,32,31,28,25,35,33,33,28,24,29,30,31,29,35,34,28,28,27,28,27,33,31],
    "ecc": [18,26,22,16,20,12,29,17,18,20,10,14],
    "sng": [17,17,11,16,16,13,13,14],
    "isa": [31,22,26,6,30,13,25,22,21,34,16,6,22,32,9,14,14,7,25,6,17,25,18,23,12,21,13,29,24,33,9,20,24,17,10,22,38,22,8,31,29,25,28,28,25,13,15,22,26,11,23,15,12,17,13,12,21,14,21,22,11,12,19,12,25,24],
    "jer": [19,37,25,31,31,30,34,22,26,25,23,17,27,22,21,21,27,23,15,18,14,30,40,10,38,24,22,17,32,24,40,44,26,22,19,32,21,28,18,16,18,22,13,30,5,28,7,47,39,46,64,34],
    "lam": [22,22,66,22,22],
    "ezk": [28,10,27,17,17,14,27,18,11,22,25,28,23,23,8,63,24,32,14,49,32,31,49,27,17,21,36,26,21,26,18,32,33,31,15,38,28,23,29,49,26,20,27,31,25,24,23,35],
    "dan": [21,49,30,37,31,28,28,27,27,21,45,13],
    "hos": [11,23,5,19,15,11,16,14,17,15,12,14,16,9],
    "jol": [20,32,21],
    "amo": [15,16,15,13,27,14,17,14,15],
    "oba": [21],
    "jon": [17,10,10,11],
    "mic": [16,13,12,13,15,16,20],
    "nam": [15,13,19],
    "hab": [17,20,19],
    "zep": [18,15,20],
    "hag": [15,23],
    "zec": [21,13,10,14,11,15,14,23,17,12,17,14,9,21],
    "mal": [14,17,18,6],
    "mat": [25,23,17,25,48,34,29,34,38,42,30,50,58,36,39,28,27,35,30,34,46,46,39,51,46,75,66,20],
    "mrk": [45,28,35,41,43,56,37,38,50,52,33,44,37,72,47,20],
    "luk": [80,52,38,44,39,49,50,56,62,42,54,59,35,35,32,31,37,43,48,47,38,71,56,53],
    "jhn": [51,25,36,54,47,71,53,59,41,42,57,50,38,31,27,33,26,40,42,31,25],
    "act": [26,47,26,37,42,15,60,40,43,48,30,25,52,28,41,40,34,28,41,38,40,30,35,27,27,32,44,31],
    "rom": [32,29,31,25,21,23,25,39,33,21,36,21,14,23,33,27],
    "1co": [31,16,23,21,13,20,40,13,27,33,34,31,13,40,58,24],
    "2co": [24,17,18,18,21,18,16,24,15,18,33,21,13],
    "gal": [24,21,29,31,26,18],
    "eph": [23,22,21,32,33,24],
    "php": [30,30,21,23],
    "col": [29,23,25,18],
    "1th": [10,20,13,18,28],
    "2th": [12,17,18],
    "1ti": [20,15,16,16,25,21],
    "2ti": [18,26,17,22],
    "tit": [16,15,15],
    "phm": [25],
    "heb": [14,18,19,16,14,20,28,13,28,39,40,29,25],
    "jas": [27,26,18,17,20],
    "1pe": [25,25,22,19,14],
    "2pe": [21,22,18],
    "1jn": [10,29,24,21,21],
    "2jn": [13],
    "3jn": [15],
    "jud": [25],
    "rev": [20,29,22,11,14,17,17,13,21,11,19,18,18,20,8,21,18,24,21,15,27,21],
}
# fmt: on


# ---------------------------------------------------------------------------
# Core classes
# ---------------------------------------------------------------------------

@dataclass
class ValidationError:
    rule: str
    message: str
    file: str | None = None
    line: int | None = None
    ref: str | None = None

    def display(self) -> str:
        parts = [f"[{self.rule}]"]
        if self.file is not None:
            parts.append(f"File: {self.file}")
        if self.line is not None:
            parts.append(f"Line: {self.line}")
        if self.ref is not None:
            parts.append(f"Ref: {self.ref}")
        parts.append(self.message)
        return " | ".join(parts)


class ErrorCollector:
    def __init__(self, max_errors: int, enabled_checks: set[str]) -> None:
        self.max_errors = max_errors
        self.enabled_checks = enabled_checks
        self.errors: list[ValidationError] = []
        self.truncated = False

    def add(
        self,
        *,
        rule: str,
        message: str,
        file: str | None = None,
        line: int | None = None,
        ref: str | None = None,
    ) -> None:
        if not self.is_enabled(rule):
            return
        if len(self.errors) >= self.max_errors:
            self.truncated = True
            return
        self.errors.append(
            ValidationError(rule=rule, message=message, file=file, line=line, ref=ref)
        )

    def is_enabled(self, check_name: str) -> bool:
        return check_name in self.enabled_checks

    def has_errors(self) -> bool:
        return bool(self.errors) or self.truncated


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def normalize_repo_path(path_value: str) -> str:
    normalized = path_value.strip().replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    while normalized.startswith("/"):
        normalized = normalized[1:]
    return normalized


def capitalize_identifier(identifier: str) -> str:
    """Capitalize the first alphabetic character: gen->Gen, 1sa->1Sa."""
    result = list(identifier)
    for i, c in enumerate(result):
        if c.isalpha():
            result[i] = c.upper()
            break
    return "".join(result)


def get_repo_name(manifest_data: dict[str, Any]) -> str:
    """Derive repo name from manifest: <lang>_<identifier>."""
    dc = manifest_data.get("dublin_core", {})
    lang_id = dc.get("language", {}).get("identifier", "")
    resource_id = dc.get("identifier", "")
    if lang_id and resource_id:
        return f"{lang_id}_{resource_id}"
    return ""


def format_verse_list(verses: list[int]) -> str:
    """Format verse numbers into ranges: [1,2,3,5,7,8,9] -> '1-3, 5, 7-9'."""
    if not verses:
        return ""
    ranges: list[str] = []
    start = end = verses[0]
    for v in verses[1:]:
        if v == end + 1:
            end = v
        else:
            ranges.append(str(start) if start == end else f"{start}-{end}")
            start = end = v
    ranges.append(str(start) if start == end else f"{start}-{end}")
    return ", ".join(ranges)


# ---------------------------------------------------------------------------
# Manifest loading / project parsing
# ---------------------------------------------------------------------------

def load_manifest(manifest_path: Path, errors: ErrorCollector) -> dict[str, Any] | None:
    try:
        with manifest_path.open("r", encoding="utf-8") as f:
            data = yaml.safe_load(f)
    except Exception as exc:
        errors.add(rule=CHECK_VALID_MANIFEST, message=f"{manifest_path} is not valid YAML: {exc}")
        return None
    if not isinstance(data, dict):
        errors.add(rule=CHECK_VALID_MANIFEST, message=f"{manifest_path} must parse to a YAML mapping.")
        return None
    return data


def parse_manifest_projects(
    manifest_data: dict[str, Any], errors: ErrorCollector
) -> list[dict[str, Any]]:
    """Return a list of project dicts, each augmented with '_normalized_path'."""
    projects = manifest_data.get("projects")
    if not isinstance(projects, list):
        errors.add(
            rule=CHECK_PROJECT_USFM_FILES_EXIST,
            message="manifest.yaml key 'projects' must be a list.",
        )
        return []

    result: list[dict[str, Any]] = []
    for idx, project in enumerate(projects, 1):
        if not isinstance(project, dict):
            errors.add(
                rule=CHECK_PROJECT_USFM_FILES_EXIST,
                message=f"manifest project #{idx} is not an object.",
            )
            continue
        raw_path = project.get("path")
        if not isinstance(raw_path, str) or not raw_path.strip():
            errors.add(
                rule=CHECK_PROJECT_USFM_FILES_EXIST,
                message=f"manifest project #{idx} has no valid 'path'.",
            )
            continue
        path = normalize_repo_path(raw_path)
        if not path.lower().endswith(".usfm"):
            errors.add(
                rule=CHECK_PROJECT_USFM_FILES_EXIST,
                message=f"manifest project path '{raw_path}' is not a .usfm file path.",
            )
            continue
        proj = dict(project)
        proj["_normalized_path"] = path
        result.append(proj)

    paths = [p["_normalized_path"] for p in result]
    if len(set(paths)) != len(paths):
        seen: set[str] = set()
        dupes: list[str] = []
        for p in paths:
            if p in seen and p not in dupes:
                dupes.append(p)
            seen.add(p)
        errors.add(
            rule=CHECK_PROJECT_USFM_FILES_EXIST,
            message=f"manifest has duplicate project path(s): {', '.join(sorted(dupes))}",
        )
    return result


# ---------------------------------------------------------------------------
# USFM header validation
# ---------------------------------------------------------------------------

def validate_usfm_header(
    lines: list[str],
    file_name: str,
    project: dict[str, Any],
    repo_name: str,
    errors: ErrorCollector,
) -> None:
    """Check the 8 required header lines."""
    header: list[str] = []
    for line in lines:
        stripped = line.strip()
        if not stripped:
            break
        if stripped.startswith("\\ts\\*") or stripped.startswith("\\c ") or stripped == "\\p":
            break
        header.append(stripped)

    identifier = project.get("identifier", "").lower()
    title = project.get("title", "")
    book_id_upper = identifier.upper()
    repo_upper = repo_name.upper()
    expected_toc3 = capitalize_identifier(identifier)

    if len(header) < 8:
        errors.add(
            rule=CHECK_USFM_HEADER,
            file=file_name,
            message=f"Header has only {len(header)} lines; expected at least 8.",
        )

    # Line 1: \id <BOOKID> <REPONAME> <one or more chars>
    if len(header) >= 1:
        expected_prefix = f"\\id {book_id_upper} {repo_upper} "
        if not header[0].startswith(expected_prefix):
            errors.add(
                rule=CHECK_USFM_HEADER, file=file_name, line=1,
                message=f"Expected line to start with '{expected_prefix}' but found '{header[0][:60]}'.",
            )

    # Line 2: \usfm 3.0
    if len(header) >= 2 and header[1] != "\\usfm 3.0":
        errors.add(
            rule=CHECK_USFM_HEADER, file=file_name, line=2,
            message=f"Expected '\\usfm 3.0' but found '{header[1]}'.",
        )

    # Line 3: \ide UTF-8
    if len(header) >= 3 and header[2] != "\\ide UTF-8":
        errors.add(
            rule=CHECK_USFM_HEADER, file=file_name, line=3,
            message=f"Expected '\\ide UTF-8' but found '{header[2]}'.",
        )

    # Line 4: \h <title>
    if len(header) >= 4:
        expected_h = f"\\h {title}"
        if header[3] != expected_h:
            errors.add(
                rule=CHECK_USFM_HEADER, file=file_name, line=4,
                message=f"Expected '{expected_h}' but found '{header[3]}'.",
            )

    # Line 5: \toc1 <one or more chars>
    if len(header) >= 5:
        if not header[4].startswith("\\toc1 ") or len(header[4]) <= 6:
            errors.add(
                rule=CHECK_USFM_HEADER, file=file_name, line=5,
                message=f"Expected '\\toc1 <text>' but found '{header[4]}'.",
            )

    # Line 6: \toc2 <one or more chars>
    if len(header) >= 6:
        if not header[5].startswith("\\toc2 ") or len(header[5]) <= 6:
            errors.add(
                rule=CHECK_USFM_HEADER, file=file_name, line=6,
                message=f"Expected '\\toc2 <text>' but found '{header[5]}'.",
            )

    # Line 7: \toc3 <capitalized identifier>
    if len(header) >= 7:
        expected_toc3_line = f"\\toc3 {expected_toc3}"
        if header[6] != expected_toc3_line:
            errors.add(
                rule=CHECK_USFM_HEADER, file=file_name, line=7,
                message=f"Expected '{expected_toc3_line}' but found '{header[6]}'.",
            )

    # Line 8: \mt <one or more chars>
    if len(header) >= 8:
        if not header[7].startswith("\\mt ") or len(header[7]) <= 4:
            errors.add(
                rule=CHECK_USFM_HEADER, file=file_name, line=8,
                message=f"Expected '\\mt <text>' but found '{header[7]}'.",
            )


# ---------------------------------------------------------------------------
# USFM content validation (chapters, verses, footnotes, paragraphs)
# ---------------------------------------------------------------------------

PARAGRAPH_MARKERS = frozenset(("\\p", "\\m", "\\pi", "\\mi", "\\nb", "\\cls"))


def validate_usfm_content(
    lines: list[str],
    file_name: str,
    book_id: str,
    errors: ErrorCollector,
) -> None:
    expected_chapters = BOOK_CHAPTER_VERSES.get(book_id)
    if expected_chapters is None:
        errors.add(
            rule=CHECK_CHAPTER_ORDER_AND_COUNT, file=file_name,
            message=f"Unknown book identifier '{book_id}'; cannot validate chapters/verses.",
        )
        return

    current_chapter = 0
    last_verse_end = 0
    chapter_verse_sets: dict[int, set[int]] = {}
    chapter_line_map: dict[int, int] = {}
    chapters_in_order: list[int] = []

    footnote_depth = 0
    prev_was_paragraph = False
    prev_paragraph_line = 0

    check_chapters = errors.is_enabled(CHECK_CHAPTER_ORDER_AND_COUNT)
    check_verses = errors.is_enabled(CHECK_VERSE_ORDER_AND_COVERAGE)
    check_footnotes = errors.is_enabled(CHECK_FOOTNOTE_SYNTAX)
    check_paragraphs = errors.is_enabled(CHECK_CONSECUTIVE_PARAGRAPH_MARKERS)

    for line_number, raw_line in enumerate(lines, 1):
        stripped = raw_line.strip()
        if not stripped:
            prev_was_paragraph = False
            continue

        # --- Chapter markers ---
        c_match = re.search(r"\\c\s+(\d+)", stripped)
        if c_match:
            chapter_num = int(c_match.group(1))
            if check_chapters:
                if chapters_in_order and chapter_num <= chapters_in_order[-1]:
                    errors.add(
                        rule=CHECK_CHAPTER_ORDER_AND_COUNT,
                        file=file_name, line=line_number, ref=str(chapter_num),
                        message=(
                            f"Chapter {chapter_num} is out of order "
                            f"(previous chapter was {chapters_in_order[-1]})."
                        ),
                    )
                if chapter_num in chapter_verse_sets:
                    errors.add(
                        rule=CHECK_CHAPTER_ORDER_AND_COUNT,
                        file=file_name, line=line_number, ref=str(chapter_num),
                        message=(
                            f"Duplicate chapter {chapter_num} "
                            f"(first seen at line {chapter_line_map[chapter_num]})."
                        ),
                    )
            chapters_in_order.append(chapter_num)
            chapter_verse_sets.setdefault(chapter_num, set())
            chapter_line_map.setdefault(chapter_num, line_number)
            current_chapter = chapter_num
            last_verse_end = 0
            prev_was_paragraph = False
            continue

        # --- Verse markers ---
        v_match = re.search(r"\\v\s+(\d+)(?:-(\d+))?", stripped)
        if v_match and current_chapter > 0:
            start_v = int(v_match.group(1))
            end_v = int(v_match.group(2)) if v_match.group(2) else start_v
            ref = (
                f"{current_chapter}:{start_v}"
                if start_v == end_v
                else f"{current_chapter}:{start_v}-{end_v}"
            )

            if check_verses:
                if start_v <= last_verse_end and last_verse_end > 0:
                    errors.add(
                        rule=CHECK_VERSE_ORDER_AND_COVERAGE,
                        file=file_name, line=line_number, ref=ref,
                        message=(
                            f"Verse {start_v} is out of order "
                            f"(previous verse ended at {last_verse_end})."
                        ),
                    )
                if current_chapter in chapter_verse_sets:
                    for v in range(start_v, end_v + 1):
                        if v in chapter_verse_sets[current_chapter]:
                            errors.add(
                                rule=CHECK_VERSE_ORDER_AND_COVERAGE,
                                file=file_name, line=line_number,
                                ref=f"{current_chapter}:{v}",
                                message=f"Duplicate verse {v} in chapter {current_chapter}.",
                            )
                        chapter_verse_sets[current_chapter].add(v)
            last_verse_end = end_v

        # --- Footnote pairing ---
        if check_footnotes:
            f_opens = len(re.findall(r"\\f ", stripped))
            f_closes = stripped.count("\\f*")
            footnote_depth += f_opens - f_closes
            if footnote_depth < 0:
                errors.add(
                    rule=CHECK_FOOTNOTE_SYNTAX,
                    file=file_name, line=line_number,
                    ref=f"{current_chapter}:{last_verse_end}" if current_chapter else None,
                    message="Closing \\f* without a matching opening \\f.",
                )
                footnote_depth = 0

        # --- Consecutive paragraph markers ---
        if check_paragraphs:
            is_para = stripped in PARAGRAPH_MARKERS
            if is_para and prev_was_paragraph:
                errors.add(
                    rule=CHECK_CONSECUTIVE_PARAGRAPH_MARKERS,
                    file=file_name, line=line_number,
                    ref=str(current_chapter) if current_chapter else None,
                    message=(
                        f"Consecutive paragraph marker at line {line_number} "
                        f"(previous at line {prev_paragraph_line})."
                    ),
                )
            if is_para:
                prev_was_paragraph = True
                prev_paragraph_line = line_number
            else:
                prev_was_paragraph = False

    # --- Post-scan: chapter count ---
    if check_chapters:
        num_expected = len(expected_chapters)
        unique_chapters = sorted(chapter_verse_sets.keys())

        if len(unique_chapters) != num_expected:
            errors.add(
                rule=CHECK_CHAPTER_ORDER_AND_COUNT, file=file_name,
                message=(
                    f"Expected {num_expected} chapter(s) but found {len(unique_chapters)}."
                ),
            )

        expected_seq = set(range(1, num_expected + 1))
        missing_ch = sorted(expected_seq - set(unique_chapters))
        if missing_ch:
            errors.add(
                rule=CHECK_CHAPTER_ORDER_AND_COUNT, file=file_name,
                message=f"Missing chapter(s): {', '.join(str(c) for c in missing_ch)}.",
            )
        extra_ch = sorted(set(unique_chapters) - expected_seq)
        if extra_ch:
            errors.add(
                rule=CHECK_CHAPTER_ORDER_AND_COUNT, file=file_name,
                message=f"Unexpected chapter(s): {', '.join(str(c) for c in extra_ch)}.",
            )

    # --- Post-scan: verse coverage per chapter ---
    if check_verses:
        for ch_idx, expected_count in enumerate(expected_chapters, 1):
            if ch_idx not in chapter_verse_sets:
                continue
            expected_vs = set(range(1, expected_count + 1))
            actual_vs = chapter_verse_sets[ch_idx]
            missing_vs = sorted(expected_vs - actual_vs)
            if missing_vs:
                errors.add(
                    rule=CHECK_VERSE_ORDER_AND_COVERAGE,
                    file=file_name, ref=str(ch_idx),
                    message=(
                        f"Missing verse(s) in chapter {ch_idx}: "
                        f"{format_verse_list(missing_vs)}. "
                        f"Expected {expected_count} verses."
                    ),
                )
            extra_vs = sorted(actual_vs - expected_vs)
            if extra_vs:
                errors.add(
                    rule=CHECK_VERSE_ORDER_AND_COVERAGE,
                    file=file_name, ref=str(ch_idx),
                    message=(
                        f"Unexpected verse(s) in chapter {ch_idx}: "
                        f"{format_verse_list(extra_vs)}. "
                        f"Expected {expected_count} verses."
                    ),
                )

    # --- Post-scan: unclosed footnotes ---
    if check_footnotes and footnote_depth > 0:
        errors.add(
            rule=CHECK_FOOTNOTE_SYNTAX, file=file_name,
            message=f"{footnote_depth} unclosed footnote(s) at end of file.",
        )


# ---------------------------------------------------------------------------
# USFM formatting validation
# ---------------------------------------------------------------------------

# Tags that are allowed immediately before \v on the same line.
_VERSE_PREFIX_RE = re.compile(
    r"^\\(q[0-9]?|qm[0-9]?|qr|qc|qa|qd|li[0-9]?|pi[0-9]?|ph[0-9]?|p|m|mi|nb|pc|cls)$"
)


def validate_usfm_formatting(
    lines: list[str],
    file_name: str,
    errors: ErrorCollector,
) -> None:
    """Check formatting / readability rules for USFM markers."""
    # Skip header lines (everything before the first blank line).
    in_header = True
    prev_line_blank = False
    prev_non_blank = ""
    prev_non_blank_line_num = 0
    current_chapter = 0
    current_verse = 0

    for line_number, raw_line in enumerate(lines, 1):
        stripped = raw_line.strip()

        # Stay in header until the first blank line after it.
        if in_header:
            if not stripped:
                in_header = False
                prev_line_blank = True
            continue

        # --- blank line handling ---
        if not stripped:
            # \c, \b, \ts\*, \p must NOT have a blank line after them
            if re.match(r"\\c\s+\d+\s*$", prev_non_blank):
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number,
                    ref=str(current_chapter) if current_chapter else None,
                    message=(
                        f"Blank line after \\c marker (line {prev_non_blank_line_num}); "
                        "\\c should not be followed by a blank line."
                    ),
                )
            elif prev_non_blank == "\\b":
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number,
                    ref=str(current_chapter) if current_chapter else None,
                    message=(
                        f"Blank line after \\b marker (line {prev_non_blank_line_num}); "
                        "\\b should not be followed by a blank line."
                    ),
                )
            elif prev_non_blank == "\\ts\\*":
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number,
                    ref=str(current_chapter) if current_chapter else None,
                    message=(
                        f"Blank line after \\ts\\* marker (line {prev_non_blank_line_num}); "
                        "\\ts\\* should not be followed by a blank line."
                    ),
                )
            elif prev_non_blank == "\\p":
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number,
                    ref=str(current_chapter) if current_chapter else None,
                    message=(
                        f"Blank line after \\p marker (line {prev_non_blank_line_num}); "
                        "\\p should not be followed by a blank line."
                    ),
                )
            if not prev_line_blank:
                prev_line_blank = True
            continue

        # --- track chapter / verse for ref ---
        c_search = re.search(r"\\c\s+(\d+)", stripped)
        if c_search:
            current_chapter = int(c_search.group(1))
            current_verse = 0
        v_search = re.search(r"\\v\s+(\d+)", stripped)
        if v_search:
            current_verse = int(v_search.group(1))

        ref = None
        if current_chapter:
            ref = (
                f"{current_chapter}:{current_verse}"
                if current_verse
                else str(current_chapter)
            )

        # === Line-isolation rules ===

        # \c <integer> must be on its own line
        if c_search and not re.match(r"\\c\s+\d+\s*$", stripped):
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message="\\c chapter marker must be on its own line.",
            )

        # \p must be on its own line  (avoid matching \pi, \ph, etc.)
        if re.search(r"\\p(?!\w)", stripped) and stripped != "\\p":
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message="\\p paragraph marker must be on its own line.",
            )

        # \ts\* must be on its own line
        if "\\ts\\*" in stripped and stripped != "\\ts\\*":
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message="\\ts\\* must be on its own line.",
            )

        # \b must be on its own line
        if re.search(r"\\b(?!\w)", stripped) and stripped != "\\b":
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message="\\b must be on its own line.",
            )

        # \b must come before \ts\*, not after
        if stripped == "\\b" and prev_non_blank == "\\ts\\*":
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message=(
                    "\\b appears after \\ts\\*; \\b should come before \\ts\\*, not after."
                ),
            )

        # === Verse formatting rules ===

        v_all = re.findall(r"\\v\s+\d+", stripped)

        # Only one \v per line
        if len(v_all) > 1:
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message=(
                    f"Multiple \\v markers on the same line ({len(v_all)} found); "
                    "each verse should start on its own line."
                ),
            )

        # Before \v only a paragraph/poetry marker is allowed, no content
        if len(v_all) >= 1:
            first_v = re.search(r"\\v\s+\d+", stripped)
            before_v = stripped[: first_v.start()].strip()
            if before_v and not _VERSE_PREFIX_RE.match(before_v):
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number, ref=ref,
                    message=(
                        f"Content before \\v marker: '{before_v[:50]}'. "
                        "Only a paragraph/poetry marker (e.g., \\q1) should precede \\v."
                    ),
                )

        # === Blank-line rules ===

        # \b must always have a blank line before it
        if stripped == "\\b" and not prev_line_blank:
            errors.add(
                rule=CHECK_USFM_FORMATTING,
                file=file_name, line=line_number, ref=ref,
                message="\\b must have a blank line before it.",
            )

        # \ts\* must have a blank line before it unless preceded by \b or \ts\*
        if stripped == "\\ts\\*" and not prev_line_blank:
            prev_is_b = prev_non_blank == "\\b"
            prev_is_ts = prev_non_blank == "\\ts\\*"
            if not prev_is_b and not prev_is_ts:
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number, ref=ref,
                    message=(
                        "\\ts\\* must have a blank line before it "
                        "(unless preceded by \\b or \\ts\\*)."
                    ),
                )

        # \p must have a blank line before it unless preceded by \ts\*, \c, or \b
        if stripped == "\\p" and not prev_line_blank:
            prev_is_ts = prev_non_blank == "\\ts\\*"
            prev_is_c = bool(re.match(r"\\c\s+\d+\s*$", prev_non_blank))
            prev_is_b = prev_non_blank == "\\b"
            if not prev_is_ts and not prev_is_c and not prev_is_b:
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number, ref=ref,
                    message=(
                        "\\p must have a blank line before it "
                        "(unless preceded by \\ts\\*, \\c, or \\b)."
                    ),
                )

        # \c must have a blank line before it unless preceded by \p, \ts\*, or \b
        if re.match(r"\\c\s+\d+\s*$", stripped) and not prev_line_blank:
            prev_is_ts = prev_non_blank == "\\ts\\*"
            prev_is_p = prev_non_blank == "\\p"
            prev_is_b = prev_non_blank == "\\b"
            if not prev_is_ts and not prev_is_p and not prev_is_b:
                errors.add(
                    rule=CHECK_USFM_FORMATTING,
                    file=file_name, line=line_number, ref=ref,
                    message=(
                        "\\c must have a blank line before it "
                        "(unless preceded by \\p, \\ts\\*, or \\b)."
                    ),
                )

        prev_line_blank = False
        prev_non_blank = stripped
        prev_non_blank_line_num = line_number


# ---------------------------------------------------------------------------
# Per-file orchestrator
# ---------------------------------------------------------------------------

def validate_usfm_file(
    usfm_path: Path,
    project: dict[str, Any],
    repo_name: str,
    errors: ErrorCollector,
) -> None:
    file_name = usfm_path.name
    identifier = project.get("identifier", "").lower()

    # Skip front/back matter for content checks
    if identifier in ("frt", "bak"):
        return

    try:
        content = usfm_path.read_text(encoding="utf-8")
    except UnicodeDecodeError as exc:
        errors.add(
            rule=CHECK_USFM_HEADER, file=file_name,
            message=f"File is not valid UTF-8: {exc}",
        )
        return

    content = content.replace("\r\n", "\n").replace("\r", "\n")
    lines = content.split("\n")

    if not lines or (len(lines) == 1 and lines[0] == ""):
        errors.add(rule=CHECK_USFM_HEADER, file=file_name, line=1, message="USFM file is empty.")
        return

    if errors.is_enabled(CHECK_USFM_HEADER):
        validate_usfm_header(lines, file_name, project, repo_name, errors)

    need_content = any(
        errors.is_enabled(c) for c in [
            CHECK_CHAPTER_ORDER_AND_COUNT,
            CHECK_VERSE_ORDER_AND_COVERAGE,
            CHECK_FOOTNOTE_SYNTAX,
            CHECK_CONSECUTIVE_PARAGRAPH_MARKERS,
        ]
    )
    if need_content:
        validate_usfm_content(lines, file_name, identifier, errors)

    if errors.is_enabled(CHECK_USFM_FORMATTING):
        validate_usfm_formatting(lines, file_name, errors)


# ---------------------------------------------------------------------------
# Output / results
# ---------------------------------------------------------------------------

def _escape_annotation_value(value: str) -> str:
    return value.replace("%", "%25").replace("\r", "%0D").replace("\n", "%0A")


def _should_emit_annotations() -> bool:
    value = os.environ.get("USFM_EMIT_ANNOTATIONS", "").strip().lower()
    return value in {"1", "true", "yes", "on"}


def _check_number(check_label: str) -> int:
    prefix = check_label.split(".", 1)[0].strip()
    try:
        return int(prefix)
    except ValueError:
        return 9999


def resolve_active_checks(check_numbers: list[int] | None) -> list[str]:
    if not check_numbers:
        return CHECK_DISPLAY_ORDER.copy()
    unique_numbers = sorted(set(check_numbers))
    invalid = [num for num in unique_numbers if num not in CHECK_NUMBER_TO_NAME]
    if invalid:
        valid = ", ".join(str(n) for n in sorted(CHECK_NUMBER_TO_NAME.keys()))
        raise ValueError(
            f"Unknown check number(s): {', '.join(str(n) for n in invalid)}. "
            f"Valid check numbers are: {valid}."
        )
    return [CHECK_NUMBER_TO_NAME[num] for num in unique_numbers]


def _error_sort_key(err: ValidationError) -> tuple[int, str, int, str]:
    return (_check_number(err.rule), err.file or "", err.line or 0, err.message)


def _group_errors_by_check(
    errors: list[ValidationError], active_checks: list[str]
) -> dict[str, list[ValidationError]]:
    grouped: dict[str, list[ValidationError]] = {c: [] for c in active_checks}
    for err in errors:
        if err.rule not in grouped:
            grouped[err.rule] = []
        grouped[err.rule].append(err)
    return grouped


def _json_output_by_check(
    grouped_errors: dict[str, list[ValidationError]], active_checks: list[str]
) -> dict[str, Any]:
    output: dict[str, Any] = {}
    for check_name in active_checks:
        num = str(_check_number(check_name))
        output[num] = {
            "name": check_name,
            "errors": [
                {
                    "file": err.file,
                    "line": err.line,
                    "ref": err.ref,
                    "message": err.message,
                }
                for err in grouped_errors.get(check_name, [])
            ],
        }
    return output


def write_step_summary(
    *, success: bool, errors: list[ValidationError], truncated: bool = False
) -> None:
    summary_path = os.environ.get("GITHUB_STEP_SUMMARY")
    if not summary_path:
        return
    lines: list[str] = []
    if success:
        lines.append("## USFM Validation")
        lines.append("")
        lines.append("All checks passed.")
    else:
        lines.append("## USFM Validation")
        lines.append("")
        lines.append(f"Found **{len(errors)}** error(s).")
        lines.append("")
        lines.append("| Check | Location | Message |")
        lines.append("|---|---|---|")
        for err in errors[:200]:
            location = err.file or "(repo)"
            if err.file and err.line is not None:
                location = f"{err.file}:{err.line}"
            message = err.message.replace("|", "\\|")
            lines.append(f"| {err.rule} | {location} | {message} |")
        if len(errors) > 200:
            lines.append("")
            lines.append(f"_Summary limited to first 200 errors (total: {len(errors)})._")
        if truncated:
            lines.append("")
            lines.append("_Validation output was truncated by --max-errors._")
    with open(summary_path, "a", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def emit_results(
    errors: ErrorCollector, active_checks: list[str], output_json: bool = False
) -> int:
    ordered_errors = sorted(errors.errors, key=_error_sort_key)
    grouped_errors = _group_errors_by_check(ordered_errors, active_checks)

    if output_json:
        print(json.dumps(_json_output_by_check(grouped_errors, active_checks), ensure_ascii=False))
        if errors.has_errors():
            write_step_summary(success=False, errors=ordered_errors, truncated=errors.truncated)
            return 1
        write_step_summary(success=True, errors=[])
        return 0

    if errors.has_errors():
        print(f"Validation failed with {len(ordered_errors)} error(s).")
    else:
        print("Validation passed: manifest.yaml and USFM checks succeeded.")

    emit_annotations = _should_emit_annotations()
    for check_name in active_checks:
        print(check_name)
        print("-" * len(check_name))
        section_errors = grouped_errors.get(check_name, [])
        if not section_errors:
            print("No validation errors found.")
            print("")
            continue
        for err in section_errors:
            line = err.display()
            print(f"- {line}")
            if emit_annotations:
                annotation = _escape_annotation_value(line)
                if err.file:
                    if err.line is not None:
                        print(f"::error file={err.file},line={err.line}::{annotation}")
                    else:
                        print(f"::error file={err.file}::{annotation}")
                else:
                    print(f"::error::{annotation}")
        print("")

    if errors.truncated:
        truncation = (
            f"Error output truncated after {errors.max_errors} entries. "
            "Re-run with --max-errors for a larger cap."
        )
        print(truncation)
        if emit_annotations:
            print(f"::error::{_escape_annotation_value(truncation)}")

    if errors.has_errors():
        write_step_summary(success=False, errors=ordered_errors, truncated=errors.truncated)
        return 1
    write_step_summary(success=True, errors=[])
    return 0


# ---------------------------------------------------------------------------
# CLI entry point
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(description="Validate USFM files and manifest.yaml.")
    parser.add_argument("--manifest", default="manifest.yaml", help="Path to manifest.yaml")
    parser.add_argument(
        "--base-branch",
        default=os.environ.get("GITHUB_BASE_REF", ""),
        help="PR base branch name (e.g. master).",
    )
    parser.add_argument(
        "--max-errors", type=int, default=1000,
        help="Maximum number of errors before truncating.",
    )
    parser.add_argument(
        "--check", dest="checks", action="append", type=int,
        help="Check number to run (repeatable). Example: --check 1 --check 4",
    )
    parser.add_argument(
        "--book", dest="books", action="append",
        help="Book identifier to validate (repeatable, case-insensitive).",
    )
    parser.add_argument(
        "--json", dest="json_output", action="store_true",
        help="Output validation results as JSON.",
    )
    args = parser.parse_args()

    try:
        active_checks = resolve_active_checks(args.checks)
    except ValueError as exc:
        print(f"Argument error: {exc}")
        return 2

    enabled_checks = set(active_checks)
    selected_books = sorted(
        {b.strip().lower() for b in (args.books or []) if isinstance(b, str) and b.strip()}
    )
    errors = ErrorCollector(max_errors=max(1, args.max_errors), enabled_checks=enabled_checks)
    manifest_path = Path(args.manifest)

    # Always load manifest (needed for header checks and project paths)
    manifest_data = load_manifest(manifest_path, errors)
    if manifest_data is None:
        return emit_results(errors, active_checks, output_json=args.json_output)

    repo_name = get_repo_name(manifest_data)
    projects = parse_manifest_projects(manifest_data, errors)

    # Check files exist
    if errors.is_enabled(CHECK_PROJECT_USFM_FILES_EXIST):
        missing = [p["_normalized_path"] for p in projects if not Path(p["_normalized_path"]).is_file()]
        if missing:
            errors.add(
                rule=CHECK_PROJECT_USFM_FILES_EXIST,
                message="Files listed in manifest but missing in repo: " + ", ".join(sorted(missing)),
            )

    # Determine which projects to validate content for
    projects_to_validate = projects
    if selected_books:
        id_to_proj: dict[str, dict[str, Any]] = {}
        for proj in projects:
            pid = proj.get("identifier", "")
            if isinstance(pid, str):
                id_to_proj[pid.lower()] = proj

        unknown = sorted(b for b in selected_books if b not in id_to_proj)
        if unknown:
            errors.add(
                rule=CHECK_PROJECT_USFM_FILES_EXIST,
                message="--book value(s) not found in manifest: " + ", ".join(unknown),
            )
        projects_to_validate = [id_to_proj[b] for b in selected_books if b in id_to_proj]

    # Content checks
    need_content = any(
        errors.is_enabled(c) for c in [
            CHECK_USFM_HEADER,
            CHECK_CHAPTER_ORDER_AND_COUNT,
            CHECK_VERSE_ORDER_AND_COVERAGE,
            CHECK_FOOTNOTE_SYNTAX,
            CHECK_CONSECUTIVE_PARAGRAPH_MARKERS,
            CHECK_USFM_FORMATTING,
        ]
    )
    if need_content:
        for proj in projects_to_validate:
            path = proj.get("_normalized_path", "")
            if path and Path(path).is_file():
                validate_usfm_file(Path(path), proj, repo_name, errors)

    return emit_results(errors, active_checks, output_json=args.json_output)


if __name__ == "__main__":
    sys.exit(main())
