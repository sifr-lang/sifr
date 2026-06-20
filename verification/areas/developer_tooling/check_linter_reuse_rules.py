#!/usr/bin/env python3
"""Validate the production-grade Sifr linter reuse rules."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
MANIFEST_DIR = REPO_ROOT / "verification" / "areas" / "developer_tooling" / "linter_manifests"
RULE_CONFIG_MANIFEST = MANIFEST_DIR / "ruff_rule_config_audit.json"
CLI_MANIFEST = MANIFEST_DIR / "lint_cli_parity.json"
RULE_METADATA_MANIFEST = MANIFEST_DIR / "lint_rule_metadata.json"
CONFIG_SCHEMA_PLACEHOLDER = MANIFEST_DIR / "lint_config_schema_placeholder.json"
SUPPRESSION_GATE_MANIFEST = MANIFEST_DIR / "suppression_gate.json"

ALLOWED_DISPOSITIONS = {"adapt", "sifr-native", "formatter-owned", "future-rules", "reject"}
ALLOWED_CONFIG_DISPOSITIONS = {"adapt", "sifr-native"}
REJECTED_CONFIG_DISPOSITIONS = {"reject", "formatter-owned", "future-rules"}
ALLOWED_SUPPRESSION_COMPLEXITIES = {
    "physical-line",
    "single-node",
    "statement-range",
    "symbol-workspace",
}
FORBIDDEN_DEPENDENCY_NAMES = {
    "ruff_linter",
    "ruff_python_semantic",
    "ruff_server",
    "ty_python_semantic",
    "ty_project",
    "ty_python_stdlib",
}
FORBIDDEN_SOURCE_PATTERNS = {
    "ruff_linter::rules": "Ruff Python rules cannot be production Sifr lint rules",
    "ruff_linter::registry": "Ruff's Python rule registry cannot be Sifr lint authority",
    "ruff_linter::linter": "Ruff's Python lint runner cannot be Sifr lint authority",
    "ruff_linter::noqa": "Ruff noqa mapping cannot be Sifr suppression authority",
    "ruff_python_semantic": "Python semantic analysis cannot be Sifr lint authority",
    "ty_python_semantic": "Python semantic analysis cannot be Sifr lint authority",
    "ty_project": "Python project/module semantics cannot be Sifr lint authority",
    "ruff_server": "Ruff Server behavior cannot be Sifr LSP semantic authority",
}
SCAN_ROOTS = [
    REPO_ROOT / "crates" / "sifr_lint",
    REPO_ROOT / "crates" / "sifr_analysis",
    REPO_ROOT / "crates" / "sifr_lsp",
    REPO_ROOT / "crates" / "sifr",
]
IMPLEMENTED_LINT_OPTION_ALIASES = {
    "path": "FILES",
    "paths": "FILES",
    "files": "FILES",
    "targets": "FILES",
    "config": "--config",
    "isolated": "--isolated",
}


class RulesError(RuntimeError):
    pass


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise RulesError(f"missing manifest: {path.relative_to(REPO_ROOT)}") from exc
    except json.JSONDecodeError as exc:
        raise RulesError(f"invalid JSON in {path.relative_to(REPO_ROOT)}: {exc}") from exc


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RulesError(message)


def relative(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)


def run(command: list[str], *, cwd: Path = REPO_ROOT) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(command, cwd=cwd, text=True, capture_output=True)
    if completed.returncode != 0:
        raise RulesError(
            f"command failed: {' '.join(command)}\n{completed.stdout}\n{completed.stderr}"
        )
    return completed


def validate_rule_config_manifest(manifest: dict[str, Any]) -> None:
    require(manifest.get("schema") == 1, "ruff rule/config manifest schema must be 1")
    source = Path(manifest.get("rule_family_source", ""))
    require(source.exists(), f"Ruff rule-family source does not exist: {source}")
    actual = sorted(path.name for path in source.iterdir() if path.is_dir())
    families = manifest.get("rule_families")
    require(isinstance(families, list) and families, "rule_families must be a non-empty array")
    directories = [family.get("directory") for family in families]
    require(len(directories) == len(set(directories)), "rule family directories must be unique")
    require(sorted(directories) == actual, "Ruff rule-family manifest does not match actual fork directories")
    for family in families:
        disposition = family.get("disposition")
        require(disposition in ALLOWED_DISPOSITIONS, f"invalid rule-family disposition: {family}")
        require(family.get("name") == family.get("directory"), f"rule family name/directory mismatch: {family}")
        require(family.get("rationale"), f"missing rule-family rationale: {family}")
        require(family.get("sifr_requirement_note"), f"missing rule-family Sifr requirement: {family}")

    pin = manifest.get("ruff_fork_pin")
    actual_pin = run(["git", "-C", str(source.parents[3]), "rev-parse", "HEAD"]).stdout.strip()
    require(pin == actual_pin, f"Ruff fork pin mismatch: manifest {pin}, actual {actual_pin}")

    surfaces = manifest.get("config_surfaces")
    require(isinstance(surfaces, list) and surfaces, "config_surfaces must be a non-empty array")
    keys = [surface.get("key") for surface in surfaces]
    require(len(keys) == len(set(keys)), "config surface keys must be unique")
    surface_by_key = {surface["key"]: surface for surface in surfaces}
    for surface in surfaces:
        disposition = surface.get("disposition")
        require(disposition in ALLOWED_DISPOSITIONS, f"invalid config disposition: {surface}")
        require(surface.get("kind") in {"config", "cli", "comment-directive", "plugin-block"}, f"invalid config kind: {surface}")
        require(surface.get("rationale"), f"missing config rationale: {surface}")
        require(surface.get("sifr_requirement_note"), f"missing config Sifr requirement: {surface}")

    accepted_keys = manifest.get("accepted_sifr_config_keys")
    require(isinstance(accepted_keys, list), "accepted_sifr_config_keys must be an array")
    for key in accepted_keys:
        require(key in surface_by_key, f"accepted Sifr config key missing from audit: {key}")
        disposition = surface_by_key[key]["disposition"]
        require(
            disposition in ALLOWED_CONFIG_DISPOSITIONS,
            f"accepted Sifr config key {key!r} has disallowed disposition {disposition!r}",
        )

    rejected_keys = manifest.get("rejected_ruff_config_keys")
    require(isinstance(rejected_keys, list), "rejected_ruff_config_keys must be an array")
    rejected_set = set(rejected_keys)
    required_rejected = {
        key
        for key, surface in surface_by_key.items()
        if surface["disposition"] in REJECTED_CONFIG_DISPOSITIONS
    }
    missing = sorted(required_rejected - rejected_set)
    require(not missing, f"rejected_ruff_config_keys missing rejected/future/formatter keys: {missing}")


def validate_cli_manifest(manifest: dict[str, Any]) -> None:
    require(manifest.get("schema") == 1, "lint CLI manifest schema must be 1")
    surfaces = manifest.get("surfaces")
    require(isinstance(surfaces, list) and surfaces, "CLI surfaces must be a non-empty array")
    names = [surface.get("ruff_surface") for surface in surfaces]
    require(len(names) == len(set(names)), "Ruff CLI surfaces must be unique")
    by_spelling: dict[str, dict[str, Any]] = {}
    for surface in surfaces:
        spelling = surface.get("sifr_spelling")
        if not spelling:
            continue
        for expanded in expand_sifr_spellings(spelling):
            by_spelling[expanded] = surface
    for surface in surfaces:
        disposition = surface.get("disposition")
        require(disposition in ALLOWED_DISPOSITIONS, f"invalid CLI disposition: {surface}")
        require(surface.get("implementation_requirement"), f"missing CLI implementation rule: {surface}")
        require(surface.get("fixture"), f"missing CLI fixture id: {surface}")
        require(isinstance(surface.get("conflicts_with"), list), f"conflicts_with must be an array: {surface}")

    output_formats = manifest.get("output_formats")
    require(isinstance(output_formats, list) and output_formats, "output_formats must be a non-empty array")
    for output_format in output_formats:
        require(output_format.get("disposition") in ALLOWED_DISPOSITIONS, f"invalid output-format disposition: {output_format}")
        require(output_format.get("schema_requirement"), f"missing output-format schema rule: {output_format}")
        require(output_format.get("fixture"), f"missing output-format fixture: {output_format}")

    exit_codes = manifest.get("exit_codes")
    require(isinstance(exit_codes, list), "exit_codes must be an array")
    require({entry.get("code") for entry in exit_codes} == {0, 1, 2, 3}, "exit-code manifest must cover 0, 1, 2, and 3")
    for entry in exit_codes:
        require(entry.get("condition"), f"missing exit-code condition: {entry}")
        require(entry.get("fixture"), f"missing exit-code fixture: {entry}")

    rejected = manifest.get("rejected_surfaces")
    require(isinstance(rejected, list), "rejected_surfaces must be an array")
    for spelling in rejected:
        require(spelling.startswith("--"), f"rejected CLI surface must be an option spelling: {spelling}")

    implemented = implemented_lint_options()
    missing = sorted(option for option in implemented if option not in by_spelling)
    require(not missing, f"implemented sifr lint options missing from CLI parity manifest: {missing}")
    exposed_rejected = sorted(option for option in implemented if option in rejected)
    require(not exposed_rejected, f"rejected Ruff CLI surfaces are exposed by sifr lint: {exposed_rejected}")


def implemented_lint_options() -> set[str]:
    cli_source = (REPO_ROOT / "crates" / "sifr" / "src" / "cli_model_and_entrypoint.rs").read_text(encoding="utf-8")
    options: set[str] = {"--config", "--isolated"}
    start = cli_source.find("Lint {")
    end = cli_source.find("Lsp {", start)
    if start != -1 and end != -1:
        lint_block = cli_source[start:end]
    else:
        lint_block = (REPO_ROOT / "crates" / "sifr" / "src" / "lint_cli.rs").read_text(encoding="utf-8")
    for match in re.finditer(
        r"#\[arg\(([^\]]*)\)\]\s*pub\(crate\)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*:",
        lint_block,
        flags=re.MULTILINE,
    ):
        attr, field = match.groups()
        if "long" in attr:
            explicit = re.search(r'long\s*=\s*"([^"]+)"', attr)
            options.add(f"--{explicit.group(1) if explicit else field.replace('_', '-')}")
        alias = IMPLEMENTED_LINT_OPTION_ALIASES.get(field)
        if alias:
            options.add(alias)
    return options


def expand_sifr_spellings(spelling: str) -> list[str]:
    if "/" not in spelling:
        return [spelling]
    expanded: list[str] = []
    for part in spelling.split("/"):
        part = part.strip()
        if part.startswith("--"):
            expanded.append(part)
    return expanded or [spelling]


def validate_rule_metadata(manifest: dict[str, Any]) -> None:
    require(manifest.get("schema") == 1, "rule metadata manifest schema must be 1")
    rules = manifest.get("rules")
    require(isinstance(rules, list) and rules, "rule metadata manifest must contain rules")
    ids = [rule.get("id") for rule in rules]
    require(len(ids) == len(set(ids)), "rule IDs must be unique")
    lib = (REPO_ROOT / "crates" / "sifr_lint" / "src" / "lib.rs").read_text(encoding="utf-8")
    rust_ids = sorted(set(re.findall(r'id:\s*"([^"]+)"', lib)))
    require(sorted(ids) == rust_ids, f"rule metadata manifest does not match sifr_lint RULES: manifest={sorted(ids)} rust={rust_ids}")
    for rule in rules:
        require(rule.get("docs_url", "").startswith("https://docs.sifr.sh/errors/"), f"rule docs URL must be Sifr-owned: {rule}")
        require(rule.get("default_severity") in {"ignore", "warn", "error"}, f"invalid rule severity: {rule}")
        require(rule.get("status") in {"stable", "experimental", "deprecated"}, f"invalid rule status: {rule}")
        require(rule.get("category"), f"missing rule category: {rule}")
        require(rule.get("fix_availability") in {"none", "safe", "unsafe", "manual"}, f"invalid fix availability: {rule}")
        complexity = rule.get("suppression_complexity")
        require(complexity in ALLOWED_SUPPRESSION_COMPLEXITIES, f"invalid suppression complexity: {rule}")


def validate_config_schema_placeholder(manifest: dict[str, Any]) -> None:
    require(manifest.get("schema") == 1, "lint config schema placeholder schema must be 1")
    require(manifest.get("authority") == "sifr.toml", "lint config authority must be sifr.toml")
    require(
        manifest.get("state") in {"placeholder-unimplemented", "implemented-lint-config"},
        "lint config schema state must be a known rules state",
    )


def validate_suppression_gate(manifest: dict[str, Any], rule_metadata: dict[str, Any]) -> None:
    require(manifest.get("schema") == 1, "suppression gate schema must be 1")
    state = manifest.get("gate_state")
    require(state in {"physical_line_only", "parser_aware"}, f"invalid suppression gate state: {state}")
    allowed = manifest.get("allowed_rule_families")
    require(isinstance(allowed, list) and allowed, "allowed_rule_families must be a non-empty array")
    require(set(allowed).issubset(ALLOWED_SUPPRESSION_COMPLEXITIES), "suppression gate allowed families contain invalid values")
    parser_api = manifest.get("parser_aware_api")
    require(parser_api == "sifr_lint::suppression::ParserAwareSuppressions", "unexpected parser-aware suppression API path")
    rules = manifest.get("updated_by_rules")
    require(isinstance(rules, str) and rules in {"lint-suppression"}, "suppression gate rules marker must be lint-suppression")
    if state == "physical_line_only":
        require(allowed == ["physical-line"], "physical_line_only gate may allow only physical-line rules")
    else:
        require(set(allowed) == ALLOWED_SUPPRESSION_COMPLEXITIES, "parser_aware gate must allow all suppression families")

    for rule in rule_metadata.get("rules", []):
        complexity = rule.get("suppression_complexity")
        require(complexity in allowed, f"rule {rule.get('id')} uses suppression complexity {complexity} outside gate {state}")
        if complexity != "physical-line":
            require_rule_source_imports_parser_api(rule, parser_api)


def require_rule_source_imports_parser_api(rule: dict[str, Any], parser_api: str) -> None:
    source = rule.get("source", "")
    candidate = source_path_for_rule_source(source)
    require(candidate is not None and candidate.exists(), f"could not resolve rule source for parser-aware check: {rule}")
    text = candidate.read_text(encoding="utf-8")
    short_api = parser_api.split("::")[-1]
    require(
        parser_api in text or short_api in text,
        f"non-physical rule {rule.get('id')} does not depend on parser-aware suppression API in {relative(candidate)}",
    )


def source_path_for_rule_source(source: str) -> Path | None:
    prefix = "sifr_lint::"
    if not source.startswith(prefix):
        return None
    module = source.removeprefix(prefix)
    parts = module.split("::")
    if parts[0] == "rules" and len(parts) > 1:
        return REPO_ROOT / "crates" / "sifr_lint" / "src" / "rules" / f"{parts[1]}.rs"
    if parts[0] == "suppressions":
        return REPO_ROOT / "crates" / "sifr_lint" / "src" / "lib.rs"
    return REPO_ROOT / "crates" / "sifr_lint" / "src" / f"{parts[0]}.rs"


def validate_forbidden_dependencies() -> None:
    cargo_toml = (REPO_ROOT / "crates" / "sifr_lint" / "Cargo.toml").read_text(encoding="utf-8")
    for dependency in FORBIDDEN_DEPENDENCY_NAMES:
        require(dependency not in cargo_toml, f"crates/sifr_lint/Cargo.toml contains forbidden dependency {dependency}")

    if shutil.which("cargo"):
        tree = run(["cargo", "tree", "-p", "sifr_lint", "--edges", "normal"]).stdout
        for dependency in FORBIDDEN_DEPENDENCY_NAMES:
            require(dependency not in tree, f"cargo tree for sifr_lint contains forbidden dependency {dependency}")

    for path in production_source_files():
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern, reason in FORBIDDEN_SOURCE_PATTERNS.items():
            require(pattern not in text, f"{relative(path)} contains {pattern!r}: {reason}")


def production_source_files() -> list[Path]:
    files: list[Path] = []
    for root in SCAN_ROOTS:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if (
                path.is_file()
                and path.suffix in {".rs", ".toml"}
                and "target" not in path.parts
                and "tests" not in path.parts
            ):
                files.append(path)
    return sorted(files)


def validate_no_rejected_feature_exposure(rule_config: dict[str, Any], cli_manifest: dict[str, Any]) -> None:
    rejected_config_terms = set(rule_config.get("rejected_ruff_config_keys", []))
    rejected_cli_terms = set(cli_manifest.get("rejected_surfaces", []))
    for path in production_source_files():
        text = path.read_text(encoding="utf-8", errors="replace")
        for term in rejected_cli_terms:
            long_name = term.removeprefix("--")
            patterns = [f'long = "{long_name}"', f'"{term}"']
            require(
                not any(pattern in text for pattern in patterns),
                f"{relative(path)} appears to expose rejected/future CLI lint surface {term!r}",
            )
        for term in rejected_config_terms:
            patterns = [
                f'get("{term}")',
                f'contains_key("{term}")',
                f'== "{term}"',
                f'match "{term}"',
            ]
            require(
                not any(pattern in text for pattern in patterns),
                f"{relative(path)} appears to expose rejected/future/formatter lint config key {term!r}",
            )


def validate_all() -> None:
    rule_config = load_json(RULE_CONFIG_MANIFEST)
    cli = load_json(CLI_MANIFEST)
    metadata = load_json(RULE_METADATA_MANIFEST)
    config_placeholder = load_json(CONFIG_SCHEMA_PLACEHOLDER)
    gate = load_json(SUPPRESSION_GATE_MANIFEST)
    validate_rule_config_manifest(rule_config)
    validate_cli_manifest(cli)
    validate_rule_metadata(metadata)
    validate_config_schema_placeholder(config_placeholder)
    validate_suppression_gate(gate, metadata)
    validate_forbidden_dependencies()
    validate_no_rejected_feature_exposure(rule_config, cli)


def write_json(path: Path, value: Any) -> None:
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run_self_test() -> None:
    rule_config = load_json(RULE_CONFIG_MANIFEST)
    cli = load_json(CLI_MANIFEST)
    metadata = load_json(RULE_METADATA_MANIFEST)
    gate = load_json(SUPPRESSION_GATE_MANIFEST)

    broken_rule_config = json.loads(json.dumps(rule_config))
    broken_rule_config["rule_families"] = broken_rule_config["rule_families"][1:]
    expect_failure(lambda: validate_rule_config_manifest(broken_rule_config), "missing Ruff rule family")

    broken_cli = json.loads(json.dumps(cli))
    broken_cli["surfaces"] = [
        surface for surface in broken_cli["surfaces"] if surface.get("sifr_spelling") != "FILES"
    ]
    expect_failure(lambda: validate_cli_manifest(broken_cli), "implemented CLI surface missing")

    broken_metadata = json.loads(json.dumps(metadata))
    broken_metadata["rules"][0]["suppression_complexity"] = "statement-range"
    expect_failure(lambda: validate_suppression_gate(gate, broken_metadata), "parser-aware suppression gate")

    with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as tmp:
        seed = Path(tmp) / "sifr_lint_Cargo.toml"
        seed.write_text('[dependencies]\nruff_linter = "0.0.0"\n', encoding="utf-8")
        expect_failure(
            lambda: require("ruff_linter" not in seed.read_text(encoding="utf-8"), "seed cargo dependency failure"),
            "forbidden dependency seed",
        )


def expect_failure(func: Any, label: str) -> None:
    try:
        func()
    except RulesError:
        return
    raise RulesError(f"self-test failed: {label} did not fail")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        if args.self_test:
            run_self_test()
            print("linter reuse rules self-test: PASS")
        else:
            validate_all()
            print("linter reuse rules: PASS")
    except RulesError as error:
        print(f"linter reuse rules: FAIL: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
