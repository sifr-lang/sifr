#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-dispatcher-malformed.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

make_dispatcher_fixture "${tmp_dir}"
make_mock_version_installers "${tmp_dir}"
python3 - "${tmp_dir}/channels.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
metadata = json.loads(path.read_text())
metadata["channels"]["beta"] = "0.1.0-alpha.1"
path.write_text(json.dumps(metadata, sort_keys=True, separators=(",", ":")) + "\n")
PY

SITE_INSTALL_ROOT="${tmp_dir}"
require_failure_contains \
  "malformed channel metadata: beta points at 0.1.0-alpha.1" \
  run_dispatcher beta
