#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

ruby -e 'require "yaml"; YAML.load_file(ARGV.fetch(0))' \
  "${REPO_ROOT}/.github/workflows/preview-release.yml" >/dev/null
