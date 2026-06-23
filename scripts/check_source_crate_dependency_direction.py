#!/usr/bin/env python3
"""Guard source, IR, stdlib, and lowering-boundary dependency direction."""

from __future__ import annotations

import argparse
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
import re
import sys
import tempfile
import tomllib


REPO_ROOT = Path(__file__).resolve().parents[1]
SELF_TEST_TMP_ROOT = REPO_ROOT / "target"

NORMAL_DEP_SECTIONS = ("dependencies", "build-dependencies")
TARGET_DEP_SECTIONS = ("dependencies", "build-dependencies")

ALL_SIFR_CRATES = {
    "sifr",
    "sifr_analysis",
    "sifr_codegen",
    "sifr_diagnostics",
    "sifr_driver",
    "sifr_format",
    "sifr_frontend",
    "sifr_ir",
    "sifr_lint",
    "sifr_lowering",
    "sifr_lsp",
    "sifr_package",
    "sifr_runtime",
    "sifr_source",
    "sifr_stdlib",
    "sifr_stdlib_model",
    "sifr_sysroot",
    "sifr_syntax",
    "sifr_type_system",
}

IR_FORBIDDEN_DEPENDENCIES = {
    "sifr",
    "sifr_analysis",
    "sifr_codegen",
    "sifr_driver",
    "sifr_frontend",
    "sifr_lint",
    "sifr_lowering",
    "sifr_lsp",
    "sifr_package",
    "sifr_stdlib",
    "sifr_stdlib_model",
    "sifr_syntax",
}

STDLIB_FORBIDDEN_DEPENDENCIES = {
    "sifr",
    "sifr_analysis",
    "sifr_codegen",
    "sifr_driver",
    "sifr_frontend",
    "sifr_lint",
    "sifr_lowering",
    "sifr_lsp",
    "sifr_package",
    "sifr_stdlib_model",
}

GENERATED_STDLIB_FORBIDDEN_DEPENDENCIES = {
    "sifr",
    "sifr_analysis",
    "sifr_codegen",
    "sifr_driver",
    "sifr_frontend",
    "sifr_ir",
    "sifr_lint",
    "sifr_lowering",
    "sifr_lsp",
    "sifr_package",
    "sifr_stdlib_model",
    "sifr_syntax",
    "sifr_sysroot",
    "sifr_type_system",
}

PARSER_CRATES = {
    "sifr_python_ast",
    "sifr_python_parser",
}

GENERATED_DEPENDENCY_SPEC_DEFINITION_PATTERNS = (
    re.compile(r"\b(?:pub\s+)?struct\s+GeneratedCargoDependency\b"),
    re.compile(r"\b(?:pub\s+)?struct\s+StdlibFeatureSpec\b"),
    re.compile(r"\b(?:pub\s+)?const\s+STDLIB_FEATURE_SPECS\b"),
    re.compile(r"\bconst\s+[A-Z0-9_]+_DEPS\s*:\s*&\s*\[GeneratedCargoDependency\]"),
    re.compile(r"\bfn\s+generated_cargo_dependencies\b"),
    re.compile(r"\bfn\s+render_dependency_spec\b"),
)


@dataclass(frozen=True)
class CrateRule:
    crate: str
    allowed_normal_dependencies: frozenset[str] | None = None
    forbidden_normal_dependencies: frozenset[str] = frozenset()
    forbidden_source_references: frozenset[str] = frozenset()
    skip_test_sources: bool = False


RULES = (
    CrateRule(
        crate="sifr_source",
        allowed_normal_dependencies=frozenset({"ruff_text_size"}),
        forbidden_source_references=frozenset(ALL_SIFR_CRATES - {"sifr_source"}),
    ),
    CrateRule(
        crate="sifr_ir",
        forbidden_normal_dependencies=frozenset(
            IR_FORBIDDEN_DEPENDENCIES | PARSER_CRATES
        ),
        forbidden_source_references=frozenset(IR_FORBIDDEN_DEPENDENCIES | PARSER_CRATES),
    ),
    CrateRule(
        crate="sifr_stdlib_model",
        forbidden_normal_dependencies=frozenset(
            STDLIB_FORBIDDEN_DEPENDENCIES | PARSER_CRATES | {"sifr_syntax"}
        ),
        forbidden_source_references=frozenset(
            STDLIB_FORBIDDEN_DEPENDENCIES | PARSER_CRATES | {"sifr_syntax"}
        ),
    ),
    CrateRule(
        crate="sifr_stdlib",
        forbidden_normal_dependencies=frozenset(
            GENERATED_STDLIB_FORBIDDEN_DEPENDENCIES | PARSER_CRATES
        ),
        forbidden_source_references=frozenset(
            GENERATED_STDLIB_FORBIDDEN_DEPENDENCIES | PARSER_CRATES
        ),
    ),
    CrateRule(
        crate="sifr_codegen",
        forbidden_normal_dependencies=frozenset({"sifr_lowering"}),
        forbidden_source_references=frozenset({"sifr_lowering"}),
        skip_test_sources=True,
    ),
    CrateRule(
        crate="sifr_lint",
        forbidden_normal_dependencies=frozenset({"sifr_lowering"}),
        forbidden_source_references=frozenset({"sifr_lowering"}),
        skip_test_sources=True,
    ),
    CrateRule(
        crate="sifr_analysis",
        forbidden_normal_dependencies=frozenset({"sifr_lowering"}),
        forbidden_source_references=frozenset({"sifr_lowering"}),
        skip_test_sources=True,
    ),
)


