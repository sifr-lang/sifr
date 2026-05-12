#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_success_contains \
  "sifr dispatcher channel=alpha version=0.1.0-alpha.1" \
  run_dispatcher index --version 0.1.0-alpha.1

require_success_contains \
  "sifr mock generated installer version=0.1.0-beta.1" \
  run_dispatcher alpha --version 0.1.0-beta.1
