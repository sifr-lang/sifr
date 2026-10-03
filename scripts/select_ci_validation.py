#!/usr/bin/env python3
"""Emit one canonical profile for the actual checked-out CI candidate."""
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "verification/runner"))
from sifr_verify.change_selection import SHA, selection


def main():
    candidate = os.environ["CANDIDATE_SHA"]
    observed = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if not SHA.fullmatch(candidate) or observed != candidate:
        raise ValueError("CI must checkout the actual candidate commit")
    event = os.environ["EVENT_NAME"]
    if event == "pull_request":
        plan = selection(ROOT, os.environ["BASE_SHA"], candidate)
        profile = plan["profile"]
        print(json.dumps(plan, sort_keys=True))
    elif event == "schedule":
        profile = "nightly"
    elif event in {"push", "merge_group", "workflow_dispatch"}:
        profile = "merge"
    else:
        raise ValueError("unknown CI event cannot select reduced coverage")
    receipt = ROOT / "target/validation-candidate/candidate.json"
    receipt.parent.mkdir(parents=True, exist_ok=True)
    receipt.write_text(json.dumps({"candidate_sha": candidate,
                                  "run_id": int(os.environ["GITHUB_RUN_ID"]),
                                  "run_attempt": int(os.environ["GITHUB_RUN_ATTEMPT"])}))
    with open(os.environ["GITHUB_OUTPUT"], "a") as output:
        print("profiles=" + json.dumps([profile]), file=output)


if __name__ == "__main__":
    main()
