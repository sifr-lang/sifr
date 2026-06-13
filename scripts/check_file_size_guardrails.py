#!/usr/bin/env python3
"""Enforce repository-wide first-party source file-size limits."""

from __future__ import annotations

import argparse
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence


MAX_SOURCE_LINES = 900

LEGACY_HIR_GUARDRAIL_PATHS = (
    "crates/sifr_lowering/src/lower/mod.rs",
    "crates/sifr_lowering/src/lower/imports.rs",
    "crates/sifr_lowering/src/lower/diagnostics.rs",
    "crates/sifr_lowering/src/lower/classes.rs",
    "crates/sifr_lowering/src/lower/typing_and_functions.rs",
    "crates/sifr_lowering/src/lower/statements.rs",
    "crates/sifr_lowering/src/lower/expressions.rs",
    "crates/sifr_lowering/src/stdlib/mod.rs",
    "crates/sifr_lowering/src/stdlib/io_json.rs",
    "crates/sifr_lowering/src/stdlib/math_test.rs",
    "crates/sifr_lowering/src/stdlib/collections_bytes_time.rs",
    "crates/sifr_lowering/src/stdlib/sys_fs.rs",
    "crates/sifr_lowering/src/stdlib/crypto_regex_uuid.rs",
    "crates/sifr_lowering/src/stdlib/platform_misc.rs",
)

LEGACY_DRIVER_GUARDRAIL_PATHS = (
    "crates/sifr_driver/src/lib.rs",
    "crates/sifr_driver/src/project/mod.rs",
    "crates/sifr_driver/src/project/build.rs",
    "crates/sifr_driver/src/tests/project_graph.rs",
)


@dataclass(frozen=True)
class SourceFile:
    rel_path: Path
    category: str


@dataclass(frozen=True)
class Violation:
    rel_path: Path
    category: str
    lines: int
    limit: int


@dataclass(frozen=True)
class RustfmtSkipViolation:
    rel_path: Path
    line: int


def resolve_repo_root(script_path: Path) -> Path:
    return script_path.resolve().parent.parent


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce the 900-line cap for maintained first-party source files."
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run guardrail self-tests against temporary fixture trees.",
    )
    return parser.parse_args()


def path_parts(rel_path: Path) -> tuple[str, ...]:
    return rel_path.as_posix().split("/")


def has_any_part(rel_path: Path, names: set[str]) -> bool:
    return bool(names.intersection(path_parts(rel_path)))


def is_excluded_source_path(rel_path: Path) -> bool:
    parts = path_parts(rel_path)
    name = rel_path.name
    if has_any_part(rel_path, {"target", "third_party", "snapshots"}):
        return True
    if name.endswith(".lock") or name == "Cargo.lock":
        return True
    if name in {"emitted.rs", "idiomatic.rs"}:
        return True
    if any(part in {"baselines", "baseline_outputs"} for part in parts):
        return True
    if any("baseline" in part for part in parts):
        return True
    return False


def category_for_path(rel_path: Path) -> str | None:
    rel = rel_path.as_posix()
    if is_excluded_source_path(rel_path):
        return None
    if rel.startswith("crates/") and rel.endswith(".rs"):
        return "rust"
    if rel.startswith("scripts/") and rel.endswith(".py"):
        return "python-tooling"
    if rel.startswith("verification/") and rel.endswith(".py"):
        return "python-verification"
    if rel.startswith("demos/") and rel.endswith(".sifr"):
        return "sifr-demo"
    if rel.startswith("crates/sifr/tests/") and rel.endswith(".sifr"):
        return "sifr-fixture"
    return None


def iter_source_files(repo_root: Path) -> Iterable[SourceFile]:
    roots = ("crates", "scripts", "verification", "demos")
    suffixes = (".rs", ".py", ".sifr")
    for root_name in roots:
        root = repo_root / root_name
        if not root.exists():
            continue
        for path in sorted(root.rglob("*")):
            if not path.is_file() or path.suffix not in suffixes:
                continue
            rel_path = path.relative_to(repo_root)
            category = category_for_path(rel_path)
            if category is not None:
                yield SourceFile(rel_path=rel_path, category=category)


def count_physical_lines(path: Path) -> int:
    with path.open("r", encoding="utf-8") as handle:
        return sum(1 for _ in handle)


def has_generated_file_marker(path: Path) -> bool:
    with path.open("r", encoding="utf-8") as handle:
        for _ in range(5):
            line = handle.readline()
            if not line:
                return False
            if "@generated" in line or "DO NOT EDIT" in line:
                return True
    return False


def collect_violations(repo_root: Path) -> list[Violation]:
    violations: list[Violation] = []
    for source_file in iter_source_files(repo_root):
        path = repo_root / source_file.rel_path
        if has_generated_file_marker(path):
            continue
        lines = count_physical_lines(path)
        if lines > MAX_SOURCE_LINES:
            violations.append(
                Violation(
                    rel_path=source_file.rel_path,
                    category=source_file.category,
                    lines=lines,
                    limit=MAX_SOURCE_LINES,
                )
            )
    return violations


def collect_rustfmt_skip_violations(repo_root: Path) -> list[RustfmtSkipViolation]:
    violations: list[RustfmtSkipViolation] = []
    for source_file in iter_source_files(repo_root):
        if source_file.category != "rust":
            continue
        path = repo_root / source_file.rel_path
        if has_generated_file_marker(path):
            continue
        for line_number, line in enumerate(
            path.read_text(encoding="utf-8").splitlines(), start=1
        ):
            if "rustfmt::skip" in line:
                violations.append(RustfmtSkipViolation(source_file.rel_path, line_number))
    return violations


