#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-channel-installer-drift.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

install_root="${tmp_dir}/install"
make_self_update_install_root_fixture "${install_root}" "0.1.0-alpha.4" "0.1.0-beta.7"
sed -i.bak 's/APP_VERSION="0.1.0-beta.7"/APP_VERSION="0.1.0-beta.6"/' \
  "${install_root}/versions/0.1.0-beta.7"

require_failure_contains \
  "immutable installer APP_VERSION drift for beta" \
  "${REPO_ROOT}/scripts/distribution/validate_self_update_metadata.sh" \
    --install-root "${install_root}"
