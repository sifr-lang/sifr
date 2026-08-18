#!/usr/bin/env python3
"""Assert semantic LSP hover and signature-help responses."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

from lsp_protocol import LspClient, LspProtocolError, file_uri
from lsp_protocol_smoke import initialize, open_document


MAIN = '''\
from helper import imported_helper
from sifr.random import randint

def generate_random() -> int | None:
    x: int = 1
    return x

def combine(left: int, right: int) -> int:
    return left + right

def main() -> int:
    x: int = 2
    y = generate_random()
    z: int = imported_helper(x)
    random_result = randint(
        0,
        100,
    )
    word: str = "generate_random"
    # generate_random in a comment must not produce semantic hover
    return combine(x, z)

def try_random() -> int | None:
    try:
        candidate: int = randint(1, 10)
        return candidate
    except ValueError:
        return None

def unicode_probe() -> None:
    print("😀", generate_random())
'''

HELPER = '''\
def imported_helper(value: int) -> int:
    return value + 1
'''

UPDATED = MAIN.replace("y = generate_random()", "y = combine(1, 2)")

PLACEHOLDER_FRAGMENTS = ["(Name)", "(Identifier)", "NonLogicalNewline", "(NonLogicalNewline)"]


def hover_value(client: LspClient, uri: str, line: int, character: int) -> str | None:
    hover = client.request(
        "textDocument/hover",
        {"textDocument": {"uri": uri}, "position": {"line": line, "character": character}},
    )
    if hover is None:
        return None
    contents = hover.get("contents") if isinstance(hover, dict) else None
    if not isinstance(contents, dict):
        raise LspProtocolError(f"hover returned invalid contents: {hover}")
    value = contents.get("value")
    if not isinstance(value, str):
        raise LspProtocolError(f"hover returned invalid markdown: {hover}")
    assert_no_placeholders(value, "hover")
    return value


def signature_help(client: LspClient, uri: str, line: int, character: int) -> dict[str, Any]:
    result = client.request(
        "textDocument/signatureHelp",
        {"textDocument": {"uri": uri}, "position": {"line": line, "character": character}},
    )
    if not isinstance(result, dict):
        raise LspProtocolError(f"signatureHelp returned invalid result: {result}")
    signatures = result.get("signatures")
    if not isinstance(signatures, list) or len(signatures) != 1:
        raise LspProtocolError(f"signatureHelp should return one signature: {result}")
    label = signatures[0].get("label")
    if not isinstance(label, str):
        raise LspProtocolError(f"signatureHelp missing label: {result}")
    assert_no_placeholders(label, "signatureHelp")
    return result


def assert_no_placeholders(value: str, label: str) -> None:
    leaked = [fragment for fragment in PLACEHOLDER_FRAGMENTS if fragment in value]
    if leaked:
        raise LspProtocolError(f"{label} leaked lexer placeholder {leaked}: {value!r}")


def assert_contains(value: str | None, required: list[str], label: str) -> None:
    if value is None:
        raise LspProtocolError(f"{label} returned null")
    missing = [fragment for fragment in required if fragment not in value]
    if missing:
        raise LspProtocolError(f"{label} missing {missing}: {value!r}")


def assert_null_hover(client: LspClient, uri: str, line: int, character: int, label: str) -> None:
    value = hover_value(client, uri, line, character)
    if value is not None:
        raise LspProtocolError(f"{label} should not return semantic hover: {value!r}")


def assert_signature(
    help_result: dict[str, Any],
    required_label: list[str],
    expected_parameters: list[str],
    expected_active_parameter: int,
    label: str,
) -> None:
    signature = help_result["signatures"][0]
    text = signature["label"]
    missing = [fragment for fragment in required_label if fragment not in text]
    if missing:
        raise LspProtocolError(f"{label} missing label fragments {missing}: {help_result}")
    parameters = signature.get("parameters")
    if not isinstance(parameters, list):
        raise LspProtocolError(f"{label} missing parameter list: {help_result}")
    actual = [item.get("label") for item in parameters if isinstance(item, dict)]
    if actual != expected_parameters:
        raise LspProtocolError(f"{label} parameters {actual!r} != {expected_parameters!r}")
    if help_result.get("activeParameter") != expected_active_parameter:
        raise LspProtocolError(
            f"{label} activeParameter {help_result.get('activeParameter')!r} != {expected_active_parameter}"
        )


def run_semantic_corpus() -> None:
    with tempfile.TemporaryDirectory(prefix="sifr-lsp-semantic-") as raw:
        root = Path(raw)
        (root / "sifr.toml").write_text(
            '[package]\nname = "lsp_semantic"\n[source]\nroot = "."\n',
            encoding="utf-8",
        )
        main = root / "main.sifr"
        helper = root / "helper.sifr"
        main.write_text(MAIN, encoding="utf-8")
        helper.write_text(HELPER, encoding="utf-8")
        uri = file_uri(main)
        client = LspClient()
        try:
            initialize(client, root)
            open_document(client, main, MAIN)

            assert_contains(
                hover_value(client, uri, 3, 6),
                ["generate_random", "->", "int", "None"],
                "function definition hover",
            )
            assert_contains(
                hover_value(client, uri, 12, 10),
                ["generate_random", "->", "int", "None"],
                "function call hover",
            )
            assert_contains(hover_value(client, uri, 4, 4), ["x", "int"], "annotated local hover")
            assert_contains(hover_value(client, uri, 11, 4), ["x", "int"], "shadowed local hover")
            assert_contains(hover_value(client, uri, 12, 4), ["y", "int", "None"], "inferred local hover")
            assert_contains(hover_value(client, uri, 7, 12), ["left", "int"], "parameter hover")
            assert_contains(
                hover_value(client, uri, 13, 13),
                ["imported_helper", "value", "int", "->", "int"],
                "project import hover",
            )
            assert_contains(
                hover_value(client, uri, 14, 20),
                ["randint", "minimum", "maximum", "Result[int, ValueError]"],
                "stdlib import hover",
            )
            assert_contains(
                hover_value(client, uri, 24, 25),
                ["randint", "minimum", "maximum", "Result[int, ValueError]"],
                "try stdlib call hover",
            )
            assert_contains(hover_value(client, uri, 24, 8), ["candidate", "int"], "try local hover")
            assert_contains(hover_value(client, uri, 25, 15), ["candidate", "int"], "try return hover")
            assert_contains(
                hover_value(client, uri, 30, 16),
                ["generate_random", "->", "int", "None"],
                "utf-16 position hover after non-ascii text",
            )

            assert_signature(
                signature_help(client, uri, 12, 24),
                ["generate_random", "->", "int", "None"],
                [],
                0,
                "zero-arg signature help",
            )
            assert_signature(
                signature_help(client, uri, 14, 28),
                ["randint", "Result[int, ValueError]"],
                ["minimum: int", "maximum: int"],
                0,
                "first randint argument signature help",
            )
            assert_signature(
                signature_help(client, uri, 16, 4),
                ["randint", "Result[int, ValueError]"],
                ["minimum: int", "maximum: int"],
                1,
                "second multiline randint argument signature help",
            )
            assert_signature(
                signature_help(client, uri, 24, 37),
                ["randint", "Result[int, ValueError]"],
                ["minimum: int", "maximum: int"],
                1,
                "try randint argument signature help",
            )
            assert_signature(
                signature_help(client, uri, 20, 22),
                ["combine", "->", "int"],
                ["left: int", "right: int"],
                1,
                "active parameter after comma signature help",
            )

            assert_null_hover(client, uri, 10, 0, "keyword hover")
            assert_null_hover(client, uri, 18, 17, "string literal hover")
            assert_null_hover(client, uri, 19, 6, "comment hover")

            client.notify(
                "textDocument/didChange",
                {
                    "textDocument": {"uri": uri, "version": 2},
                    "contentChanges": [{"text": UPDATED}],
                },
            )
            client.wait_for_notification("textDocument/publishDiagnostics")
            assert_contains(
                hover_value(client, uri, 12, 8),
                ["combine", "left", "right", "->", "int"],
                "updated document hover",
            )
            assert_signature(
                signature_help(client, uri, 12, 20),
                ["combine", "->", "int"],
                ["left: int", "right: int"],
                1,
                "updated document signature help",
            )

            client.request("shutdown", {})
            client.notify("exit", {})
        finally:
            client.close()


def run_self_test() -> None:
    try:
        assert_no_placeholders("x (Name)", "negative semantic corpus")
    except LspProtocolError:
        print("LSP semantic editor corpus self-test: PASS")
        return
    raise SystemExit("LSP semantic editor corpus self-test failed: placeholder passed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0
    try:
        run_semantic_corpus()
    except (LspProtocolError, OSError, json.JSONDecodeError) as error:
        print(f"LSP semantic editor corpus: FAIL: {error}", file=sys.stderr)
        return 1
    print("LSP semantic editor corpus: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
