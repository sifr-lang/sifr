#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MANIFEST="${REPO_ROOT}/verification/platform/golden/manifest.json"

if [[ ! -f "${MANIFEST}" ]]; then
  echo "platform golden manifest not found: ${MANIFEST}" >&2
  exit 2
fi

python3 - "$MANIFEST" "$REPO_ROOT" <<'PY'
import json
import os
import pathlib
import subprocess
import sys

manifest_path = pathlib.Path(sys.argv[1])
repo_root = pathlib.Path(sys.argv[2])
manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
closed = {
    item.strip()
    for item in os.environ.get("SIFR_PLATFORM_CLOSED_MILESTONES", "").split(",")
    if item.strip()
}

passed = 0
skipped = 0
for entry in manifest.get("entries", []):
    missing = [milestone for milestone in entry.get("blocked_until", []) if milestone not in closed]
    if missing:
        skipped += 1
        print(
            f"[platform-golden] skip {entry['program']} blocked_until={','.join(missing)}"
        )
        continue

    result = subprocess.run(
        entry["command"],
        cwd=repo_root,
        shell=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    combined = result.stdout + result.stderr
    expected_exit = int(entry.get("expected_exit", 0))
    if result.returncode != expected_exit:
        print(
            f"[platform-golden] fail {entry['program']} exit={result.returncode} expected={expected_exit}",
            file=sys.stderr,
        )
        print(combined, file=sys.stderr)
        sys.exit(1)
    for needle in entry.get("expected_stdout_contains", []):
        if needle not in result.stdout:
            print(
                f"[platform-golden] fail {entry['program']} missing stdout: {needle}",
                file=sys.stderr,
            )
            print(combined, file=sys.stderr)
            sys.exit(1)
    for needle in entry.get("expected_diagnostic_contains", []):
        if needle not in combined:
            print(
                f"[platform-golden] fail {entry['program']} missing diagnostic: {needle}",
                file=sys.stderr,
            )
            print(combined, file=sys.stderr)
            sys.exit(1)
    passed += 1
    print(f"[platform-golden] pass {entry['program']}")

print(f"[platform-golden] summary pass={passed} skip={skipped}")
PY
