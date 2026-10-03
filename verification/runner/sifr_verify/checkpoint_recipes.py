"""Only audited recipes may consume input-bound local correctness evidence."""
from __future__ import annotations

import argparse
from datetime import UTC, datetime
import json
import os
from pathlib import Path
import re
import shutil
import sys
import uuid

from .checkpoint_runtime import isolated_python, stdlib_identity
from .correctness_checkpoints import Checkpoints, Completed, UnknownClosure
from .execution_evidence import write_evidence
from .execution_identity import EvidenceError, artifact_identity, digest, execution_key
from .paths import REPO_ROOT
from .profile_commands import run_command
from .schemas import load_json, load_schema, validate_data


POLICY = REPO_ROOT / "verification/policy/correctness_checkpoints.json"
SUPPORTED = {"hir-maintainability": "scripts/check_hir_maintainability_guardrails.py"}
REQUIRED_PATHS = {"hir-maintainability": ["internal_docs/hir_maintainability_guardrails.md",
    "crates/sifr_lowering/src/lower.rs", "crates/sifr_lowering/src/stdlib.rs"]}


def load_policy(path: Path = POLICY) -> dict:
    policy = load_json(path)
    validate_data(policy, load_schema("correctness_checkpoints.schema.json"), source=str(path))
    for field in ("max_age_seconds", "max_retained_bytes", "disk_reserve_bytes"):
        if type(policy[field]) is not int or policy[field] <= 0:
            raise EvidenceError("checkpoint policy budgets must be positive integers")
    if policy["disk_reserve_bytes"] < 8 * 1024**3:
        raise EvidenceError("checkpoint writes must preserve the cloud disk reserve")
    seen = set()
    for recipe in policy["recipes"]:
        if (recipe["guard"] not in SUPPORTED or recipe["script"] != SUPPORTED[recipe["guard"]]
                or recipe["id"] != "guard/" + recipe["guard"] or recipe["id"] in seen
                or recipe["consumed_paths"] != REQUIRED_PATHS[recipe["guard"]]
                or not re.fullmatch(r"[0-9a-f]{64}", recipe["audited_script_sha256"])):
            raise EvidenceError("checkpoint recipe has no unique audited implementation")
        seen.add(recipe["id"])
    return policy


def raw_file(path: Path, data: bytes):
    with path.open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    path.chmod(0o400)


def run_guard(guard: str, *, env: dict[str, str], root: Path = REPO_ROOT,
              command_runner=run_command, cadence: str = "cloud"):
    policy = load_policy(root / "verification/policy/correctness_checkpoints.json")
    recipes = [row for row in policy["recipes"] if row["guard"] == guard]
    script = SUPPORTED.get(guard)
    if script is None:
        raise EvidenceError("unknown checkpoint guard")
    recipe = recipes[0] if recipes else None
    if (recipe is None or cadence not in recipe["cadence"] or
            sys.platform not in recipe["supported_hosts"] or
            artifact_identity(root / script)["sha256"] != recipe["audited_script_sha256"]):
        print(f"[sifr-checkpoint] guard={guard} state=fresh reason=unknown-dependency-closure")
        command_runner(["python3", script], env=env)
        return None
    producer = {"kind": policy["protocol"], "uid": os.getuid(), "worktree": str(root.resolve())}
    command = isolated_python(script)
    identifier = recipe["id"]
    additional = 2 * policy["max_retained_bytes"]
    if shutil.disk_usage(root).free < policy["disk_reserve_bytes"] + additional:
        # Optional reuse cannot suppress the required assertion. This path
        # launches only the read-only guard and publishes no checkpoint.
        print(f"[sifr-checkpoint] guard={guard} state=fresh reason=checkpoint-capacity-unavailable")
        command_runner(command, env=env)
        return None
    observations = root / "target/verification/checkpoint-observations" / str(uuid.uuid4())
    observations.mkdir(parents=True, mode=0o700)
    index = 0
    def observe(payload):
        nonlocal index
        index += 1
        write_evidence(observations / f"{index:04d}.json", {
            "schema_version": 1, "claim": "checkpoint-observation",
            "observed_at": datetime.now(UTC).isoformat(), "guard": guard,
            "producer": producer, **payload})
        print(f"[sifr-checkpoint] guard={guard} state={payload['state']} observations={observations}")
    def factory():
        if artifact_identity(root / script)["sha256"] != recipe["audited_script_sha256"]:
            raise EvidenceError("audited recipe changed during checkpoint execution")
        paths = [root / value for value in recipe["consumed_paths"]]
        selection = {"required_kinds": {identifier: "validation"},
            "recipe": recipe, "policy": policy, "cadence": cadence,
            "path_presence": {str(path): path.exists() for path in paths}}
        try:
            key = execution_key(selection=selection, commands=[command],
                artifacts=[path for path in paths if path.is_file()], producer=producer,
                services={}, env=env, root=root)
            key["inputs"]["runtime"]["checkpoint_stdlib"] = stdlib_identity(script, env, cwd=root)
        except (EvidenceError, OSError) as error:
            raise UnknownClosure("runtime dependency closure unavailable; execute fresh") from error
        key["input_digest"] = digest(key["inputs"])
        return key
    def callback():
        try:
            outcome = command_runner(command, env=env)
        except BaseException as error:
            outcome = getattr(error, "outcome", None)
            if outcome is not None:
                raw_file(observations / "stdout", outcome.stdout)
                raw_file(observations / "stderr", outcome.stderr)
            raise
        raw_file(observations / "stdout", outcome.stdout)
        raw_file(observations / "stderr", outcome.stderr)
        if outcome.returncode != 0 or outcome.cause != "exit" or outcome.truncated:
            raise EvidenceError("checkpoint requires complete actual validation execution")
        return Completed(records=[{"id": identifier, "state": "passed",
            "phases": ["selected", "executed", "passed"], "execution_kind": "validation",
            "executed_count": 1, "elapsed_seconds": outcome.elapsed_seconds,
            "infrastructure": "none"}], artifacts=[observations / "stdout", observations / "stderr"])
    store = Checkpoints(root / "target/verification/correctness-checkpoints", producer=producer,
        max_age_seconds=policy["max_age_seconds"], max_retained_bytes=policy["max_retained_bytes"])
    return store.run(factory=factory, callback=callback, observe=observe)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run/reuse one audited correctness recipe; no full-profile claim.")
    parser.add_argument("--guard", required=True, choices=sorted(SUPPORTED))
    args = parser.parse_args(argv)
    run_guard(args.guard, env=os.environ.copy(), cadence="manual-recovery")
    return 0
