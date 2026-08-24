#!/usr/bin/env python3
"""Validate submodule ownership metadata and restoration entrypoints."""

from __future__ import annotations

import argparse
import configparser
import re
import tempfile
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class ExpectedSubmodule:
    path: str
    url: str
    branch: str


EXPECTED_SUBMODULES = [
    ExpectedSubmodule(
        "third_party/ruff",
        "https://github.com/sifr-lang/ruff.git",
        "sifr/0.16.4-maintenance",
    ),
    ExpectedSubmodule(
        "editor_integrations",
        "https://github.com/sifr-lang/editor-integrations.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/algorithmic_compatibility/corpora/leetcode",
        "https://github.com/sifr-lang/leetcode.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/package_management/corpora/demo_repositories/sifr-demo-json",
        "https://github.com/sifr-lang/sifr-demo-json.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/package_management/corpora/demo_repositories/sifr-demo-http",
        "https://github.com/sifr-lang/sifr-demo-http.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/package_management/corpora/demo_repositories/sifr-demo-test-support",
        "https://github.com/sifr-lang/sifr-demo-test-support.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/package_management/corpora/demo_repositories/sifr-demo-app",
        "https://github.com/sifr-lang/sifr-demo-app.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/package_management/corpora/demo_repositories/sifr-demo-workspace",
        "https://github.com/sifr-lang/sifr-demo-workspace.git",
        "main",
    ),
    ExpectedSubmodule(
        "verification/areas/developer_tooling/corpora/sifr-large-lsp-verification",
        "https://github.com/sifr-lang/sifr-large-lsp-verification.git",
        "main",
    ),
]

# These paths are called out separately from generic unclassified submodules so
# migrations regress with an owner-specific error message.
STALE_SUBMODULE_PATHS = {
    "audits/leetcode",
    "verification/sifr-large-lsp-verification",
    "verification/package_management/demo_repositories/sifr-demo-json",
    "verification/package_management/demo_repositories/sifr-demo-http",
    "verification/package_management/demo_repositories/sifr-demo-test-support",
    "verification/package_management/demo_repositories/sifr-demo-app",
    "verification/package_management/demo_repositories/sifr-demo-workspace",
}

WORKFLOW_STEP_RE = re.compile(r"(?ms)^(\s*)-\s+(?P<body>.*?)(?=^\1-\s+|\Z)")
CHECKOUT_USES_RE = re.compile(
    r"(?m)^\s*uses:\s+actions/checkout@[^\s#]+(?:\s+#.*)?\s*$"
)
SUBMODULES_RECURSIVE_RE = re.compile(r"(?m)^\s+submodules:\s+[\"']?recursive[\"']?\s*$")


def parse_gitmodules(text: str) -> dict[str, dict[str, str]]:
    parser = configparser.ConfigParser()
    parser.read_string(text)
    entries: dict[str, dict[str, str]] = {}
    for section in parser.sections():
        if not section.startswith("submodule "):
            continue
        path = parser.get(section, "path", fallback="")
        if path:
            entries[path] = {
                "url": parser.get(section, "url", fallback=""),
                "branch": parser.get(section, "branch", fallback=""),
            }
    return entries


def validate_gitmodules(text: str) -> list[str]:
    failures: list[str] = []
    try:
        entries = parse_gitmodules(text)
    except configparser.Error as error:
        return [f".gitmodules is not valid config: {error}"]

    expected_by_path = {entry.path: entry for entry in EXPECTED_SUBMODULES}
    for expected in EXPECTED_SUBMODULES:
        actual = entries.get(expected.path)
        if actual is None:
            failures.append(f"missing submodule entry: {expected.path}")
            continue
        if actual["url"] != expected.url:
            failures.append(
                f"submodule {expected.path} has unexpected url: {actual['url']}"
            )
        if actual["branch"] != expected.branch:
            failures.append(f"submodule {expected.path} must track {expected.branch}")

    for path in sorted(entries):
        if path in STALE_SUBMODULE_PATHS:
            failures.append(f"stale submodule path remains in .gitmodules: {path}")
        if path not in expected_by_path:
            failures.append(f"unclassified submodule path in .gitmodules: {path}")
    return failures


def validate_clone_script(text: str) -> list[str]:
    failures: list[str] = []
    for required in [
        "git submodule sync --recursive",
        "git submodule update --init --recursive",
        "git submodule update --init --recursive --remote",
        "git submodule status --recursive",
    ]:
        if required not in text:
            failures.append(f"scripts/clone_subrepos.sh missing `{required}`")
    return failures


def validate_workflow(path: Path, text: str) -> list[str]:
    failures: list[str] = []
    checkout_blocks = [
        match
        for match in WORKFLOW_STEP_RE.finditer(text)
        if CHECKOUT_USES_RE.search(match.group("body"))
    ]
    if not checkout_blocks:
        return failures
    display_path = path
    try:
        display_path = path.relative_to(REPO_ROOT)
    except ValueError:
        pass
    for index, match in enumerate(checkout_blocks, start=1):
        if not SUBMODULES_RECURSIVE_RE.search(match.group("body")):
            failures.append(
                f"{display_path} checkout #{index} does not initialize submodules recursively"
            )
    return failures


