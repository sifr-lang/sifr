#!/usr/bin/env python3
"""Reject Sifr-owned pre-v1 compatibility mechanisms and validate retained contracts."""

from __future__ import annotations

import argparse
import json
import os
import re
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
CONTRACTS_PATH = REPO_ROOT / "verification/compatibility/retained_compatibility_contracts.json"
SCAN_ROOTS = (
    Path("crates"),
    Path("stdlib"),
    Path("verification"),
    Path(".github/workflows"),
    Path("docs"),
    Path("internal_docs"),
    Path("demos"),
    Path("scripts"),
    Path("editor_integrations"),
    Path("plans/issues/archive"),
    Path("plans/reviews/archive"),
)
TEXT_EXTENSIONS = {".json", ".md", ".mdx", ".py", ".rs", ".sifr", ".toml", ".yml", ".yaml"}
SKIP_DIR_NAMES = {
    ".git",
    ".venv",
    "__pycache__",
    "generated",
    "node_modules",
    "snapshots",
    "target",
    "third_party",
    "vendor",
}
EXCLUDED_FILES = {
    Path("verification/areas/developer_tooling/check_no_pre_v1_compatibility.py"),
    Path("verification/compatibility/retained_compatibility_contracts.json"),
    Path("docs/releases/pre-v1-breaking-changes.mdx"),
}
REQUIRED_CONTRACT_IDS = {
    "retained-dlpack-protocol",
    "retained-lsp-utf16-default",
    "retained-cargo-metadata",
    "retained-cargo-semver",
    "retained-ipc-negotiation",
    "retained-host-portability",
    "retained-cancellation-cleanup",
    "retained-configuration-defaults",
    "retained-translation-fallbacks",
    "retained-vendored-compatibility",
    "retained-external-file-formats",
    "retained-phase40-legacy-index",
    "retained-lint-deprecated-status",
}
ALLOWED_CONTRACT_KINDS = {
    "external-dependency",
    "external-format",
    "external-protocol",
    "current-product",
}
PLACEHOLDER_WORDS = {"later", "nobody", "placeholder", "tbd", "todo", "unknown", "unowned"}


def joined(*parts: str) -> str:
    """Build forbidden spellings without exempting a copied guard implementation."""
    return "".join(parts)


@dataclass(frozen=True)
class Rule:
    rule_id: str
    description: str
    patterns: tuple[re.Pattern[str], ...]
    path_prefixes: tuple[str, ...] = ()

    def applies_to(self, relative: str) -> bool:
        return not self.path_prefixes or relative.startswith(self.path_prefixes)


def words_pattern(words: tuple[str, ...]) -> re.Pattern[str]:
    return re.compile(r"\b(?:" + "|".join(re.escape(word) for word in words) + r")\b")


PUBLIC_STDLIB_ALIASES = (
    "abs_val",
    "pow_val",
    "min_val",
    "max_val",
    "round_val",
    "random_int",
    "random_float",
    "random_uniform",
    "platform_system",
    "platform_arch",
    "platform_node",
    "platform_release",
    "platform_version",
    "platform_processor",
    "time_now",
    "time_format",
    "get_args",
    "sys_exit",
    "sys_version",
    "sys_platform",
    "sys_maxsize",
    "env_get",
    "env_set",
    "env_unset",
    "env_keys",
    "env_values",
    "env_items",
    "json_loads",
    "json_dumps",
    "toml_loads",
    "fnmatch_filter",
    "html_escape",
    "html_unescape",
    "calendar_isleap",
    "calendar_weekday",
    "calendar_monthrange",
    "parse_url",
    "build_url",
)


