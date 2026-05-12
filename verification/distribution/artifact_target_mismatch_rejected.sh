#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-target-reject.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.13"
artifact_dir="${tmp_dir}/artifacts"
binary="${tmp_dir}/sifr"
installer="${tmp_dir}/installer.sh"

make_mock_binary "${binary}" "target mismatch fixture"
"${REPO_ROOT}/scripts/distribution/build_preview_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" >/dev/null
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

require_failure_contains \
  "unsupported target: riscv64-unknown-linux-gnu" \
  env SIFR_TARGET="riscv64-unknown-linux-gnu" SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" sh "${installer}"
