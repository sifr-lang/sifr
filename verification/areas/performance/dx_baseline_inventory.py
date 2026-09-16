#!/usr/bin/env python3
"""Freeze the pre-migration fixture and assertion authorities without flattening them."""
import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = "0f819c2f04bf5b2891074c55ba26369ddf4f13bd"


def capture():
    names = subprocess.check_output(
        ["git", "ls-tree", "-r", "--name-only", BASE], cwd=ROOT, text=True
    ).splitlines()
    records = []
    # Keep complete declarative assertions/profile settings, including negative
    # expectations and native/application flags. Imperative harness authorities
    # remain pinned source blobs; do not guess an assertion depth from a suffix.
    for name in names:
        role = None
        if name.startswith("verification/profiles/") and name.endswith(".json"):
            role = "verification-profile"
        elif name.startswith("verification/") and name.endswith(".json") and (
            "manifest" in name or "matrix" in name or "fixtures" in name
        ):
            role = "declarative-assertion-authority"
        elif name.endswith(".sifr") and name.startswith(("crates/", "verification/", "demos/")):
            role = "fixture"
        elif name.startswith(("verification/runner/", "crates/sifr/tests/", "verification/areas/")) and name.endswith((".py", ".rs", ".sh")):
            role = "imperative-assertion-authority"
        if role is None:
            continue
        blob = subprocess.check_output(["git", "show", f"{BASE}:{name}"], cwd=ROOT)
        record = {"id": name, "role": role, "sha256": hashlib.sha256(blob).hexdigest(),
                  "source": f"{BASE}:{name}"}
        if role in ("verification-profile", "declarative-assertion-authority"):
            record["declaration"] = json.loads(blob)
        records.append(record)
    return {
        "schema_version": 1, "source_commit": BASE,
        "application_profile_authority": [
            f"{BASE}:crates/sifr_driver/src/lib.rs",
            f"{BASE}:verification/runner/e2e/run_e2e_pass.sh",
        ],
        "preservation_rule": (
            "Each fixture retains every assertion of its pinned declarative or imperative "
            "consumer. Release linking/runtime, negative diagnostics and snapshots remain "
            "distinct obligations; compiler/verification profile labels cannot replace them."
        ),
        "records": records,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = capture()
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(f"{len(result['records'])} frozen authorities/fixtures; sha256="
          f"{hashlib.sha256(args.output.read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
