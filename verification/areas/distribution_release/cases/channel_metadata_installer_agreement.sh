#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-agreement.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7"

"${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
  --install-root "${install_root}"
