#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-incident-recovery.XXXXXX")"
cleanup() {
  rm -rf "${DEMO_ROOT}"
}
trap cleanup EXIT HUP INT TERM

python3 - "${REPO_ROOT}" "${DEMO_ROOT}" <<'PY'
import sys
from pathlib import Path

repo_root = Path(sys.argv[1])
demo_root = Path(sys.argv[2])
sys.path.insert(0, str(repo_root / "verification" / "areas" / "distribution_release"))

from governance.common import GovernanceError, load_json_strict
from governance.incident_fixture import plan_fixture_recovery
from governance.incident_recovery_selftest import (
    build_roll_forward_fixture,
    build_rollback_fixture,
    execute_fixture_installer,
    run_fixture,
    scrub_credentials,
)

print("1. Reserve a rollback generation, retain the failed attempt, and resume")
rollback = build_rollback_fixture(demo_root / "rollback")
with scrub_credentials():
    failed = run_fixture(rollback, mode="initial", fail_at="after-reservation")
    completed = run_fixture(rollback, mode="resume")
index = load_json_strict(rollback.root / "live" / "channels.json", require_canonical=True)
print(
    f"burned_generation=21 failed={failed['failure']} "
    f"realized_generation={index['generation']} stable={index['channels']['stable']}"
)

print("2. Require explicit consent before a working affected client downgrades")
try:
    plan_fixture_recovery(
        fixture_root=rollback.root,
        current_version="0.1.1",
        entrypoint="self-update",
        force=False,
    )
except GovernanceError as error:
    print(f"refused={error}")
working = plan_fixture_recovery(
    fixture_root=rollback.root,
    current_version="0.1.1",
    entrypoint="self-update",
    force=True,
)
working_state = rollback.root / "installations" / "working-client" / "version"
execute_fixture_installer(working, working_state)
print(f"working_client={working_state.read_text().strip()}")

print("3. Recover a broken installation through the stable out-of-band entrypoint")
broken = plan_fixture_recovery(
    fixture_root=rollback.root,
    current_version="0.1.1",
    entrypoint="out-of-band",
    force=True,
)
broken_state = rollback.root / "installations" / "broken-client" / "version"
execute_fixture_installer(broken, broken_state)
print(f"out_of_band={broken_state.read_text().strip()} signoff={completed['signoff']}")

print("4. Resolve a first-GA incident by atomic roll-forward")
forward = build_roll_forward_fixture(demo_root / "roll-forward")
with scrub_credentials():
    result = run_fixture(forward, mode="initial")
forward_index = load_json_strict(
    forward.root / "live" / "channels.json",
    require_canonical=True,
)
print(
    f"generation={result['generation']} stable={forward_index['channels']['stable']} "
    f"withdrawn={forward_index['releases']['0.1.0']['incident_id']}"
)
PY
