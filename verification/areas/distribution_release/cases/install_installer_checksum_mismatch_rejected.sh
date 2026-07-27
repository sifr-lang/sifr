#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

printf '%s\n' '# tampered after metadata generation' \
  >>"${MOCK_DISPATCHER_TMP_DIR}/github-releases/0.1.0/sifr-installer-0.1.0"

require_failure_contains \
  "installer SHA-256 mismatch for 0.1.0" \
  run_dispatcher index

