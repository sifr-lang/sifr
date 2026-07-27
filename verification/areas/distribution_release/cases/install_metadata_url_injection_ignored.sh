#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

use_mock_dispatcher_fixture

attacker_dir="${MOCK_DISPATCHER_TMP_DIR}/attacker"
mkdir -p "${attacker_dir}"
printf '%s\n' '{"schema_version":2}' >"${attacker_dir}/channels.json"

output="$(
  SIFR_CHANNEL_METADATA_URL="file://${attacker_dir}/channels.json" \
  SIFR_INSTALLER_RELEASE_BASE_URL="file://${attacker_dir}/releases" \
    run_dispatcher stable
)"
[[ "${output}" == *"sifr mock generated installer version=0.1.0"* ]] || {
  echo "${output}" >&2
  exit 1
}
