#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

require_failure_contains \
  "stable channel installs are disabled" \
  run_dispatcher index --channel stable
