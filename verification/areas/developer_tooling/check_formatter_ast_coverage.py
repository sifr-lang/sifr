#!/usr/bin/env python3
"""Validate Sifr formatter AST coverage and corpus samples."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_DIR = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "formatter_manifests"
CORPUS_DIR = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "formatter_corpus"
RUFF_FORMATTER = REPO_ROOT / "third_party" / "ruff" / "crates" / "ruff_python_formatter"
RUFF_AST_NODES = REPO_ROOT / "third_party" / "ruff" / "crates" / "ruff_python_ast" / "src" / "nodes.rs"
RUFF_PARSER_STATEMENT = (
    REPO_ROOT / "third_party" / "ruff" / "crates" / "ruff_python_parser" / "src" / "parser" / "statement.rs"
)

REQUIRED_EXTENSIONS = {
    "param_default_borrow",
    "param_mut",
    "param_own",
    "param_own_mut",
    "param_mut_own_tolerant",
    "sifr_type_annotations",
    "sifr_generics",
    "match_case",
    "ownership_aware_collections",
    "formatter_pragmas",
    "docstring_code_snippets",
}

AST_MARKERS = {
    "param_default_borrow": [(RUFF_AST_NODES, "AstParamOwnership::Borrow")],
    "param_mut": [(RUFF_AST_NODES, "AstParamMutability::Mutable")],
    "param_own": [(RUFF_AST_NODES, "AstParamOwnership::Own")],
    "param_own_mut": [(RUFF_AST_NODES, "own_mut()")],
    "param_mut_own_tolerant": [(RUFF_PARSER_STATEMENT, '"mut"'), (RUFF_PARSER_STATEMENT, '"own"')],
    "sifr_type_annotations": [(RUFF_AST_NODES, "annotation: Option<Box<Expr>>")],
    "sifr_generics": [(RUFF_AST_NODES, "TypeParam")],
    "match_case": [(RUFF_AST_NODES, "StmtMatch"), (RUFF_AST_NODES, "MatchCase")],
    "ownership_aware_collections": [(RUFF_AST_NODES, "ExprList"), (RUFF_AST_NODES, "ExprDict")],
    "formatter_pragmas": [(RUFF_FORMATTER / "src" / "comments" / "fmt.rs", "fmt: off")],
    "docstring_code_snippets": [(RUFF_FORMATTER / "src" / "string" / "docstring.rs", "python3|py3|sifr")],
}


class CoverageError(Exception):
    pass


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise CoverageError(f"failed to read {path.relative_to(REPO_ROOT)}: {error}") from error
    except json.JSONDecodeError as error:
        raise CoverageError(f"malformed JSON in {path.relative_to(REPO_ROOT)}: {error}") from error
    if not isinstance(value, dict):
        raise CoverageError(f"{path.relative_to(REPO_ROOT)} root must be an object")
    return value


def run(command: list[str], *, cwd: Path = REPO_ROOT, input_text: str | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=cwd,
        input=input_text,
        text=True,
        capture_output=True,
        check=False,
    )


def require(condition: bool, message: str, failures: list[str]) -> None:
    if not condition:
        failures.append(message)


def discover_extensions() -> set[str]:
    discovered: set[str] = set()
    for extension, markers in AST_MARKERS.items():
        present = True
        for path, marker in markers:
            if not path.is_file() or marker not in path.read_text(encoding="utf-8"):
                present = False
                break
        if present:
            discovered.add(extension)
    return discovered


def formatter_snapshot_for_fixture(fixture: Path) -> Path:
    relative = fixture.relative_to(RUFF_FORMATTER / "resources" / "test" / "fixtures" / "ruff")
    suffix = relative.as_posix().replace("/", "__")
    return RUFF_FORMATTER / "tests" / "snapshots" / f"format@{suffix}.snap"


def corpus_fixtures(corpus: dict[str, Any]) -> dict[str, dict[str, Any]]:
    raw = corpus.get("fixtures")
    if not isinstance(raw, list):
        raise CoverageError("formatter corpus manifest fixtures must be a list")
    fixtures: dict[str, dict[str, Any]] = {}
    for item in raw:
        if not isinstance(item, dict):
            raise CoverageError("formatter corpus entries must be objects")
        fixture_id = item.get("id")
        if not isinstance(fixture_id, str) or not fixture_id:
            raise CoverageError("formatter corpus fixture missing non-empty id")
        if fixture_id in fixtures:
            raise CoverageError(f"duplicate formatter corpus fixture id: {fixture_id}")
        fixtures[fixture_id] = item
    return fixtures


def resolve_corpus_path(relative: str) -> Path:
    return (CORPUS_DIR / relative).resolve()


def validate_manifests(
    ast_manifest: dict[str, Any],
    corpus: dict[str, Any],
    discovered: set[str],
) -> list[str]:
    failures: list[str] = []
    rows = ast_manifest.get("rows")
    if not isinstance(rows, list):
        return ["AST coverage manifest rows must be a list"]
    fixtures = corpus_fixtures(corpus)
    row_ids = {row.get("id") for row in rows if isinstance(row, dict)}

    require(REQUIRED_EXTENSIONS <= row_ids, f"AST coverage manifest missing required rows: {sorted(REQUIRED_EXTENSIONS - row_ids)}", failures)
    require(discovered <= row_ids, f"discovered Sifr AST extensions lack formatter rows: {sorted(discovered - row_ids)}", failures)
    require(row_ids <= REQUIRED_EXTENSIONS, f"AST coverage manifest has unreviewed rows: {sorted(row_ids - REQUIRED_EXTENSIONS)}", failures)

    for row in rows:
        if not isinstance(row, dict):
            failures.append("AST coverage rows must be objects")
            continue
        row_id = row.get("id")
        fork_fixture = row.get("fork_fixture")
        wrapper_fixture = row.get("sifr_wrapper_fixture")
        if row.get("classification") == "not-applicable":
            approval = row.get("review_approval")
            require(
                isinstance(approval, str) and "Claude" in approval and "http" in approval,
                f"{row_id}: not-applicable formatter coverage requires reviewer approval link",
                failures,
            )
            continue
        for field, value in [("fork_fixture", fork_fixture), ("sifr_wrapper_fixture", wrapper_fixture)]:
            require(isinstance(value, str) and value, f"{row_id}: missing {field}", failures)
            if isinstance(value, str):
                require(not value.startswith("pending:"), f"{row_id}: unresolved {field} {value!r}", failures)
        if isinstance(fork_fixture, str) and not fork_fixture.startswith("pending:"):
            fork_path = REPO_ROOT / fork_fixture
            require(fork_path.is_file(), f"{row_id}: fork formatter fixture does not exist: {fork_fixture}", failures)
            if fork_path.is_file() and RUFF_FORMATTER in fork_path.parents:
                snapshot = formatter_snapshot_for_fixture(fork_path)
                require(snapshot.is_file(), f"{row_id}: fork formatter snapshot is missing: {snapshot.relative_to(REPO_ROOT)}", failures)
        if isinstance(wrapper_fixture, str) and wrapper_fixture.startswith("formatter_corpus:"):
            fixture_id = wrapper_fixture.split(":", maxsplit=1)[1]
            fixture = fixtures.get(fixture_id)
            require(fixture is not None, f"{row_id}: unknown formatter corpus fixture {fixture_id!r}", failures)
            if fixture is not None:
                extensions = fixture.get("extensions", [])
                require(isinstance(extensions, list) and row_id in extensions, f"{row_id}: corpus fixture {fixture_id!r} does not list this extension", failures)
        elif isinstance(wrapper_fixture, str):
            require(False, f"{row_id}: wrapper fixture must use formatter_corpus:<id>", failures)

    for fixture_id, fixture in fixtures.items():
        kind = fixture.get("kind")
        require(kind in {"stable", "canonicalize"}, f"{fixture_id}: invalid fixture kind {kind!r}", failures)
        extensions = fixture.get("extensions")
        require(isinstance(extensions, list) and extensions, f"{fixture_id}: missing extension list", failures)
        unknown = sorted(set(extensions or []) - REQUIRED_EXTENSIONS)
        require(not unknown, f"{fixture_id}: unknown extension ids {unknown}", failures)
        if kind == "stable":
            path = fixture.get("path")
            require(isinstance(path, str), f"{fixture_id}: stable fixture missing path", failures)
            if isinstance(path, str):
                require(resolve_corpus_path(path).is_file(), f"{fixture_id}: fixture path missing: {path}", failures)
        if kind == "canonicalize":
            for field in ["input", "expected"]:
                value = fixture.get(field)
                require(isinstance(value, str), f"{fixture_id}: canonical fixture missing {field}", failures)
                if isinstance(value, str):
                    require(resolve_corpus_path(value).is_file(), f"{fixture_id}: fixture {field} missing: {value}", failures)
    return failures


def write_config(root: Path, config: dict[str, Any] | None) -> None:
    if not config:
        return
    lines = ["[format]"]
    for key, value in config.items():
        if isinstance(value, bool):
            rendered = "true" if value else "false"
        elif isinstance(value, int):
            rendered = str(value)
        else:
            rendered = json.dumps(value)
        lines.append(f"{key} = {rendered}")
    (root / "sifr.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")


def cargo_target_dir() -> Path:
    return Path(os.environ.get("CARGO_TARGET_DIR", REPO_ROOT / "target")).resolve()


def build_sifr() -> Path:
    completed = run(["cargo", "build", "-q", "-p", "sifr"])
    if completed.returncode != 0:
        raise CoverageError(f"failed to build sifr formatter CLI:\n{completed.stderr}")
    binary = cargo_target_dir() / "debug" / "sifr"
    if not binary.is_file():
        raise CoverageError(f"sifr binary was not produced at {binary}")
    return binary


def run_formatter_samples(corpus: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    fixtures = corpus_fixtures(corpus)
    sifr = build_sifr()
    target_dir = cargo_target_dir()
    target_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=target_dir) as tmp:
        temp_root = Path(tmp)
        for fixture_id, fixture in fixtures.items():
            fixture_root = temp_root / fixture_id
            fixture_root.mkdir()
            write_config(fixture_root, fixture.get("config"))
            if fixture["kind"] == "stable":
                source = resolve_corpus_path(str(fixture["path"]))
                candidate = fixture_root / source.name
                shutil.copyfile(source, candidate)
                assert_stable_sample(sifr, fixture_id, candidate, fixture_root, failures)
            else:
                source = resolve_corpus_path(str(fixture["input"]))
                expected = resolve_corpus_path(str(fixture["expected"])).read_text(encoding="utf-8")
                candidate = fixture_root / source.name.replace(".input", "")
                shutil.copyfile(source, candidate)
                formatted = run([str(sifr), "fmt", "--no-cache", str(candidate)], cwd=fixture_root)
                if formatted.returncode != 0:
                    failures.append(f"{fixture_id}: formatter failed:\n{formatted.stderr}")
                    continue
                actual = candidate.read_text(encoding="utf-8")
                require(actual == expected, f"{fixture_id}: formatted output did not match expected fixture", failures)
                assert_stable_sample(sifr, fixture_id, candidate, fixture_root, failures)
        assert_invalid_source_diagnostic(sifr, temp_root, failures)
        assert_config_matrix(sifr, temp_root, failures)
    return failures


def assert_stable_sample(
    sifr: Path,
    fixture_id: str,
    candidate: Path,
    cwd: Path,
    failures: list[str],
) -> None:
    before = candidate.read_text(encoding="utf-8")
    check = run([str(sifr), "fmt", "--check", "--no-cache", str(candidate)], cwd=cwd)
    require(check.returncode == 0, f"{fixture_id}: formatted sample failed --check:\n{check.stderr}", failures)
    first = run([str(sifr), "fmt", "--no-cache", str(candidate)], cwd=cwd)
    require(first.returncode == 0, f"{fixture_id}: first formatter pass failed:\n{first.stderr}", failures)
    after_first = candidate.read_text(encoding="utf-8")
    second = run([str(sifr), "fmt", "--no-cache", str(candidate)], cwd=cwd)
    require(second.returncode == 0, f"{fixture_id}: second formatter pass failed:\n{second.stderr}", failures)
    after_second = candidate.read_text(encoding="utf-8")
    require(after_first == after_second, f"{fixture_id}: formatter is not idempotent", failures)
    if before != after_first:
        failures.append(f"{fixture_id}: stable fixture was not checked in formatted form")


def assert_invalid_source_diagnostic(sifr: Path, temp_root: Path, failures: list[str]) -> None:
    invalid = temp_root / "invalid_formatter_source.sifr"
    invalid.write_text("def broken(:\n", encoding="utf-8")
    completed = run([str(sifr), "fmt", "--check", "--no-cache", str(invalid)], cwd=temp_root)
    require(completed.returncode != 0, "invalid formatter source unexpectedly succeeded", failures)
    diagnostic_output = completed.stdout + completed.stderr
    require("SIFR-FMT-0001" in diagnostic_output or "formatter could not parse" in diagnostic_output, "invalid formatter source did not report a formatter parse diagnostic", failures)


def assert_config_matrix(sifr: Path, temp_root: Path, failures: list[str]) -> None:
    project = temp_root / "config_matrix"
    project.mkdir()
    (project / "sifr.toml").write_text(
        "[format]\nline-length = 40\ndocstring-code-format = true\ndocstring-code-line-length = \"dynamic\"\n",
        encoding="utf-8",
    )
    source = project / "main.sifr"
    source.write_text(
        "def documented(mut own values:list[int])->list[int]:\n"
        "    \"\"\"\n"
        "    ```sifr\n"
        "    def sample( mut own items:list[int])->list[int]:\n"
        "        return [item for item in items if item>0]\n"
        "    ```\n"
        "    \"\"\"\n"
        "    return values\n",
        encoding="utf-8",
    )
    completed = run([str(sifr), "fmt", "--no-cache", str(source)], cwd=project)
    require(completed.returncode == 0, f"config matrix formatter run failed:\n{completed.stderr}", failures)
    formatted = source.read_text(encoding="utf-8")
    require("own mut values: list[int]" in formatted, "config matrix did not format Sifr parameter conventions", failures)
    require("own mut items: list[int]" in formatted, "config matrix did not enable docstring code formatting", failures)
    require("item > 0" in formatted, "config matrix did not format docstring snippet expressions", failures)


def run_positive() -> None:
    ast_manifest = load_json(MANIFEST_DIR / "ast_coverage.json")
    corpus = load_json(CORPUS_DIR / "manifest.json")
    discovered = discover_extensions()
    failures = validate_manifests(ast_manifest, corpus, discovered)
    failures.extend(run_formatter_samples(corpus))
    if failures:
        print("formatter AST coverage: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        raise SystemExit(1)
    print("formatter AST coverage: PASS")


def run_self_test() -> None:
    corpus = {
        "schema": 1,
        "fixtures": [
            {
                "id": "parameters",
                "kind": "stable",
                "path": "fixtures/parameters.sifr",
                "extensions": ["param_default_borrow"],
            }
        ],
    }
    ast_manifest = {
        "schema": 1,
        "rows": [
            {
                "id": "param_default_borrow",
                "fork_fixture": "pending:m2",
                "sifr_wrapper_fixture": "formatter_corpus:parameters",
            }
        ],
    }
    failures = validate_manifests(ast_manifest, corpus, {"param_default_borrow"})
    if not any("unresolved fork_fixture" in failure for failure in failures):
        raise SystemExit("formatter AST coverage self-test failed: pending fixture passed")
    failures = validate_manifests(ast_manifest, corpus, {"param_default_borrow", "param_mut"})
    if not any("discovered Sifr AST extensions" in failure for failure in failures):
        raise SystemExit("formatter AST coverage self-test failed: missing discovered extension passed")
    ast_manifest["rows"][0]["classification"] = "not-applicable"
    ast_manifest["rows"][0].pop("fork_fixture")
    failures = validate_manifests(ast_manifest, corpus, {"param_default_borrow"})
    if not any("requires reviewer approval" in failure for failure in failures):
        raise SystemExit("formatter AST coverage self-test failed: unapproved non-applicable row passed")
    print("formatter AST coverage self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
    else:
        run_positive()
    return 0


if __name__ == "__main__":
    sys.exit(main())
