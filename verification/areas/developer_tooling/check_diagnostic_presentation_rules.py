#!/usr/bin/env python3
"""Validate the diagnostic presentation rules."""

from __future__ import annotations

import argparse
import json
import shutil
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]

REQUIRED_FIELDS = [
    "code",
    "severity",
    "message",
    "message_template",
    "args",
    "url",
    "spans",
    "children",
    "help",
    "suggestions",
]

DIAGNOSTIC_FIXTURES = [
    "decimal_invalid_literal",
    "multiline_span_rendering",
    "presentation_rules_cases",
]

CLI_FORMAT_CASES = {
    "check": "crates/sifr/src/check_and_package_commands.rs",
    "build": "crates/sifr/src/diagnostic_rendering_and_run.rs",
    "run": "crates/sifr/src/diagnostic_rendering_and_run.rs",
    "emit": "crates/sifr/src/check_and_package_commands.rs",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def read_text(root: Path, relative: str) -> str:
    path = root / relative
    if not path.is_file():
        raise AssertionError(f"required file missing: {relative}")
    return path.read_text(encoding="utf-8")


def read_json(root: Path, relative: str) -> Any:
    return json.loads(read_text(root, relative))


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def check_profile_runner_wiring(root: Path) -> None:
    text = read_text(root, "verification/runner/sifr_verify/profile_runner.py")
    require('"developer_tooling"' in text, "profile_runner.py missing developer_tooling area route")
    require('"diagnostic-rules"' in text, "profile_runner.py missing diagnostic rules suite route")


def check_schema_lock(root: Path) -> None:
    lock = read_json(root, "verification/areas/developer_tooling/diagnostic_presentation_schema_lock.json")
    locked = lock.get("rendered_diagnostic_required_fields")
    require(locked == REQUIRED_FIELDS, "schema lock does not enumerate RenderedDiagnostic fields")

    schema = read_json(root, "docs/schemas/diagnostics.schema.json")
    rendered = schema.get("$defs", {}).get("RenderedDiagnostic", {})
    required = rendered.get("required", [])
    properties = rendered.get("properties", {})
    for field in REQUIRED_FIELDS:
        require(field in required, f"diagnostic schema missing required field: {field}")
        require(field in properties, f"diagnostic schema missing property: {field}")


def baseline_path(root: Path, fixture: str, name: str) -> Path:
    return (
        root
        / "verification/areas/diagnostics/fixtures/diagnostics"
        / fixture
        / "baselines"
        / name
    )


def check_fixture_baselines(root: Path) -> None:
    base = root / "verification/areas/diagnostics/fixtures/diagnostics"
    for fixture in DIAGNOSTIC_FIXTURES:
        fixture_dir = base / fixture
        require(fixture_dir.is_dir(), f"required diagnostic fixture missing: {fixture}")
        require((fixture_dir / "main.sifr").is_file(), f"fixture missing main.sifr: {fixture}")
        for diagnostic_format in ("human", "compact", "json"):
            for stream in ("stdout", "stderr", "exit-code"):
                artifact = baseline_path(
                    root, fixture, f"check-{diagnostic_format}.{stream}.txt"
                )
                require(artifact.is_file(), f"required baseline missing: {artifact.relative_to(root)}")


def check_manifest_cases(root: Path) -> None:
    manifest = read_json(root, "verification/areas/diagnostics/manifest.json")
    baseline_suite = next(
        (
            suite
            for suite in manifest.get("suites", [])
            if isinstance(suite, dict) and suite.get("name") == "baselines"
        ),
        None,
    )
    require(baseline_suite is not None, "diagnostics area manifest missing baselines suite")
    cases = {case.get("id"): case for case in baseline_suite.get("cases", [])}
    for fixture in ("decimal_invalid_literal", "multiline_span_rendering"):
        case = cases.get(fixture)
        require(case is not None, f"diagnostics baselines suite missing case: {fixture}")
        require(
            case.get("diagnostic_formats") == ["human", "json", "compact"],
            f"diagnostics case has wrong format lock: {fixture}",
        )


def check_human_baselines(root: Path) -> None:
    decimal = baseline_path(root, "decimal_invalid_literal", "check-human.stderr.txt").read_text(
        encoding="utf-8"
    )
    require("error[SIFR-DECIMAL-0001]:" in decimal, "human baseline missing severity/code")
    require(":3:30" in decimal and "  --> " in decimal, "human baseline missing location")
    require('Decimal("12.34.56")' in decimal, "human baseline missing source snippet")
    require("^^^^^^^^^^" in decimal, "human baseline missing visual highlight marker")
    require(
        "https://sifr.sh/docs/errors/SIFR-DECIMAL-0001" in decimal,
        "human baseline missing docs URL",
    )

    multiline = baseline_path(
        root, "multiline_span_rendering", "check-human.stderr.txt"
    ).read_text(encoding="utf-8")
    require(":3:5" in multiline, "multiline human baseline missing primary location")
    require("   4 |" in multiline and "   6 |" in multiline, "multiline baseline missing span lines")
    require(multiline.count("^") >= 4, "multiline baseline missing per-line highlights")

    presentation = baseline_path(
        root, "presentation_rules_cases", "check-human.stderr.txt"
    ).read_text(encoding="utf-8")
    require("::: " in presentation, "human rules baseline missing related span")
    require("= related span" in presentation, "human rules baseline missing related label")
    require("= location: <unavailable>" in presentation, "human rules baseline missing spanless fallback")
    require("= note: child note rendered" in presentation, "human rules baseline missing child note")
    require("= help: child help rendered" in presentation, "human rules baseline missing help")
    require("= suggestion: replace value" in presentation, "human rules baseline missing suggestion")
    require("\r" not in presentation, "human rules baseline must not print CR characters")


def check_compact_baselines(root: Path) -> None:
    for fixture in DIAGNOSTIC_FIXTURES:
        compact = baseline_path(root, fixture, "check-compact.stderr.txt").read_text(
            encoding="utf-8"
        )
        lines = compact.splitlines()
        require(lines, f"compact baseline empty: {fixture}")
        require("error(s)" not in lines[0], f"compact summary still uses old count spelling: {fixture}")
        require("help item" not in lines[0], f"compact summary counts help items: {fixture}")
        require(lines[0].count(",") == 2, f"compact summary is not severity-only: {fixture}")
        for line in lines[1:]:
            fields = line.split(" ", 3)
            require(len(fields) == 4, f"compact line does not expose four stable fields: {line}")
            require(fields[0] in {"E", "W", "N"}, f"compact line has bad severity field: {line}")
            require(fields[1].startswith("SIFR-"), f"compact line has bad code field: {line}")
        require("url:" not in compact, f"compact baseline emits URL by default: {fixture}")
        require("  |" not in compact, f"compact baseline emits snippets: {fixture}")
        require("(x" not in compact, f"compact baseline still has grouped counts: {fixture}")


def check_json_baselines(root: Path) -> None:
    for fixture in ("decimal_invalid_literal", "multiline_span_rendering"):
        payload = read_json(
            root,
            f"verification/areas/diagnostics/fixtures/diagnostics/{fixture}/baselines/check-json.stderr.txt",
        )
        first = payload[0]
        for field in REQUIRED_FIELDS:
            require(field in first, f"json baseline missing RenderedDiagnostic field: {field}")
    multiline = read_json(
        root,
        "verification/areas/diagnostics/fixtures/diagnostics/multiline_span_rendering/baselines/check-json.stderr.txt",
    )
    require(
        len(multiline[0]["spans"][0]["lines"]) > 1,
        "multiline json baseline does not lock multiline span lines",
    )
    presentation = read_json(
        root,
        "verification/areas/diagnostics/fixtures/diagnostics/presentation_rules_cases/baselines/check-json.stderr.txt",
    )
    require(
        "\r" in presentation[0]["spans"][0]["lines"][0]["text"],
        "json rules baseline must preserve CR source text semantics",
    )
    require(presentation[0]["suggestions"], "json rules baseline missing suggestions")


def check_renderer_ownership(root: Path) -> None:
    presentation = read_text(root, "crates/sifr_diagnostics/src/render/presentation.rs")
    for symbol in (
        "render_human_diagnostics",
        "render_compact_diagnostics",
        "render_json_diagnostics",
        "highlight_marker",
        "terminal_line_text",
    ):
        require(symbol in presentation, f"sifr_diagnostics presentation missing {symbol}")
    require("CompactKey" not in presentation, "compact renderer still defines CompactKey grouping")
    require("(x{})" not in presentation, "compact renderer still renders grouped counts")

    cli = read_text(root, "crates/sifr/src/diagnostic_rendering_and_run.rs")
    for symbol in (
        "sifr_diagnostics::render_human_diagnostics",
        "sifr_diagnostics::render_compact_diagnostics",
        "sifr_diagnostics::render_json_diagnostics",
    ):
        require(symbol in cli, f"CLI renderer does not delegate to {symbol}")
    require(
        "fn render_compact_diagnostics" not in cli,
        "CLI still owns a local compact diagnostic renderer",
    )


def check_command_format_routing(root: Path) -> None:
    for command, relative in CLI_FORMAT_CASES.items():
        text = read_text(root, relative)
        require(
            "diagnostic_format: DiagnosticFormat" in text,
            f"{command} command lacks diagnostic format parameter coverage",
        )
        require(
            "render_diagnostics(" in text,
            f"{command} command lacks render_diagnostics routing coverage",
        )
    tests = read_text(root, "crates/sifr/src/diagnostics_and_packages_tests.rs")
    for command in CLI_FORMAT_CASES:
        require(command in tests, f"CLI format-selection regression coverage missing {command}")


def run_checks(root: Path) -> None:
    check_profile_runner_wiring(root)
    check_schema_lock(root)
    check_fixture_baselines(root)
    check_manifest_cases(root)
    check_human_baselines(root)
    check_compact_baselines(root)
    check_json_baselines(root)
    check_renderer_ownership(root)
    check_command_format_routing(root)


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def seed_minimal_repo(root: Path) -> None:
    write(
        root / "verification/runner/sifr_verify/profile_runner.py",
        'run_command(uv_area_command("--area", "developer_tooling", "--suite", "diagnostic-rules"))\n',
    )
    write(
        root / "verification/areas/developer_tooling/diagnostic_presentation_schema_lock.json",
        json.dumps({"rendered_diagnostic_required_fields": REQUIRED_FIELDS}),
    )
    write(
        root / "docs/schemas/diagnostics.schema.json",
        json.dumps(
            {
                "$defs": {
                    "RenderedDiagnostic": {
                        "required": REQUIRED_FIELDS,
                        "properties": {field: {} for field in REQUIRED_FIELDS},
                    }
                }
            }
        ),
    )
    manifest_cases = []
    for fixture in DIAGNOSTIC_FIXTURES:
        fixture_dir = root / "verification/areas/diagnostics/fixtures/diagnostics" / fixture
        write(fixture_dir / "main.sifr", "def subject():\n    pass\n")
        write(fixture_dir / "baselines/check-human.stdout.txt", "\n")
        write(fixture_dir / "baselines/check-human.exit-code.txt", "1\n")
        write(fixture_dir / "baselines/check-compact.stdout.txt", "\n")
        write(fixture_dir / "baselines/check-compact.exit-code.txt", "1\n")
        write(fixture_dir / "baselines/check-json.stdout.txt", "\n")
        write(fixture_dir / "baselines/check-json.exit-code.txt", "1\n")
    write(
        root
        / "verification/areas/diagnostics/fixtures/diagnostics/decimal_invalid_literal/baselines/check-human.stderr.txt",
        'error[SIFR-DECIMAL-0001]: msg\n  --> file.sifr:3:30\n   3 | Decimal("12.34.56")\n     | ^^^^^^^^^^\n  = docs: https://sifr.sh/docs/errors/SIFR-DECIMAL-0001\n',
    )
    write(
        root
        / "verification/areas/diagnostics/fixtures/diagnostics/multiline_span_rendering/baselines/check-human.stderr.txt",
        "error[SIFR-FLOW-0007]: msg\n  --> file.sifr:3:5\n   4 | line\n   6 | line\n   | ^^^^\n",
    )
    write(
        root
        / "verification/areas/diagnostics/fixtures/diagnostics/presentation_rules_cases/baselines/check-human.stderr.txt",
        "error[SIFR-TYPE-0002]: msg\n  ::: file.sifr:3:5\n  = related span\n  = location: <unavailable>\n  = note: child note rendered\n  = help: child help rendered\n  = suggestion: replace value\n",
    )
    for fixture, code in (
        ("decimal_invalid_literal", "SIFR-DECIMAL-0001"),
        ("multiline_span_rendering", "SIFR-FLOW-0007"),
        ("presentation_rules_cases", "SIFR-TYPE-0002"),
    ):
        write(
            root / f"verification/areas/diagnostics/fixtures/diagnostics/{fixture}/baselines/check-compact.stderr.txt",
            f"1 error, 0 warnings, 0 notes\nE {code} file.sifr:1:1 msg\n",
        )
    write(
        root
        / "verification/areas/diagnostics/fixtures/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt",
        json.dumps([diagnostic_json("SIFR-DECIMAL-0001", ["line"])]),
    )
    write(
        root
        / "verification/areas/diagnostics/fixtures/diagnostics/multiline_span_rendering/baselines/check-json.stderr.txt",
        json.dumps([diagnostic_json("SIFR-FLOW-0007", ["line1", "line2"])]),
    )
    presentation_json = diagnostic_json("SIFR-TYPE-0002", ["line\r"])
    presentation_json["suggestions"] = [{"message": "replace value"}]
    write(
        root
        / "verification/areas/diagnostics/fixtures/diagnostics/presentation_rules_cases/baselines/check-json.stderr.txt",
        json.dumps([presentation_json]),
    )
    for fixture in ("decimal_invalid_literal", "multiline_span_rendering"):
        manifest_cases.append(
            {
                "id": fixture,
                "entry": f"verification/areas/diagnostics/fixtures/diagnostics/{fixture}/main.sifr",
                "command": "check",
                "expect_exit_code": 1,
                "diagnostic_formats": ["human", "json", "compact"],
            }
        )
    write(
        root / "verification/areas/diagnostics/manifest.json",
        json.dumps({"suites": [{"name": "baselines", "cases": manifest_cases}]}),
    )
    write(
        root / "crates/sifr_diagnostics/src/render/presentation.rs",
        "fn render_human_diagnostics() {}\nfn render_compact_diagnostics() {}\n"
        "fn render_json_diagnostics() {}\nfn highlight_marker() {}\nfn terminal_line_text() {}\n",
    )
    write(
        root / "crates/sifr/src/diagnostic_rendering_and_run.rs",
        "diagnostic_format: DiagnosticFormat\nrender_diagnostics(\n"
        "sifr_diagnostics::render_human_diagnostics();\n"
        "sifr_diagnostics::render_compact_diagnostics();\n"
        "sifr_diagnostics::render_json_diagnostics();\n",
    )
    write(
        root / "crates/sifr/src/check_and_package_commands.rs",
        "diagnostic_format: DiagnosticFormat\nrender_diagnostics(\n",
    )
    write(
        root / "crates/sifr/src/diagnostics_and_packages_tests.rs",
        "check build run emit\n",
    )


def diagnostic_json(code: str, lines: list[str]) -> dict[str, Any]:
    return {
        "code": code,
        "severity": "Error",
        "message": "msg",
        "message_template": "{message}",
        "args": {},
        "url": f"https://sifr.sh/docs/errors/{code}",
        "spans": [
            {
                "file": "file.sifr",
                "byte_start": 0,
                "byte_end": 1,
                "line": 1,
                "column": 1,
                "end_line": 1,
                "end_column": 2,
                "is_primary": True,
                "label": None,
                "lines": [
                    {"text": text, "highlight_start": 1, "highlight_end": 2}
                    for text in lines
                ],
            }
        ],
        "children": [],
        "help": None,
        "suggestions": [],
    }


def expect_self_test_failure(description: str, expected: str, mutate: Any) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        seed_minimal_repo(root)
        mutate(root)
        try:
            run_checks(root)
        except AssertionError as error:
            if expected not in str(error):
                raise AssertionError(
                    f"{description}: expected {expected!r}, got {error!s}"
                ) from error
            return
        raise AssertionError(f"{description}: expected failure")


def run_self_tests() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        seed_minimal_repo(root)
        run_checks(root)

    expect_self_test_failure(
        "missing fixture",
        "required diagnostic fixture missing",
        lambda root: shutil.rmtree(
            root / "verification/areas/diagnostics/fixtures/diagnostics/multiline_span_rendering"
        ),
    )
    expect_self_test_failure(
        "missing baseline",
        "required baseline missing",
        lambda root: (
            root
            / "verification/areas/diagnostics/fixtures/diagnostics/decimal_invalid_literal/baselines/check-human.stderr.txt"
        ).unlink(),
    )
    expect_self_test_failure(
        "missing schema field",
        "schema lock does not enumerate",
        lambda root: write(
            root / "verification/areas/developer_tooling/diagnostic_presentation_schema_lock.json",
            json.dumps({"rendered_diagnostic_required_fields": REQUIRED_FIELDS[:-1]}),
        ),
    )
    expect_self_test_failure(
        "missing run-all wiring",
        "missing diagnostic rules suite route",
        lambda root: write(
            root / "verification/runner/sifr_verify/profile_runner.py",
            'run_command(uv_area_command("--area", "developer_tooling"))\n',
        ),
    )
    print("diagnostic presentation rules self-test: PASS")


def main() -> int:
    args = parse_args()
    try:
        if args.self_test:
            run_self_tests()
        else:
            run_checks(REPO_ROOT)
            print("diagnostic presentation rules: PASS")
    except AssertionError as error:
        print(f"diagnostic presentation rules: FAIL: {error}", flush=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
