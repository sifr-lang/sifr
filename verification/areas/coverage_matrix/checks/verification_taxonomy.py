#!/usr/bin/env python3
"""Reject delivery-plan taxonomy in active verification and crate surfaces."""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[4]

ACTIVE_ROOTS = (
    REPO_ROOT / "verification" / "areas" / "developer_tooling",
    REPO_ROOT / "verification" / "areas" / "package_management",
    REPO_ROOT / "verification" / "areas" / "coverage_matrix",
    REPO_ROOT / "verification" / "areas" / "diagnostics" / "data",
    REPO_ROOT / "verification" / "areas" / "distribution_release",
    REPO_ROOT / "verification" / "areas" / "core_language" / "data" / "validation_contracts",
    REPO_ROOT / "verification" / "areas" / "performance",
    REPO_ROOT / "verification" / "areas" / "runtime_platform",
    REPO_ROOT / "verification" / "areas" / "stdlib_parity" / "data",
    REPO_ROOT / "verification" / "areas" / "stdlib_parity" / "tools",
    REPO_ROOT / "verification" / "profiles",
    REPO_ROOT / "verification" / "runner",
    REPO_ROOT / "crates",
)

TEXT_EXTENSIONS = {
    ".json",
    ".md",
    ".py",
    ".rs",
    ".sifr",
    ".toml",
    ".yml",
    ".yaml",
}

FILENAME_PATTERNS = (
    re.compile(r"(^|[-_])(phase|milestone|wave)([-_]|$)", re.IGNORECASE),
    re.compile(r"(^|[-_])m\d+([._-]|$)", re.IGNORECASE),
)

ALLOW_TEXT_PATTERNS = (
    re.compile(r"\b(?:WorkspaceTracePhase|SingleOwnerCompilerPhase|LintPhase|PhaseExecution|ProgressPhase)\b"),
    re.compile(r"\b(?:phase_plan|empty_phase_plan|phase_has_enabled_rules|mark_phase_readonly)\b"),
    re.compile(r"\b(?:record_compiler_phase_trace|build phase|compiler phase|trace phases|phase=)\b", re.IGNORECASE),
    re.compile(r"\b" + "exp" + r"_m1\b"),
)

LEGACY_FIELD_PATTERNS = (
    "implementation_" + "milestone",
    "updated_by_" + "milestone",
    "future-" + "phase",
    "closes_in_" + "wave",
    "closes_in_sub" + "wave",
)

TEXT_PATTERNS = (
    re.compile(r"\bPhase\s+\d+\b"),
    re.compile(r"\bMilestone\s+\d+\b"),
    re.compile(r"\bWave\s+\d+\b"),
    re.compile(r"\b(?:phase|milestone|wave)[_-][a-z0-9][a-z0-9_-]*\b", re.IGNORECASE),
    re.compile(r"\b[a-z][a-z0-9_]*_m\d+[a-z0-9_]*\b", re.IGNORECASE),
    re.compile(r"\bm\d+[_-][a-z0-9][a-z0-9_-]*\b", re.IGNORECASE),
    re.compile(r"\b[a-z][a-z0-9_-]*[_-]m\d+\b", re.IGNORECASE),
    re.compile(r"\bm\d+-[a-z0-9][a-z0-9_-]*\b", re.IGNORECASE),
    re.compile(r"\bM\d+(?:\.\d+)?\b"),
    re.compile(r"\b(?:" + "|".join(re.escape(pattern) for pattern in LEGACY_FIELD_PATTERNS) + r")\b"),
)


@dataclass(frozen=True)
class Failure:
    path: Path
    line: int | None
    text: str

    def render(self) -> str:
        location = repo_path(self.path)
        if self.line is not None:
            location = f"{location}:{self.line}"
        return f"{location}: {self.text}"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()

    self_test_status = run_self_test(quiet=True)
    if self_test_status != 0:
        return self_test_status
    failures = collect_failures(ACTIVE_ROOTS)
    if failures:
        for failure in failures:
            print(f"verification-taxonomy error: {failure.render()}", file=sys.stderr)
        return 1
    print("verification taxonomy ok: active verification and crate surfaces use compiler/codebase terminology")
    return 0


