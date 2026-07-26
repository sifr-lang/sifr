#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-candidate.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0"
binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/sifr-installer-${version}"
sysroot_root="${tmp_dir}/mock-sysroot"

make_mock_binary "${binary}" "sifr stable candidate fixture"
make_mock_sysroot_root "${sysroot_root}"
"${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" \
  --sysroot-root "${sysroot_root}" >/dev/null

"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

grep -F 'APP_VERSION="0.1.0"' "${installer}" >/dev/null
grep -F 'APP_CHANNEL="stable"' "${installer}" >/dev/null

for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu
do
  install_root="${tmp_dir}/install-${target}"
  SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_INSTALL_DIR="${install_root}/bin" \
    SIFR_INSTALL_MANIFEST_DIR="${install_root}" \
    SIFR_NO_MODIFY_PATH=1 \
    sh "${installer}" >/dev/null
  grep -F '"version": "0.1.0"' "${install_root}/install.json" >/dev/null
  grep -F '"channel": "stable"' "${install_root}/install.json" >/dev/null
  grep -F '"sifr-version" = "0.1.0"' "${install_root}/sysroot.toml" >/dev/null
  grep -F "\"target-triple\" = \"${target}\"" "${install_root}/sysroot.toml" >/dev/null
done

echo "stable candidate artifact generation: PASS"
