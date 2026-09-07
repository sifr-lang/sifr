"""Source-emission and negative-seed checks for generated Rust quality."""

from __future__ import annotations

import re
import shutil
from pathlib import Path
from typing import Any, Callable, Sequence

from quality_policy import PATTERN_POLICIES

RUST_RAW_STRING_RE = re.compile(r'r(?P<hashes>#*)"(?P<body>.*?)"(?P=hashes)')
RUST_NORMAL_STRING_RE = re.compile(r'"(?P<body>(?:\\.|[^"\\])*)"')
GENERATED_SOURCE_CONTEXT_RE = re.compile(
    r"\b(format!|emit_line|RustExpr::Ident|RustType::Named|RustLiteral::Str|push_str)\b"
)
SOURCE_FORBIDDEN_POLICY_IDS = {
    "allow-attribute",
    "expect",
    "panic",
    "todo",
    "unimplemented",
    "unsafe",
    "unwrap",
}


def codegen_source_files(repo_root: Path) -> list[Path]:
    src_root = repo_root / "crates" / "sifr_codegen" / "src"
    return sorted(
        path
        for path in src_root.rglob("*.rs")
        if path.name not in {"tests.rs"}
        and not path.name.endswith("_tests.rs")
        and "tests" not in path.relative_to(src_root).parts
    )


def rust_string_literals(line: str) -> list[str]:
    literals: list[str] = []
    raw_ranges: list[tuple[int, int]] = []
    for match in RUST_RAW_STRING_RE.finditer(line):
        literals.append(match.group("body"))
        raw_ranges.append(match.span())
    for match in RUST_NORMAL_STRING_RE.finditer(line):
        if any(start <= match.start() < end for start, end in raw_ranges):
            continue
        literals.append(match.group("body"))
    return literals


def scan_codegen_source_emissions(repo_root: Path) -> list[str]:
    violations: list[str] = []
    for path in codegen_source_files(repo_root):
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if not GENERATED_SOURCE_CONTEXT_RE.search(line):
                continue
            for literal in rust_string_literals(line):
                for policy in PATTERN_POLICIES:
                    if policy.id not in SOURCE_FORBIDDEN_POLICY_IDS:
                        continue
                    if policy.pattern.search(literal):
                        violations.append(
                            f"{path}:{line_number}: emitted source contains {policy.id}"
                        )
    return violations


def gate_intrinsic_panic_lint(
    repo_root: Path,
    timed_case: Callable[..., Any],
) -> None:
    def check_intrinsic_layout() -> None:
        codegen_lib = repo_root / "crates" / "sifr_codegen" / "src" / "lib.rs"
        source = codegen_lib.read_text(encoding="utf-8")
        forbidden = ("fn emit_intrinsic_call(", "pub(crate) fn emit_intrinsic_call(")
        for marker in forbidden:
            if marker in source:
                raise RuntimeError(f"retired intrinsic emitter monolith returned: {marker}")
        source_violations = scan_codegen_source_emissions(repo_root)
        if source_violations:
            raise RuntimeError(
                "\n".join(
                    ["forbidden emitted constructs in codegen source", *source_violations]
                )
            )

    timed_case(
        "generated_code_quality",
        "intrinsic-panic-lint/no-retired-emit-intrinsic-call",
        check_intrinsic_layout,
    )
    print("generated-code intrinsic panic lint passed")


def assert_negative_rustfmt(
    seed: Path,
    run_root: Path,
    run_command: Callable[..., Any],
) -> None:
    run_root.mkdir(parents=True, exist_ok=True)
    target = run_root / "negative-format.rs"
    shutil.copyfile(seed, target)
    result = run_command(["rustfmt", "--check", str(target)], check=False)
    if result.returncode == 0:
        raise RuntimeError("negative rustfmt seed unexpectedly passed")


def assert_negative_clippy(
    seed: Path,
    run_root: Path,
    expected_lint: str,
    run_command: Callable[..., Any],
    strict_clippy_args: Sequence[str],
    parse_diagnostics: Callable[[str, Path], dict[str, Any]],
    cargo_target_dir: Path,
) -> None:
    crate_root = run_root / "negative-clippy"
    src = crate_root / "src"
    src.mkdir(parents=True, exist_ok=True)
    (crate_root / "Cargo.toml").write_text(
        '[package]\nname = "negative_clippy"\nversion = "0.1.0"\nedition = "2024"\n\n[workspace]\n',
        encoding="utf-8",
    )
    shutil.copyfile(seed, src / "main.rs")
    result = run_command(
        [
            "cargo",
            "clippy",
            "--message-format=json",
            "--manifest-path",
            str(crate_root / "Cargo.toml"),
            "--",
            *strict_clippy_args,
        ],
        check=False,
        cargo_target_dir=cargo_target_dir,
    )
    if result.returncode == 0:
        raise RuntimeError("negative clippy seed unexpectedly passed")
    diagnostics = parse_diagnostics(result.stdout, crate_root)
    if expected_lint not in diagnostics:
        raise RuntimeError(
            f"negative Clippy seed {seed.name} did not trigger {expected_lint}; "
            f"got {diagnostics}"
        )


def run_strict_clippy(
    crate_root: Path,
    run_command: Callable[..., Any],
    strict_clippy_args: Sequence[str],
    cargo_target_dir: Path,
) -> Any:
    manifest = crate_root / "Cargo.toml"
    # Each gate invocation owns a fresh target. Cargo therefore emits the same
    # diagnostics on every run without cleaning artifacts from another process
    # that happens to share the materialization cache.
    return run_command(
        [
            "cargo",
            "clippy",
            "--message-format=json",
            "--manifest-path",
            str(manifest),
            "--locked",
            "--",
            *strict_clippy_args,
        ],
        check=False,
        cargo_target_dir=cargo_target_dir,
    )


def assert_negative_determinism(a: Path, b: Path) -> None:
    if compare_bytes(a.read_bytes(), b.read_bytes()):
        raise RuntimeError("negative determinism seed unexpectedly matched")


def compare_bytes(left: bytes, right: bytes) -> bool:
    return left == right
