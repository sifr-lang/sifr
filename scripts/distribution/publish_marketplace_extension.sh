#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/publish_marketplace_extension.sh \
  --package <file> --publisher <id> --extension <id> --version <version> \
  --expected-sha256 <sha256> --verified-out <file>

Verify and reuse an exact Marketplace version, or publish the exact qualified
VSIX once when it is absent. The raw Marketplace VSIX is written to
--verified-out after byte and metadata verification.
EOF
}

PACKAGE=""
PUBLISHER=""
EXTENSION=""
VERSION=""
EXPECTED_SHA256=""
VERIFIED_OUT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --package) PACKAGE="${2:-}"; shift 2 ;;
    --publisher) PUBLISHER="${2:-}"; shift 2 ;;
    --extension) EXTENSION="${2:-}"; shift 2 ;;
    --version) VERSION="${2:-}"; shift 2 ;;
    --expected-sha256) EXPECTED_SHA256="${2:-}"; shift 2 ;;
    --verified-out) VERIFIED_OUT="${2:-}"; shift 2 ;;
    --help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "${PACKAGE}" || -z "${PUBLISHER}" || -z "${EXTENSION}" || \
      -z "${VERSION}" || -z "${EXPECTED_SHA256}" || -z "${VERIFIED_OUT}" ]]; then
  usage >&2
  exit 2
fi
if [[ -e "${VERIFIED_OUT}" || -L "${VERIFIED_OUT}" ]]; then
  echo "--verified-out must not already exist" >&2
  exit 2
fi
if [[ -z "${VSCE_BIN:-}" || ! -x "${VSCE_BIN}" ]]; then
  echo "VSCE_BIN must name the executable pinned Marketplace publisher" >&2
  exit 2
fi
if [[ ! "${PUBLISHER}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ || \
      ! "${EXTENSION}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]]; then
  echo "publisher and extension must use Marketplace identifier characters" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFY="${SCRIPT_DIR}/verify_marketplace_vsix.py"
python3 "${VERIFY}" \
  --vsix "${PACKAGE}" \
  --expected-sha256 "${EXPECTED_SHA256}" \
  --publisher "${PUBLISHER}" \
  --extension "${EXTENSION}" \
  --version "${VERSION}"

RAW_URL="https://${PUBLISHER}.gallery.vsassets.io/_apis/public/gallery/publisher/${PUBLISHER}/extension/${EXTENSION}/${VERSION}/assetbyname/Microsoft.VisualStudio.Services.VSIXPackage"
TEMP_ROOT="$(mktemp -d)"
trap 'rm -rf "${TEMP_ROOT}"' EXIT

fetch_raw() {
  local destination="$1"
  curl --silent --show-error --location \
    --connect-timeout 10 --max-time 60 \
    --output "${destination}" --write-out '%{http_code}' \
    "${RAW_URL}"
}

RAW_PACKAGE="${TEMP_ROOT}/marketplace.vsix"
STATUS="$(fetch_raw "${RAW_PACKAGE}")"
case "${STATUS}" in
  200)
    ;;
  404)
    rm -f "${RAW_PACKAGE}"
    if [[ -z "${VSCE_PAT:-}" ]]; then
      echo "VSCE_PAT is required when the Marketplace version is absent" >&2
      exit 2
    fi
    GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="${VSCE_PAT}" \
      "${VSCE_BIN}" publish --packagePath "${PACKAGE}"
    for _ in {1..12}; do
      STATUS="$(fetch_raw "${RAW_PACKAGE}")"
      if [[ "${STATUS}" == "200" ]]; then
        break
      fi
      if [[ "${STATUS}" != "404" ]]; then
        echo "Marketplace raw asset returned HTTP ${STATUS}" >&2
        exit 2
      fi
      rm -f "${RAW_PACKAGE}"
      sleep 5
    done
    if [[ "${STATUS}" != "200" ]]; then
      echo "Marketplace raw asset did not converge after publication" >&2
      exit 2
    fi
    ;;
  *)
    echo "Marketplace raw asset returned HTTP ${STATUS}" >&2
    exit 2
    ;;
esac

python3 "${VERIFY}" \
  --vsix "${RAW_PACKAGE}" \
  --expected-sha256 "${EXPECTED_SHA256}" \
  --publisher "${PUBLISHER}" \
  --extension "${EXTENSION}" \
  --version "${VERSION}"
mv "${RAW_PACKAGE}" "${VERIFIED_OUT}"
