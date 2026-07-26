#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-metadata.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

metadata_path="${tmp_dir}/channels.json"
alpha_version="0.1.0-alpha.4"
beta_version="0.1.0-beta.7"

generate_channel_metadata_fixture "${metadata_path}" "${alpha_version}" "${beta_version}"

test -f "${metadata_path}"

python3 - "${metadata_path}" "${alpha_version}" "${beta_version}" <<'PY'
import json
import pathlib
import sys

metadata = json.loads(pathlib.Path(sys.argv[1]).read_text())
if set(metadata) != {"schema_version", "generation", "ga_status", "channels", "releases"}:
    raise SystemExit(f"unexpected metadata fields: {metadata!r}")
if metadata["schema_version"] != 2 or metadata["generation"] != 1:
    raise SystemExit("governance epoch or generation drifted")
if metadata["ga_status"] != "preview":
    raise SystemExit("fixture must remain preview metadata")
if metadata["channels"] != {"alpha": sys.argv[2], "beta": sys.argv[3]}:
    raise SystemExit("channel mapping drifted")
if set(metadata["releases"]) != {sys.argv[2], sys.argv[3]}:
    raise SystemExit("release records drifted")
PY
