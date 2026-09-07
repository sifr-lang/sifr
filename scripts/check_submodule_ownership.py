#!/usr/bin/env python3
"""Validate submodule ownership metadata and restoration entrypoints."""

from __future__ import annotations

import argparse
import configparser
import json
import subprocess
import textwrap
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


@dataclass(frozen=True)
class ExpectedSubmodule:
    path: str
    url: str
    branch: str | None


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
    *[
        ExpectedSubmodule(
            f"third_party/libpg_query/{major}",
            "https://github.com/pganalyze/libpg_query.git",
            None,
        )
        for major in range(13, 19)
    ],
    ExpectedSubmodule(
        "third_party/wasi-virt",
        "https://github.com/bytecodealliance/WASI-Virt.git",
        None,
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

# Use the same Ruby/Psych parser as the release workflow contracts, without
# introducing a Python environment/lock dependency. Never execute YAML tags.
WORKFLOW_YAML_PARSER = r"""
require "yaml"
require "json"

def validate_keys(node)
  if node.is_a?(Psych::Nodes::Mapping)
    keys = node.children.each_slice(2).map do |key, _|
      abort "workflow mapping keys must be scalars" unless key.is_a?(Psych::Nodes::Scalar)
      key.value
    end
    abort "duplicate workflow mapping key" unless keys.uniq.length == keys.length
  end
  (node.children || []).each { |child| validate_keys(child) }
end

begin
  text = STDIN.read
  tree = YAML.parse_stream(text)
  abort "expected one workflow document" unless tree.children.length == 1
  validate_keys(tree)
  workflow = YAML.safe_load(text, permitted_classes: [], permitted_symbols: [], aliases: true)
  puts JSON.generate(workflow)
rescue Psych::Exception, JSON::JSONError, SystemStackError => error
  abort "invalid workflow YAML: #{error.message}"
end
"""


@dataclass(frozen=True)
class NonSourceCheckout:
    workflow: str
    job: str
    condition: str
    inputs: dict[str, object]
    purpose: str


# The publication root executes first-party governance scripts and consumes
# already-built release artifacts. Only stable-source builds/publishes the
# editor and needs the submodule graph. Evidence trees provide JSON records.
# Match the full input identity, job, and condition, never a step name or ordinal.
NON_SOURCE_CHECKOUTS = (
    NonSourceCheckout(
        "release-publication.yml", "publish",
        "env.STABLE_MUTATION_OPERATION != 'true'",
        {"ref": "${{ inputs.source_commit }}", "fetch-depth": 0,
         "persist-credentials": False},
        "preview/bootstrap governance and prebuilt release artifacts",
    ),
    NonSourceCheckout(
        "release-publication.yml", "publish",
        "env.STABLE_MUTATION_OPERATION == 'true'",
        {"fetch-depth": 0, "persist-credentials": False},
        "stable publication governance scripts at the workflow revision",
    ),
    NonSourceCheckout(
        "release-publication.yml", "publish",
        "env.STABLE_CANDIDATE_OPERATION == 'true'",
        {"ref": "${{ inputs.evidence_commit }}", "fetch-depth": 0,
         "path": "stable-evidence", "persist-credentials": False},
        "stable candidate evidence records",
    ),
    NonSourceCheckout(
        "release-publication.yml", "publish",
        "env.INCIDENT_OPERATION == 'true'",
        {"ref": "${{ inputs.incident_commit }}", "fetch-depth": 0,
         "path": "incident-evidence", "persist-credentials": False},
        "incident evidence records",
    ),
    NonSourceCheckout(
        "release-publication-prepare.yml", "prepare",
        "${{ inputs.governance_mode == 'ga-activation' || "
        "inputs.governance_mode == 'normal' || "
        "inputs.governance_mode == 'incident-roll-forward' }}",
        {"ref": "${{ inputs.evidence_commit }}", "fetch-depth": 0,
         "path": "stable-evidence", "persist-credentials": False},
        "stable candidate evidence records for preparation",
    ),
    NonSourceCheckout(
        "release-publication-prepare.yml", "prepare",
        "${{ inputs.governance_mode == 'rollback' || "
        "inputs.governance_mode == 'incident-roll-forward' }}",
        {"ref": "${{ inputs.incident_commit }}", "fetch-depth": 0,
         "path": "incident-evidence", "persist-credentials": False},
        "incident evidence records for preparation",
    ),
)


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
        if expected.branch is None:
            if actual["branch"]:
                failures.append(
                    f"submodule {expected.path} must remain commit-pinned without branch tracking"
                )
        elif actual["branch"] != expected.branch:
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


def validate_workflow(path: Path, text: str, root: Path = REPO_ROOT) -> list[str]:
    display_path = path
    try:
        display_path = path.relative_to(root)
    except ValueError:
        pass
    try:
        parsed = subprocess.run(
            ["ruby", "-e", WORKFLOW_YAML_PARSER], input=text,
            text=True, capture_output=True, timeout=30, check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return [f"{display_path} cannot parse workflow YAML with Ruby: {error}"]
    if parsed.returncode:
        return [f"{display_path} cannot parse workflow YAML: {parsed.stderr.strip()}"]
    try:
        workflow = json.loads(parsed.stdout)
    except ValueError as error:
        return [f"{display_path} invalid YAML parser response: {error}"]
    if not isinstance(workflow, dict) or not isinstance(workflow.get("jobs"), dict):
        return [f"{display_path} workflow must contain a jobs mapping"]

    failures: list[str] = []
    for job_id, job in workflow["jobs"].items():
        location = f"{display_path} job {job_id}"
        if not isinstance(job, dict):
            failures.append(f"{location} must be a mapping")
            continue
        if "steps" not in job and isinstance(job.get("uses"), str):
            continue  # Reusable workflow call; no local steps to inspect.
        steps = job.get("steps")
        if not isinstance(steps, list):
            failures.append(f"{location} must contain a steps sequence")
            continue
        for index, step in enumerate(steps, start=1):
            step_location = f"{location} step #{index}"
            if not isinstance(step, dict):
                failures.append(f"{step_location} must be a mapping")
                continue
            uses = step.get("uses", "")
            if not isinstance(uses, str):
                failures.append(f"{step_location} uses must be a string")
                continue
            if not uses.strip().lower().startswith("actions/checkout@"):
                continue
            inputs = step.get("with", {})
            if not isinstance(inputs, dict):
                failures.append(f"{step_location} checkout with must be a mapping")
                continue
            if inputs.get("submodules") == "recursive":
                continue
            identity = {key: value for key, value in inputs.items() if key != "submodules"}
            if any(
                display_path == Path(".github/workflows") / owner.workflow
                and job_id == owner.job and step.get("if") == owner.condition
                and identity == owner.inputs
                for owner in NON_SOURCE_CHECKOUTS
            ):
                continue
            failures.append(
                f"{step_location} checkout does not initialize submodules recursively "
                "and is not a classified governance/evidence checkout"
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
        failures.extend(validate_workflow(path, path.read_text(encoding="utf-8"), root))
    return failures


def run_workflow_self_tests() -> None:
    path = REPO_ROOT / ".github/workflows/test.yml"

    def workflow(steps: str) -> str:
        return "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n" + textwrap.indent(
            steps, "      "
        )

    def check(label: str, text: str, *, accepted: bool = False, at: Path = path) -> None:
        failures = validate_workflow(at, text)
        if bool(failures) == accepted:
            raise SystemExit(f"submodule ownership self-test failed: {label}: {failures}")

    for label, step in (
        ("unnamed checkout", "- uses: actions/checkout@v7\n"),
        ("named checkout", "- name: Checkout\n  uses: actions/checkout@v7\n"),
        ("commented pin", "- uses: actions/checkout@0123456789abcdef # v7\n"),
        ("named commented pin", "- name: Checkout\n  uses: actions/checkout@abc # v7\n"),
        ("quoted action", '- uses: "actions/checkout@v7"\n'),
        ("flow style", "- {uses: actions/checkout@v7}\n"),
        ("folded action", "- uses: >-\n    actions/checkout@v7\n"),
        ("case insensitive action", "- uses: Actions/Checkout@v7\n"),
    ):
        check(label, workflow(step))
        if "flow" not in label:
            check(label + " recursive", workflow(step + "  with:\n    submodules: recursive\n"),
                  accepted=True)
    check("recursive flow style", workflow(
        "- {uses: 'actions/checkout@v7', with: {submodules: recursive}}\n"
    ), accepted=True)
    check("inline comment on recursive", workflow(
        "- uses: actions/checkout@v7\n  with:\n    submodules: 'recursive' # restore all\n"
    ), accepted=True)
    for value in ("true", "false", "null", "[recursive]", "'${{ inputs.submodules }}'"):
        check("nonliteral recursive " + value, workflow(
            f"- uses: actions/checkout@v7\n  with:\n    submodules: {value}\n"
        ))
    check("neighbor step cannot supply recursive", workflow(
        "- uses: actions/checkout@v7\n"
        "- uses: unrelated/action@v1\n  with:\n    submodules: recursive\n"
    ))
    check("run block cannot supply recursive", workflow(
        "- uses: actions/checkout@v7\n- run: |\n    submodules: recursive\n"
    ))
    check("job env cannot supply recursive", workflow(
        "- uses: actions/checkout@v7\n"
    ).replace("    steps:", "    env:\n      submodules: recursive\n    steps:"))
    check("step env cannot supply recursive", workflow(
        "- uses: actions/checkout@v7\n  env:\n    submodules: recursive\n"
    ))
    check("commented recursive cannot supply input", workflow(
        "- uses: actions/checkout@v7\n  # with: {submodules: recursive}\n"
    ))
    check("all jobs inspected", workflow(
        "- uses: actions/checkout@v7\n  with: {submodules: recursive}\n"
    ) + "  other:\n    steps:\n      - uses: actions/checkout@v7\n")
    check("literal checkout text is not a step", workflow(
        "- run: |\n    uses: actions/checkout@v7\n"
        "    - uses: actions/checkout@v7\n"
        "# - uses: actions/checkout@v7\n"
    ), accepted=True)
    check("reusable workflow", "jobs:\n  test:\n    uses: ./.github/workflows/build.yml\n",
          accepted=True)
    check("anchored checkout", workflow(
        "- &source\n  uses: actions/checkout@v7\n  with: {submodules: recursive}\n"
        "- *source\n"
    ), accepted=True)
    check("anchored missing recursive", workflow(
        "- &source\n  uses: actions/checkout@v7\n- *source\n"
    ))
    for label, invalid in (
        ("malformed YAML", "jobs: ["),
        ("multiple documents", "jobs: {}\n---\njobs: {}\n"),
        ("nonmapping workflow", "- uses: actions/checkout@v7\n"),
        ("missing jobs", "name: workflow\n"),
        ("nonmapping jobs", "jobs: []\n"),
        ("nonmapping job", "jobs: {test: false}\n"),
        ("missing steps", "jobs: {test: {runs-on: ubuntu-latest}}\n"),
        ("nonsequence steps", "jobs: {test: {steps: {uses: actions/checkout@v7}}}\n"),
        ("nonmapping step", workflow("- invalid\n")),
        ("nonstring uses", workflow("- uses: [actions/checkout@v7]\n")),
        ("nonmapping with", workflow("- uses: actions/checkout@v7\n  with: recursive\n")),
        ("duplicate uses", workflow("- uses: actions/checkout@v7\n  uses: unrelated/action@v1\n")),
        ("unsafe YAML tag", "jobs: !ruby/object:Object {}\n"),
    ):
        check(label, invalid)

    for owner in NON_SOURCE_CHECKOUTS:
        at = REPO_ROOT / ".github/workflows" / owner.workflow
        step = {"uses": "actions/checkout@v7", "if": owner.condition, "with": owner.inputs}

        def document(candidate: dict[str, object], job: str = owner.job) -> str:
            # JSON is a YAML subset and keeps exact expressions quoted.
            return json.dumps({"jobs": {job: {"steps": [candidate]}}})

        check(owner.purpose, document(step), accepted=True, at=at)
        check("wrong workflow: " + owner.purpose, document(step))
        check("wrong job: " + owner.purpose, document(step, "other"), at=at)
        check("wrong condition: " + owner.purpose, document({**step, "if": "true"}), at=at)
        for key, value in (
            ("ref", "${{ inputs.source_commit }}" if "evidence" in owner.purpose else "other"),
            ("path", "stable-source"),
            ("repository", "other/source"),
        ):
            check("changed " + key + ": " + owner.purpose, document({
                **step, "with": {**owner.inputs, key: value},
            }), at=at)
        check("name cannot grant classification: " + owner.purpose, document({
            "uses": "actions/checkout@v7", "name": owner.purpose,
        }), at=at)


def run_self_test() -> None:
    valid_gitmodules = "".join(
        f'[submodule "{entry.path}"]\n'
        f"\tpath = {entry.path}\n"
        f"\turl = {entry.url}\n"
        + (f"\tbranch = {entry.branch}\n" if entry.branch is not None else "")
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
    pinned = next(entry for entry in EXPECTED_SUBMODULES if entry.branch is None)
    tracked_pin = valid_gitmodules.replace(
        f"\turl = {pinned.url}\n",
        f"\turl = {pinned.url}\n\tbranch = main\n",
        1,
    )
    if not any(
        "must remain commit-pinned" in failure
        for failure in validate_gitmodules(tracked_pin)
    ):
        raise SystemExit("submodule ownership self-test failed: tracked pin accepted")
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
    run_workflow_self_tests()
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
