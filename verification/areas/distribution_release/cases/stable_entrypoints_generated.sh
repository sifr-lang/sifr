#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

for dispatcher in index stable; do
  [[ -f "${SITE_INSTALL_ROOT}/${dispatcher}" ]] || {
    echo "stable installer entrypoint missing: ${SITE_INSTALL_ROOT}/${dispatcher}" >&2
    exit 1
  }
  grep -q 'DEFAULT_CHANNEL="stable"' "${SITE_INSTALL_ROOT}/${dispatcher}"
done

grep -q '^# Entrypoint: index$' "${SITE_INSTALL_ROOT}/index"
grep -q '^# Entrypoint: beta$' "${SITE_INSTALL_ROOT}/beta"
[[ "$(sha256_fixture_file "${SITE_INSTALL_ROOT}/index")" != \
   "$(sha256_fixture_file "${SITE_INSTALL_ROOT}/beta")" ]] || {
  echo "index and beta dispatcher attestations must remain distinguishable" >&2
  exit 1
}

preview_root="$(mktemp -d "${TMPDIR:-/tmp}/sifr-preview-dispatchers.XXXXXX")"
cleanup() {
  rm -rf "${preview_root}"
}
trap cleanup EXIT HUP INT TERM
"${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
  --install-root "${preview_root}" \
  --default-channel beta >/dev/null
grep -q 'DEFAULT_CHANNEL="beta"' "${preview_root}/index"
[[ "$(sha256_fixture_file "${preview_root}/index")" != \
   "$(sha256_fixture_file "${preview_root}/beta")" ]] || {
  echo "preview index and beta dispatcher attestations must remain distinguishable" >&2
  exit 1
}
