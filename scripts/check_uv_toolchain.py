#!/usr/bin/env python3
"""Check maintained uv selections without installing uv or accessing the network.

Root-owned projects/workflows are discovered from Git, including new files.
Fork-owned toolchain adoption belongs to its gitlink update, not this root
invariant. Fixture projects and vendored/upstream projects are not toolchains.
Ruby/Psych is the existing workflow-parser prerequisite; no uv environment is
needed to validate the tool that would create that environment.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import tomllib
from pathlib import Path, PurePosixPath

from check_submodule_ownership import WORKFLOW_YAML_PARSER

REPO_ROOT = Path(__file__).resolve().parents[1]
CANONICAL_PROJECT = "verification/pyproject.toml"
EXCLUDED_PARTS = {
    "vendor", "third_party", "fixtures", "corpora", "snapshots", "target",
    ".venv", "plans",
}
# Release-qualified archive digest retained from Item 3 / PR #3495:
# https://github.com/astral-sh/uv/releases/download/0.12.5/
# uv-x86_64-unknown-linux-gnu.tar.gz.sha256
# A version/platform change must add its qualified digest explicitly. Never
# reuse an old release's digest or infer one from the workflow being checked.
ARCHIVE_CHECKSUMS = {
    ("0.12.5", "x86_64-unknown-linux-gnu"):
        "68a509da24b06b4223a1c0175fb5eb5bc79342b76cbeff0cfe51ac3f5b17b6b2",
}
# Only qualified hosted runner identities are accepted. Expressions, custom
# labels, and new platforms fail clearly until their archive is qualified.
RUNNER_PLATFORMS = {
    "ubuntu-24.04": "x86_64-unknown-linux-gnu",
    "ubuntu-24.04-arm": "aarch64-unknown-linux-gnu",
    "macos-15": "aarch64-apple-darwin",
    "windows-2025": "x86_64-pc-windows-msvc",
}


def maintained(path: str) -> bool:
    return not EXCLUDED_PARTS.intersection(PurePosixPath(path).parts)


def discover_inputs(root: Path) -> dict[str, str]:
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root, text=True, capture_output=True, check=True,
    )
    inputs = {}
    for name in sorted(set(result.stdout.split("\0")) - {""}):
        path = PurePosixPath(name)
        if not maintained(name):
            continue
        if path.name not in {"pyproject.toml", "uv.toml", "uv.lock", ".uv-version"} and not (
            name.startswith(".github/workflows/") and path.suffix in {".yml", ".yaml"}
        ):
            continue
        local = root / name
        if local.is_symlink():
            raise ValueError(f"{name}: toolchain inputs must not be symlinks")
        # A deleted tracked file must remain visible to validation.
        inputs[name] = local.read_text(encoding="utf-8")
    return inputs


def parse_workflows(inputs: dict[str, str]) -> dict[str, object]:
    workflows = {}
    for name, text in inputs.items():
        if not name.startswith(".github/workflows/"):
            continue
        result = subprocess.run(
            ["ruby", "-e", WORKFLOW_YAML_PARSER], input=text, text=True,
            capture_output=True, timeout=30, check=False,
        )
        if result.returncode:
            raise ValueError(f"{name}: invalid workflow YAML: {result.stderr.strip()}")
        workflows[name] = json.loads(result.stdout)
    return workflows


def project_pins(inputs: dict[str, str]) -> tuple[dict[str, str], list[str]]:
    pins = {}
    failures = []
    for name, text in inputs.items():
        if PurePosixPath(name).name not in {"pyproject.toml", "uv.toml", ".uv-version"}:
            continue
        try:
            if name.endswith(".uv-version"):
                requirement = "==" + text.strip()
            else:
                document = tomllib.loads(text)
                config = document if name.endswith("/uv.toml") or name == "uv.toml" else (
                    document.get("tool", {}).get("uv", {})
                )
                requirement = config.get("required-version")
            if not isinstance(requirement, str) or not re.fullmatch(r"==\d+\.\d+\.\d+", requirement):
                failures.append(f"{name}: uv required-version must be an exact ==X.Y.Z pin")
            else:
                pins[name] = requirement[2:]
        except (ValueError, AttributeError) as error:
            failures.append(f"{name}: invalid toolchain configuration: {error}")
    for name in inputs:
        if PurePosixPath(name).name == "uv.lock":
            project = str(PurePosixPath(name).with_name("pyproject.toml"))
            if project not in inputs:
                failures.append(f"{name}: missing maintained project {project}")
    if CANONICAL_PROJECT not in pins:
        failures.append(f"{CANONICAL_PROJECT}: missing canonical exact uv pin")
    else:
        expected = pins[CANONICAL_PROJECT]
        for name, version in pins.items():
            if version != expected:
                failures.append(f"{name}: uv {version} disagrees with canonical {expected}")
    return pins, failures


def validate(inputs: dict[str, str], workflows: dict[str, object]) -> tuple[list[str], int]:
    pins, failures = project_pins(inputs)
    expected = pins.get(CANONICAL_PROJECT)
    setups = 0
    for name, document in workflows.items():
        if not isinstance(document, dict) or not isinstance(document.get("jobs"), dict):
            failures.append(f"{name}: workflow must contain a jobs mapping")
            continue
        for job_id, job in document["jobs"].items():
            location = f"{name} job {job_id}"
            if not isinstance(job, dict):
                failures.append(f"{location}: job must be a mapping")
                continue
            if "uses" in job and "steps" not in job:
                continue
            steps = job.get("steps")
            if not isinstance(steps, list):
                failures.append(f"{location}: steps must be a sequence")
                continue
            for index, step in enumerate(steps, 1):
                label = f"{location} step {index}"
                if not isinstance(step, dict) or not isinstance(step.get("uses", ""), str):
                    failures.append(f"{label}: malformed step/action")
                    continue
                action = step.get("uses", "").strip().lower().split("@", 1)[0]
                if action != "astral-sh/setup-uv":
                    continue
                setups += 1
                settings = step.get("with")
                if not isinstance(settings, dict):
                    failures.append(f"{label}: setup-uv requires version-file and checksum inputs")
                    continue
                reference = settings.get("version-file")
                if reference != CANONICAL_PROJECT:
                    failures.append(f"{label}: version-file must reference {CANONICAL_PROJECT}")
                if "version" in settings:
                    failures.append(f"{label}: version overrides the canonical version-file selection")
                if "working-directory" in settings:
                    failures.append(f"{label}: working-directory can redirect the version-file")
                runner = job.get("runs-on")
                platform = RUNNER_PLATFORMS.get(runner) if isinstance(runner, str) else None
                if platform is None:
                    failures.append(f"{label}: unsupported runner platform {runner!r}; qualify it explicitly")
                    continue
                digest = ARCHIVE_CHECKSUMS.get((expected, platform))
                if digest is None:
                    failures.append(f"{label}: missing platform checksum for uv {expected} / {platform}")
                elif settings.get("checksum") != digest:
                    failures.append(f"{label}: checksum must match uv {expected} / {platform}: {digest}")
    if not setups:
        failures.append("no maintained setup-uv steps discovered")
    return failures, setups


def self_test() -> None:
    import copy
    import tempfile

    version, platform = next(iter(ARCHIVE_CHECKSUMS))
    pin = f'[tool.uv]\nrequired-version = "=={version}"\n'
    inputs = {CANONICAL_PROJECT: pin, "demos/new_project/pyproject.toml": pin}
    settings = {"version-file": CANONICAL_PROJECT, "checksum": ARCHIVE_CHECKSUMS[(version, platform)]}
    step = {"uses": "astral-sh/setup-uv@immutable", "with": settings}
    workflow = {"jobs": {"check": {"runs-on": "ubuntu-24.04", "steps": [step]}}}
    name = ".github/workflows/example.yml"
    workflows = {name: workflow}
    checks = 0

    def check(data: dict[str, str], documents: dict[str, object], diagnostic: str | None = None) -> None:
        nonlocal checks
        failures, _ = validate(data, documents)
        assert (not failures) if diagnostic is None else any(diagnostic in f for f in failures), failures
        checks += 1

    check(inputs, workflows)
    for path in inputs:
        for replacement, diagnostic in [
            ('[tool.uv]\nrequired-version = ">=0.1.0"', "exact"),
            ('[tool.uv]\nrequired-version = "==99.0.0"', "disagrees"),
            ("[tool.uv]", "exact"),
            ('[tool.uv]\nrequired-version = 1', "exact"),
            ("invalid = [", "invalid"),
        ]:
            check({**inputs, path: replacement}, workflows, diagnostic)
    check({"demos/new_project/pyproject.toml": pin}, workflows, "canonical")
    check({**inputs, "new/uv.lock": ""}, workflows, "missing maintained project")
    check({**inputs, "new/uv.toml": 'required-version = "==99.0.0"'}, workflows, "disagrees")
    check({**inputs, ".uv-version": "99.0.0"}, workflows, "disagrees")
    for key, value, diagnostic in [
        ("version-file", "missing/pyproject.toml", "version-file"),
        ("version-file", "${{ env.PROJECT }}", "version-file"),
        ("version-file", "../verification/pyproject.toml", "version-file"),
        ("version", version, "overrides"),
        ("working-directory", "another", "redirect"),
        ("checksum", "a" * 64, "checksum"),
        ("checksum", "${{ env.CHECKSUM }}", "checksum"),
    ]:
        changed = copy.deepcopy(workflows)
        changed[name]["jobs"]["check"]["steps"][0]["with"][key] = value
        check(inputs, changed, diagnostic)
    for key in settings:
        changed = copy.deepcopy(workflows)
        del changed[name]["jobs"]["check"]["steps"][0]["with"][key]
        check(inputs, changed, key)
    for runner, diagnostic in [
        ("ubuntu-24.04-arm", "missing platform checksum"),
        ("macos-15", "missing platform checksum"),
        ("windows-2025", "missing platform checksum"),
        ("ubuntu-future", "unsupported runner"),
        ("${{ matrix.os }}", "unsupported runner"),
        (["self-hosted", "linux"], "unsupported runner"),
        (None, "unsupported runner"),
    ]:
        changed = copy.deepcopy(workflows)
        changed[name]["jobs"]["check"]["runs-on"] = runner
        check(inputs, changed, diagnostic)
    check({key: pin.replace(version, "99.0.0") for key in inputs}, workflows, "missing platform checksum")
    check(inputs, {}, "no maintained setup-uv")
    for bad in [None, [], {"jobs": []}, {"jobs": {"bad": None}}, {"jobs": {"bad": {"steps": "text"}}}]:
        check(inputs, {name: bad}, ":")

    # Parse real YAML syntax, not a regex representation of steps. A fake
    # checksum in neighboring run text must never satisfy the setup input.
    yaml = f"""jobs:
  check:
    runs-on: ubuntu-24.04
    steps:
      - name: setup
        uses: 'astral-sh/setup-uv@immutable' # comment
        with: {{version-file: {CANONICAL_PROJECT}, checksum: {settings['checksum']}}}
