#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-bootstrap.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

releases_json="${tmp_dir}/releases.json"
channels_file="${tmp_dir}/channels.json"

cat >"${releases_json}" <<'JSON'
[
  {"tagName":"0.1.0-beta.8","isDraft":true,"isPrerelease":true},
  {"tagName":"0.1.0-beta.7+ignored","isDraft":false,"isPrerelease":true},
  {"tagName":"0.1.0-beta.6","isDraft":false,"isPrerelease":true},
  {"tagName":"0.1.0-alpha.4","isDraft":false,"isPrerelease":true},
  {"tagName":"0.1.0-alpha.3","isDraft":false,"isPrerelease":true},
  {"tagName":"0.1.0","isDraft":false,"isPrerelease":false}
]
JSON

"${REPO_ROOT}/scripts/distribution/bootstrap_channel_metadata.py" \
  --releases-json "${releases_json}" \
  --channel beta \
  --version 0.1.0-beta.9 \
  --out "${channels_file}"

python3 - "${channels_file}" <<'PY'
import json
import pathlib
import sys

metadata = json.loads(pathlib.Path(sys.argv[1]).read_text())
expected = {
    "schema_version": 1,
    "channels": {
        "alpha": "0.1.0-alpha.4",
        "beta": "0.1.0-beta.9",
    },
}
if metadata != expected:
    raise SystemExit(f"unexpected metadata: {metadata!r}")
PY

cat >"${releases_json}" <<'JSON'
[
  {"tagName":"0.1.0-beta.6","isDraft":false,"isPrerelease":true}
]
JSON

require_failure_contains \
  "missing public alpha prerelease" \
  "${REPO_ROOT}/scripts/distribution/bootstrap_channel_metadata.py" \
    --releases-json "${releases_json}" \
    --channel beta \
    --version 0.1.0-beta.7 \
    --out "${channels_file}"

cat >"${releases_json}" <<'JSON'
[
  {"tagName":"0.1.0-alpha.4","isDraft":false,"isPrerelease":true},
  {"tagName":"0.1.0-beta.9","isDraft":false,"isPrerelease":true}
]
JSON

require_failure_contains \
  "refusing to downgrade beta channel from 0.1.0-beta.9 to 0.1.0-beta.8" \
  "${REPO_ROOT}/scripts/distribution/bootstrap_channel_metadata.py" \
    --releases-json "${releases_json}" \
    --channel beta \
    --version 0.1.0-beta.8 \
    --out "${channels_file}"
