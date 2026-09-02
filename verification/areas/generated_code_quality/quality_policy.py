"""Strict generated-Rust checks with explicit, monotonically shrinking debt."""

from __future__ import annotations

import collections
import copy
import dataclasses
import hashlib
import json
import re
from pathlib import Path
from typing import Any, Iterable

STRICT_CLIPPY_ARGS = [
    "-D",
    "warnings",
    "-W",
    "clippy::pedantic",
    "-W",
    "clippy::nursery",
    "-W",
    "clippy::arithmetic_side_effects",
    "-W",
    "clippy::indexing_slicing",
    "-W",
    "clippy::cast_possible_truncation",
    "-W",
    "clippy::cast_possible_wrap",
    "-W",
    "clippy::cast_sign_loss",
    "-W",
    "clippy::unwrap_used",
    "-W",
    "clippy::expect_used",
    "-W",
    "clippy::panic",
    "-W",
    "clippy::unimplemented",
    "-W",
    "clippy::todo",
    "-W",
    "clippy::exit",
]


@dataclasses.dataclass(frozen=True)
class PatternPolicy:
    id: str
    owner_item: int
    pattern: re.Pattern[str]


PATTERN_POLICIES = (
    PatternPolicy("unwrap", 3, re.compile(r"\.unwrap\s*\(")),
    PatternPolicy("expect", 3, re.compile(r"\.expect\s*\(")),
    PatternPolicy("panic", 3, re.compile(r"\bpanic\s*!")),
    PatternPolicy("todo", 3, re.compile(r"\btodo\s*!")),
    PatternPolicy("unimplemented", 3, re.compile(r"\bunimplemented\s*!")),
    PatternPolicy("unsafe", 3, re.compile(r"\bunsafe\b")),
    PatternPolicy("allow-attribute", 8, re.compile(r"#\s*!?\s*\[\s*allow\s*\(")),
    PatternPolicy("unreachable", 3, re.compile(r"\bunreachable\s*!")),
    PatternPolicy("process-abort", 3, re.compile(r"\b(?:std|::std)::process::abort\s*\(")),
    PatternPolicy("process-exit", 3, re.compile(r"\b(?:std|::std)::process::exit\s*\(")),
    PatternPolicy(
        "direct-index",
        4,
        re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*\s*\[[^\]\n]+\]"),
    ),
    PatternPolicy("signed-to-usize", 11, re.compile(r"\bas\s+usize\b")),
)
PATTERN_BY_ID = {policy.id: policy for policy in PATTERN_POLICIES}
DEBT_CATEGORIES = {"safety", "rustfmt", "clippy", "freshness"}


@dataclasses.dataclass(frozen=True)
class Violation:
    policy_id: str
    path: str
    line: int
    text: str


def debt_selection_id(entry_ids: Iterable[str]) -> str:
    digest = hashlib.sha256()
    for entry_id in entry_ids:
        digest.update(entry_id.encode("utf-8"))
        digest.update(b"\0")
    return f"selection-{digest.hexdigest()[:16]}"


