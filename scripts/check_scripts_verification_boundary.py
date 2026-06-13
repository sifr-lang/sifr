#!/usr/bin/env python3
"""Validate that scripts/ does not own verification implementation."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]

# Strict manifest of source files intentionally allowed under scripts/.
# If one of these scripts is retired, delete the manifest entry with the file.
ALLOWED_SCRIPT_FILES = {
    "archive_issues.sh",
    "archive_reviews.sh",
    "archive_reviews_and_issues.sh",
    "check_codegen_rawcode_gate.sh",
    "check_audits_normalization.py",
    "check_cursor_hygiene.py",
    "check_diagnostic_cancel_usage.py",
    "check_diagnostic_transport_cleanup.py",
    "check_file_size_guardrails.py",
    "check_hir_maintainability_guardrails.py",
    "check_scripts_verification_boundary.py",
    "check_sifr_driver_maintainability_guardrails.py",
    "check_source_crate_dependency_direction.py",
    "check_submodule_ownership.py",
    "clone_subrepos.sh",
    "generate_unicode_tables.py",
    "run_all_tests.sh",
    "distribution/build_preview_artifacts.sh",
    "distribution/create_new_version.sh",
    "distribution/generate_dispatchers.sh",
    "distribution/generate_version_installer.sh",
}

FORBIDDEN_SCRIPT_PATHS = {
    "check_codegen_binary_size.sh",
    "check_e2e_report_determinism.sh",
    "check_e2e_sequential_parallel_equivalence.sh",
    "ci_e2e_throughput.sh",
    "phase_contract_gate_check.py",
    "run_e2e_pass.sh",
    "run_integer_model_closure_perf.py",
    "run_verification_hardening.py",
    "run_verification_hardening/__init__.py",
    "run_verification_hardening/core.py",
    "run_verification_hardening/fixedbugs_and_crashes.py",
    "run_verification_hardening/main_flow.py",
    "run_verification_hardening/oss_and_determinism.py",
    "run_verification_hardening/property_and_fuzz.py",
    "run_verification_hardening/self_tests_and_baselines.py",
    "validate_phase15_backlog.py",
}

# Substring patterns for stale active references. Keep these intentionally coarse:
# active surfaces should use the new location rather than quote retired entrypoints.
STALE_REFERENCE_PATTERNS = {
    "scripts/run_e2e_pass.sh": "verification/runner/e2e/run_e2e_pass.sh",
    "scripts/run_smoke_fuzz_property.sh": (
        "uv run --project verification --locked python -m sifr_verify "
        "areas run --area fuzz_property --suite cargo-smoke --suite property --suite fuzz-smoke"
    ),
    "scripts/check_e2e_report_determinism.sh": "verification/runner/e2e/check_report_determinism.sh",
    "scripts/check_e2e_sequential_parallel_equivalence.sh": (
        "verification/runner/e2e/check_sequential_parallel_equivalence.sh"
    ),
    "scripts/run_verification_hardening.py": "uv run --project verification --locked python -m sifr_verify.hardening",
    "scripts/check_codegen_binary_size.sh": "verification/areas/performance/tools/check_codegen_binary_size.sh",
    "scripts/ci_e2e_throughput.sh": "verification/areas/performance/tools/ci_e2e_throughput.sh",
    "scripts/run_integer_model_closure_perf.py": "verification/areas/performance/tools/run_integer_model_closure_perf.py",
    "scripts/phase_contract_gate_check.py": "",
    "scripts/validate_phase15_backlog.py": "",
    "scripts.run_verification_hardening": "sifr_verify.hardening",
    "verification/validation_lanes/": "verification/areas/core_language/data/",
}

REFERENCE_PATHS = [
    ".github",
    "AGENTS.md",
    "README.md",
    "demos",
    "internal_docs/architecture.md",
    "verification/policy",
    "scripts",
    "verification/areas",
    "verification/determinism",
    "verification/integer_model_closure_hardening.md",
    "verification/runner",
]

PERSONAL_PATH_REFERENCE_PATHS = [
    ".github",
    ".cursor",
    "AGENTS.md",
    "CLAUDE.md",
    "README.md",
    "demos",
    "docs",
    "scripts",
    "verification/areas/developer_tooling/linter_manifests",
    "verification/areas/stdlib_parity/data",
    "verification/areas/stdlib_parity/reports",
    "verification/areas/stdlib_parity/tools",
    "verification/profiles",
    "verification/runner",
]

PERSONAL_PATH_PATTERNS = {
    "/Users/yaseralnajjar/": "use a repo-relative path or an environment variable",
}


def scripts_files(root: Path) -> set[str]:
    scripts_root = root / "scripts"
    return {
        path.relative_to(scripts_root).as_posix()
        for path in scripts_root.rglob("*")
        if path.is_file()
    }


def validate_script_file_set(actual: set[str]) -> list[str]:
    failures: list[str] = []
    for path in sorted(actual - ALLOWED_SCRIPT_FILES - FORBIDDEN_SCRIPT_PATHS):
        failures.append(f"unclassified scripts/ file remains: scripts/{path}")
    for path in sorted(FORBIDDEN_SCRIPT_PATHS.intersection(actual)):
        failures.append(f"verification-owned script remains in scripts/: scripts/{path}")
    for path in sorted(actual):
        if "__pycache__" in Path(path).parts or path.endswith(".pyc"):
            failures.append(f"generated Python bytecode must not live under scripts/: scripts/{path}")
    for path in sorted(ALLOWED_SCRIPT_FILES - actual):
        failures.append(f"allowed scripts/ file is missing: scripts/{path}")
    return failures


def validate_scripts_tree(root: Path) -> list[str]:
    return validate_script_file_set(scripts_files(root))


def iter_reference_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for raw_path in REFERENCE_PATHS:
        path = root / raw_path
        if path.is_file():
            files.append(path)
            continue
        if path.is_dir():
            files.extend(
                candidate
                for candidate in path.rglob("*")
                if candidate.is_file()
                and candidate.relative_to(root).as_posix() != "scripts/check_scripts_verification_boundary.py"
                and ".git" not in candidate.parts
                and "__pycache__" not in candidate.parts
                and candidate.suffix not in {".pyc", ".png", ".webp", ".lock"}
            )
    return sorted(set(files))


def validate_references(root: Path) -> list[str]:
    failures: list[str] = []
    for path in iter_reference_files(root):
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        display = path.relative_to(root)
        for stale, replacement in STALE_REFERENCE_PATTERNS.items():
            if stale in text:
                hint = f"; use `{replacement}`" if replacement else ""
                failures.append(f"{display} references stale path `{stale}`{hint}")
    return failures


def iter_tracked_reference_files(root: Path, raw_paths: list[str]) -> list[Path]:
    completed = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--", *raw_paths],
        check=True,
        text=True,
        capture_output=True,
    )
    return [
        root / line
        for line in completed.stdout.splitlines()
        if line
        and line != "scripts/check_scripts_verification_boundary.py"
        and (root / line).is_file()
    ]


def validate_personal_paths(root: Path) -> list[str]:
    failures: list[str] = []
    for path in iter_tracked_reference_files(root, PERSONAL_PATH_REFERENCE_PATHS):
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        failures.extend(validate_personal_path_text(path.relative_to(root), text))
    return failures


def validate_personal_path_text(display: Path, text: str) -> list[str]:
    failures: list[str] = []
    for stale, replacement in PERSONAL_PATH_PATTERNS.items():
        if stale in text:
            failures.append(f"{display} references personal path `{stale}`; {replacement}")
    return failures


def validate(root: Path) -> list[str]:
    failures = validate_scripts_tree(root)
    failures.extend(validate_references(root))
    failures.extend(validate_personal_paths(root))
    return failures


def run_self_test() -> None:
    missing = validate_script_file_set(ALLOWED_SCRIPT_FILES - {"run_all_tests.sh"})
    if not any("allowed scripts/ file is missing" in failure for failure in missing):
        raise SystemExit("scripts boundary self-test failed: missing allowed script not detected")
    forbidden = validate_script_file_set(ALLOWED_SCRIPT_FILES | {"run_e2e_pass.sh"})
    if not any("verification-owned script remains" in failure for failure in forbidden):
        raise SystemExit("scripts boundary self-test failed: forbidden script not detected")
    unclassified = validate_script_file_set(ALLOWED_SCRIPT_FILES | {"new_verification_gate.sh"})
    if not any("unclassified scripts/ file remains" in failure for failure in unclassified):
        raise SystemExit("scripts boundary self-test failed: unclassified script not detected")

    text = "use scripts/run_verification_hardening.py"
    stale_found = any(stale in text for stale in STALE_REFERENCE_PATTERNS)
    if not stale_found:
        raise SystemExit("scripts boundary self-test failed: stale reference pattern not detected")

    personal_path_failures = validate_personal_path_text(
        Path("example.md"),
        "CPython checkout: `/Users/yaseralnajjar/work/sifr/cpython`",
    )
    if not any("references personal path" in failure for failure in personal_path_failures):
        raise SystemExit("scripts boundary self-test failed: personal path pattern not detected")
    personal_path_clean = validate_personal_path_text(
        Path("example.md"),
        "CPython checkout: `../cpython`",
    )
    if personal_path_clean:
        raise SystemExit("scripts boundary self-test failed: relative path flagged as personal")

    pycache = validate_script_file_set(
        ALLOWED_SCRIPT_FILES
        | {"run_verification_hardening/__pycache__/x.cpython-313.pyc"}
    )
    if not any("generated Python bytecode" in failure for failure in pycache):
        raise SystemExit("scripts boundary self-test failed: pycache path not detected")
    print("scripts verification-boundary self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate(REPO_ROOT)
    if failures:
        print("scripts verification-boundary guardrail: FAIL")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("scripts verification-boundary guardrail: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
