#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"
use_mock_dispatcher_fixture

require_failure_contains \
  "stable channel installs are disabled" \
  run_dispatcher index --channel stable
