#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-site-facts.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

out="${tmp_dir}/site-publication-facts.json"
"${REPO_ROOT}/scripts/distribution/generate_site_publication_facts.py" \
  --out "${out}" \
  --source-commit "$(printf 'a%.0s' {1..40})" \
  --site-base-commit "$(printf '1%.0s' {1..40})" \
  --release-plan-sha256 "$(printf 'b%.0s' {1..64})" \
  --publication-attempt "run-42-1" \
  --release-index-generation 9 \
  --release-index-sha256 "$(printf 'c%.0s' {1..64})" \
  --dispatcher-default-channel beta \
  --dispatcher-index-sha256 "$(printf 'd%.0s' {1..64})" \
  --dispatcher-stable-sha256 "$(printf 'e%.0s' {1..64})" \
  --dispatcher-alpha-sha256 "$(printf 'f%.0s' {1..64})" \
  --dispatcher-beta-sha256 "$(printf '0%.0s' {1..64})"

python3 - "${out}" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
raw = path.read_bytes()
value = json.loads(raw)
assert raw.endswith(b"\n")
assert raw == (
    json.dumps(value, sort_keys=True, separators=(",", ":")).encode() + b"\n"
)
assert value["contract"] == "sifr-site-publication-binding-v1"
assert value["publication_attempt"] == "run-42-1"
assert value["site_base_commit"] == "1" * 40
assert value["release_index"]["generation"] == 9
assert value["dispatcher_default_channel"] == "beta"
assert set(value["dispatchers"]) == {"index", "stable", "alpha", "beta"}
PY

require_failure_contains \
  "refusing to overwrite" \
  "${REPO_ROOT}/scripts/distribution/generate_site_publication_facts.py" \
    --out "${out}" \
    --source-commit "$(printf 'a%.0s' {1..40})" \
    --site-base-commit "$(printf '1%.0s' {1..40})" \
    --release-plan-sha256 "$(printf 'b%.0s' {1..64})" \
    --publication-attempt "run-42-1" \
    --release-index-generation 9 \
    --release-index-sha256 "$(printf 'c%.0s' {1..64})" \
    --dispatcher-default-channel beta \
    --dispatcher-index-sha256 "$(printf 'd%.0s' {1..64})" \
    --dispatcher-stable-sha256 "$(printf 'e%.0s' {1..64})" \
    --dispatcher-alpha-sha256 "$(printf 'f%.0s' {1..64})" \
    --dispatcher-beta-sha256 "$(printf '0%.0s' {1..64})"
