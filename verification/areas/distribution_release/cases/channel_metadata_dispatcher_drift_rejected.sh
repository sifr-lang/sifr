#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-dispatcher-drift.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
channels_file="${tmp_dir}/channels.json"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7" beta
sed -i.bak 's#https://github.com/sifr-lang/sifr/releases/download/channels/channels.json#https://sifr.sh/install/metadata/channels.json#' \
  "${install_root}/index"
generate_channel_metadata_fixture "${channels_file}" "0.1.0-alpha.4" "0.1.0-beta.7"

require_failure_contains \
  "dispatcher does not resolve channels from GitHub: index" \
  "${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
    --install-root "${install_root}" \
    --channels-file "${channels_file}"