def format_violation(violation: Violation) -> str:
    return (
        f"{violation.rel_path}: {violation.lines} lines "
        f"(limit {violation.limit}, category {violation.category})"
    )


def write_lines(path: Path, count: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("x\n" * count, encoding="utf-8")


def assert_no_violations(repo_root: Path) -> None:
    violations = collect_violations(repo_root)
    if violations:
        formatted = "\n".join(format_violation(violation) for violation in violations)
        raise AssertionError(f"expected no violations, got:\n{formatted}")
    rustfmt_skip_violations = collect_rustfmt_skip_violations(repo_root)
    if rustfmt_skip_violations:
        formatted = "\n".join(
            f"{violation.rel_path}:{violation.line}"
            for violation in rustfmt_skip_violations
        )
        raise AssertionError(f"expected no rustfmt skip violations, got:\n{formatted}")


def assert_violation(repo_root: Path, expected_rel: str, expected_category: str) -> None:
    violations = collect_violations(repo_root)
    if len(violations) != 1:
        raise AssertionError(f"expected one violation, got {len(violations)}")
    violation = violations[0]
    output = format_violation(violation)
    expected_fragments = (
        expected_rel,
        "901 lines",
        f"limit {MAX_SOURCE_LINES}",
        f"category {expected_category}",
    )
    missing = [fragment for fragment in expected_fragments if fragment not in output]
    if missing:
        raise AssertionError(f"failure output missing {missing}: {output}")


def assert_paths_are_included(paths: Sequence[str]) -> None:
    missing = [path for path in paths if category_for_path(Path(path)) is None]
    if missing:
        formatted = "\n".join(f"- {path}" for path in missing)
        raise AssertionError(f"guardrail patterns do not include legacy paths:\n{formatted}")


def run_self_test() -> None:
    assert_paths_are_included(LEGACY_HIR_GUARDRAIL_PATHS)
    assert_paths_are_included(LEGACY_DRIVER_GUARDRAIL_PATHS)

    with tempfile.TemporaryDirectory(prefix="sifr-file-size-guardrail-") as temp_dir:
        repo_root = Path(temp_dir)

        passing_included = (
            "crates/example/src/lib.rs",
            "scripts/tool.py",
            "verification/tooling/check.py",
            "demos/sample.sifr",
            "crates/sifr/tests/e2e/pass/sample.sifr",
        )
        for rel in passing_included:
            write_lines(repo_root / rel, MAX_SOURCE_LINES)
        assert_no_violations(repo_root)

    with tempfile.TemporaryDirectory(prefix="sifr-file-size-guardrail-") as temp_dir:
        repo_root = Path(temp_dir)
        write_lines(repo_root / "scripts/oversized.py", MAX_SOURCE_LINES + 1)
        assert_violation(repo_root, "scripts/oversized.py", "python-tooling")

    excluded_oversized = (
        "target/generated.rs",
        "third_party/vendor/tool.py",
        "crates/example/src/snapshots/output.rs",
        "verification/areas/performance/baselines/result.py",
        "Cargo.lock",
        "crates/example/src/emitted.rs",
        "crates/example/src/idiomatic.rs",
    )
    with tempfile.TemporaryDirectory(prefix="sifr-file-size-guardrail-") as temp_dir:
        repo_root = Path(temp_dir)
        for rel in excluded_oversized:
            write_lines(repo_root / rel, MAX_SOURCE_LINES + 1)
        assert_no_violations(repo_root)

    with tempfile.TemporaryDirectory(prefix="sifr-file-size-guardrail-") as temp_dir:
        repo_root = Path(temp_dir)
        generated_path = repo_root / "crates/example/src/generated_tables.rs"
        generated_path.parent.mkdir(parents=True, exist_ok=True)
        generated_path.write_text(
            "// @generated by scripts/example.py; DO NOT EDIT.\n" + ("x\n" * MAX_SOURCE_LINES),
            encoding="utf-8",
        )
        assert_no_violations(repo_root)

    with tempfile.TemporaryDirectory(prefix="sifr-file-size-guardrail-") as temp_dir:
        repo_root = Path(temp_dir)
        skip_path = repo_root / "crates/example/src/lib.rs"
        skip_path.parent.mkdir(parents=True, exist_ok=True)
        skip_path.write_text("#[rustfmt::skip]\nfn compact() {}\n", encoding="utf-8")
        rustfmt_skip_violations = collect_rustfmt_skip_violations(repo_root)
        if len(rustfmt_skip_violations) != 1:
            raise AssertionError(
                f"expected one rustfmt skip violation, got {len(rustfmt_skip_violations)}"
            )


def main() -> int:
    args = parse_args()
    if args.self_test:
        run_self_test()
        print("file-size guardrails self-test: PASS")
        return 0

    repo_root = resolve_repo_root(Path(__file__))
    violations = collect_violations(repo_root)
    rustfmt_skip_violations = collect_rustfmt_skip_violations(repo_root)
    if violations or rustfmt_skip_violations:
        print("file-size guardrails: FAIL")
        for violation in violations:
            print(f"- {format_violation(violation)}")
        for violation in rustfmt_skip_violations:
            print(f"- {violation.rel_path}:{violation.line}: rustfmt::skip is not allowed")
        return 1

    scanned = sum(1 for _ in iter_source_files(repo_root))
    print(
        f"file-size guardrails: PASS ({scanned} files, limit {MAX_SOURCE_LINES} lines)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
