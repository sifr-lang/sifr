#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_success_contains \
  "sifr dispatcher channel=stable version=0.1.0" \
  run_dispatcher index

require_success_contains \
  "sifr mock generated installer version=0.1.0" \
  run_dispatcher index
