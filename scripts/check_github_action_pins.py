#!/usr/bin/env python3
"""Validate every maintained third-party GitHub Action selection."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = REPO_ROOT / "verification/policy/github_actions.json"
USE_RE = re.compile(
    r"^(?P<indent>\s*)(?:-\s*)?uses:\s*(?P<target>\S+?)"
    r"(?:\s+#\s*(?P<label>.+?))?\s*$"
)
SHA_RE = re.compile(r"[0-9a-f]{40}")


def load_policy() -> dict[str, Any]:
    return json.loads(POLICY_PATH.read_text(encoding="utf-8"))


def policy_pins(policy: dict[str, Any]) -> tuple[dict[str, tuple[str, str]], list[str]]:
    failures: list[str] = []
    if set(policy) != {"schema_version", "maintained_workflow_roots", "actions"}:
        failures.append("policy top-level fields drifted")
    if policy.get("schema_version") != 1:
        failures.append("policy schema_version must be 1")

    pins: dict[str, tuple[str, str]] = {}
    for item in policy.get("actions", []):
        if not isinstance(item, dict) or set(item) != {"name", "version", "sha"}:
            failures.append(
                "every action policy entry must contain name, version, and sha"
            )
            continue
        name = item.get("name")
        version = item.get("version")
        sha = item.get("sha")
        if (
            not isinstance(name, str)
            or re.fullmatch(r"[^/@\s]+/[^/@\s]+", name) is None
        ):
            failures.append(f"invalid action name: {name!r}")
            continue
        if name in pins:
            failures.append(f"duplicate action policy: {name}")
            continue
        if not isinstance(version, str) or not version:
            failures.append(f"{name}: version label must be non-empty")
            continue
        if not isinstance(sha, str) or SHA_RE.fullmatch(sha) is None:
            failures.append(f"{name}: sha must be one lowercase 40-character commit")
            continue
        pins[name] = (sha, version)
    return pins, failures


def step_has_digest_error(lines: list[str], use_index: int) -> bool:
    use_line = lines[use_index]
    use_indent = len(use_line) - len(use_line.lstrip())
    step_indent = use_indent
    if not use_line.lstrip().startswith("- uses:"):
        for earlier in reversed(lines[:use_index]):
            stripped = earlier.lstrip()
            indent = len(earlier) - len(stripped)
            if stripped.startswith("- ") and indent < use_indent:
                step_indent = indent
                break

    for line in lines[use_index + 1 :]:
        stripped = line.lstrip()
        indent = len(line) - len(stripped)
        if stripped.startswith("- ") and indent == step_indent:
            break
        if stripped == "digest-mismatch: error":
            return True
    return False


def validate_workflow_text(
    path: str,
    text: str,
    pins: dict[str, tuple[str, str]],
    seen: set[str],
) -> list[str]:
    failures: list[str] = []
    lines = text.splitlines()
    for index, line in enumerate(lines):
        match = USE_RE.match(line)
        if match is None:
            continue
        target = match.group("target")
        if target.startswith("./"):
            continue
        if "@" not in target:
            failures.append(f"{path}:{index + 1}: external action has no selector")
            continue
        name, selector = target.rsplit("@", 1)
        expected = pins.get(name)
        if expected is None:
            failures.append(f"{path}:{index + 1}: unowned external action {name}")
            continue
        expected_sha, expected_version = expected
        if selector != expected_sha:
            failures.append(
                f"{path}:{index + 1}: {name} must use {expected_sha}, got {selector}"
            )
        if match.group("label") != expected_version:
            failures.append(
                f"{path}:{index + 1}: {name} comment must be {expected_version!r}"
            )
        if name == "actions/download-artifact" and not step_has_digest_error(
            lines, index
        ):
            failures.append(
                f"{path}:{index + 1}: download-artifact must fail on digest mismatch"
            )
        seen.add(name)
    return failures


def validate_repository(
    policy: dict[str, Any], pins: dict[str, tuple[str, str]]
) -> tuple[list[str], int]:
    failures: list[str] = []
    seen: set[str] = set()
    reference_count = 0
    roots = policy.get("maintained_workflow_roots", [])
    if not isinstance(roots, list) or len(roots) != len(set(roots)):
        return ["maintained_workflow_roots must be a unique list"], 0

    for relative_root in roots:
        if not isinstance(relative_root, str):
            failures.append("maintained workflow roots must be strings")
            continue
        workflow_dir = REPO_ROOT / relative_root / ".github/workflows"
        if not workflow_dir.is_dir():
            failures.append(
                f"maintained workflow root is unavailable: {relative_root}; "
                "initialize recursive submodules"
            )
            continue
        paths = sorted((*workflow_dir.glob("*.yml"), *workflow_dir.glob("*.yaml")))
        for path in paths:
            text = path.read_text(encoding="utf-8")
            path_failures = validate_workflow_text(
                str(path.relative_to(REPO_ROOT)), text, pins, seen
            )
            failures.extend(path_failures)
            reference_count += sum(
                1
                for line in text.splitlines()
                if (match := USE_RE.match(line)) is not None
                and not match.group("target").startswith("./")
            )

    unused = sorted(set(pins).difference(seen))
    if unused:
        failures.append(f"policy actions have no maintained consumer: {unused}")
    return failures, reference_count


def run_self_test() -> None:
    pins = {
        "actions/checkout": ("a" * 40, "v7.0.1"),
        "actions/download-artifact": ("b" * 40, "v8.0.1"),
    }
    valid = """steps:
  - uses: actions/checkout@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa # v7.0.1
  - uses: actions/download-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb # v8.0.1
    with:
      digest-mismatch: error
"""
    seen: set[str] = set()
    if validate_workflow_text("valid.yml", valid, pins, seen):
        raise SystemExit("GitHub Action pin self-test rejected valid pins")
    invalid = valid.replace("@" + "a" * 40, "@v7").replace(
        "      digest-mismatch: error\n", ""
    )
    failures = validate_workflow_text("invalid.yml", invalid, pins, set())
    if len(failures) != 2:
        raise SystemExit("GitHub Action pin self-test accepted mutable or weak pins")
    print("GitHub Action pin self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    policy = load_policy()
    pins, failures = policy_pins(policy)
    repository_failures, reference_count = validate_repository(policy, pins)
    failures.extend(repository_failures)
    if failures:
        print("GitHub Action pins: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(
        f"GitHub Action pins: PASS (actions={len(pins)}, references={reference_count})"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
