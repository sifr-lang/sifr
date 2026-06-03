#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-metadata.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
alpha_version="0.1.0-alpha.4"
beta_version="0.1.0-beta.7"

"${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
  --install-root "${install_root}" \
  --alpha-version "${alpha_version}" \
  --beta-version "${beta_version}" >/dev/null

metadata_path="${install_root}/metadata/channels.json"
test -f "${metadata_path}"

python3 - "${metadata_path}" "${alpha_version}" "${beta_version}" <<'PY'
import json
import pathlib
import sys

metadata = json.loads(pathlib.Path(sys.argv[1]).read_text())
expected = {
    "schema_version": 1,
    "channels": {
        "alpha": sys.argv[2],
        "beta": sys.argv[3],
    },
}
if metadata != expected:
    raise SystemExit(f"unexpected metadata: {metadata!r}")
if list(metadata["channels"].keys()) != ["alpha", "beta"]:
    raise SystemExit("channel ordering drifted")
PY
