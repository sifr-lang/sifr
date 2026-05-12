#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact-all-targets.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
version="0.1.0-beta.7"

make_mock_binary "${binary}" "sifr all-target fixture"
"${REPO_ROOT}/scripts/distribution/build_preview_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" >/dev/null

"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu
do
  install_dir="${tmp_dir}/install-${target}"
  SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_INSTALL_DIR="${install_dir}" \
    sh "${installer}" >/dev/null
  output="$("${install_dir}/sifr")"
  if [[ "${output}" != "sifr all-target fixture" ]]; then
    echo "unexpected installed binary output for ${target}: ${output}" >&2
    exit 1
  fi
done
