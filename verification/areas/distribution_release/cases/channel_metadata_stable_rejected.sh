#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-stable.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
channels_file="${tmp_dir}/channels.json"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7"
generate_channel_metadata_fixture "${channels_file}" "0.1.0-alpha.4" "0.1.0-beta.7"

python3 - "${channels_file}" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
metadata = json.loads(path.read_text())
metadata["channels"]["stable"] = "1.0.0"
path.write_text(json.dumps(metadata, indent=2) + "\n")
PY

require_failure_contains \
  "$.channels: unknown field(s): stable" \
  "${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
    --install-root "${install_root}" \
    --channels-file "${channels_file}"
