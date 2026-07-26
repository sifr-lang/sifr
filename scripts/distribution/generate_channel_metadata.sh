#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/generate_channel_metadata.sh [options]

Generate the canonical schema-v2 governed release index.

Required:
  --out <file>
  --generation <positive-integer>
  --alpha-release <file>    JSON object with exactly {version, release}
  --beta-release <file>     JSON object with exactly {version, release}

Optional:
  --stable-release <file>   Required only with --ga-status active
  --ga-status preview|active
EOF
}

OUT=""
GENERATION=""
GA_STATUS="preview"
ALPHA_RELEASE=""
BETA_RELEASE=""
STABLE_RELEASE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out) OUT="${2:-}"; shift 2 ;;
    --generation) GENERATION="${2:-}"; shift 2 ;;
    --ga-status) GA_STATUS="${2:-}"; shift 2 ;;
    --alpha-release) ALPHA_RELEASE="${2:-}"; shift 2 ;;
    --beta-release) BETA_RELEASE="${2:-}"; shift 2 ;;
    --stable-release) STABLE_RELEASE="${2:-}"; shift 2 ;;
    --help) usage; exit 0 ;;
    *) echo "unknown argument: $1" >&2; usage >&2; exit 2 ;;
  esac
done

if [[ -z "${OUT}" || -z "${GENERATION}" || -z "${ALPHA_RELEASE}" || -z "${BETA_RELEASE}" ]]; then
  echo "--out, --generation, --alpha-release, and --beta-release are required" >&2
  exit 2
fi
if [[ "${GA_STATUS}" != "preview" && "${GA_STATUS}" != "active" ]]; then
  echo "--ga-status must be preview or active" >&2
  exit 2
fi
if [[ "${GA_STATUS}" == "active" && -z "${STABLE_RELEASE}" ]]; then
  echo "--stable-release is required for active GA metadata" >&2
  exit 2
fi
if [[ "${GA_STATUS}" == "preview" && -n "${STABLE_RELEASE}" ]]; then
  echo "--stable-release is forbidden for preview metadata" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
args=(
  generate-release-index
  --out "${OUT}"
  --generation "${GENERATION}"
  --ga-status "${GA_STATUS}"
  --release "${ALPHA_RELEASE}"
  --release "${BETA_RELEASE}"
)

alpha_version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "${ALPHA_RELEASE}")"
beta_version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "${BETA_RELEASE}")"
args+=(--channel "alpha=${alpha_version}" --channel "beta=${beta_version}")
if [[ -n "${STABLE_RELEASE}" ]]; then
  stable_version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "${STABLE_RELEASE}")"
  args+=(--release "${STABLE_RELEASE}" --channel "stable=${stable_version}")
fi

python3 "${SCRIPT_DIR}/release_governance.py" "${args[@]}"
