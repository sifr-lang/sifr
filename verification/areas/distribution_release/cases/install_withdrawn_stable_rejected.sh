#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

python3 - "${MOCK_DISPATCHER_TMP_DIR}/channels.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
release = payload["releases"]["0.1.0"]
release["status"] = "withdrawn"
release["incident_id"] = "incident-fixture"
path.write_text(
    json.dumps(payload, sort_keys=True, separators=(",", ":")) + "\n",
    encoding="utf-8",
)
PY

require_failure_contains \
  "version 0.1.0 is not an active governed release" \
  run_dispatcher index