def load_manifest(root: Path, crate: str) -> dict:
    manifest_path = root / "crates" / crate / "Cargo.toml"
    if not manifest_path.exists():
        return {}
    return tomllib.loads(manifest_path.read_text(encoding="utf-8"))


def normal_dependencies(manifest: dict) -> set[str]:
    dependencies: set[str] = set()
    for section in NORMAL_DEP_SECTIONS:
        dependencies.update(manifest.get(section, {}))
    for target in manifest.get("target", {}).values():
        for section in TARGET_DEP_SECTIONS:
            dependencies.update(target.get(section, {}))
    return dependencies


def is_test_source(path: Path) -> bool:
    name = path.name
    return (
        "tests" in path.parts
        or name == "lib_codegen_tests.rs"
        or name.endswith("_tests.rs")
        or name == "tests.rs"
    )


def crate_source_files(root: Path, crate: str, *, skip_test_sources: bool) -> list[Path]:
    src = root / "crates" / crate / "src"
    if not src.exists():
        return []
    files = sorted(src.rglob("*.rs"))
    if skip_test_sources:
        files = [path for path in files if not is_test_source(path.relative_to(src))]
    return files


def references_crate(text: str, crate: str) -> bool:
    return re.search(rf"\buse\s+{re.escape(crate)}\b", text) is not None or re.search(
        rf"\b{re.escape(crate)}::", text
    ) is not None


def validate_crate_rule(root: Path, rule: CrateRule) -> list[str]:
    failures: list[str] = []
    manifest = load_manifest(root, rule.crate)
    if not manifest:
        failures.append(f"{rule.crate}: missing Cargo.toml")
        return failures

    dependencies = normal_dependencies(manifest)
    if rule.allowed_normal_dependencies is not None:
        unexpected = sorted(dependencies - rule.allowed_normal_dependencies)
        if unexpected:
            failures.append(
                f"{rule.crate}: unexpected normal dependency/dependencies: "
                + ", ".join(unexpected)
            )

    forbidden_dependencies = sorted(dependencies & rule.forbidden_normal_dependencies)
    if forbidden_dependencies:
        failures.append(
            f"{rule.crate}: forbidden normal dependency/dependencies: "
            + ", ".join(forbidden_dependencies)
        )

    for path in crate_source_files(
        root, rule.crate, skip_test_sources=rule.skip_test_sources
    ):
        text = path.read_text(encoding="utf-8", errors="replace")
        for crate in sorted(rule.forbidden_source_references):
            if references_crate(text, crate):
                rel = path.relative_to(root)
                failures.append(f"{rule.crate}: {rel} references {crate}")
    return failures


def generated_dependency_spec_violations(root: Path) -> list[str]:
    failures: list[str] = []
    crates_root = root / "crates"
    for path in sorted(crates_root.rglob("*.rs")):
        rel = path.relative_to(root)
        if rel.parts[:2] == ("crates", "sifr_stdlib_model"):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern in GENERATED_DEPENDENCY_SPEC_DEFINITION_PATTERNS:
            if pattern.search(text):
                failures.append(
                    f"{rel} defines generated dependency spec pattern {pattern.pattern!r}"
                )
                break
    return failures


def validate(root: Path) -> list[str]:
    failures: list[str] = []
    for rule in RULES:
        failures.extend(validate_crate_rule(root, rule))
    failures.extend(generated_dependency_spec_violations(root))
    return failures


def write_manifest(crate_root: Path, name: str, deps: list[str] | None = None) -> None:
    crate_root.mkdir(parents=True, exist_ok=True)
    dependency_lines = ""
    for dependency in deps or []:
        dependency_lines += f'{dependency} = {{ path = "../{dependency}" }}\n'
    crate_root.joinpath("Cargo.toml").write_text(
        f"""[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies]
{dependency_lines}""",
        encoding="utf-8",
    )
    src = crate_root / "src"
    src.mkdir(exist_ok=True)
    src.joinpath("lib.rs").write_text("", encoding="utf-8")


