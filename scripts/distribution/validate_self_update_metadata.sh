#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/validate_self_update_metadata.sh --install-root <dir>

Validate that self-update channel metadata, preview dispatchers, and immutable
version installers agree.
EOF
}

INSTALL_ROOT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install-root)
      INSTALL_ROOT="${2:-}"
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

extract_var() {
  local file="$1"
  local name="$2"
  sed -n "s/^${name}=\"\\(.*\\)\"$/\\1/p" "${file}" | head -n 1
}

metadata_path="${INSTALL_ROOT}/metadata/channels.json"
[[ -f "${metadata_path}" ]] || fail "channel metadata missing: ${metadata_path}"

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
    raise SystemExit("stable channel metadata is disabled until Phase 39")
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
done

index_alpha="$(extract_var "${INSTALL_ROOT}/index" ALPHA_VERSION)"
index_beta="$(extract_var "${INSTALL_ROOT}/index" BETA_VERSION)"
alpha_alpha="$(extract_var "${INSTALL_ROOT}/alpha" ALPHA_VERSION)"
alpha_beta="$(extract_var "${INSTALL_ROOT}/alpha" BETA_VERSION)"
beta_alpha="$(extract_var "${INSTALL_ROOT}/beta" ALPHA_VERSION)"
beta_beta="$(extract_var "${INSTALL_ROOT}/beta" BETA_VERSION)"

[[ "${index_alpha}" == "${alpha_alpha}" && "${index_alpha}" == "${beta_alpha}" ]] || fail "dispatcher ALPHA_VERSION drift"
[[ "${index_beta}" == "${alpha_beta}" && "${index_beta}" == "${beta_beta}" ]] || fail "dispatcher BETA_VERSION drift"
[[ "${metadata_alpha}" == "${index_alpha}" ]] || fail "metadata alpha version drift: metadata=${metadata_alpha} dispatcher=${index_alpha}"
[[ "${metadata_beta}" == "${index_beta}" ]] || fail "metadata beta version drift: metadata=${metadata_beta} dispatcher=${index_beta}"

validate_installer() {
  local channel="$1"
  local version="$2"
  local installer="${INSTALL_ROOT}/versions/${version}"
  [[ -f "${installer}" ]] || fail "immutable installer missing for ${channel}: ${installer}"
  local installer_version
  installer_version="$(extract_var "${installer}" APP_VERSION)"
  [[ -n "${installer_version}" ]] || fail "immutable installer missing APP_VERSION for ${channel}: ${installer}"
  [[ "${installer_version}" == "${version}" ]] || fail "immutable installer APP_VERSION drift for ${channel}: metadata=${version} installer=${installer_version}"
}

validate_installer alpha "${metadata_alpha}"
validate_installer beta "${metadata_beta}"
