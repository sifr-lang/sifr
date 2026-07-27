#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-rc-rejection.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

require_failure_contains \
  "version must be stable SemVer or a prerelease using -alpha.N or -beta.N" \
  "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
    --version 0.1.0-rc.1 \
    --output-dir "${tmp_dir}/artifacts" \
    --cargo-build

make_dispatcher_fixture "${tmp_dir}/install"
generate_channel_metadata_fixture \
  "${tmp_dir}/channels.json" \
  "0.1.0-alpha.1" \
  "0.1.0-beta.1"
SITE_INSTALL_ROOT="${tmp_dir}/install"
DISPATCH_RELEASE_BASE_URL="file://${tmp_dir}/github-releases"
DISPATCH_CHANNEL_METADATA_URL="file://${tmp_dir}/channels.json"
require_failure_contains \
  "unknown release channel: rc" \
  run_dispatcher index --channel rc
require_failure_contains \
  "version must be stable SemVer or a prerelease using -alpha.N or -beta.N" \
  run_dispatcher index --version 0.1.0-rc.1

if rg -n 'alpha\|beta\|rc|\(alpha\|beta\|rc\)|^[[:space:]]*rc\)' \
  "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
  "${REPO_ROOT}/scripts/distribution/create_new_version.sh" \
  "${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
  "${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  "${REPO_ROOT}/scripts/distribution/trigger_preview_release.sh" \
  "${REPO_ROOT}/crates/sifr/src/self_update_cli.rs" \
  "${REPO_ROOT}/crates/sifr/src/self_update_metadata.rs" \
  "${REPO_ROOT}/crates/sifr/src/self_update_receipt.rs" \
  "${REPO_ROOT}/crates/sifr/src/self_update_runner.rs" \
  "${REPO_ROOT}/.github/workflows/preview-release.yml"
then
  echo "live release surface still contains rc acceptance" >&2
  exit 1
fi