RULES = (
    Rule(
        "public-bigint",
        "public bigint type support",
        (
            re.compile(r"\bType::" + joined("Big", "Int") + r"\b"),
            re.compile(r"\bKnownType::" + joined("Big", "Int") + r"\b"),
            re.compile(r"['\"]" + joined("big", "int") + r"['\"]"),
            re.compile(joined("SIFR-INT-", "0011") + r"|" + joined("SIFR-TYPE-", "0006")),
        ),
        ("crates/", "stdlib/", "demos/", "verification/"),
    ),
    Rule(
        "list-set-helpers",
        "list-backed set helper",
        (words_pattern(("new_set", "set_from_list", "set_add", "set_contains", "set_remove", "set_len", "set_union", "set_intersection")),),
        ("stdlib/sifr/",),
    ),
    Rule(
        "copy-heap-bisect",
        "copy-returning heapq or bisect helper",
        (words_pattern(("heapify_copy", "heappush_copy", "heappop_val", "heappop_rest", "insort_left_copy", "insort_right_copy")),),
        ("stdlib/sifr/",),
    ),
    Rule(
        "legacy-verification-schema",
        "legacy verification profile or report reader",
        (
            re.compile(r"(?:get|pop|contains_key)\s*\(\s*['\"]" + joined("legacy", "_facade") + r"['\"]"),
            re.compile(r"add_argument\s*\([^\n]*" + joined("--hardening", "-summary")),
            re.compile(r"schema_version[^\n]*(?:==|<=|in)\s*(?:1\b|[^\n]*\b1\b)"),
        ),
        ("verification/runner/", "verification/schemas/", ".github/workflows/"),
    ),
    Rule(
        "legacy-source-roots",
        "legacy source.roots reader or multi-root handler",
        (
            re.compile(r"(?:get|contains_key)\s*\(\s*['\"]" + joined("root", "s") + r"['\"]"),
            re.compile(joined("source", ".roots")),
        ),
        ("crates/sifr_package/src/manifest/production.rs", "crates/sifr_driver/src/workspace/mod.rs"),
    ),
    Rule(
        "legacy-source-default",
        "non-src package source-root default",
        (
            re.compile(r"unwrap_or_else\s*\(\|\|\s*['\"](?!src['\"])[^'\"]+['\"]\.to_string\(\)"),
            re.compile(r"None\s*=>[^\n]*PackageSourceRoot\s*\(\s*PathBuf::from\s*\(\s*['\"](?!src['\"])[^'\"]+['\"]"),
        ),
        ("crates/sifr_package/src/manifest/", "crates/sifr_driver/src/workspace/"),
    ),
    Rule(
        "legacy-manifest-targets",
        "manifest export or binary-table reader",
        (
            re.compile(r"let\s+\w+\s*=\s*[^\n]*(?:get|table)\s*\([^\n]*['\"]" + joined("export", "s") + r"['\"]"),
            re.compile(r"let\s+\w+\s*=\s*[^\n]*(?:get|table)\s*\([^\n]*['\"]" + joined("b", "in") + r"['\"]"),
            re.compile(r"\bstruct\s+\w+[^\n]*\b(?:" + joined("export", "s") + r"|" + joined("b", "in") + r")\s*:"),
        ),
        ("crates/sifr_package/src/manifest/", "crates/sifr_driver/src/workspace/"),
    ),
    Rule(
        "expect-stdout-harness",
        "legacy expect-stdout harness expectation",
        (
            re.compile(r"def\s+" + joined("extract_expect", "_stdout") + r"\b"),
            re.compile(r"(?:startswith|strip_prefix)\s*\(\s*['\"]#\s*" + joined("expect", "-stdout")),
        ),
        ("crates/", "verification/", "scripts/"),
    ),
    Rule(
        "hidden-compat-names",
        "hidden compatibility name recognition",
        (
            re.compile(joined("__compat_", "sifr_") + r"[A-Za-z0-9_]*"),
        ),
    ),
    Rule(
        "workspace-diagnostic-codes",
        "legacy workspace diagnostic code",
        (re.compile(joined("SIFR-WORKSPACE-", "010") + r"[1-4]"),),
    ),
    Rule(
        "pre-session-source-wrappers",
        "pre-session source-provider wrapper",
        (
            words_pattern(
                (
                    "load_project_with_provider",
                    "find_workspace_root_with_provider",
                    "discover_sifr_toml_with_provider",
                    "collect_sifr_files_with_provider",
                    "read_source_with_provider",
                )
            ),
        ),
    ),
    Rule(
        "collapsing-package-resolution",
        "Option-returning package import resolution wrapper",
        (re.compile(r"pub\s+fn\s+" + joined("resolve", "_import") + r"\s*\("),),
        ("crates/sifr_package/src/imports/source_map.rs",),
    ),
    Rule(
        "string-rust-types",
        "string-based Rust type rendering",
        (
            re.compile(r"\bType::" + joined("rust", "_type") + r"\b"),
            re.compile(joined("rust_type_for", "_struct_field")),
            re.compile(r"\." + joined("rust", "_type") + r"\s*\("),
            re.compile(r"RustType::Named\s*\([^\n]*(?:display_name|render_type)"),
        ),
        ("crates/",),
    ),
    Rule(
        "flat-install-layout",
        "flat installation-layout discovery",
        (
            words_pattern(("default_manifest_path",)),
            re.compile(r"current_executable\.parent\(\)[^\n]*" + joined("install", ".json")),
        ),
        ("crates/sifr/src/", "crates/sifr_sysroot/src/"),
    ),
)


