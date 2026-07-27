#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-preview-index.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

make_dispatcher_fixture "${tmp_dir}"
mkdir -p \
  "${tmp_dir}/github-releases/0.1.0-alpha.1" \
  "${tmp_dir}/github-releases/0.1.0-beta.1"
for version in 0.1.0-alpha.1 0.1.0-beta.1; do
  printf '%s\n' '#!/bin/sh' "echo ${version}" \
    >"${tmp_dir}/github-releases/${version}/sifr-installer-${version}"
done
generate_channel_metadata_fixture \
  "${tmp_dir}/channels.json" \
  "0.1.0-alpha.1" \
  "0.1.0-beta.1"

SITE_INSTALL_ROOT="${tmp_dir}"
require_failure_contains \
  "stable channel installs require active GA metadata" \
  run_dispatcher index
