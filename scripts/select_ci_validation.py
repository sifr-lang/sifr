#!/usr/bin/env python3
"""Emit one canonical profile for the actual checked-out CI candidate."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "verification/runner"))
from sifr_verify.change_selection import SHA, selection
from validation_pr_candidate import verify_pr_candidate


def main():
    candidate = os.environ["CANDIDATE_SHA"]
    observed = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    if not SHA.fullmatch(candidate) or observed != candidate:
        raise ValueError("CI must checkout the actual candidate commit")
    event = os.environ["EVENT_NAME"]
    if event == "pull_request":
        verify_pr_candidate(ROOT, candidate, os.environ["BASE_SHA"], os.environ["PR_HEAD_SHA"])
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
    reuse = False
    if event == 'push':
        from validation_main_reuse import api, select
        decision = {'state': 'fresh', 'candidate_sha': candidate, 'reused_jobs': [],
                    'reason': ['not the protected main branch']}
        if os.environ.get('GITHUB_REF') == 'refs/heads/main':
            try:
                decision = select(api, os.environ['GITHUB_REPOSITORY'], candidate,
                                  int(os.environ['GITHUB_RUN_ID']), now=datetime.now(timezone.utc))
            except Exception as error:
                decision['reason'] = ['prior producer unavailable: ' + type(error).__name__]
        decision.update(run_id=int(os.environ['GITHUB_RUN_ID']), run_attempt=int(os.environ['GITHUB_RUN_ATTEMPT']))
        (receipt.parent / 'main-reuse.json').write_text(json.dumps(decision, indent=2) + '\n')
        reuse = decision['state'] == 'reused'
        print(json.dumps(decision, sort_keys=True))
    with open(os.environ["GITHUB_OUTPUT"], "a") as output:
        print("profiles=" + json.dumps([profile]), file=output)
        print("reuse=" + str(reuse).lower(), file=output)


if __name__ == "__main__":
    main()
