#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_failure_contains \
  "--version 0.1.0-alpha.1 conflicts with selected channel beta" \
  run_dispatcher index --channel beta --version 0.1.0-alpha.1

require_failure_contains \
  "SIFR_CHANNEL=alpha conflicts with --channel beta" \
  env SIFR_CHANNEL=alpha SIFR_INSTALL_BASE_URL="${DISPATCH_BASE_URL}" sh "${SITE_INSTALL_ROOT}/index" --channel beta
