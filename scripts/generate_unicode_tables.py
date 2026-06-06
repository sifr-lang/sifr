#!/usr/bin/env python3
"""Generate Unicode 17.0.0 property tables for sifr_runtime."""

from __future__ import annotations

import argparse
import urllib.request
from dataclasses import dataclass
from pathlib import Path


UNICODE_VERSION = "17.0.0"
BASE_URL = f"https://www.unicode.org/Public/{UNICODE_VERSION}/ucd"
UNICODE_DATA_URL = f"{BASE_URL}/UnicodeData.txt"
EAST_ASIAN_WIDTH_URL = f"{BASE_URL}/EastAsianWidth.txt"
CASE_FOLDING_URL = f"{BASE_URL}/CaseFolding.txt"
OUT_PATH = Path("crates/sifr_runtime/src/unicode_data/generated.rs")


@dataclass(frozen=True)
class PropertyRecord:
    start: int
    end: int
    category: str
    bidi: str
    combining: int
    mirrored: bool
    decomposition: str
    decimal: int | None
    digit: int | None
    numeric: str


@dataclass(frozen=True)
class WidthRange:
    start: int
    end: int
    width: str


@dataclass(frozen=True)
class CaseFold:
    codepoint: int
    mapping: str


def fetch_text(url: str) -> str:
    with urllib.request.urlopen(url, timeout=60) as response:
        return response.read().decode("utf-8")


def rust_str(value: str) -> str:
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def rust_char_string(codepoints: list[int]) -> str:
    return "".join(f"\\u{{{codepoint:X}}}" for codepoint in codepoints)


def rust_char_string_literal(value: str) -> str:
    return f'"{value}"'


def parse_hex_range(token: str) -> tuple[int, int]:
    if ".." in token:
        start, end = token.split("..", 1)
        return int(start, 16), int(end, 16)
    value = int(token, 16)
    return value, value


def parse_int(value: str) -> int | None:
    if value == "":
        return None
    return int(value)


def record_key(record: PropertyRecord) -> tuple[object, ...]:
    return (
        record.category,
        record.bidi,
        record.combining,
        record.mirrored,
        record.decomposition,
        record.decimal,
        record.digit,
        record.numeric,
    )


def parse_unicode_data(text: str) -> list[PropertyRecord]:
    records: list[PropertyRecord] = []
    pending_first: tuple[int, list[str]] | None = None

    for raw_line in text.splitlines():
        if not raw_line or raw_line.startswith("#"):
            continue
        fields = raw_line.split(";")
        codepoint = int(fields[0], 16)
        name = fields[1]

        if name.endswith(", First>"):
            pending_first = (codepoint, fields)
            continue
        if name.endswith(", Last>"):
            if pending_first is None:
                raise ValueError(f"range end without start at U+{codepoint:04X}")
            start, first_fields = pending_first
            records.append(record_from_fields(start, codepoint, first_fields))
            pending_first = None
            continue

        records.append(record_from_fields(codepoint, codepoint, fields))

    if pending_first is not None:
        start, _fields = pending_first
        raise ValueError(f"range start without end at U+{start:04X}")

    merged: list[PropertyRecord] = []
    for record in records:
        if merged and merged[-1].end + 1 == record.start and record_key(merged[-1]) == record_key(record):
            previous = merged[-1]
            merged[-1] = PropertyRecord(
                previous.start,
                record.end,
                previous.category,
                previous.bidi,
                previous.combining,
                previous.mirrored,
                previous.decomposition,
                previous.decimal,
                previous.digit,
                previous.numeric,
            )
        else:
            merged.append(record)
    return merged


def record_from_fields(start: int, end: int, fields: list[str]) -> PropertyRecord:
    return PropertyRecord(
        start=start,
        end=end,
        category=fields[2],
        bidi=fields[4],
        combining=int(fields[3]),
        mirrored=fields[9] == "Y",
        decomposition=fields[5],
        decimal=parse_int(fields[6]),
        digit=parse_int(fields[7]),
        numeric=fields[8],
    )


