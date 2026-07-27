#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_failure_contains \
  "version 1.0.0 is not an active governed release" \
  run_dispatcher index --version 1.0.0

require_success_contains \
  "sifr dispatcher channel=stable version=0.1.0" \
  run_dispatcher index --version 0.1.0