def validate_repo(root: Path) -> list[str]:
    failures: list[str] = []
    failures.extend(
        validate_gitmodules((root / ".gitmodules").read_text(encoding="utf-8"))
    )
    failures.extend(
        validate_clone_script(
            (root / "scripts" / "clone_subrepos.sh").read_text(encoding="utf-8")
        )
    )
    workflow_paths = sorted((root / ".github" / "workflows").glob("*.yml"))
    workflow_paths.extend(sorted((root / ".github" / "workflows").glob("*.yaml")))
    for path in workflow_paths:
        failures.extend(validate_workflow(path, path.read_text(encoding="utf-8")))
    return failures


def run_self_test() -> None:
    valid_gitmodules = "".join(
        f'[submodule "{entry.path}"]\n'
        f"\tpath = {entry.path}\n"
        f"\turl = {entry.url}\n"
        f"\tbranch = {entry.branch}\n"
        for entry in EXPECTED_SUBMODULES
    )
    if validate_gitmodules(valid_gitmodules):
        raise SystemExit(
            "submodule ownership self-test failed: valid metadata rejected"
        )
    missing_entry = valid_gitmodules.replace(
        f'[submodule "{EXPECTED_SUBMODULES[0].path}"]\n'
        f"\tpath = {EXPECTED_SUBMODULES[0].path}\n"
        f"\turl = {EXPECTED_SUBMODULES[0].url}\n"
        f"\tbranch = {EXPECTED_SUBMODULES[0].branch}\n",
        "",
    )
    if not any(
        "missing submodule entry" in failure
        for failure in validate_gitmodules(missing_entry)
    ):
        raise SystemExit("submodule ownership self-test failed: missing entry accepted")
    wrong_url = valid_gitmodules.replace(
        EXPECTED_SUBMODULES[0].url, "https://example.invalid/ruff.git", 1
    )
    if not any(
        "unexpected url" in failure for failure in validate_gitmodules(wrong_url)
    ):
        raise SystemExit("submodule ownership self-test failed: wrong url accepted")
    missing_branch = valid_gitmodules.replace("\tbranch = main\n", "", 1)
    if not any(
        "must track" in failure for failure in validate_gitmodules(missing_branch)
    ):
        raise SystemExit(
            "submodule ownership self-test failed: missing branch accepted"
        )
    stale_path = valid_gitmodules + (
        '[submodule "audits/leetcode"]\n'
        "\tpath = audits/leetcode\n"
        "\turl = https://github.com/sifr-lang/leetcode.git\n"
        "\tbranch = main\n"
    )
    if not any(
        "stale submodule path" in failure for failure in validate_gitmodules(stale_path)
    ):
        raise SystemExit("submodule ownership self-test failed: stale path accepted")
    unclassified_path = valid_gitmodules + (
        '[submodule "verification/new-corpus"]\n'
        "\tpath = verification/new-corpus\n"
        "\turl = https://github.com/sifr-lang/new-corpus.git\n"
        "\tbranch = main\n"
    )
    if not any(
        "unclassified submodule path" in failure
        for failure in validate_gitmodules(unclassified_path)
    ):
        raise SystemExit(
            "submodule ownership self-test failed: unclassified path accepted"
        )
    valid_clone_script = (
        "git submodule sync --recursive\n"
        "git submodule update --init --recursive\n"
        "git submodule update --init --recursive --remote\n"
        "git submodule status --recursive"
    )
    if validate_clone_script(valid_clone_script):
        raise SystemExit(
            "submodule ownership self-test failed: valid clone script rejected"
        )
    invalid_clone_script = valid_clone_script.replace(
        "git submodule status --recursive", ""
    )
    if not any(
        "missing" in failure for failure in validate_clone_script(invalid_clone_script)
    ):
        raise SystemExit(
            "submodule ownership self-test failed: incomplete clone script accepted"
        )
    workflow = "- uses: actions/checkout@v5\n"
    with tempfile.TemporaryDirectory() as tmpdir:
        path = Path(tmpdir) / "workflow.yml"
        if not validate_workflow(path, workflow):
            raise SystemExit(
                "submodule ownership self-test failed: checkout without submodules accepted"
            )
        pinned_checkout = (
            "- uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 "
            "# v7.0.1\n"
        )
        if not validate_workflow(path, pinned_checkout):
            raise SystemExit(
                "submodule ownership self-test failed: commented pin bypassed validation"
            )
        named_checkout = "- name: Checkout\n  uses: actions/checkout@0123456789abcdef\n"
        if not validate_workflow(path, named_checkout):
            raise SystemExit(
                "submodule ownership self-test failed: named checkout without submodules accepted"
            )
        valid_workflow = (
            "- name: Checkout\n"
            "  uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1\n"
            "  with:\n"
            "    submodules: 'recursive'\n"
        )
        if validate_workflow(path, valid_workflow):
            raise SystemExit(
                "submodule ownership self-test failed: valid workflow rejected"
            )
    print("submodule ownership self-test: PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        run_self_test()
        return 0

    failures = validate_repo(REPO_ROOT)
    if failures:
        print("submodule ownership guardrail: FAIL")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("submodule ownership guardrail: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
