"""Shared Rust source masking for repository policy checks."""

from __future__ import annotations

from pathlib import Path
import re


CFG_TEST_MODULE = re.compile(
    r"(?m)^\s*#\[cfg\(test\)\]\s*\n\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+\w+\s*\{"
)
RAW_STRING_START = re.compile(r'(?:br|cr|r)(?P<hashes>#{0,255})"')


def is_test_path(path: Path) -> bool:
    """Return whether a Rust path is owned only by tests."""
    return any("test" in part for part in path.parts)


def _blank(output: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if output[index] != "\n":
            output[index] = " "


def _char_literal_end(text: str, start: int) -> int | None:
    quote = start + 1 if text.startswith("b'", start) else start
    if quote >= len(text) or text[quote] != "'" or quote + 2 >= len(text):
        return None
    content = quote + 1
    if text[content] != "\\":
        return content + 2 if text[content + 1] == "'" else None
    escape = content + 1
    if escape >= len(text):
        return None
    if text[escape] == "u" and escape + 1 < len(text) and text[escape + 1] == "{":
        brace = text.find("}", escape + 2)
        end = brace + 2
        return end if brace >= 0 and end <= len(text) and text[brace + 1] == "'" else None
    if text[escape] == "x":
        closing = escape + 3
    else:
        closing = escape + 1
    return closing + 1 if closing < len(text) and text[closing] == "'" else None


def mask_rust_non_code(text: str) -> str:
    """Mask comments and literals while preserving byte offsets and newlines."""
    output = list(text)
    index = 0
    while index < len(text):
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = len(text) if end < 0 else end
            _blank(output, index, end)
            index = end
            continue
        if text.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            _blank(output, index, end)
            index = end
            continue
        char_end = _char_literal_end(text, index)
        if char_end is not None:
            _blank(output, index, char_end)
            index = char_end
            continue
        raw = RAW_STRING_START.match(text, index)
        if raw:
            terminator = '"' + raw.group("hashes")
            end = text.find(terminator, raw.end())
            end = len(text) if end < 0 else end + len(terminator)
            _blank(output, index, end)
            index = end
            continue
        quote_index = index
        if text[index] in {"b", "c"} and index + 1 < len(text) and text[index + 1] == '"':
            quote_index += 1
        if text[quote_index] == '"':
            end = quote_index + 1
            while end < len(text):
                if text[end] == "\\":
                    end += 2
                elif text[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            _blank(output, index, end)
            index = end
            continue
        index += 1
    return "".join(output)


def mask_rust_literals(text: str) -> str:
    """Mask Rust literals while retaining comments and code.

    Character literals are recognized only when a closing quote proves the
    token shape. A lifetime or loop label therefore remains visible as code.
    """
    output = list(text)
    index = 0
    while index < len(text):
        char_end = _char_literal_end(text, index)
        if char_end is not None:
            _blank(output, index, char_end)
            index = char_end
            continue
        raw = RAW_STRING_START.match(text, index)
        if raw:
            terminator = '"' + raw.group("hashes")
            end = text.find(terminator, raw.end())
            end = len(text) if end < 0 else end + len(terminator)
            _blank(output, index, end)
            index = end
            continue
        quote_index = index
        if text[index] in {"b", "c"} and index + 1 < len(text) and text[index + 1] == '"':
            quote_index += 1
        if text[quote_index] == '"':
            end = quote_index + 1
            while end < len(text):
                if text[end] == "\\":
                    end += 2
                elif text[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            _blank(output, index, end)
            index = end
            continue
        index += 1
    return "".join(output)


def mask_rust_comments(text: str) -> str:
    """Mask line and nested block comments while preserving string literals."""
    output = list(text)
    index = 0
    while index < len(text):
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = len(text) if end < 0 else end
            _blank(output, index, end)
            index = end
            continue
        if text.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < len(text) and depth:
                if text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            _blank(output, index, end)
            index = end
            continue
        char_end = _char_literal_end(text, index)
        if char_end is not None:
            index = char_end
            continue
        raw = RAW_STRING_START.match(text, index)
        if raw:
            terminator = '"' + raw.group("hashes")
            end = text.find(terminator, raw.end())
            index = len(text) if end < 0 else end + len(terminator)
            continue
        quote_index = index
        if text[index] in {"b", "c"} and index + 1 < len(text) and text[index + 1] == '"':
            quote_index += 1
        if text[quote_index] == '"':
            end = quote_index + 1
            while end < len(text):
                if text[end] == "\\":
                    end += 2
                elif text[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            index = end
            continue
        index += 1
    return "".join(output)


def strip_cfg_test_modules(text: str) -> str:
    """Blank inline ``#[cfg(test)]`` modules without hiding later production code."""
    code = mask_rust_non_code(text)
    output = list(text)
    for match in CFG_TEST_MODULE.finditer(code):
        opening = code.find("{", match.start(), match.end())
        if opening < 0:
            continue
        depth = 0
        end = opening
        while end < len(code):
            if code[end] == "{":
                depth += 1
            elif code[end] == "}":
                depth -= 1
                if depth == 0:
                    end += 1
                    break
            end += 1
        _blank(output, match.start(), end)
    return "".join(output)
