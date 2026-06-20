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
install_root="${tmp_dir}/install-root"
release_root="${tmp_dir}/github-releases"
install_dir="${tmp_dir}/installed"
binary="${tmp_dir}/sifr"

make_mock_binary "${binary}" "dispatcher generated installer fixture"
"${REPO_ROOT}/scripts/distribution/build_preview_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" >/dev/null
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${release_root}/${version}/sifr-installer-${version}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null
"${REPO_ROOT}/scripts/distribution/generate_channel_metadata.sh" \
  --out "${tmp_dir}/channels.json" \
  --alpha-version "0.1.0-alpha.1" \
  --beta-version "${version}" >/dev/null
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
  sh "${install_root}/index" >/dev/null

output="$("${install_dir}/sifr")"
if [[ "${output}" != "dispatcher generated installer fixture" ]]; then
  echo "dispatcher did not delegate to generated installer: ${output}" >&2
  exit 1
fi
