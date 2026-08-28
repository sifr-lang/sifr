#!/usr/bin/env python3
"""Prevent unreviewed growth in complexity, public API, and dependency fan-out."""

from __future__ import annotations

import argparse
import ast
import json
from pathlib import Path
import re
import sys
import tempfile
import tomllib
from typing import Any

from check_file_size_guardrails import (
    SourceFile,
    count_physical_lines,
    has_generated_file_marker,
    iter_source_files,
)
from rust_source_policy import mask_rust_non_code


REPO_ROOT = Path(__file__).resolve().parents[1]
BASELINE_PATH = REPO_ROOT / "verification" / "policy" / "maintainability_ratchets.json"
COMPLEX_FILE_LINES = 600
NEAR_LIMIT_LINES = 800
COMPLEX_FUNCTION_LINES = 80
COMPLEX_FUNCTION_DECISIONS = 20
RUST_FN = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\b[^;{]*\{")
RUST_DECISION = re.compile(
    r"\b(?:if|else|match|for|while|loop)\b|&&|\|\||\?"
)
PUBLIC_ITEM = re.compile(
    r"(?m)^\s*pub\s+(?:(?:async|unsafe|const)\s+)*(?:fn|struct|enum|trait|type|const|static|mod)\b"
)
PUBLIC_GLOB = re.compile(r"(?m)^\s*pub\s+use\s+[^;]*::\s*\*\s*;")
DEAD_CODE_ALLOW = re.compile(
    r"(?m)^\s*#!?\s*\[\s*allow\s*\([^)]*\bdead_code\b[^)]*\)\s*\]"
)
RATCHET_FIELDS = (
    "public_items",
    "public_glob_exports",
    "dead_code_allowances",
    "direct_dependencies",
)


def component_for(rel_path: Path) -> str:
    parts = rel_path.parts
    if len(parts) >= 2 and parts[0] == "crates":
        return f"crates/{parts[1]}"
    return parts[0]


def matching_brace(masked: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(masked)):
        char = masked[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def rust_function_metrics(rel_path: Path, text: str) -> dict[str, dict[str, int]]:
    masked = mask_rust_non_code(text)
    functions: dict[str, dict[str, int]] = {}
    occurrences: dict[str, int] = {}
    for match in RUST_FN.finditer(masked):
        opening = masked.find("{", match.start(), match.end())
        closing = matching_brace(masked, opening)
        if closing is None:
            continue
        name = match.group(1)
        occurrences[name] = occurrences.get(name, 0) + 1
        start_line = masked.count("\n", 0, match.start()) + 1
        end_line = masked.count("\n", 0, closing) + 1
        body = masked[opening : closing + 1]
        key = f"{rel_path.as_posix()}::{name}#{occurrences[name]}@{start_line}"
        functions[key] = {
            "lines": end_line - start_line + 1,
            "decisions": len(RUST_DECISION.findall(body)),
        }
    return functions


def python_function_metrics(rel_path: Path, text: str) -> dict[str, dict[str, int]]:
    try:
        tree = ast.parse(text)
    except SyntaxError:
        return {}
    functions: dict[str, dict[str, int]] = {}
    parents: list[str] = []

    class Visitor(ast.NodeVisitor):
        def visit_ClassDef(self, node: ast.ClassDef) -> None:
            parents.append(node.name)
            self.generic_visit(node)
            parents.pop()

        def visit_FunctionDef(self, node: ast.FunctionDef) -> None:
            self._visit_function(node)

        def visit_AsyncFunctionDef(self, node: ast.AsyncFunctionDef) -> None:
            self._visit_function(node)

        def _visit_function(self, node: ast.FunctionDef | ast.AsyncFunctionDef) -> None:
            qualified = ".".join([*parents, node.name])
            key = f"{rel_path.as_posix()}::{qualified}@{node.lineno}"
            decisions = sum(
                isinstance(
                    child,
                    (
                        ast.If,
                        ast.IfExp,
                        ast.For,
                        ast.AsyncFor,
                        ast.While,
                        ast.Match,
                        ast.Try,
                        ast.BoolOp,
                        ast.comprehension,
                    ),
                )
                for child in ast.walk(node)
            )
            functions[key] = {
                "lines": (node.end_lineno or node.lineno) - node.lineno + 1,
                "decisions": decisions,
            }
            parents.append(node.name)
            self.generic_visit(node)
            parents.pop()

    Visitor().visit(tree)
    return functions


def cargo_dependency_count(path: Path) -> int:
    payload = tomllib.loads(path.read_text(encoding="utf-8"))
    names: set[str] = set()
    for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
        table = payload.get(table_name, {})
        if isinstance(table, dict):
            names.update(str(name) for name in table)
    targets = payload.get("target", {})
    if isinstance(targets, dict):
        for target in targets.values():
            if not isinstance(target, dict):
                continue
            for table_name in ("dependencies", "dev-dependencies", "build-dependencies"):
                table = target.get(table_name, {})
                if isinstance(table, dict):
                    names.update(str(name) for name in table)
    return len(names)


def source_files(root: Path) -> list[SourceFile]:
    return list(iter_source_files(root))


def collect_metrics(root: Path) -> dict[str, Any]:
    components: dict[str, dict[str, int]] = {}
    complex_files: dict[str, int] = {}
    near_limit_files: dict[str, int] = {}
    complex_functions: dict[str, dict[str, int]] = {}

    for source in source_files(root):
        path = root / source.rel_path
        if has_generated_file_marker(path):
            continue
        component = component_for(source.rel_path)
        component_metrics = components.setdefault(
            component,
            {field: 0 for field in RATCHET_FIELDS},
        )
        lines = count_physical_lines(path)
        if lines >= COMPLEX_FILE_LINES:
            complex_files[source.rel_path.as_posix()] = lines
        if lines >= NEAR_LIMIT_LINES:
            near_limit_files[source.rel_path.as_posix()] = lines
        text = path.read_text(encoding="utf-8")
        functions: dict[str, dict[str, int]] = {}
        if source.rel_path.suffix == ".rs":
            functions = rust_function_metrics(source.rel_path, text)
            component_metrics["public_items"] += len(PUBLIC_ITEM.findall(text))
            component_metrics["public_glob_exports"] += len(PUBLIC_GLOB.findall(text))
            component_metrics["dead_code_allowances"] += len(DEAD_CODE_ALLOW.findall(text))
        elif source.rel_path.suffix == ".py":
            functions = python_function_metrics(source.rel_path, text)
        for key, metrics in functions.items():
            if (
                metrics["lines"] >= COMPLEX_FUNCTION_LINES
                or metrics["decisions"] >= COMPLEX_FUNCTION_DECISIONS
            ):
                complex_functions[key] = metrics

    crates_root = root / "crates"
    if crates_root.is_dir():
        for manifest in sorted(crates_root.glob("*/Cargo.toml")):
            component = f"crates/{manifest.parent.name}"
            component_metrics = components.setdefault(
                component,
                {field: 0 for field in RATCHET_FIELDS},
            )
            component_metrics["direct_dependencies"] = cargo_dependency_count(manifest)

    return {
        "schema_version": 1,
        "thresholds": {
            "complex_file_lines": COMPLEX_FILE_LINES,
            "near_limit_lines": NEAR_LIMIT_LINES,
            "complex_function_lines": COMPLEX_FUNCTION_LINES,
            "complex_function_decisions": COMPLEX_FUNCTION_DECISIONS,
        },
        "components": dict(sorted(components.items())),
        "complex_files": dict(sorted(complex_files.items())),
        "near_limit_files": dict(sorted(near_limit_files.items())),
        "complex_functions": dict(sorted(complex_functions.items())),
    }


def compare_limits(
    current: dict[str, Any], baseline: dict[str, Any], label: str
) -> list[str]:
    errors: list[str] = []
    for key, value in current.items():
        if key not in baseline:
            errors.append(f"new {label}: {key}")
            continue
        expected = baseline[key]
        if isinstance(value, dict):
            for metric, actual in value.items():
                limit = expected.get(metric, -1)
                if actual > limit:
                    errors.append(f"{label} grew: {key} {metric}={actual} limit={limit}")
        elif value > expected:
            errors.append(f"{label} grew: {key} lines={value} limit={expected}")
    return errors


def validate(current: dict[str, Any], baseline: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    if baseline.get("schema_version") != 1:
        errors.append("baseline schema_version must be 1")
    if current["thresholds"] != baseline.get("thresholds"):
        errors.append("ratchet thresholds differ from the committed baseline")
    baseline_components = baseline.get("components", {})
    for component, metrics in current["components"].items():
        expected = baseline_components.get(component)
        if expected is None:
            errors.append(f"new maintainability component: {component}")
            continue
        for field in RATCHET_FIELDS:
            if metrics[field] > expected.get(field, -1):
                errors.append(
                    f"{component} {field} grew: {metrics[field]} > {expected.get(field)}"
                )
    errors.extend(compare_limits(current["complex_files"], baseline.get("complex_files", {}), "complex file"))
    errors.extend(compare_limits(current["near_limit_files"], baseline.get("near_limit_files", {}), "near-limit file"))
    errors.extend(
        compare_limits(
            current["complex_functions"],
            baseline.get("complex_functions", {}),
            "complex function",
        )
    )
    return errors


def validate_architecture_authority(root: Path) -> list[str]:
    path = root / "internal_docs" / "architecture.md"
    if not path.is_file():
        return []
    text = path.read_text(encoding="utf-8")
    errors: list[str] = []
    if "**Implementation responsibilities:**" in text:
        errors.append(
            "architecture mixes current rules with legacy implementation responsibilities"
        )
    if "**Future**. When this document conflicts with code or an executable guard" not in text:
        errors.append("architecture is missing its explicit future-authority contract")
    return errors


def validate_flow_graph_decision(root: Path) -> list[str]:
    path = root / "verification" / "policy" / "flow_graph_retention.json"
    if not path.is_file():
        return ["flow-graph retention decision is missing"]
    payload = json.loads(path.read_text(encoding="utf-8"))
    errors: list[str] = []
    if payload.get("schema_version") != 1 or payload.get("decision") != "keep-snapshot-artifact":
        errors.append("flow-graph retention decision has an unsupported shape")
    for consumer in payload.get("production_consumers", []):
        if not root.joinpath(str(consumer)).is_file():
            errors.append(f"flow-graph consumer evidence is missing: {consumer}")
    documentation = root / str(payload.get("documentation", ""))
    if not documentation.is_file() or "Decision: **keep the graph" not in documentation.read_text(
        encoding="utf-8"
    ):
        errors.append("flow-graph retention documentation is missing its keep decision")
    return errors


def write_baseline(metrics: dict[str, Any]) -> None:
    BASELINE_PATH.write_text(
        json.dumps(metrics, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )


def run_self_test() -> int:
    with tempfile.TemporaryDirectory(prefix="maintainability-ratchets-") as temp:
        root = Path(temp)
        crate = root / "crates" / "probe"
        source = crate / "src" / "lib.rs"
        source.parent.mkdir(parents=True)
        (crate / "Cargo.toml").write_text(
            '[package]\nname="probe"\nversion="0.1.0"\nedition="2024"\n',
            encoding="utf-8",
        )
        source.write_text("pub fn stable() {}\n", encoding="utf-8")
        baseline = collect_metrics(root)

        source.write_text("pub fn stable() {}\npub struct Added;\n", encoding="utf-8")
        if not any("public_items grew" in error for error in validate(collect_metrics(root), baseline)):
            raise AssertionError("public API mutation escaped the ratchet")

        source.write_text(
            "pub fn stable() {}\n" + "fn complex() {\n" + "if true {}\n" * 79 + "}\n",
            encoding="utf-8",
        )
        if not any("new complex function" in error for error in validate(collect_metrics(root), baseline)):
            raise AssertionError("function-complexity mutation escaped the ratchet")

        source.write_text("pub fn stable() {}\n" + "// line\n" * 800, encoding="utf-8")
        errors = validate(collect_metrics(root), baseline)
        if not any("new complex file" in error for error in errors) or not any(
            "new near-limit file" in error for error in errors
        ):
            raise AssertionError("source-concentration mutation escaped the ratchet")

        source.write_text("pub fn stable() {}\n", encoding="utf-8")
        (crate / "Cargo.toml").write_text(
            '[package]\nname="probe"\nversion="0.1.0"\nedition="2024"\n'
            '[dependencies]\nserde="1"\n',
            encoding="utf-8",
        )
        if not any("direct_dependencies grew" in error for error in validate(collect_metrics(root), baseline)):
            raise AssertionError("dependency fan-out mutation escaped the ratchet")

        architecture = root / "internal_docs" / "architecture.md"
        architecture.parent.mkdir(parents=True)
        architecture.write_text(
            "**Future**. When this document conflicts with code or an executable guard\n",
            encoding="utf-8",
        )
        if validate_architecture_authority(root):
            raise AssertionError("valid architecture authority contract was rejected")
        architecture.write_text(
            "**Future**. When this document conflicts with code or an executable guard\n"
            "**Implementation responsibilities:**\n",
            encoding="utf-8",
        )
        if not validate_architecture_authority(root):
            raise AssertionError("mixed architecture authority escaped the ratchet")

    print("maintainability ratchet self-test: PASS")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--write-baseline", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return run_self_test()
    current = collect_metrics(REPO_ROOT)
    if args.write_baseline:
        write_baseline(current)
        print(f"wrote maintainability baseline: {BASELINE_PATH.relative_to(REPO_ROOT)}")
        return 0
    baseline = json.loads(BASELINE_PATH.read_text(encoding="utf-8"))
    errors = validate(current, baseline)
    errors.extend(validate_architecture_authority(REPO_ROOT))
    errors.extend(validate_flow_graph_decision(REPO_ROOT))
    if errors:
        for error in errors:
            print(f"maintainability ratchet error: {error}", file=sys.stderr)
        return 1
    print(
        "maintainability ratchets: PASS "
        f"({len(current['components'])} components, "
        f"{len(current['complex_functions'])} complex functions, "
        f"{len(current['near_limit_files'])} near-limit files)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
