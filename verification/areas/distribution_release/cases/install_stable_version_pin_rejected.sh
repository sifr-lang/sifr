#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_failure_contains \
  "stable-looking version pins are disabled" \
  run_dispatcher index --version 1.0.0

require_failure_contains \
  "stable-looking version pins are disabled" \
  run_dispatcher index --version 0.1.0
