#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/run_stable_public_smoke.sh \
  --repository OWNER/REPO --version X.Y.Z --index PATH --dispatchers DIR \
  --asset-digests PATH --marketplace-vsix PATH --out DIR
EOF
  exit 2
}

repository=""
version=""
index=""
dispatchers=""
asset_digests=""
marketplace_vsix=""
out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --version) version="${2:-}"; shift 2 ;;
    --index) index="${2:-}"; shift 2 ;;
    --dispatchers) dispatchers="${2:-}"; shift 2 ;;
    --asset-digests) asset_digests="${2:-}"; shift 2 ;;
    --marketplace-vsix) marketplace_vsix="${2:-}"; shift 2 ;;
    --out) out="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ "${repository}" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ &&
  "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || usage
for path in "${index}" "${dispatchers}/index" "${dispatchers}/stable" \
  "${asset_digests}" "${marketplace_vsix}"
do
  [[ -f "${path}" && ! -L "${path}" ]] || usage
done
[[ ! -e "${out}" && ! -L "${out}" ]] || {
  echo "stable-public-smoke: --out must not already exist" >&2
  exit 2
}
test -z "${SIFR_TEST_CHANNEL_METADATA_PATH:-}" || {
  echo "stable-public-smoke: qualification metadata override must be absent" >&2
  exit 2
}
jq -e '
  type == "object"
  and length > 0
  and all(
    to_entries[];
    (.key | test("^[A-Za-z0-9][A-Za-z0-9._+-]*$"))
    and (.value | test("^[0-9a-f]{64}$"))
  )
' "${asset_digests}" >/dev/null || {
  echo "stable-public-smoke: invalid asset digest map" >&2
  exit 2
}
cmp <(jq -cS . "${asset_digests}") "${asset_digests}" || {
  echo "stable-public-smoke: asset digest map must be canonical JSON" >&2
  exit 2
}
mkdir "${out}"

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
  echo "stable-public-smoke: public bytes did not converge: ${url}" >&2
  return 2
}

download_until_matches \
  "https://github.com/${repository}/releases/download/channels/channels.json" \
  "${index}" \
  "${out}/governed-index.json"
scripts/distribution/release_governance.py validate \
  --kind release-index \
  --input "${out}/governed-index.json" \
  --require-canonical
jq -e \
  --arg version "${version}" \
  '.ga_status == "active"
   and .channels.stable == $version
   and .releases[$version].channel == "stable"
   and .releases[$version].status == "active"' \
  "${out}/governed-index.json" >/dev/null || {
  echo "stable-public-smoke: public index did not activate the expected stable release" >&2
  exit 2
}

download_until_matches \
  https://sifr.sh/install \
  "${dispatchers}/index" \
  "${out}/install-dispatcher"
download_until_matches \
  https://sifr.sh/install/stable \
  "${dispatchers}/stable" \
  "${out}/stable-dispatcher"

version_assets_root="$(mktemp -d)"
install_root="$(mktemp -d)"
trap 'rm -rf "${version_assets_root}" "${install_root}"' EXIT
while IFS= read -r name; do
  expected_sha="$(jq -r --arg name "${name}" '.[$name]' "${asset_digests}")"
  destination="${version_assets_root}/${name}"
  curl -fsSL --connect-timeout 10 --max-time 120 \
    "https://github.com/${repository}/releases/download/${version}/${name}" \
    -o "${destination}"
  actual_sha="$(sha256sum "${destination}" | awk '{print $1}')"
  test "${actual_sha}" = "${expected_sha}" || {
    echo "stable-public-smoke: public asset digest drifted: ${name}" >&2
    exit 2
  }
done < <(jq -r 'keys[]' "${asset_digests}")
cp "${asset_digests}" "${out}/version-assets.json"

GH_TOKEN="" \
SITE_TOKEN="" \
VSCE_PAT="" \
HOME="${install_root}" \
SIFR_INSTALL_DIR="${install_root}/bin" \
SIFR_SYSROOT_INSTALL_DIR="${install_root}" \
SIFR_NO_MODIFY_PATH=1 \
  sh "${out}/stable-dispatcher"
GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  "${install_root}/bin/sifr" self update --dry-run --format json \
  >"${out}/installed-self-update.json"
scripts/distribution/release_governance.py validate \
  --kind self-update-plan \
  --input "${out}/installed-self-update.json"
jq -e \
  --arg version "${version}" \
  '.current_version == $version
   and .target_version == $version
   and .resolved_channel == "stable"
   and .would_run_installer == false' \
  "${out}/installed-self-update.json" >/dev/null || {
  echo "stable-public-smoke: installed stable release did not resolve a no-op" >&2
  exit 2
}
cp "${marketplace_vsix}" "${out}/marketplace.vsix"