def rust_code_lines(source: str) -> list[str]:
    output = list(source)
    index = 0
    block_depth = 0
    while index < len(source):
        if block_depth:
            if source.startswith("/*", index):
                output[index : index + 2] = "  "
                block_depth += 1
                index += 2
            elif source.startswith("*/", index):
                output[index : index + 2] = "  "
                block_depth -= 1
                index += 2
            else:
                if source[index] != "\n":
                    output[index] = " "
                index += 1
            continue
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end == -1 else end
            output[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            output[index : index + 2] = "  "
            block_depth = 1
            index += 2
            continue
        raw_match = re.match(r"(?:br|r)(?P<hashes>#{0,255})\"", source[index:])
        if raw_match is not None:
            delimiter = f'\"{raw_match.group("hashes")}'
            end = source.find(delimiter, index + raw_match.end())
            end = len(source) if end == -1 else end + len(delimiter)
            for offset in range(index, end):
                if source[offset] != "\n":
                    output[offset] = " "
            index = end
            continue
        if source.startswith('b"', index) or source[index] == '"':
            start = index
            index += 2 if source.startswith('b"', index) else 1
            escaped = False
            while index < len(source):
                character = source[index]
                index += 1
                if character == '"' and not escaped:
                    break
                escaped = character == "\\" and not escaped
                if character != "\\":
                    escaped = False
            for offset in range(start, index):
                if source[offset] != "\n":
                    output[offset] = " "
            continue
        if source[index] == "'":
            char_match = re.match(r"'(?:\\.|[^\\'\n])'", source[index:])
            if char_match is not None:
                end = index + char_match.end()
                output[index:end] = " " * (end - index)
                index = end
                continue
        index += 1
    return "".join(output).splitlines()


def pattern_matches(policy: PatternPolicy, code: str) -> bool:
    for match in policy.pattern.finditer(code):
        if policy.id == "direct-index" and re.fullmatch(r"let\s*\[[^\n]+", match.group(0)):
            continue
        return True
    return False


def scan_files(paths: Iterable[Path], root: Path) -> list[Violation]:
    violations: list[Violation] = []
    for path in paths:
        relative = path.relative_to(root).as_posix() if path.is_relative_to(root) else path.name
        source = path.read_text(encoding="utf-8")
        original_lines = source.splitlines()
        code_lines = rust_code_lines(source)
        for line_number, (line, code) in enumerate(
            zip(original_lines, code_lines, strict=True),
            start=1,
        ):
            for policy in PATTERN_POLICIES:
                if pattern_matches(policy, code):
                    violations.append(Violation(policy.id, relative, line_number, line.strip()))
    return violations


def violation_summary(violations: Iterable[Violation]) -> dict[str, dict[str, Any]]:
    grouped: dict[str, list[Violation]] = collections.defaultdict(list)
    for violation in violations:
        grouped[violation.policy_id].append(violation)
    summary: dict[str, dict[str, Any]] = {}
    for policy_id, items in sorted(grouped.items()):
        signatures = sorted(f"{item.path}:{item.line}:{item.text}" for item in items)
        digest = hashlib.sha256("\n".join(signatures).encode("utf-8")).hexdigest()
        summary[policy_id] = {"count": len(items), "signature_sha256": digest}
    return summary


def parse_clippy_diagnostics(
    output: str,
    workspace: Path | None = None,
) -> dict[str, dict[str, Any]]:
    grouped: dict[str, list[str]] = collections.defaultdict(list)
    for line in output.splitlines():
        try:
            message = json.loads(line)
        except json.JSONDecodeError:
            continue
        if message.get("reason") != "compiler-message":
            continue
        diagnostic = message.get("message")
        if not isinstance(diagnostic, dict) or diagnostic.get("level") not in {"warning", "error"}:
            continue
        code = diagnostic.get("code")
        code_value = code.get("code") if isinstance(code, dict) else None
        if not isinstance(code_value, str):
            code_value = f"rustc::{diagnostic.get('message', 'uncoded-error')}"
        spans = diagnostic.get("spans")
        primary = next(
            (
                span
                for span in spans
                if isinstance(span, dict) and span.get("is_primary") is True
            ),
            {},
        ) if isinstance(spans, list) else {}
        filename = str(primary.get("file_name", "<unknown>"))
        if workspace is not None:
            filename = filename.replace(str(workspace), "<workspace>")
        signature = "|".join(
            (
                str(diagnostic.get("message", "")),
                filename,
                str(primary.get("line_start", 0)),
                str(primary.get("column_start", 0)),
            )
        )
        grouped[code_value].append(signature)
    return {
        code: {
            "count": len(signatures),
            "signature_sha256": hashlib.sha256(
                "\n".join(sorted(signatures)).encode("utf-8")
            ).hexdigest(),
        }
        for code, signatures in sorted(grouped.items())
    }


def output_signature(output: str, workspace: Path) -> dict[str, str]:
    normalized = output.replace(str(workspace), "<workspace>")
    return {
        "diagnostic_sha256": hashlib.sha256(normalized.encode("utf-8")).hexdigest(),
    }


def merge_signature_summaries(
    records: Iterable[tuple[str, dict[str, dict[str, Any]]]],
) -> dict[str, dict[str, Any]]:
    grouped: dict[str, list[tuple[str, dict[str, Any]]]] = collections.defaultdict(list)
    for entry_id, summary in records:
        for finding, signature in summary.items():
            grouped[finding].append((entry_id, signature))
    return {
        finding: {
            "count": sum(signature["count"] for _, signature in signatures),
            "signature_sha256": hashlib.sha256(
                "\n".join(
                    sorted(
                        f"{entry_id}|{signature['count']}|{signature['signature_sha256']}"
                        for entry_id, signature in signatures
                    )
                ).encode("utf-8")
            ).hexdigest(),
        }
        for finding, signatures in sorted(grouped.items())
    }


def merge_output_signatures(records: Iterable[tuple[str, dict[str, str]]]) -> dict[str, Any]:
    signatures = sorted(
        f"{entry_id}|{signature['diagnostic_sha256']}"
        for entry_id, signature in records
    )
    if not signatures:
        return {}
    return {
        "count": len(signatures),
        "signature_sha256": hashlib.sha256("\n".join(signatures).encode("utf-8")).hexdigest(),
    }


def compact_clippy_summary(summary: dict[str, dict[str, Any]]) -> dict[str, Any]:
    if not summary:
        return {}
    signatures = [
        f"{lint}|{signature['count']}|{signature['signature_sha256']}"
        for lint, signature in sorted(summary.items())
    ]
    return {
        "count": sum(signature["count"] for signature in summary.values()),
        "lint_count": len(summary),
        "signature_sha256": hashlib.sha256("\n".join(signatures).encode("utf-8")).hexdigest(),
    }


def validate_clippy_lint_owners(
    summary: dict[str, Any],
    debt: dict[str, Any],
    *,
    require_exact: bool = False,
) -> None:
    lint_owners = debt["clippy"]["lint_owners"]
    unknown = sorted(set(summary).difference(lint_owners))
    if unknown:
        raise RuntimeError(f"Clippy debt contains lints without exact owners: {unknown}")
    if require_exact:
        governed_lints = {
            lint
            for entry in debt["clippy"]["entries"].values()
            for lint in entry
        }
        stale = sorted(set(lint_owners).difference(governed_lints))
        if stale:
            raise RuntimeError(f"Clippy debt contains stale lint owners: {stale}")


def load_debt(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict) or payload.get("schema_version") != 2:
        raise ValueError("generated quality debt must use schema_version 2")
    if payload.get("closure_item") != 12:
        raise ValueError("generated quality debt must expire at Item 12")
    validate_debt_owners(payload)
    return payload


def compare_exact_debt(
    *,
    category: str,
    entry_id: str,
    actual: Any,
    debt: dict[str, Any],
) -> None:
    category_debt = debt.get(category)
    if not isinstance(category_debt, dict):
        raise ValueError(f"missing debt category: {category}")
    entries = category_debt.get("entries")
    if not isinstance(entries, dict):
        raise ValueError(f"debt category {category} must contain entries")
    expected = entries.get(entry_id)
    if actual in ({}, [], None, False):
        if expected not in (None, {}, [], False):
            raise RuntimeError(f"{entry_id}: {category} debt was fixed; remove its stale baseline")
        return
    if expected is None:
        raise RuntimeError(f"{entry_id}: new unowned {category} debt: {actual}")
    if actual != expected:
        raise RuntimeError(
            f"{entry_id}: {category} debt changed\nexpected={expected}\nactual={actual}"
        )


def validate_debt_owners(debt: dict[str, Any]) -> None:
    structural_keys = set(debt).difference(
        {"schema_version", "baseline_commit", "closure_item", "policy"}
    )
    if structural_keys != DEBT_CATEGORIES:
        raise ValueError("generated quality debt must contain the exact category set")
    baseline_commit = debt.get("baseline_commit")
    if not isinstance(baseline_commit, str) or re.fullmatch(r"[0-9a-f]{40}", baseline_commit) is None:
        raise ValueError("generated quality debt must name a lowercase 40-character baseline SHA")
    policy = debt.get("policy")
    if not isinstance(policy, str) or not policy.strip():
        raise ValueError("generated quality debt must name its policy")
    for category in sorted(DEBT_CATEGORIES):
        section = debt.get(category)
        if not isinstance(section, dict):
            raise ValueError(f"missing debt section: {category}")
        owner_items = section.get("owner_items")
        if not isinstance(owner_items, list) or not owner_items:
            raise ValueError(f"{category} debt must name owner_items")
        if any(not isinstance(item, int) or isinstance(item, bool) or item not in range(1, 12) for item in owner_items):
            raise ValueError(f"{category} debt owner_items must be Items 1 through 11")
        allowed_fields = {"owner_items", "entries", "lint_owners"} if category == "clippy" else {"owner_items", "entries"}
        if set(section) != allowed_fields:
            raise ValueError(f"{category} debt must contain the exact required fields")
        lint_owners = section.get("lint_owners")
        if category == "clippy" and (
            not isinstance(lint_owners, dict)
            or any(
                not isinstance(lint, str)
                or not lint.strip()
                or not isinstance(owner, int)
                or isinstance(owner, bool)
                or owner not in owner_items
                for lint, owner in lint_owners.items()
            )
        ):
            raise ValueError("clippy debt lint_owners must map lint names to listed owner items")
        entries = section.get("entries")
        if not isinstance(entries, dict):
            raise ValueError(f"{category} debt must contain an entries object")
        for entry_id, entry in entries.items():
            if not isinstance(entry_id, str) or not entry_id.strip():
                raise ValueError(f"{category} debt entry ids must be non-empty text")
            validate_debt_entry(category, entry_id, entry, owner_items, lint_owners)


def validate_signature_summary(summary: object, label: str) -> None:
    if not isinstance(summary, dict) or set(summary) != {"count", "signature_sha256"}:
        raise ValueError(f"{label} must contain count and signature_sha256")
    count = summary.get("count")
    signature = summary.get("signature_sha256")
    if not isinstance(count, int) or isinstance(count, bool) or count < 1:
        raise ValueError(f"{label} count must be a positive integer")
    if not isinstance(signature, str) or re.fullmatch(r"[0-9a-f]{64}", signature) is None:
        raise ValueError(f"{label} signature_sha256 must be a lowercase SHA-256")


def validate_debt_entry(
    category: str,
    entry_id: str,
    entry: object,
    owner_items: list[int],
    lint_owners: object,
) -> None:
    label = f"{category} debt entry {entry_id}"
    if category == "safety":
        if not isinstance(entry, dict) or not entry:
            raise ValueError(f"{label} must contain policy signatures")
        for policy_id, summary in entry.items():
            policy = PATTERN_BY_ID.get(policy_id)
            if policy is None:
                raise ValueError(f"{label} names unknown policy {policy_id}")
            if policy.owner_item not in owner_items:
                raise ValueError(f"{label} policy {policy_id} has an unlisted owner")
            validate_signature_summary(summary, f"{label} policy {policy_id}")
        return
    if category == "clippy":
        if not isinstance(entry, dict) or not entry:
            raise ValueError(f"{label} must contain per-lint signatures")
        if not isinstance(lint_owners, dict):
            raise ValueError(f"{label} requires lint owners")
        for lint, summary in entry.items():
            if lint not in lint_owners:
                raise ValueError(f"{label} names lint without an exact owner: {lint}")
            validate_signature_summary(summary, f"{label} lint {lint}")
        return
    if category == "rustfmt":
        validate_signature_summary(entry, label)
        return
    if not isinstance(entry, dict) or set(entry) != {"stale", "orphans"}:
        raise ValueError(f"{label} must contain stale and orphans lists")
    for field in ("stale", "orphans"):
        values = entry[field]
        if not isinstance(values, list) or any(
            not isinstance(value, str) or not value.strip() for value in values
        ):
            raise ValueError(f"{label} {field} must be a text list")
    if not entry["stale"] and not entry["orphans"]:
        raise ValueError(f"{label} must not be empty")


def require_empty_debt(debt: dict[str, Any]) -> None:
    validate_debt_owners(debt)
    remaining = {
        category: debt[category]["entries"]
        for category in sorted(DEBT_CATEGORIES)
        if debt[category]["entries"]
    }
    if debt["clippy"]["lint_owners"]:
        remaining["clippy_lint_owners"] = debt["clippy"]["lint_owners"]
    if remaining:
        raise RuntimeError(f"generated quality debt must be empty for phase closure: {remaining}")


def validate_debt_selection_ids(debt: dict[str, Any], allowed: set[str]) -> None:
    validate_debt_owners(debt)
    for category in ("safety", "rustfmt", "clippy"):
        unknown = set(debt[category]["entries"]).difference(allowed)
        if category == "safety":
            unknown.discard("checked-demo-snapshots")
        if unknown:
            raise ValueError(f"{category} debt contains stale or unknown selections: {sorted(unknown)}")


def _expect_invalid(debt: dict[str, Any], expected: str) -> None:
    try:
        validate_debt_owners(debt)
    except ValueError as error:
        if expected in str(error):
            return
        raise AssertionError(f"expected {expected!r}, got {error!s}") from error
    raise AssertionError(f"expected {expected!r}, debt was accepted")


def run_debt_self_test(debt: dict[str, Any]) -> None:
    validate_debt_owners(debt)

    one_lint = {
        "clippy::self_test": {
            "count": 1,
            "signature_sha256": "0" * 64,
        }
    }
    first_companion = merge_signature_summaries([("companion-a", one_lint)])
    moved_companion = merge_signature_summaries([("companion-b", one_lint)])
    if first_companion == moved_companion:
        raise AssertionError("merged lint signatures ignored companion identity")

    lexical_fixture = "// panic!()\nlet text = \"values[0] as usize\";\n/* unsafe {} */\n"
    if any(
        pattern_matches(policy, line)
        for line in rust_code_lines(lexical_fixture)
        for policy in PATTERN_POLICIES
    ):
        raise AssertionError("Rust lexical scanner inspected comments or string contents")
    slice_pattern = "let [head, middle @ .., tail] = values.as_slice() else { return; };"
    if pattern_matches(PATTERN_BY_ID["direct-index"], slice_pattern):
        raise AssertionError("direct-index scanner rejected a refutable slice pattern")

    missing_category = copy.deepcopy(debt)
    missing_category.pop("freshness")
    _expect_invalid(missing_category, "exact category set")

    unknown_category = copy.deepcopy(debt)
    unknown_category["unknown"] = {"owner_items": [1], "entries": {}}
    _expect_invalid(unknown_category, "exact category set")

    ownerless = copy.deepcopy(debt)
    ownerless["safety"]["owner_items"] = []
    _expect_invalid(ownerless, "must name owner_items")

    invalid_owner = copy.deepcopy(debt)
    invalid_owner["safety"]["owner_items"] = [12]
    _expect_invalid(invalid_owner, "Items 1 through 11")

    missing_entries = copy.deepcopy(debt)
    missing_entries["safety"].pop("entries")
    _expect_invalid(missing_entries, "exact required fields")

    missing_lint_owners = copy.deepcopy(debt)
    missing_lint_owners["clippy"].pop("lint_owners")
    _expect_invalid(missing_lint_owners, "exact required fields")

    malformed_entry = copy.deepcopy(debt)
    malformed_entry["rustfmt"]["entries"]["self-test"] = {"count": 1, "signature_sha256": "bad"}
    _expect_invalid(malformed_entry, "signature_sha256 must be a lowercase SHA-256")

    malformed_clippy = copy.deepcopy(debt)
    malformed_clippy["clippy"]["entries"]["self-test"] = {
        "clippy::arithmetic_side_effects": {
            "count": 1,
            "signature_sha256": "bad",
        }
    }
    _expect_invalid(malformed_clippy, "signature_sha256 must be a lowercase SHA-256")

    ownerless_clippy = copy.deepcopy(debt)
    ownerless_clippy["clippy"]["entries"]["self-test"] = {
        "clippy::self_test_ownerless": {
            "count": 1,
            "signature_sha256": "0" * 64,
        }
    }
    _expect_invalid(ownerless_clippy, "lint without an exact owner")

    stale_selection = copy.deepcopy(debt)
    stale_selection["rustfmt"]["entries"]["stale-selection"] = {
        "count": 1,
        "signature_sha256": "0" * 64,
    }
    try:
        validate_debt_selection_ids(stale_selection, set())
    except ValueError as error:
        if "stale or unknown selections" not in str(error):
            raise
    else:
        raise AssertionError("stale debt selection was accepted")

    stale_lint_owner = copy.deepcopy(debt)
    stale_lint_owner["clippy"]["lint_owners"]["clippy::self_test_stale"] = 8
    try:
        validate_clippy_lint_owners({}, stale_lint_owner, require_exact=True)
    except RuntimeError as error:
        if "stale lint owners" not in str(error):
            raise
    else:
        raise AssertionError("stale Clippy lint owner was accepted")

    cross_selection_owner = copy.deepcopy(debt)
    cross_selection_owner["clippy"]["lint_owners"]["clippy::self_test_other_selection"] = 10
    cross_selection_owner["clippy"]["entries"]["self-test-other-selection"] = {
        "clippy::self_test_other_selection": {
            "count": 1,
            "signature_sha256": "0" * 64,
        }
    }
    validate_clippy_lint_owners({}, cross_selection_owner, require_exact=True)

    for category in sorted(DEBT_CATEGORIES):
        try:
            compare_exact_debt(
                category=category,
                entry_id="self-test-new-debt",
                actual={"synthetic": 1},
                debt=debt,
            )
        except RuntimeError as error:
            if f"new unowned {category} debt" not in str(error):
                raise
        else:
            raise AssertionError(f"unowned {category} debt mutation was accepted")

    nonempty = copy.deepcopy(debt)
    nonempty["safety"]["entries"]["self-test"] = {
        "panic": {"count": 1, "signature_sha256": "0" * 64}
    }
    try:
        require_empty_debt(nonempty)
    except RuntimeError as error:
        if "must be empty" not in str(error):
            raise
    else:
        raise AssertionError("phase closure accepted non-empty debt")


def assert_negative_pattern(seed: Path, expected_policy: str) -> None:
    violations = scan_files([seed], seed.parent)
    actual = {violation.policy_id for violation in violations}
    if expected_policy not in actual:
        raise RuntimeError(
            f"negative safety seed {seed.name} did not trigger {expected_policy}; got {sorted(actual)}"
        )
