#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-installer-drift.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
channels_file="${tmp_dir}/channels.json"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7"
mkdir -p "${install_root}/versions"
touch "${install_root}/versions/0.1.0-beta.7"
generate_channel_metadata_fixture "${channels_file}" "0.1.0-alpha.4" "0.1.0-beta.7"

require_failure_contains \
  "website must not publish immutable version installers" \
  "${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
    --install-root "${install_root}" \
    --channels-file "${channels_file}"
