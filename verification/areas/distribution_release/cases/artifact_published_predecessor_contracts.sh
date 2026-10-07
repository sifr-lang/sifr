#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"
uv run --project verification --locked python -m unittest discover \
  -s verification/areas/sysroot_release -p 'published_*tests.py'
