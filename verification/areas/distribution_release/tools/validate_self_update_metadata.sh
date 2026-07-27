#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"

usage() {
  cat <<'EOF'
Usage: verification/areas/distribution_release/tools/validate_self_update_metadata.sh --install-root <dir> --channels-file <file>

Validate that self-update channel metadata is well-formed and the website
install root contains only GitHub-backed bootstrap dispatchers.
EOF
}

INSTALL_ROOT=""
CHANNELS_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-root)
      INSTALL_ROOT="${2:-}"
      shift 2
      ;;
    --channels-file)
      CHANNELS_FILE="${2:-}"
      shift 2
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

fail() {
  echo "self-update metadata validation: $*" >&2
  exit 2
}

[[ -n "${INSTALL_ROOT}" ]] || fail "--install-root is required"
[[ -d "${INSTALL_ROOT}" ]] || fail "install root not found: ${INSTALL_ROOT}"
[[ -n "${CHANNELS_FILE}" ]] || fail "--channels-file is required"
[[ -f "${CHANNELS_FILE}" ]] || fail "channel metadata missing: ${CHANNELS_FILE}"

metadata_path="${CHANNELS_FILE}"

governance_output="$(
  python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" \
    validate \
    --kind release-index \
    --input "${metadata_path}" 2>&1
)" || fail "${governance_output}"

metadata_values="$(
  python3 - "${metadata_path}" 2>&1 <<'PY'
import json
import pathlib
import re
import sys

metadata_path = pathlib.Path(sys.argv[1])
try:
    metadata = json.loads(metadata_path.read_text())
except json.JSONDecodeError as error:
    raise SystemExit(f"channel metadata is malformed: {error}")

if not isinstance(metadata, dict):
    raise SystemExit("channel metadata must be a JSON object")
channels = metadata.get("channels")
if not isinstance(channels, dict):
    raise SystemExit("channel metadata channels must be an object")
ga_status = metadata.get("ga_status")
expected_channels = (
    {"alpha", "beta"} if ga_status == "preview"
    else {"alpha", "beta", "stable"} if ga_status == "active"
    else None
)
if expected_channels is None:
    raise SystemExit("channel metadata ga_status must be preview or active")
if set(channels) != expected_channels:
    unknown = sorted(set(channels) - {"alpha", "beta", "stable"})
    if unknown:
        raise SystemExit(f"unknown self-update channel in metadata: {unknown[0]}")
    raise SystemExit(
        f"{ga_status} channel metadata must contain exactly "
        + ", ".join(sorted(expected_channels))
    )
if list(channels) != sorted(expected_channels):
    raise SystemExit("channel metadata ordering drifted")

for channel in sorted(expected_channels):
    version = channels[channel]
    if not isinstance(version, str):
        raise SystemExit(f"metadata channel {channel} must map to a version")
    pattern = (
        r"[0-9]+\.[0-9]+\.[0-9]+"
        if channel == "stable"
        else rf"[0-9]+\.[0-9]+\.[0-9]+-{channel}\.[0-9]+"
    )
    if not re.fullmatch(pattern, version):
        raise SystemExit(f"metadata channel {channel} points at {version}")
    release = metadata["releases"].get(version)
    if not isinstance(release, dict) or release.get("status") != "active":
        raise SystemExit(f"metadata channel {channel} does not point at an active release")
    print(f"{channel}={version}")
print(f"ga_status={ga_status}")
PY
)" || fail "${metadata_values}"

metadata_alpha=""
metadata_beta=""
metadata_stable=""
metadata_ga_status=""
while IFS='=' read -r key value; do
  case "${key}" in
    alpha)
      metadata_alpha="${value}"
      ;;
    beta)
      metadata_beta="${value}"
      ;;
    stable)
      metadata_stable="${value}"
      ;;
    ga_status)
      metadata_ga_status="${value}"
      ;;
  esac
done <<<"${metadata_values}"

[[ -n "${metadata_alpha}" && -n "${metadata_beta}" ]] || fail "metadata versions could not be extracted"
if [[ "${metadata_ga_status}" == "active" ]]; then
  [[ -n "${metadata_stable}" ]] || fail "active metadata stable version could not be extracted"
elif [[ "${metadata_ga_status}" == "preview" ]]; then
  [[ -z "${metadata_stable}" ]] || fail "preview metadata must not extract a stable version"
else
  fail "metadata ga_status could not be extracted"
fi

for dispatcher in index stable alpha beta; do
  [[ -f "${INSTALL_ROOT}/${dispatcher}" ]] || fail "dispatcher missing: ${INSTALL_ROOT}/${dispatcher}"
  grep -q 'CHANNEL_METADATA_URL="https://github.com/sifr-lang/sifr/releases/download/channels/channels.json"' \
    "${INSTALL_ROOT}/${dispatcher}" || fail "dispatcher does not resolve channels from GitHub: ${dispatcher}"
  grep -q 'INSTALLER_RELEASE_BASE_URL="https://github.com/sifr-lang/sifr/releases/download"' \
    "${INSTALL_ROOT}/${dispatcher}" || fail "dispatcher does not download installers from GitHub: ${dispatcher}"
  grep -q 'sifr-installer-${resolved_version}' \
    "${INSTALL_ROOT}/${dispatcher}" || fail "dispatcher does not use GitHub installer assets: ${dispatcher}"
done

grep -q 'DEFAULT_CHANNEL="stable"' "${INSTALL_ROOT}/index" || fail "index dispatcher must default to stable"
grep -q 'DEFAULT_CHANNEL="stable"' "${INSTALL_ROOT}/stable" || fail "stable dispatcher must default to stable"
grep -q 'DEFAULT_CHANNEL="alpha"' "${INSTALL_ROOT}/alpha" || fail "alpha dispatcher must default to alpha"
grep -q 'DEFAULT_CHANNEL="beta"' "${INSTALL_ROOT}/beta" || fail "beta dispatcher must default to beta"

[[ ! -e "${INSTALL_ROOT}/metadata/channels.json" ]] || fail "website must not publish metadata/channels.json"
[[ ! -d "${INSTALL_ROOT}/versions" ]] || fail "website must not publish immutable version installers"