def seed_valid_repo(root: Path) -> None:
    allowed_deps = {
        "sifr_source": ["ruff_text_size"],
        "sifr_ir": ["sifr_diagnostics", "sifr_type_system"],
        "sifr_stdlib": ["sifr_runtime"],
        "sifr_stdlib_model": ["sifr_type_system"],
        "sifr_codegen": ["sifr_ir", "sifr_stdlib_model"],
        "sifr_lint": ["sifr_frontend", "sifr_ir"],
        "sifr_analysis": ["sifr_frontend", "sifr_lint"],
    }
    for crate in ALL_SIFR_CRATES | {"ruff_text_size"}:
        write_manifest(root / "crates" / crate, crate, allowed_deps.get(crate, []))
    stdlib_src = root / "crates" / "sifr_stdlib_model" / "src" / "features.rs"
    stdlib_src.write_text("pub struct GeneratedCargoDependency;\n", encoding="utf-8")
    codegen_src = root / "crates" / "sifr_codegen" / "src"
    codegen_src.joinpath("lib.rs").write_text(
        "pub fn read_specs() { let _ = sifr_stdlib_model::STDLIB_FEATURE_SPECS; }\n",
        encoding="utf-8",
    )
    codegen_src.joinpath("lib_codegen_tests.rs").write_text(
        "use sifr_lowering::lower_module;\n",
        encoding="utf-8",
    )


def assert_self_test_case(
    label: str, mutate: Callable[[Path], object], expected_fragment: str, failures: list[str]
) -> None:
    SELF_TEST_TMP_ROOT.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=SELF_TEST_TMP_ROOT) as tmp:
        root = Path(tmp)
        seed_valid_repo(root)
        mutate(root)
        found = validate(root)
    if not any(expected_fragment in failure for failure in found):
        failures.append(f"{label}: seeded violation was not detected; found={found!r}")


def run_self_test() -> int:
    failures: list[str] = []
    SELF_TEST_TMP_ROOT.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=SELF_TEST_TMP_ROOT) as tmp:
        root = Path(tmp)
        seed_valid_repo(root)
        found = validate(root)
    if found:
        failures.append(f"positive fixture unexpectedly failed: {found!r}")

    assert_self_test_case(
        "sifr_source upward dependency",
        lambda root: write_manifest(
            root / "crates" / "sifr_source", "sifr_source", ["sifr_diagnostics"]
        ),
        "sifr_source: unexpected normal dependency",
        failures,
    )
    assert_self_test_case(
        "sifr_ir parser dependency",
        lambda root: write_manifest(
            root / "crates" / "sifr_ir", "sifr_ir", ["sifr_python_parser"]
        ),
        "sifr_ir: forbidden normal dependency",
        failures,
    )
    assert_self_test_case(
        "sifr_ir source reference",
        lambda root: (
            root / "crates" / "sifr_ir" / "src" / "lib.rs"
        ).write_text("use sifr_lowering::LoweringResult;\n", encoding="utf-8"),
        "sifr_ir: crates/sifr_ir/src/lib.rs references sifr_lowering",
        failures,
    )
    assert_self_test_case(
        "sifr_stdlib_model codegen dependency",
        lambda root: write_manifest(
            root / "crates" / "sifr_stdlib_model", "sifr_stdlib_model", ["sifr_codegen"]
        ),
        "sifr_stdlib_model: forbidden normal dependency",
        failures,
    )
    assert_self_test_case(
        "sifr_codegen lowering dependency",
        lambda root: write_manifest(
            root / "crates" / "sifr_codegen", "sifr_codegen", ["sifr_lowering"]
        ),
        "sifr_codegen: forbidden normal dependency",
        failures,
    )
    assert_self_test_case(
        "sifr_codegen production source reference",
        lambda root: (
            root / "crates" / "sifr_codegen" / "src" / "lib.rs"
        ).write_text("pub fn leak() { sifr_lowering::lower_module(); }\n", encoding="utf-8"),
        "sifr_codegen: crates/sifr_codegen/src/lib.rs references sifr_lowering",
        failures,
    )
    assert_self_test_case(
        "sifr_lint lowering dependency",
        lambda root: write_manifest(
            root / "crates" / "sifr_lint", "sifr_lint", ["sifr_lowering"]
        ),
        "sifr_lint: forbidden normal dependency",
        failures,
    )
    assert_self_test_case(
        "sifr_analysis lowering dependency",
        lambda root: write_manifest(
            root / "crates" / "sifr_analysis", "sifr_analysis", ["sifr_lowering"]
        ),
        "sifr_analysis: forbidden normal dependency",
        failures,
    )
    assert_self_test_case(
        "generated dependency spec outside stdlib",
        lambda root: (
            root
            / "crates"
            / "sifr_codegen"
            / "src"
            / "lib.rs"
        ).write_text("pub struct GeneratedCargoDependency;\n", encoding="utf-8"),
        "generated dependency spec pattern",
        failures,
    )

    if failures:
        print("source crate dependency-direction self-test: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("source crate dependency-direction self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()

    failures = validate(REPO_ROOT)
    if failures:
        print("source crate dependency-direction guardrail: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("source crate dependency-direction guardrail: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
