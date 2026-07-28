#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo "usage: run_schema_bootstrap_public_smoke.sh --repository OWNER/REPO --version X.Y.Z-beta.N --index PATH --dispatchers DIR --out DIR" >&2
  exit 2
}

repository=""
version=""
index=""
dispatchers=""
out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --version) version="${2:-}"; shift 2 ;;
    --index) index="${2:-}"; shift 2 ;;
    --dispatchers) dispatchers="${2:-}"; shift 2 ;;
    --out) out="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ -n "${repository}" && -n "${version}" && -n "${index}" &&
  -n "${dispatchers}" && -n "${out}" ]] || usage
[[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-beta\.[0-9]+$ ]] || usage
test -f "${index}" && test -f "${dispatchers}/index" &&
  test -f "${dispatchers}/stable" || usage
test -z "${SIFR_TEST_CHANNEL_METADATA_PATH:-}" || {
  echo "schema-bootstrap-smoke: qualification metadata override must be absent" >&2
  exit 2
}
mkdir -p "${out}"

download_until_matches() {
  local url="$1"
  local expected="$2"
  local destination="$3"
  local deadline=$((SECONDS + 180))
  while (( SECONDS < deadline )); do
    if curl -fsSL \
      -H "Cache-Control: no-cache" \
      "${url}?sifr_publication_smoke=${RANDOM}-${SECONDS}" \
      -o "${destination}" &&
      cmp "${expected}" "${destination}"
    then
      return 0
    fi
    sleep 5
  done
  echo "schema-bootstrap-smoke: public bytes did not converge: ${url}" >&2
  return 2
}

live_index="${out}/governance-index.txt"
download_until_matches \
  "https://github.com/${repository}/releases/download/channels/channels.json" \
  "${index}" \
  "${live_index}"
scripts/distribution/release_governance.py validate \
  --kind release-index \
  --input "${live_index}" \
  --require-canonical

default_dispatcher="${out}/dispatcher-default.txt"
stable_dispatcher="${out}/stable-dispatcher.sh"
download_until_matches https://sifr.sh/install "${dispatchers}/index" "${default_dispatcher}"
download_until_matches \
  https://sifr.sh/install/stable \
  "${dispatchers}/stable" \
  "${stable_dispatcher}"

install_root="$(mktemp -d)"
stable_root="$(mktemp -d)"
stable_log="${out}/dispatcher-stable-rejection.txt"
if HOME="${stable_root}" \
  SIFR_INSTALL_DIR="${stable_root}/bin" \
  SIFR_SYSROOT_INSTALL_DIR="${stable_root}" \
  SIFR_NO_MODIFY_PATH=1 \
  sh "${stable_dispatcher}" >"${stable_log}" 2>&1
then
  echo "schema-bootstrap-smoke: stable dispatcher activated during preview" >&2
  exit 2
fi
grep -F "stable channel installs require active GA metadata" "${stable_log}" >/dev/null || {
  echo "schema-bootstrap-smoke: stable rejection was not the governed preview result" >&2
  exit 2
}

HOME="${install_root}" \
SIFR_INSTALL_DIR="${install_root}/bin" \
SIFR_SYSROOT_INSTALL_DIR="${install_root}" \
  SIFR_NO_MODIFY_PATH=1 \
  sh "${default_dispatcher}"
"${install_root}/bin/sifr" self update --dry-run --format json \
  >"${out}/installed-self-update.txt"
scripts/distribution/release_governance.py validate \
  --kind self-update-plan \
  --input "${out}/installed-self-update.txt"
test "$(
  jq -r '.target_version + ":" + .resolved_channel + ":" + (.would_run_installer | tostring)' \
    "${out}/installed-self-update.txt"
)" = "${version}:beta:false" || {
  echo "schema-bootstrap-smoke: installed beta did not resolve a public no-op plan" >&2
  exit 2
}
