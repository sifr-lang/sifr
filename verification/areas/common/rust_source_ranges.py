"""Locate Rust source ranges that architecture scans may classify separately."""

from __future__ import annotations

import re


INLINE_TEST_MODULE_PATTERN = re.compile(
    r"(?m)^#\[cfg\(test\)\]\s*(?:#\[path\s*=\s*\"[^\"]+\"\]\s*)?mod\s+\w+\s*\{"
)


def inline_test_module_ranges(text: str) -> list[tuple[int, int]]:
    ranges: list[tuple[int, int]] = []
    for match in INLINE_TEST_MODULE_PATTERN.finditer(text):
        opening = text.find("{", match.start(), match.end())
        closing = matching_rust_brace(text, opening)
        if closing is not None:
            ranges.append((match.start(), closing))
    return ranges


def matching_rust_brace(text: str, opening: int) -> int | None:
    if opening < 0:
        return None
    depth = 0
    index = opening
    mode = "code"
    block_depth = 0
    raw_hashes = 0
    while index < len(text):
        pair = text[index : index + 2]
        char = text[index]
        if mode == "line-comment":
            if char == "\n":
                mode = "code"
        elif mode == "block-comment":
            if pair == "/*":
                block_depth += 1
                index += 1
            elif pair == "*/":
                block_depth -= 1
                index += 1
                if block_depth == 0:
                    mode = "code"
        elif mode == "string":
            if char == "\\":
                index += 1
            elif char == '"':
                mode = "code"
        elif mode == "char":
            if char == "\\":
                index += 1
            elif char == "'":
                mode = "code"
        elif mode == "raw-string":
            terminator = '"' + ("#" * raw_hashes)
            if text.startswith(terminator, index):
                index += len(terminator) - 1
                mode = "code"
        else:
            raw_match = re.match(r'r(#{0,16})"', text[index:])
            if pair == "//":
                mode = "line-comment"
                index += 1
            elif pair == "/*":
                mode = "block-comment"
                block_depth = 1
                index += 1
            elif raw_match is not None:
                raw_hashes = len(raw_match.group(1))
                index += len(raw_match.group(0)) - 1
                mode = "raw-string"
            elif char == '"':
                mode = "string"
            elif char == "'" and "'" in text[index + 1 : index + 5]:
                mode = "char"
            elif char == "{":
                depth += 1
            elif char == "}":
                depth -= 1
                if depth == 0:
                    return index
        index += 1
    return None
