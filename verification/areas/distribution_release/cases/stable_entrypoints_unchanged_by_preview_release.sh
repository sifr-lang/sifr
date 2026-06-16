#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

if [[ -e "${SITE_INSTALL_ROOT}/stable" ]]; then
  echo "stable installer entrypoint must not exist before stable channel enablement: ${SITE_INSTALL_ROOT}/stable" >&2
  exit 1
fi

if rg -n 'DEFAULT_CHANNEL="stable"|STABLE_VERSION|/stable' "${SITE_INSTALL_ROOT}" >/tmp/sifr-stable-entrypoint-rg.$$ 2>/dev/null; then
  cat /tmp/sifr-stable-entrypoint-rg.$$ >&2
  rm -f /tmp/sifr-stable-entrypoint-rg.$$
  exit 1
fi
rm -f /tmp/sifr-stable-entrypoint-rg.$$