@dataclass(frozen=True)
class Failure:
    rule_id: str
    path: Path
    line: int
    description: str

    def render(self) -> str:
        return f"{self.path.as_posix()}:{self.line}: {self.rule_id}: {self.description}"


class ContractError(ValueError):
    """The retained-contract registry does not name exact current ownership."""


def validate_contracts(payload: Any, root: Path) -> None:
    if not isinstance(payload, dict) or set(payload) != {"schema_version", "contracts"}:
        raise ContractError("top-level fields must be schema_version and contracts")
    if payload["schema_version"] != 1:
        raise ContractError("schema_version must be 1")
    rows = payload["contracts"]
    if not isinstance(rows, list) or len(rows) != len(REQUIRED_CONTRACT_IDS):
        raise ContractError("registry must contain exactly the required retained contracts")
    seen: set[str] = set()
    for index, row in enumerate(rows):
        if not isinstance(row, dict) or set(row) != {"id", "kind", "owner", "contract", "evidence"}:
            raise ContractError(f"contract[{index}] fields drifted")
        row_id = row["id"]
        if not isinstance(row_id, str) or not row_id:
            raise ContractError(f"contract[{index}] has no id")
        if row_id in seen:
            raise ContractError(f"duplicate contract id: {row_id}")
        seen.add(row_id)
        if row["kind"] not in ALLOWED_CONTRACT_KINDS:
            raise ContractError(f"{row_id}: kind is not external or current-product")
        owner = row["owner"]
        if not isinstance(owner, str) or not owner.startswith("contract:"):
            raise ContractError(f"{row_id}: owner must be an exact contract token")
        for field in ("owner", "contract"):
            value = row[field]
            if not isinstance(value, str) or not value.strip():
                raise ContractError(f"{row_id}: {field} must be non-empty")
            words = set(re.split(r"[^a-z0-9]+", value.lower()))
            if words & PLACEHOLDER_WORDS:
                raise ContractError(f"{row_id}: {field} contains an ownership placeholder")
        evidence = row["evidence"]
        if not isinstance(evidence, list) or not evidence or not all(isinstance(item, str) and item for item in evidence):
            raise ContractError(f"{row_id}: evidence must be a non-empty path list")
        for item in evidence:
            if Path(item).is_absolute() or not (root / item).exists():
                raise ContractError(f"{row_id}: evidence path does not exist: {item}")
    if seen != REQUIRED_CONTRACT_IDS:
        raise ContractError(f"retained contract ids drifted: {sorted(seen ^ REQUIRED_CONTRACT_IDS)}")


def should_skip(relative: Path) -> bool:
    if relative in EXCLUDED_FILES or relative.suffix not in TEXT_EXTENSIONS:
        return True
    if relative.name == "emitted.rs" or relative.name.endswith(".snap"):
        return True
    if set(relative.parts) & SKIP_DIR_NAMES:
        return True
    return (
        len(relative.parts) >= 3
        and relative.parts[0] == "plans"
        and relative.parts[2] == "archive"
    )


def public_stdlib_alias(line: str, relative: str) -> str | None:
    if not relative.startswith("stdlib/sifr/"):
        return None
    declaration = re.match(r"\s*def\s+([A-Za-z_][A-Za-z0-9_]*)\b", line)
    if declaration and declaration.group(1) in PUBLIC_STDLIB_ALIASES:
        return declaration.group(1)
    if not line.lstrip().startswith("from _sifr."):
        return None
    imports = line.split(" import ", 1)
    if len(imports) != 2:
        return None
    for item in imports[1].split(","):
        pieces = item.strip().split()
        if pieces and pieces[0] in PUBLIC_STDLIB_ALIASES:
            if len(pieces) < 3 or pieces[1] != "as" or not pieces[2].startswith("_"):
                return pieces[0]
    return None


def scan(root: Path) -> list[Failure]:
    failures: list[Failure] = []
    for scan_root in SCAN_ROOTS:
        absolute_root = root / scan_root
        if not absolute_root.exists():
            continue
        for current, dirnames, filenames in os.walk(absolute_root):
            dirnames[:] = [name for name in dirnames if name not in SKIP_DIR_NAMES]
            for filename in filenames:
                path = Path(current) / filename
                relative = path.relative_to(root)
                if should_skip(relative):
                    continue
                try:
                    lines = path.read_text(encoding="utf-8").splitlines()
                except UnicodeDecodeError:
                    continue
                relative_text = relative.as_posix()
                if re.search(joined("SIFR-WORKSPACE-", "010") + r"[1-4]", filename):
                    failures.append(Failure("workspace-diagnostic-codes", relative, 1, "legacy workspace diagnostic code filename"))
                for line_number, line in enumerate(lines, 1):
                    alias = public_stdlib_alias(line, relative_text)
                    if alias is not None:
                        failures.append(Failure("public-stdlib-aliases", relative, line_number, f"public alias {alias}"))
                    for rule in RULES:
                        if rule.applies_to(relative_text) and any(pattern.search(line) for pattern in rule.patterns):
                            failures.append(Failure(rule.rule_id, relative, line_number, rule.description))
    return failures


