#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-missing-target.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.11"
artifact_dir="${tmp_dir}/artifacts"
binary="${tmp_dir}/sifr"
make_mock_binary "${binary}" "missing target fixture"
build_mock_preview_artifacts "${version}" "${artifact_dir}" "${binary}"
rm "${artifact_dir}/sifr-${version}-aarch64-unknown-linux-gnu.tar.gz"

require_failure_contains \
  "artifact missing for target aarch64-unknown-linux-gnu" \
  "${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
    --version "${version}" \
    --artifact-dir "${artifact_dir}" \
    --out "${tmp_dir}/installer.sh"