"""
    check(inputs, parse_workflows({name: yaml}))
    check(inputs, parse_workflows({name: yaml.replace(f", checksum: {settings['checksum']}", "") +
          f"      - run: |\n          echo 'checksum: {settings['checksum']}'\n"}), "checksum")
    for invalid in ["jobs: [", "jobs: {}\njobs: {}", "---\njobs: {}\n---\njobs: {}"]:
        try:
            parse_workflows({name: invalid})
        except ValueError:
            checks += 1
        else:
            raise AssertionError("invalid YAML accepted")

    # Use a private miniature repository so actual discovery, newly added
    # owners, excluded upstream trees, and missing tracked files are tested.
    with tempfile.TemporaryDirectory(prefix="sifr-uv-selftest-") as directory:
        root = Path(directory)
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        files = {**inputs, name: yaml, "new/pyproject.toml": pin,
                 "vendor/pyproject.toml": "invalid", "third_party/ruff/pyproject.toml": "invalid",
                 "verification/fixtures/example/pyproject.toml": "invalid"}
        for path, text in files.items():
            target = root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(text, encoding="utf-8")
        discovered = discover_inputs(root)
        assert set(discovered) == {*inputs, name, "new/pyproject.toml"}, discovered
        check(discovered, parse_workflows(discovered))
        (root / "new/pyproject.toml").write_text(pin.replace(version, "99.0.0"), encoding="utf-8")
        check(discover_inputs(root), workflows, "disagrees")
        subprocess.run(["git", "-C", str(root), "add", "new/pyproject.toml"], check=True)
        (root / "new/pyproject.toml").unlink()
        try:
            discover_inputs(root)
        except FileNotFoundError:
            checks += 1
        else:
            raise AssertionError("deleted tracked project accepted")
    print(f"uv toolchain self-test passed ({checks} checks)")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    try:
        inputs = discover_inputs(REPO_ROOT)
        failures, setups = validate(inputs, parse_workflows(inputs))
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        failures, setups = [str(error)], 0
    if failures:
        for failure in failures:
            print(f"error: {failure}")
        return 1
    pins, _ = project_pins(inputs)
    print(f"uv {pins[CANONICAL_PROJECT]} toolchain invariant passed "
          f"({len(pins)} exact pins, {setups} setup-uv steps)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
