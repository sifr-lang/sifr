#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

use_mock_dispatcher_fixture
canonical="${MOCK_DISPATCHER_TMP_DIR}/canonical.json"
cp "${MOCK_DISPATCHER_TMP_DIR}/channels.json" "${canonical}"

reject_shape() {
  local name="$1"
  local python_mutation="$2"
  cp "${canonical}" "${MOCK_DISPATCHER_TMP_DIR}/channels.json"
  python3 - "${MOCK_DISPATCHER_TMP_DIR}/channels.json" "${python_mutation}" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
value = json.loads(path.read_text())
exec(sys.argv[2], {"value": value})
path.write_text(json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n")
PY
  require_failure_contains \
    "exact canonical schema-v2 shape" \
    run_dispatcher stable
  printf '%s\n' "rejected ${name}"
}

reject_shape "schema v1" 'value["schema_version"] = 1'
reject_shape "versionless" 'value.pop("schema_version")'
reject_shape "version-negotiated" 'value["schema_version"] = [1, 2]'
reject_shape "dual-format" 'value["version"] = 1'
