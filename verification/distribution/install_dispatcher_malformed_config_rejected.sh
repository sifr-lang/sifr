#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-dispatcher-malformed.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

make_dispatcher_fixture "${tmp_dir}" "0.1.0-alpha.1" "0.1.0-beta.1"
sed -i.bak 's/BETA_VERSION="0.1.0-beta.1"/BETA_VERSION="0.1.0-alpha.1"/' "${tmp_dir}/index"

SITE_INSTALL_ROOT="${tmp_dir}"
DISPATCH_BASE_URL="file://${tmp_dir}"
require_failure_contains \
  "malformed dispatcher configuration: beta points at 0.1.0-alpha.1" \
  run_dispatcher index
