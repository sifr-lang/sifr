"""Execute both independent corpus profiles inside the admitted process budget."""
from __future__ import annotations
import json
from pathlib import Path
import time
from sifr_verify.batch_execution import execute_batch
from sifr_verify.build_allocation import positive
from sifr_verify.corpus_ownership import CORPUS_CASE
from sifr_verify.errors import VerificationError
from corpus_profiles import validate_development


def run_profiles(artifacts: dict, root: Path, snapshot: Path, base: Path,
                 env: dict[str, str], release_version: str) -> None:
    workers = min(2, positive(env.get("SIFR_VERIFY_PROCESS_SLOTS", "1"),
                              "SIFR_VERIFY_PROCESS_SLOTS"))
    environment = dict(env)
    environment.pop("SIFR_DX8_CORPUS_CASES", None)
    environment.pop("SIFR_DX8_SEMANTIC_TARGET", None)
    environment.pop("SIFR_DX8_CORPUS_OUTPUT", None)
    if workers > 1:
        environment.update(RAYON_NUM_THREADS="1", CARGO_BUILD_JOBS="1")
    commands = [
        [artifacts["development"]["driver_test"], "--exact", CORPUS_CASE, "--ignored", "--nocapture"],
        [artifacts["oracle"], "--fixtures", str(root / "crates/sifr/tests/e2e/pass")],
    ]
    names = ("development", "release")
    overrides = [
        {"SIFR_SYSROOT": str(root), "SIFR_DX8_CORPUS_OUTPUT": str(base / names[0])},
        {"SIFR_SYSROOT": str(snapshot), "SIFR_RELEASE_VERSION": release_version,
         "SIFR_DX8_CORPUS_OUTPUT": str(base / names[1])},
    ]
    outcomes = {}
    started = time.monotonic()

    def save():
        report = {"workers": workers, "inner_rayon": environment.get("RAYON_NUM_THREADS"),
                  "elapsed_seconds": time.monotonic() - started,
                  "rows": [outcomes[i] for i in sorted(outcomes)]}
        (base / "execution.json").write_text(json.dumps(report, indent=2) + "\n")

    def completed(index, outcome):
        if type(index) is not int or index not in (0, 1) or index in outcomes:
            raise VerificationError("unknown or duplicate corpus profile outcome")
        name = names[index]
        (base / (name + ".stdout")).write_bytes(outcome.stdout)
        (base / (name + ".stderr")).write_bytes(outcome.stderr)
        error = None
        try:
            if outcome.returncode or outcome.cause != "exit" or outcome.truncated:
                raise VerificationError(f"{name}: {outcome.cause}, exit {outcome.returncode}, truncated={outcome.truncated}")
            if index == 0:
                validate_development(outcome)
        except (VerificationError, RuntimeError) as failure:
            error = str(failure)
        outcomes[index] = {"profile": name, "seconds": outcome.elapsed_seconds,
                           "status": "fail" if error else "pass", "error": error}
        save()

    save()
    try:
        execute_batch(commands, cwd=root, env=environment, env_overrides=overrides,
                      max_parallel=workers, completed=completed,
                      deadline_seconds=2400, limit_bytes=16 * 1024 * 1024)
    finally:
        save()
    if set(outcomes) != {0, 1} or any(row["status"] != "pass" for row in outcomes.values()):
        raise VerificationError(f"corpus profiles failed or incomplete; evidence: {base}")
