#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-missing-installer.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
channels_file="${tmp_dir}/channels.json"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7"
mkdir -p "${install_root}/metadata"
cp "${install_root}/channels.json" "${install_root}/metadata/channels.json"
"${REPO_ROOT}/scripts/distribution/generate_channel_metadata.sh" \
  --out "${channels_file}" \
  --alpha-version "0.1.0-alpha.4" \
  --beta-version "0.1.0-beta.7" >/dev/null

require_failure_contains \
  "website must not publish metadata/channels.json" \
  "${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
    --install-root "${install_root}" \
    --channels-file "${channels_file}"
