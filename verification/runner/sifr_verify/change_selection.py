"""Commit-bound selection of the registered core or complete merge coverage."""
from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import PurePosixPath, Path

from .errors import VerificationError
from .paths import REPO_ROOT
from .profiles import build_profile_plan, load_profile

SHA = re.compile(r"[0-9a-f]{40}")
SHARED = ("crates/", "stdlib/", "vendor/", "verification/runner/", "verification/profiles/",
          "verification/schemas/", "verification/contracts/", ".github/", "scripts/", ".cargo/")
SHARED_FILES = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".gitmodules", "sysroot.toml",
                "verification/pyproject.toml", "verification/uv.lock", "AGENTS.md"}


# Finite reverse-consumer closure, not a documentation-prefix exemption. These
# checks remain in the core: taxonomy, stale-drafts and HIR/driver guards. The
# exact active plan is prose; release requests and unproven records stay broad.
CORE_PROSE = frozenset({
    "README.md",
    "internal_docs/hir_maintainability_guardrails.md",
    "internal_docs/sifr_driver_maintainability_guardrails.md",
    "plans/issues/active/ad-hoc-validation-contracts-and-resource-aware-execution.md",
})


class SelectionError(VerificationError):
    """Selected execution is not bound to the observed committed checkout."""


def classify(paths: list[str] | None, *, error: str | None = None,
             unsafe_changes: tuple[str, ...] = ()) -> dict:
    reasons = ["registered create-pr feedback core retained in full"]
    broad = paths is None or error is not None
    if broad:
        reasons.append("unavailable diff: conservative merge coverage")
    if unsafe_changes:
        broad = True
        reasons.extend(f"non-content change requires merge coverage: {path}" for path in unsafe_changes)
    for path in sorted(set(paths or [])):
        parts = PurePosixPath(path).parts
        if (not path or path.startswith("/") or ".." in parts or "\\" in path
                or PurePosixPath(path).as_posix() != path
                or any(ord(char) < 32 or ord(char) == 127 for char in path)
                or path in SHARED_FILES or path.startswith(SHARED)):
            broad = True
            reasons.append(f"shared or invalid input: {path}")
        elif path in CORE_PROSE:
            reasons.append(f"all declared prose consumers retained in mandatory core: {path}")
        elif len(parts) >= 4 and parts[:2] == ("verification", "areas"):
            broad = True
            reasons.append(f"area input has specialist or unproven reverse consumers: {path}")
        else:
            broad = True
            reasons.append(f"unknown input requires merge coverage: {path}")
    profile = "merge" if broad else "create-pr"
    selected = load_profile(profile)
    return {"schema_version": 1, "profile": profile, "reasons": reasons,
            "profile_plan": build_profile_plan(profile),
            "guardrail_jobs": [{"id": job, "reason": "mandatory canonical guardrail"}
                               for job in selected["guardrail_steps"]],
            "toolchain_jobs": [{"id": job, "reason": "complete canonical " + profile + " coverage"}
                               for job in selected["toolchain_steps"]],
            "changed_paths": sorted(set(paths or [])), "diff_available": not (paths is None or error),
            "execution_state": "not-executed", "error": error,
            "selected_jobs": [{"area": row["area"], "suites": row["suites"],
                               "reason": "complete canonical " + profile + " selection"}
                              for row in selected["selected_areas"]]}


def git(repo: Path, *args: str) -> bytes:
    return subprocess.check_output(["git", *args], cwd=repo, stderr=subprocess.PIPE)


def selection(repo: Path, base: str, head: str) -> dict:
    paths, error, unsafe_changes = None, None, ()
    try:
        for commit in (base, head):
            if not SHA.fullmatch(commit) or git(repo, "cat-file", "-t", commit).strip() != b"commit":
                raise ValueError("base/head must identify full existing commit objects")
        raw = git(repo, "diff", "--no-renames", "--raw", "--no-abbrev", "-z", base, head, "--")
        fields = raw.split(b"\0")
        if fields[-1] != b"" or (len(fields) - 1) % 2:
            raise ValueError("incomplete raw diff")
        paths, unsafe = [], []
        for index in range(0, len(fields) - 1, 2):
            metadata = fields[index].decode("ascii").split()
            if len(metadata) != 5 or not metadata[0].startswith(":"):
                raise ValueError("invalid raw diff metadata")
            path = fields[index + 1].decode("utf-8", "strict")
            paths.append(path)
            # Add/delete, executable modes, symlinks and gitlinks are broader
            # than the reviewed regular-file content closure, even at a known path.
            if (metadata[0], metadata[1], metadata[4]) != (":100644", "100644", "M"):
                unsafe.append(path)
        unsafe_changes = tuple(unsafe)
    except (OSError, ValueError, subprocess.CalledProcessError) as failure:
        error = type(failure).__name__ + ": cannot establish complete commit diff"
    result = classify(paths, error=error, unsafe_changes=unsafe_changes)
    return result | {"base_commit": base, "candidate_commit": head}


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="sifr_verify changes")
    parser.add_argument("command", choices=("plan", "run"))
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--compact-resources", action="store_true",
                        help="Run using the canonical compact resource policy (run only).")
    args = parser.parse_args(argv)
    if args.command == "plan" and args.compact_resources:
        parser.error("--compact-resources applies only to changes run")
    result = selection(REPO_ROOT, args.base, args.head)
    print(json.dumps(result, indent=2, sort_keys=True), flush=True)
    if args.command == "plan":
        return 0
    # A failed diff broadens coverage; an unbound candidate cannot execute under
    # a falsely named identity. Never run against uncommitted source changes.
    if (not SHA.fullmatch(args.head) or git(REPO_ROOT, "rev-parse", "HEAD").decode().strip() != args.head
            or git(REPO_ROOT, "status", "--porcelain", "--untracked-files=all")):
        raise SelectionError("changes run requires the exact clean candidate checkout")
    from .profile_runner import run_profile
    return run_profile(result["profile"], ["--compact-resources"] if args.compact_resources else [])
