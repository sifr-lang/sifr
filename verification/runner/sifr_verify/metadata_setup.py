"""Prepare source metadata using the exact compiler artifact reported by Cargo."""
from __future__ import annotations
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

from scripts.distribution.metadata_artifact import require_native_binary
from .paths import REPO_ROOT
from .process_execution import execute
from .profile_commands import CommandFailed


def build_and_prepare(command, env, executor=execute, runner=subprocess.run, validator=require_native_binary):
    if command[:2] != ["cargo", "build"]:
        raise ValueError("metadata preparation requires an explicit Cargo build")
    artifacts = set()
    pending = {"stdout": "", "stderr": ""}
    def line(stream, text):
        if stream == "stdout":
            try:
                event = json.loads(text)
            except ValueError:
                event = None
            if isinstance(event, dict):
                if (event.get("reason") == "compiler-artifact"
                        and event.get("target", {}).get("name") == "sifr"
                        and "bin" in event.get("target", {}).get("kind", [])
                        and event.get("executable")
                        and not event.get("profile", {}).get("test")):
                    artifacts.add(event["executable"])
                elif event.get("reason") == "compiler-message":
                    print(event["message"].get("rendered", ""), end="")
                return
        print(text, file=sys.stderr if stream == "stderr" else sys.stdout)
    def emit(stream, data):
        pending[stream] += data.decode("utf-8", errors="replace")
        while "\n" in pending[stream]:
            text, pending[stream] = pending[stream].split("\n", 1)
            line(stream, text)
    outcome = executor([*command, "--message-format=json-render-diagnostics"],
                       cwd=REPO_ROOT, env=env, emit=emit,
                       deadline_seconds=float(env.get("SIFR_VERIFY_SAFETY_DEADLINE_SECONDS", "2400")))
    for stream, text in pending.items():
        if text:
            line(stream, text)
    if outcome.returncode:
        raise CommandFailed(outcome.returncode)
    if len(artifacts) != 1:
        raise ValueError("Cargo must report exactly one source compiler artifact for metadata preparation")
    binary = Path(artifacts.pop())
    validator(binary, False)
    # No guessed target/debug path or host/target substitution. The canonical CLI
    # retains its own compiled target and identity; it reports the durable cache key.
    with tempfile.TemporaryDirectory(prefix="sifr-metadata-preparation-") as directory:
        result = runner([str(binary), "sysroot", "build-metadata", "--source-root", str(REPO_ROOT),
                         "--output", str(Path(directory) / "stdlib.sifrmeta")], env=env,
                        cwd=REPO_ROOT, check=True)
    return result


def main():
    command = sys.argv[1:]
    if command[:1] == ["--"]:
        command = command[1:]
    build_and_prepare(command, os.environ.copy())


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, CommandFailed, subprocess.CalledProcessError) as error:
        raise SystemExit(f"metadata preparation failed: {error}") from error
