#!/usr/bin/env python3
"""Validate the diagnostic source-canonicalization phase contract."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
from typing import Any, Callable

REPO_ROOT = Path(__file__).resolve().parents[3]

NEW_IMPORT_CODES = {
    "SIFR-IMPORT-0005": "IMPORT_AMBIGUOUS_SOURCE_MODULE",
    "SIFR-IMPORT-0006": "IMPORT_NAMESPACE_COLLISION",
    "SIFR-IMPORT-0007": "IMPORT_CYCLE",
}

LEGACY_WORKSPACE_IMPORT_CODES = {
    "SIFR-WORKSPACE-0101": "SIFR-IMPORT-0002",
    "SIFR-WORKSPACE-0102": "SIFR-IMPORT-0005",
    "SIFR-WORKSPACE-0103": "SIFR-IMPORT-0006",
    "SIFR-WORKSPACE-0104": "SIFR-IMPORT-0007",
}

PARSER_FIXTURES = {
    "parser_bad_indent": "SIFR-PARSE-0002",
    "parser_unterminated_string": "SIFR-PARSE-0003",
    "parser_invalid_call_order": "SIFR-PARSE-0006",
    "parser_empty_declaration": "SIFR-PARSE-0007",
    "parser_invalid_declaration": "SIFR-PARSE-0002",
    "parser_invalid_match_pattern": "SIFR-PARSE-0008",
    "parser_unsupported_syntax": "SIFR-PARSE-0009",
}

PROJECT_FIXTURES = {
    "workspace_missing_import_canonical": "SIFR-IMPORT-0002",
    "workspace_ambiguous_import_canonical": "SIFR-IMPORT-0005",
    "workspace_namespace_collision_canonical": "SIFR-IMPORT-0006",
}

CYCLE_FIXTURES = {
    "import_cycle_source_spans": "SIFR-IMPORT-0007",
}

PACKAGE_FIXTURES = {
    "package_missing_import_canonical": "SIFR-IMPORT-0002",
    "package_ambiguous_import_canonical": "SIFR-IMPORT-0005",
}

PACKAGE_FATAL_FIXTURES = {
    "package_fatal_source_map_no_import_ambiguity": "SIFR-PACKAGE-0713",
}

SPAN_FIELDS = (
    "file",
    "byte_start",
    "byte_end",
    "line",
    "column",
    "end_line",
    "end_column",
    "is_primary",
    "lines",
)


@dataclass(frozen=True)
class CommandResult:
    exit_code: int
    stdout: str
    stderr: str
    argv: list[str]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def read_text(root: Path, relative: str) -> str:
    path = root / relative
    if not path.is_file():
        raise AssertionError(f"required file missing: {relative}")
    return path.read_text(encoding="utf-8")


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def check_run_all_wiring(root: Path) -> None:
    text = read_text(root, "scripts/run_all_tests.sh")
    require("--area developer_tooling" in text, "run_all_tests.sh missing developer_tooling area route")
    require("--suite diagnostic-contracts" in text, "run_all_tests.sh missing diagnostic contracts suite route")


def check_required_fixtures(root: Path) -> None:
    for fixture in PARSER_FIXTURES:
        path = root / "verification/areas/diagnostics/fixtures/diagnostics" / fixture / "main.sifr"
        require(path.is_file(), f"required parser fixture missing: {fixture}")
    for fixture in PROJECT_FIXTURES | CYCLE_FIXTURES:
        path = root / "verification/areas/project_workspace/fixtures/project" / fixture / "main.sifr"
        require(path.is_file(), f"required project fixture missing: {fixture}")
    for fixture in PACKAGE_FIXTURES | PACKAGE_FATAL_FIXTURES:
        path = root / "crates/sifr/tests/verification/package" / fixture
        require((path / "Cargo.toml").is_file(), f"package fixture missing Cargo.toml: {fixture}")
        require((path / "sifr.toml").is_file(), f"package fixture missing sifr.toml: {fixture}")
    help_path = (
        root
        / "crates/sifr/tests/verification/package/package_diagnostic_help_preserved/sifr.toml"
    )
    require(help_path.is_file(), "package help-preservation fixture missing")


def check_registry_and_docs(root: Path) -> None:
    registry = read_text(root, "crates/sifr_diagnostics/src/codes/registry.rs")
    entries = read_text(
        root,
        "crates/sifr_diagnostics/src/codes/registry/registry_entries/parsing_names_and_types.rs",
    )
    docs_index = read_text(root, "docs/errors/diagnostic-codes.md")
    for code, constant in NEW_IMPORT_CODES.items():
        require(
            f"pub const {constant}: Self" in registry,
            f"{code} missing DiagnosticCode constant {constant}",
        )
        require(
            f"DiagnosticCode::{constant}" in registry,
            f"{code} missing from ACTIVE_DIAGNOSTIC_CODES",
        )
        require(f'"{code}"' in entries, f"{code} missing active registry entry")
        require(
            (root / f"docs/errors/{code}.md").is_file(),
            f"{code} docs page missing",
        )
        require(f"[`{code}`]" in docs_index, f"{code} missing from diagnostic docs index")


def check_legacy_code_docs(root: Path) -> None:
    for legacy, replacement in LEGACY_WORKSPACE_IMPORT_CODES.items():
        text = read_text(root, f"docs/errors/{legacy}.md")
        require(
            replacement in text and "legacy" in text.lower(),
            f"{legacy} docs must name legacy replacement {replacement}",
        )


def cargo_debug_dir() -> Path:
    target_dir = os.environ.get("CARGO_TARGET_DIR")
    if target_dir:
        path = Path(target_dir)
        if not path.is_absolute():
            path = REPO_ROOT / path
    else:
        path = REPO_ROOT / "target"
    return path / "debug"


@lru_cache(maxsize=1)
def sifr_binary() -> Path:
    proc = subprocess.run(
        ["cargo", "build", "-q", "-p", "sifr"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
        timeout=300,
    )
    if proc.returncode != 0:
        raise AssertionError(
            "failed to build sifr diagnostic-contract binary:\n"
            f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
        )
    binary = cargo_debug_dir() / "sifr"
    require(binary.is_file(), f"sifr binary missing after build: {binary}")
    return binary


@lru_cache(maxsize=1)
def diagnostic_contract_harness_binary() -> Path:
    proc = subprocess.run(
        ["cargo", "build", "-q", "-p", "sifr_driver", "--bin", "diagnostic_contract_harness"],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
        timeout=300,
    )
    if proc.returncode != 0:
        raise AssertionError(
            "failed to build diagnostic contract harness:\n"
            f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
        )
    binary = cargo_debug_dir() / "diagnostic_contract_harness"
    require(binary.is_file(), f"diagnostic contract harness missing after build: {binary}")
    return binary


def run_runtime_harness() -> None:
    proc = subprocess.run(
        [str(diagnostic_contract_harness_binary())],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=False,
        timeout=120,
    )
    if proc.returncode != 0:
        raise AssertionError(
            "diagnostic contract harness failed:\n"
            f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
        )


def run_sifr(args: list[str], cwd: Path = REPO_ROOT) -> CommandResult:
    argv = [str(sifr_binary()), *args]
    proc = subprocess.run(
        argv,
        cwd=cwd,
        text=True,
        capture_output=True,
        check=False,
        timeout=300,
    )
    return CommandResult(proc.returncode, proc.stdout, proc.stderr, argv)


def parse_json_stderr(result: CommandResult) -> list[dict[str, Any]]:
    require(result.exit_code != 0, f"command unexpectedly succeeded: {' '.join(result.argv)}")
    try:
        payload = json.loads(result.stderr)
    except json.JSONDecodeError as error:
        raise AssertionError(
            f"stderr was not diagnostic JSON for {' '.join(result.argv)}:\n{result.stderr}"
        ) from error
    require(isinstance(payload, list) and payload, "diagnostic JSON payload is empty")
    first = payload[0]
    require(isinstance(first, dict), "diagnostic JSON entry is not an object")
    return payload


def primary_span(diagnostic: dict[str, Any]) -> dict[str, Any]:
    spans = diagnostic.get("spans")
    require(isinstance(spans, list) and spans, "diagnostic has no spans")
    primary = next(
        (span for span in spans if isinstance(span, dict) and span.get("is_primary") is True),
        spans[0],
    )
    for field in SPAN_FIELDS:
        require(field in primary, f"primary span missing field: {field}")
    require(primary.get("file") != "<unknown>", "primary span file is <unknown>")
    require(primary.get("line", 0) >= 1, "primary span line must be 1-based")
    require(primary.get("column", 0) >= 1, "primary span column must be 1-based")
    require(primary.get("lines"), "primary span missing snippet lines")
    return primary


def assert_json_contract(
    result: CommandResult,
    *,
    expected_code: str,
    case_id: str,
    forbidden_codes: set[str] | None = None,
    forbidden_prefixes: tuple[str, ...] = (),
    required_args: set[str] | None = None,
    require_help: bool = False,
    require_span: bool = True,
) -> None:
    payload = parse_json_stderr(result)
    codes = {str(item.get("code")) for item in payload if isinstance(item, dict)}
    require(expected_code in codes, f"{case_id}: expected {expected_code}, got {sorted(codes)}")
    for diagnostic in payload:
        if diagnostic.get("code") != expected_code:
            continue
        if require_span:
            primary_span(diagnostic)
        args = diagnostic.get("args")
        require(isinstance(args, dict), "diagnostic args must be an object")
        for arg in required_args or set():
            require(arg in args, f"{expected_code} missing JSON arg: {arg}")
        if require_help:
            require(diagnostic.get("help"), f"{expected_code} dropped help text")
        break
    if forbidden_codes:
        leaked = sorted(codes.intersection(forbidden_codes))
        require(not leaked, f"{case_id}: retired workspace import code leaked: {leaked}")
    for prefix in forbidden_prefixes:
        leaked = sorted(code for code in codes if code.startswith(prefix))
        require(not leaked, f"{case_id}: forbidden diagnostic family leaked: {leaked}")


def assert_text_format(
    *,
    entry: Path,
    expected_code: str,
    diagnostic_format: str,
    cwd: Path = REPO_ROOT,
) -> None:
    result = run_sifr(
        ["--diagnostic-format", diagnostic_format, "check", str(entry)],
        cwd=cwd,
    )
    require(result.exit_code != 0, f"{diagnostic_format} command unexpectedly succeeded")
    stderr = result.stderr
    require(expected_code in stderr, f"{diagnostic_format} output missing {expected_code}")
    require(
        "<unknown>" not in stderr,
        f"{diagnostic_format} output still uses <unknown> for {entry}",
    )
    if diagnostic_format == "human":
        require("-->" in stderr, "human output missing source location arrow")
    if diagnostic_format == "compact":
        require(
            f"E {expected_code} " in stderr,
            "compact output missing stable severity/code/location fields",
        )


def check_parser_runtime_contract(root: Path) -> None:
    base = root / "verification/areas/diagnostics/fixtures/diagnostics"
    for fixture, code in PARSER_FIXTURES.items():
        entry = base / fixture / "main.sifr"
        json_result = run_sifr(["--diagnostic-format", "json", "check", str(entry)])
        assert_json_contract(json_result, expected_code=code, case_id=fixture)
        assert_text_format(entry=entry, expected_code=code, diagnostic_format="human")
        assert_text_format(entry=entry, expected_code=code, diagnostic_format="compact")


def check_project_runtime_contract(root: Path) -> None:
    base = root / "verification/areas/project_workspace/fixtures/project"
    required_args = {
        "workspace_missing_import_canonical": {"resolution_scope", "tried_paths"},
        "workspace_ambiguous_import_canonical": {"resolution_scope", "candidate_paths"},
        "workspace_namespace_collision_canonical": {"resolved_path", "parent_path"},
    }
    forbidden = set(LEGACY_WORKSPACE_IMPORT_CODES)
    for fixture, code in PROJECT_FIXTURES.items():
        entry = base / fixture / "main.sifr"
        json_result = run_sifr(["--diagnostic-format", "json", "check", str(entry)])
        assert_json_contract(
            json_result,
            expected_code=code,
            case_id=fixture,
            forbidden_codes=forbidden,
            required_args=required_args[fixture],
        )
        assert_text_format(entry=entry, expected_code=code, diagnostic_format="human")
        assert_text_format(entry=entry, expected_code=code, diagnostic_format="compact")


def check_cycle_runtime_contract(root: Path) -> None:
    base = root / "verification/areas/project_workspace/fixtures/project"
    for fixture, code in CYCLE_FIXTURES.items():
        entry = base / fixture / "main.sifr"
        json_result = run_sifr(["--diagnostic-format", "json", "check", str(entry)])
        assert_json_contract(
            json_result,
            expected_code=code,
            case_id=fixture,
            forbidden_codes=set(LEGACY_WORKSPACE_IMPORT_CODES),
            required_args={"cycle", "cycle_edges"},
        )
        assert_text_format(entry=entry, expected_code=code, diagnostic_format="human")
        assert_text_format(entry=entry, expected_code=code, diagnostic_format="compact")


def check_package_runtime_contract(root: Path) -> None:
    base = root / "crates/sifr/tests/verification/package"
    required_args = {
        "package_missing_import_canonical": {
            "resolution_scope",
            "tried_paths",
            "written_module_path",
            "package_import_origin",
        },
        "package_ambiguous_import_canonical": {
            "resolution_scope",
            "candidate_paths",
            "written_module_path",
            "package_import_origin",
        },
    }
    for fixture, code in PACKAGE_FIXTURES.items():
        package = base / fixture
        entry = next(package.glob("src*/main.sifr"))
        json_result = run_sifr(
            ["--diagnostic-format", "json", "check", str(entry.relative_to(package))],
            cwd=package,
        )
        assert_json_contract(
            json_result,
            expected_code=code,
            case_id=fixture,
            forbidden_codes=set(LEGACY_WORKSPACE_IMPORT_CODES),
            forbidden_prefixes=("SIFR-PACKAGE-",),
            required_args=required_args[fixture],
        )
        assert_text_format(
            entry=entry.relative_to(package),
            expected_code=code,
            diagnostic_format="human",
            cwd=package,
        )
        assert_text_format(
            entry=entry.relative_to(package),
            expected_code=code,
            diagnostic_format="compact",
            cwd=package,
        )
    for fixture, code in PACKAGE_FATAL_FIXTURES.items():
        package = base / fixture
        entry = next(package.glob("src*/main.sifr"))
        json_result = run_sifr(
            ["--diagnostic-format", "json", "check", str(entry.relative_to(package))],
            cwd=package,
        )
        assert_json_contract(
            json_result,
            expected_code=code,
            case_id=fixture,
            forbidden_prefixes=("SIFR-IMPORT-",),
            required_args={"origin_kind", "manifest_path", "manifest_key"},
            require_span=False,
        )

def check_package_help_contract(root: Path) -> None:
    help_fixture = (
        root / "crates/sifr/tests/verification/package/package_diagnostic_help_preserved"
    )
    help_result = run_sifr(
        ["--diagnostic-format", "json", "package", "--list", "--allow-dirty", "--no-verify"],
        cwd=help_fixture,
    )
    assert_json_contract(
        help_result,
        expected_code="SIFR-PACKAGE-0701",
        case_id="package_diagnostic_help_preserved",
        required_args={"origin_kind", "manifest_path", "manifest_key"},
        require_help=True,
        require_span=False,
    )


def run_static_checks(root: Path) -> None:
    check_run_all_wiring(root)
    check_required_fixtures(root)
    check_registry_and_docs(root)
    check_legacy_code_docs(root)


def run_checks(root: Path) -> None:
    run_static_checks(root)
    run_runtime_harness()
    check_package_help_contract(root)


def seed_minimal_repo(root: Path) -> None:
    write(
        root / "scripts/run_all_tests.sh",
        "uv run --project verification --locked python -m sifr_verify areas run "
        "--area developer_tooling --suite diagnostic-contracts\n",
    )
    write(root / "crates/sifr_diagnostics/src/codes/registry.rs", registry_seed())
    write(
        root
        / "crates/sifr_diagnostics/src/codes/registry/registry_entries/parsing_names_and_types.rs",
        "\n".join(f'active_entry!("{code}")' for code in NEW_IMPORT_CODES),
    )
    index_lines = []
    for code in NEW_IMPORT_CODES:
        write(root / f"docs/errors/{code}.md", f"# {code}\n")
        index_lines.append(f"| [`{code}`]({code}.md) | Error | seeded. |")
    for legacy, replacement in LEGACY_WORKSPACE_IMPORT_CODES.items():
        write(root / f"docs/errors/{legacy}.md", f"# {legacy}\nLegacy alias for {replacement}.\n")
    write(root / "docs/errors/diagnostic-codes.md", "\n".join(index_lines))
    for fixture in PARSER_FIXTURES:
        write(
            root / f"verification/areas/diagnostics/fixtures/diagnostics/{fixture}/main.sifr",
            "def main():\n    pass\n",
        )
    for fixture in PROJECT_FIXTURES | CYCLE_FIXTURES:
        write(
            root / f"verification/areas/project_workspace/fixtures/project/{fixture}/main.sifr",
            "def main():\n    pass\n",
        )
    for fixture in PACKAGE_FIXTURES | PACKAGE_FATAL_FIXTURES:
        write(root / f"crates/sifr/tests/verification/package/{fixture}/Cargo.toml", "[package]\n")
        write(root / f"crates/sifr/tests/verification/package/{fixture}/sifr.toml", "[package]\n")
    write(
        root
        / "crates/sifr/tests/verification/package/package_diagnostic_help_preserved/sifr.toml",
        "[package]\n",
    )


def registry_seed() -> str:
    constants = []
    active = []
    for code, constant in NEW_IMPORT_CODES.items():
        constants.append(f'pub const {constant}: Self = Self::new("{code}", Severity::Error);')
        active.append(f"DiagnosticCode::{constant},")
    return (
        "struct DiagnosticCode;\n"
        "enum Severity { Error }\n"
        "impl DiagnosticCode { const fn new(_: &str, _: Severity) -> Self { Self }\n"
        + "\n".join(constants)
        + "}\npub const ACTIVE_DIAGNOSTIC_CODES: &[DiagnosticCode] = &[\n"
        + "\n".join(active)
        + "\n];\n"
    )


def expect_self_test_failure(
    description: str,
    expected: str,
    mutate: Callable[[Path], None],
) -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        seed_minimal_repo(root)
        mutate(root)
        try:
            run_static_checks(root)
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
        run_static_checks(root)

    expect_self_test_failure(
        "missing parser fixture",
        "required parser fixture missing",
        lambda root: shutil.rmtree(
            root / "verification/areas/diagnostics/fixtures/diagnostics/parser_bad_indent"
        ),
    )
    expect_self_test_failure(
        "missing active code",
        "missing from ACTIVE_DIAGNOSTIC_CODES",
        lambda root: write(
            root / "crates/sifr_diagnostics/src/codes/registry.rs",
            registry_seed().replace("DiagnosticCode::IMPORT_CYCLE,", ""),
        ),
    )
    expect_self_test_failure(
        "missing legacy migration docs",
        "docs must name legacy replacement",
        lambda root: write(root / "docs/errors/SIFR-WORKSPACE-0104.md", "# legacy\n"),
    )

    spanless = CommandResult(
        1,
        "",
        json.dumps(
            [
                {
                    "code": "SIFR-IMPORT-0002",
                    "args": {},
                    "spans": [],
                }
            ]
        ),
        ["sifr"],
    )
    try:
        assert_json_contract(spanless, expected_code="SIFR-IMPORT-0002", case_id="spanless")
    except AssertionError as error:
        require("diagnostic has no spans" in str(error), "spanless self-test failed oddly")
    else:
        raise AssertionError("spanless diagnostic self-test expected failure")

    print("diagnostic source canonicalization contract self-test: PASS")


def main() -> int:
    args = parse_args()
    try:
        if args.self_test:
            run_self_tests()
        else:
            run_checks(REPO_ROOT)
            print("diagnostic source canonicalization contract: PASS")
    except AssertionError as error:
        print(f"diagnostic source canonicalization contract: FAIL: {error}", flush=True)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
