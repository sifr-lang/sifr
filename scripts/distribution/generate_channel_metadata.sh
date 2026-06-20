#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/generate_channel_metadata.sh --out <file> --alpha-version <version> --beta-version <version>

Generate the Sifr self-update channel metadata JSON.
EOF
}

OUT=""
ALPHA_VERSION=""
BETA_VERSION=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out)
      OUT="${2:-}"
      shift 2
      ;;
    --alpha-version)
      ALPHA_VERSION="${2:-}"
      shift 2
      ;;
    --beta-version)
      BETA_VERSION="${2:-}"
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

if [[ -z "${OUT}" || -z "${ALPHA_VERSION}" || -z "${BETA_VERSION}" ]]; then
  echo "--out, --alpha-version, and --beta-version are required" >&2
  usage >&2
  exit 2
fi

preview_version_channel() {
  local version="$1"
  if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$ ]]; then
    return 1
  fi
  printf '%s\n' "${BASH_REMATCH[1]}"
}

if [[ "$(preview_version_channel "${ALPHA_VERSION}")" != "alpha" ]]; then
  echo "alpha version must use an -alpha.N prerelease label: ${ALPHA_VERSION}" >&2
  exit 2
fi

if [[ "$(preview_version_channel "${BETA_VERSION}")" != "beta" ]]; then
  echo "beta version must use a -beta.N prerelease label: ${BETA_VERSION}" >&2
  exit 2
fi

mkdir -p "$(dirname "${OUT}")"
cat >"${OUT}" <<EOF
{
  "schema_version": 1,
  "channels": {
    "alpha": "${ALPHA_VERSION}",
    "beta": "${BETA_VERSION}"
  }
}
EOF