def collect_failures(roots: tuple[Path, ...]) -> list[Failure]:
    failures: list[Failure] = []
    seen: set[Path] = set()
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if path in seen or should_skip(path):
                continue
            seen.add(path)
            if path.is_file():
                failures.extend(validate_filename(path))
                failures.extend(validate_text(path))
    return failures


def should_skip(path: Path) -> bool:
    parts = set(path.relative_to(REPO_ROOT).parts) if path.is_relative_to(REPO_ROOT) else set(path.parts)
    if parts & {".git", "__pycache__", "target", "third_party"}:
        return True
    if "reports" in parts:
        return True
    return path.is_file() and path.suffix not in TEXT_EXTENSIONS


def validate_filename(path: Path) -> list[Failure]:
    name = path.name
    return [
        Failure(path, None, f"filename contains delivery-plan taxonomy: {name}")
        for pattern in FILENAME_PATTERNS
        if pattern.search(name)
    ]


def validate_text(path: Path) -> list[Failure]:
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return []
    failures: list[Failure] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        checked_line = line
        for pattern in ALLOW_TEXT_PATTERNS:
            checked_line = pattern.sub("", checked_line)
        for pattern in TEXT_PATTERNS:
            if pattern.search(checked_line):
                failures.append(
                    Failure(path, line_number, f"line contains delivery-plan taxonomy: {line.strip()[:160]}")
                )
                break
    return failures


def run_self_test(*, quiet: bool = False) -> int:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        good = root / "compiler_contract.rs"
        good.write_text(
            "enum WorkspaceTracePhase { Parse }\nfn record_compiler_phase_trace() {}\n",
            encoding="utf-8",
        )
        bad_text = root / "active_manifest.json"
        bad_label = "Phase" + " 99 closeout"
        bad_file = "milestone" + "_99_tests.rs"
        bad_text.write_text(f'{{"label": "{bad_label}"}}\n', encoding="utf-8")
        bad_name = root / bad_file
        bad_name.write_text("// compiler test\n", encoding="utf-8")
        bad_mixed = root / "mixed_allowed_and_bad.rs"
        bad_mixed_label = "Milestone" + " 99"
        bad_mixed.write_text(f"// compiler phase trace; {bad_mixed_label}\n", encoding="utf-8")
        bad_prefix = root / "prefix_m_taxonomy.json"
        bad_prefix_label = "m" + "5_closure_evidence"
        bad_prefix.write_text(f'{{"{bad_prefix_label}": true}}\n', encoding="utf-8")
        bad_suffix = root / "suffix_m_taxonomy.json"
        bad_suffix_label = "blocked-on-runtime-" + "m1"
        bad_suffix.write_text(f'{{"state": "{bad_suffix_label}"}}\n', encoding="utf-8")
        bad_bare = root / "bare_m_taxonomy.md"
        bad_bare_label = "M" + "1"
        bad_bare.write_text(f"{bad_bare_label} owns a delivery slice.\n", encoding="utf-8")
        failures = collect_failures((root,))
    rendered = "\n".join(failure.render() for failure in failures)
    if (
        bad_label not in rendered
        or bad_file not in rendered
        or "mixed_allowed_and_bad.rs" not in rendered
        or bad_prefix_label not in rendered
        or bad_suffix_label not in rendered
        or bad_bare_label not in rendered
        or "compiler_contract.rs" in rendered
    ):
        print(f"verification taxonomy self-test failed: {rendered}", file=sys.stderr)
        return 1
    if not quiet:
        print("verification taxonomy self-test ok")
    return 0


def repo_path(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return path.as_posix()


if __name__ == "__main__":
    raise SystemExit(main())