def parse_east_asian_width(text: str) -> list[WidthRange]:
    ranges: list[WidthRange] = []
    for raw_line in text.splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        range_token, width = [part.strip() for part in line.split(";", 1)]
        start, end = parse_hex_range(range_token)
        ranges.append(WidthRange(start, end, width))
    return ranges


def parse_case_folding(text: str) -> list[CaseFold]:
    mappings: dict[int, str] = {}
    for raw_line in text.splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        codepoint_text, status, mapping_text, _name = [part.strip() for part in line.split(";", 3)]
        if status not in {"C", "F"}:
            continue
        codepoint = int(codepoint_text, 16)
        mapping = rust_char_string([int(part, 16) for part in mapping_text.split()])
        mappings[codepoint] = mapping
    return [CaseFold(codepoint, mappings[codepoint]) for codepoint in sorted(mappings)]


def write_generated(
    property_records: list[PropertyRecord],
    width_ranges: list[WidthRange],
    case_folds: list[CaseFold],
    out_path: Path,
) -> None:
    lines: list[str] = [
        "// @generated by scripts/generate_unicode_tables.py; DO NOT EDIT.",
        f"pub(crate) const UNICODE_DATA_VERSION: &str = {rust_str(UNICODE_VERSION)};",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct UnicodePropertyRecord {",
        "    pub start: u32,",
        "    pub end: u32,",
        "    pub category: &'static str,",
        "    pub bidi: &'static str,",
        "    pub combining: u8,",
        "    pub mirrored: bool,",
        "    pub decomposition: &'static str,",
        "    pub decimal: i16,",
        "    pub digit: i16,",
        "    pub numeric: &'static str,",
        "}",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct UnicodeWidthRange {",
        "    pub start: u32,",
        "    pub end: u32,",
        "    pub width: &'static str,",
        "}",
        "",
        "#[derive(Clone, Copy, Debug)]",
        "pub(crate) struct UnicodeCaseFold {",
        "    pub codepoint: u32,",
        "    pub mapping: &'static str,",
        "}",
        "",
        "#[rustfmt::skip]",
        "pub(crate) const PROPERTY_RANGES: &[UnicodePropertyRecord] = &[",
    ]
    for record in property_records:
        decimal = -1 if record.decimal is None else record.decimal
        digit = -1 if record.digit is None else record.digit
        lines.append(
            "    UnicodePropertyRecord { "
            f"start: 0x{record.start:X}, end: 0x{record.end:X}, "
            f"category: {rust_str(record.category)}, bidi: {rust_str(record.bidi)}, "
            f"combining: {record.combining}, mirrored: {str(record.mirrored).lower()}, "
            f"decomposition: {rust_str(record.decomposition)}, "
            f"decimal: {decimal}, digit: {digit}, numeric: {rust_str(record.numeric)} "
            "},"
        )
    lines.extend([
        "    ];",
        "",
        "#[rustfmt::skip]",
        "pub(crate) const EAST_ASIAN_WIDTH_RANGES: &[UnicodeWidthRange] = &[",
    ])
    for width_range in width_ranges:
        lines.append(
            "    UnicodeWidthRange { "
            f"start: 0x{width_range.start:X}, end: 0x{width_range.end:X}, "
            f"width: {rust_str(width_range.width)} "
            "},"
        )
    lines.extend([
        "    ];",
        "",
        "#[rustfmt::skip]",
        "pub(crate) const CASE_FOLDING: &[UnicodeCaseFold] = &[",
    ])
    for case_fold in case_folds:
        lines.append(
            "    UnicodeCaseFold { "
            f"codepoint: 0x{case_fold.codepoint:X}, mapping: {rust_char_string_literal(case_fold.mapping)} "
            "},"
        )
    lines.append("    ];")
    lines.append("")

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines), encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=OUT_PATH)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    property_records = parse_unicode_data(fetch_text(UNICODE_DATA_URL))
    width_ranges = parse_east_asian_width(fetch_text(EAST_ASIAN_WIDTH_URL))
    case_folds = parse_case_folding(fetch_text(CASE_FOLDING_URL))
    write_generated(property_records, width_ranges, case_folds, args.output)


if __name__ == "__main__":
    main()
