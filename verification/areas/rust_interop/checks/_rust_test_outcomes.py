"""Bind declared evidence outcomes to assertions in the selected Rust test."""

from __future__ import annotations

import re
import tempfile
from pathlib import Path
from typing import Any

RUST_FUNCTION = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(")
RUST_CALL = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*(?:\([^;{}]*\))?\s*\(")
DIAGNOSTIC_CONST = re.compile(
    r"pub\s+const\s+([A-Z][A-Z0-9_]*)\s*:\s*Self\s*="
    r'.*?Self::new\(\s*"(SIFR-[A-Z0-9-]+)"',
    re.DOTALL,
)
DIAGNOSTIC_USE = re.compile(r"DiagnosticCode::([A-Z][A-Z0-9_]*)")
ASSERTION_MACRO = re.compile(r"\bassert(?:_eq|_ne|_matches)?!\s*\(")
RUNTIME_STATE_TYPES = {
    "closed": "HandleStateError::Closed",
    "poisoned": "HandleStateError::Poisoned",
}


def validate_bound_test_outcome(
    failures: list[str],
    *,
    repo_root: Path,
    label: str,
    source_path: Path,
    test_name: str,
    evidence: Any,
) -> None:
    """Require the bound test's reachable local bodies to assert the claim."""
    if not isinstance(evidence, dict):
        return
    expected_result = evidence.get("expected_result")
    if expected_result not in {"diagnostic", "runtime-error-state"}:
        return
    bodies = _rust_function_bodies(source_path)
    reachable = _reachable_function_text(bodies, test_name)
    if reachable is None:
        failures.append(
            f"{label}.validation cannot inspect outcome assertions for {test_name}"
        )
        return
    assertions = _rust_assertions(reachable)
    if expected_result == "diagnostic":
        expected_diagnostic = evidence.get("expected_diagnostic")
        if not isinstance(expected_diagnostic, str):
            return
        constants = _diagnostic_constants(repo_root)
        asserted_codes = set(
            re.findall(r"SIFR-RUST-[A-Z]+-[0-9]{4}", assertions)
        )
        asserted_codes.update(
            constants[name]
            for name in DIAGNOSTIC_USE.findall(assertions)
            if name in constants
        )
        if expected_diagnostic not in asserted_codes:
            failures.append(
                f"{label}.validation bound test does not assert declared diagnostic "
                f"{expected_diagnostic}"
            )
        return

    expected_state = evidence.get("expected_runtime_state")
    state_type = RUNTIME_STATE_TYPES.get(str(expected_state))
    if state_type is not None and state_type not in assertions:
        failures.append(
            f"{label}.validation bound test does not assert declared runtime state "
            f"{expected_state}"
        )


def _diagnostic_constants(repo_root: Path) -> dict[str, str]:
    registry = repo_root / "crates" / "sifr_diagnostics" / "src" / "codes" / "registry.rs"
    if not registry.is_file():
        return {}
    return {
        name: code
        for name, code in DIAGNOSTIC_CONST.findall(
            registry.read_text(encoding="utf-8")
        )
    }


def _rust_function_bodies(source_path: Path) -> dict[str, str]:
    source = source_path.read_text(encoding="utf-8")
    masked = _mask_rust_noncode(source)
    bodies: dict[str, str] = {}
    for match in RUST_FUNCTION.finditer(masked):
        opening = masked.find("{", match.end())
        if opening < 0:
            continue
        semicolon = masked.find(";", match.end(), opening)
        if semicolon >= 0:
            continue
        closing = _matching_brace(masked, opening)
        if closing is not None:
            bodies[match.group(1)] = source[opening + 1 : closing]
    return bodies


def _reachable_function_text(
    bodies: dict[str, str],
    test_name: str,
) -> str | None:
    if test_name not in bodies:
        return None
    pending = [test_name]
    seen: set[str] = set()
    reachable: list[str] = []
    while pending:
        name = pending.pop()
        if name in seen:
            continue
        seen.add(name)
        body = bodies.get(name)
        if body is None:
            continue
        reachable.append(body)
        masked = _mask_rust_noncode(body)
        pending.extend(
            called
            for called in RUST_CALL.findall(masked)
            if called in bodies and called not in seen
        )
    return "\n".join(reachable)


def _matching_brace(masked: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == "{":
            depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def _rust_assertions(source: str) -> str:
    """Return exact balanced assertion macros from reachable function bodies."""
    masked = _mask_rust_noncode(source)
    assertions: list[str] = []
    for assertion in ASSERTION_MACRO.finditer(masked):
        opening = masked.find("(", assertion.start())
        closing = _matching_delimiter(masked, opening, "(", ")")
        if closing is not None:
            assertions.append(source[assertion.start() : closing + 1])
    return "\n".join(assertions)


def _matching_delimiter(
    masked: str,
    opening: int,
    opening_character: str,
    closing_character: str,
) -> int | None:
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == opening_character:
            depth += 1
        elif masked[index] == closing_character:
            depth -= 1
            if depth == 0:
                return index
    return None


def _mask_rust_noncode(source: str) -> str:
    """Blank comments and literals while preserving offsets and code braces."""
    masked = list(source)
    index = 0
    block_depth = 0
    while index < len(source):
        if block_depth:
            if source.startswith("/*", index):
                block_depth += 1
                masked[index : index + 2] = "  "
                index += 2
            elif source.startswith("*/", index):
                block_depth -= 1
                masked[index : index + 2] = "  "
                index += 2
            else:
                if source[index] != "\n":
                    masked[index] = " "
                index += 1
            continue
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end < 0 else end
            masked[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            block_depth = 1
            masked[index : index + 2] = "  "
            index += 2
            continue
        raw = re.match(r'(?:b|c)?r(#+)?"', source[index:])
        if raw is not None:
            hashes = raw.group(1) or ""
            terminator = '"' + hashes
            end = source.find(terminator, index + len(raw.group(0)))
            end = len(source) if end < 0 else end + len(terminator)
            _blank_preserving_newlines(masked, source, index, end)
            index = end
            continue
        quoted = re.match(r'(?:b|c)?"', source[index:])
        if quoted is not None:
            end = _quoted_literal_end(source, index + len(quoted.group(0)))
            _blank_preserving_newlines(masked, source, index, end)
            index = end
            continue
        character = re.match(r"(?:b)?'(?:\\.|[^'\\])+'", source[index:])
        if character is not None:
            end = index + len(character.group(0))
            masked[index:end] = " " * (end - index)
            index = end
            continue
        index += 1
    return "".join(masked)


def _quoted_literal_end(source: str, index: int) -> int:
    escaped = False
    while index < len(source):
        character = source[index]
        index += 1
        if character == '"' and not escaped:
            return index
        escaped = character == "\\" and not escaped
        if character != "\\":
            escaped = False
    return len(source)


def _blank_preserving_newlines(
    masked: list[str],
    source: str,
    start: int,
    end: int,
) -> None:
    for index in range(start, end):
        if source[index] != "\n":
            masked[index] = " "


def run_self_test() -> tuple[int, str | None]:
    """Mutation-test literal, constant, helper, and state assertion binding."""
    with tempfile.TemporaryDirectory(prefix="sifr-rust-outcome-") as raw_root:
        root = Path(raw_root)
        registry = root / "crates/sifr_diagnostics/src/codes/registry.rs"
        registry.parent.mkdir(parents=True)
        registry.write_text(
            'pub const RUST_TYPE_PROBE_FAILURE: Self = '
            'Self::new("SIFR-RUST-TYPE-0001", Severity::Error);\n',
            encoding="utf-8",
        )
        source = root / "evidence.rs"
        source.write_text(
            "fn helper() { assert_eq!(code, "
            'DiagnosticCode::RUST_TYPE_PROBE_FAILURE); }\n'
            "#[test]\nfn diagnostic_test() { helper(); }\n"
            '#[test]\nfn unrelated_test() { assert_eq!(code, "SIFR-RUST-PANIC-0001"); }\n'
            "#[test]\nfn state_test() { "
            "assert_eq!(error, HandleStateError::Closed); }\n",
            encoding="utf-8",
        )
        cases = (
            (
                "diagnostic control",
                "diagnostic_test",
                {
                    "expected_result": "diagnostic",
                    "expected_diagnostic": "SIFR-RUST-TYPE-0001",
                },
                None,
            ),
            (
                "unrelated diagnostic",
                "diagnostic_test",
                {
                    "expected_result": "diagnostic",
                    "expected_diagnostic": "SIFR-RUST-PANIC-0001",
                },
                "does not assert declared diagnostic",
            ),
            (
                "state control",
                "state_test",
                {
                    "expected_result": "runtime-error-state",
                    "expected_runtime_state": "closed",
                },
                None,
            ),
            (
                "state mismatch",
                "state_test",
                {
                    "expected_result": "runtime-error-state",
                    "expected_runtime_state": "poisoned",
                },
                "does not assert declared runtime state",
            ),
        )
        for name, test_name, evidence, expected in cases:
            failures: list[str] = []
            validate_bound_test_outcome(
                failures,
                repo_root=root,
                label=name,
                source_path=source,
                test_name=test_name,
                evidence=evidence,
            )
            if expected is None and failures:
                return len(cases), f"{name} was rejected: {failures}"
            if expected is not None and not any(
                expected in failure for failure in failures
            ):
                return len(cases), f"{name} did not report {expected!r}: {failures}"
        return len(cases), None
