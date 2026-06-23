#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-matching-target.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-alpha.9"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
install_root="${tmp_dir}/install"
install_dir="${install_root}/bin"
target="aarch64-unknown-linux-gnu"

make_target_specific_artifacts "${version}" "${artifact_dir}"
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

SIFR_TARGET="${target}" \
  SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
  SIFR_INSTALL_DIR="${install_dir}" \
  sh "${installer}" >/dev/null

if ! grep -F "target=${target}" "${install_dir}/sifr" >/dev/null; then
  echo "installer selected wrong target artifact payload" >&2
  exit 1
fi
