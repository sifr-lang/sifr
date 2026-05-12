#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_failure_contains \
  "immutable generated installer unavailable" \
  run_dispatcher index --version 0.1.0-alpha.404
