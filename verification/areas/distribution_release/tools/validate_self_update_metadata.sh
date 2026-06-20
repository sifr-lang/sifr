#!/usr/bin/env bash

set -euo pipefail

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
if set(metadata) != {"schema_version", "channels"}:
    raise SystemExit("channel metadata contains unsupported fields")
schema_version = metadata.get("schema_version")
if type(schema_version) is not int or schema_version != 1:
    raise SystemExit("channel metadata schema_version must be 1")

channels = metadata.get("channels")
if not isinstance(channels, dict):
    raise SystemExit("channel metadata channels must be an object")
if "stable" in channels:
    raise SystemExit("stable channel metadata is disabled until stable channels are supported")
if set(channels) != {"alpha", "beta"}:
    unknown = sorted(set(channels) - {"alpha", "beta"})
    if unknown:
        raise SystemExit(f"unknown self-update channel in metadata: {unknown[0]}")
    raise SystemExit("channel metadata must contain alpha and beta channels")
if list(channels) != ["alpha", "beta"]:
    raise SystemExit("channel metadata ordering drifted")

for channel in ("alpha", "beta"):
    version = channels[channel]
    if not isinstance(version, str):
        raise SystemExit(f"metadata channel {channel} must map to a version")
    if not re.fullmatch(rf"[0-9]+\.[0-9]+\.[0-9]+-{channel}\.[0-9]+", version):
        raise SystemExit(f"metadata channel {channel} points at {version}")
    print(f"{channel}={version}")
PY
)" || fail "${metadata_values}"

metadata_alpha=""
metadata_beta=""
while IFS='=' read -r key value; do
  case "${key}" in
    alpha)
      metadata_alpha="${value}"
      ;;
    beta)
      metadata_beta="${value}"
      ;;
  esac
done <<<"${metadata_values}"

[[ -n "${metadata_alpha}" && -n "${metadata_beta}" ]] || fail "metadata versions could not be extracted"

for dispatcher in index alpha beta; do
  [[ -f "${INSTALL_ROOT}/${dispatcher}" ]] || fail "dispatcher missing: ${INSTALL_ROOT}/${dispatcher}"
  grep -q 'CHANNEL_METADATA_URL="https://github.com/sifr-lang/sifr/releases/download/channels/channels.json"' \
    "${INSTALL_ROOT}/${dispatcher}" || fail "dispatcher does not resolve channels from GitHub: ${dispatcher}"
  grep -q 'INSTALLER_RELEASE_BASE_URL="https://github.com/sifr-lang/sifr/releases/download"' \
    "${INSTALL_ROOT}/${dispatcher}" || fail "dispatcher does not download installers from GitHub: ${dispatcher}"
  grep -q 'sifr-installer-${resolved_version}' \
    "${INSTALL_ROOT}/${dispatcher}" || fail "dispatcher does not use GitHub installer assets: ${dispatcher}"
done

grep -q 'DEFAULT_CHANNEL="beta"' "${INSTALL_ROOT}/index" || fail "index dispatcher must default to beta"
grep -q 'DEFAULT_CHANNEL="alpha"' "${INSTALL_ROOT}/alpha" || fail "alpha dispatcher must default to alpha"
grep -q 'DEFAULT_CHANNEL="beta"' "${INSTALL_ROOT}/beta" || fail "beta dispatcher must default to beta"

[[ ! -e "${INSTALL_ROOT}/metadata/channels.json" ]] || fail "website must not publish metadata/channels.json"
[[ ! -d "${INSTALL_ROOT}/versions" ]] || fail "website must not publish immutable version installers"