def run_self_test() -> None:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        mutations = {
            "public-bigint": ("crates/compiler/src/types.rs", 'let name = "bigint";'),
            "public-stdlib-aliases": ("stdlib/sifr/math.sifr", "def abs_val(x: float) -> float:"),
            "list-set-helpers": ("stdlib/sifr/collections.sifr", "def set_from_list(values):"),
            "copy-heap-bisect": ("stdlib/sifr/heapq.sifr", "def heapify_copy(values):"),
            "legacy-verification-schema": ("verification/runner/profile.py", 'parser.add_argument("--hardening-summary")'),
            "legacy-source-roots": ("crates/sifr_package/src/manifest/production.rs", 'source.get("roots")'),
            "legacy-source-default": (
                "crates/sifr_driver/src/workspace/mod.rs",
                'let root = source_root.unwrap_or_else(|| "source".to_string());',
            ),
            "legacy-manifest-targets": ("crates/sifr_package/src/manifest/read.rs", 'let exports = value.get("exports");'),
            "expect-stdout-harness": ("verification/runner/harness.py", 'def extract_expect_stdout(source):'),
            "hidden-compat-names": ("docs/compiler.md", "__compat_sifr_sync_Lock"),
            "workspace-diagnostic-codes": ("crates/diagnostics/src/lib.rs", "SIFR-WORKSPACE-0102"),
            "pre-session-source-wrappers": ("internal_docs/query.md", "FrontendContext::load_project_with_provider"),
            "collapsing-package-resolution": ("crates/sifr_package/src/imports/source_map.rs", "pub fn resolve_import(&self) -> Option<PathBuf>"),
            "string-rust-types": ("crates/codegen/src/types.rs", "value.rust_type()"),
            "flat-install-layout": ("crates/sifr/src/update.rs", "fn default_manifest_path() {}"),
        }
        for path_text, text in mutations.values():
            path = root / path_text
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text + "\n", encoding="utf-8")
        archive = root / "plans/issues/archive/history.md"
        archive.parent.mkdir(parents=True, exist_ok=True)
        archive.write_text("__compat_sifr_sync_Archived\n", encoding="utf-8")
        generated = root / "demos/example/emitted.rs"
        generated.parent.mkdir(parents=True, exist_ok=True)
        generated.write_text("SIFR-WORKSPACE-0101\n", encoding="utf-8")
        retained = root / "crates/sifr_lint/src/lib.rs"
        retained.parent.mkdir(parents=True, exist_ok=True)
        retained.write_text("enum RuleStatus { Deprecated }\n", encoding="utf-8")
        failures = scan(root)
        excluded_paths = {
            archive.relative_to(root),
            generated.relative_to(root),
        }
        leaked_exclusions = sorted(
            failure.path.as_posix()
            for failure in failures
            if failure.path in excluded_paths
        )
        if leaked_exclusions:
            raise SystemExit(
                "no-compatibility guard scanned excluded archive/generated mutations: "
                + ", ".join(leaked_exclusions)
            )
    observed = {failure.rule_id for failure in failures}
    expected = set(mutations)
    if observed != expected:
        raise SystemExit(f"no-compatibility guard self-test drifted: expected {sorted(expected)}, got {sorted(observed)}")
    print("final no-compatibility guard self-test: PASS")


def run_contract_self_test(payload: dict[str, Any]) -> None:
    changed = json.loads(json.dumps(payload))
    changed["contracts"][0]["owner"] = "contract:todo"
    try:
        validate_contracts(changed, REPO_ROOT)
    except ContractError:
        return
    raise ContractError("retained-contract mutation unexpectedly passed")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    try:
        payload = json.loads(CONTRACTS_PATH.read_text(encoding="utf-8"))
        validate_contracts(payload, REPO_ROOT)
        if args.self_test:
            run_self_test()
            run_contract_self_test(payload)
            print("retained compatibility contract self-test: PASS")
            return 0
    except (OSError, json.JSONDecodeError, ContractError) as error:
        print(f"final no-compatibility guard: {error}")
        return 1
    failures = scan(REPO_ROOT)
    if failures:
        for failure in failures:
            print(f"final no-compatibility guard: {failure.render()}")
        return 1
    print("final no-compatibility guard: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
