#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-stable.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7"

python3 - "${install_root}/metadata/channels.json" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
metadata = json.loads(path.read_text())
metadata["channels"]["stable"] = "1.0.0"
path.write_text(json.dumps(metadata, indent=2) + "\n")
PY

require_failure_contains \
  "stable channel metadata is disabled until Phase 39" \
  "${REPO_ROOT}/scripts/distribution/validate_self_update_metadata.sh" \
    --install-root "${install_root}"
