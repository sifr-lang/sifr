#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-generated.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.10"
artifact_dir="${tmp_dir}/artifacts"
release_root="${tmp_dir}/github-releases"
install_root="${tmp_dir}/installed"
install_dir="${install_root}/bin"
binary="${tmp_dir}/sifr"

make_mock_binary "${binary}" "dispatcher generated installer fixture"
build_mock_preview_artifacts "${version}" "${artifact_dir}" "${binary}"
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${release_root}/${version}/sifr-installer-${version}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null
generate_channel_metadata_fixture \
  "${tmp_dir}/channels.json" \
  "0.1.0-alpha.1" \
  "${version}"
"${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
  --install-root "${install_root}" \
  --channel-metadata-url "file://${tmp_dir}/channels.json" \
  --installer-release-base-url "file://${release_root}" >/dev/null
make_mock_version_installers "${install_root}"

SIFR_INSTALLER_RELEASE_BASE_URL="file://${release_root}" \
  SIFR_CHANNEL_METADATA_URL="file://${tmp_dir}/channels.json" \
  SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
  SIFR_TARGET="x86_64-unknown-linux-gnu" \
  SIFR_INSTALL_DIR="${install_dir}" \
  sh "${install_root}/beta" >/dev/null

if ! grep -F 'dispatcher generated installer fixture' "${install_dir}/sifr" >/dev/null; then
  echo "dispatcher did not delegate to generated installer payload" >&2
  exit 1
fi
