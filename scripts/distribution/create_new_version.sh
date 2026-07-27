#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/create_new_version.sh --channel alpha|beta --version <preview> --dry-run [options]

Render and validate a publication plan. This command has no mutation mode;
GitHub publication is owned exclusively by release-publication.yml.

Required:
  --channel alpha|beta       Preview channel to publish
  --version <preview>        SemVer prerelease, for example 0.1.0-beta.2
  --dry-run                  Render the exact non-mutating plan
  --site-repo <path>         Clean sifr-website checkout at the proposed base
  --release-index <file>     Canonical schema-v2 governed release index

Options:
  --base-ref <ref>           Exact Sifr source ref (default: HEAD)
  --help                     Show this help
EOF
}

CHANNEL=""
VERSION=""
DRY_RUN=0
BASE_REF="HEAD"
SITE_REPO="${SIFR_SITE_REPO:-}"
RELEASE_INDEX=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel) CHANNEL="${2:-}"; shift 2 ;;
    --version) VERSION="${2:-}"; shift 2 ;;
    --dry-run) DRY_RUN=1; shift ;;
    --base-ref) BASE_REF="${2:-}"; shift 2 ;;
    --site-repo) SITE_REPO="${2:-}"; shift 2 ;;
    --release-index) RELEASE_INDEX="${2:-}"; shift 2 ;;
    --real-run|--mutation-mode|--artifact-dir|--binary|--sysroot-root|--work-dir)
      echo "create-new-version: local mutation and artifact modes are removed; use release-publication.yml" >&2
      exit 2
      ;;
    --help) usage; exit 0 ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

fail() {
  echo "create-new-version: $*" >&2
  exit 2
}

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
  fi
}

preview_channel_for_version() {
  local version="$1"
  if [[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$ ]]; then
    printf '%s\n' "${BASH_REMATCH[1]}"
    return
  fi
  return 1
}

extract_var() {
  local file="$1"
  local name="$2"
  sed -n "s/^${name}=\"\\(.*\\)\"$/\\1/p" "${file}" | head -n 1
}

[[ "${DRY_RUN}" -eq 1 ]] || fail "--dry-run is required; local mutation is disabled"
[[ "${CHANNEL}" == "alpha" || "${CHANNEL}" == "beta" ]] ||
  fail "--channel must be alpha or beta"
[[ -n "${VERSION}" ]] || fail "--version is required"
version_channel="$(preview_channel_for_version "${VERSION}")" ||
  fail "version must be a semver prerelease using -alpha.N or -beta.N: ${VERSION}"
[[ "${version_channel}" == "${CHANNEL}" ]] ||
  fail "version ${VERSION} belongs to ${version_channel}, not ${CHANNEL}"
[[ -n "${SITE_REPO}" && -d "${SITE_REPO}/.git" ]] ||
  fail "--site-repo must name a Git checkout"
[[ -n "${RELEASE_INDEX}" && -f "${RELEASE_INDEX}" ]] ||
  fail "--release-index must name the canonical schema-v2 index"

python3 "${SCRIPT_DIR}/release_governance.py" validate \
  --kind release-index \
  --input "${RELEASE_INDEX}" \
  --require-canonical >/dev/null

source_sha="$(git -C "${REPO_ROOT}" rev-parse "${BASE_REF}^{commit}")"
[[ "${source_sha}" =~ ^[0-9a-f]{40}$ ]] || fail "base ref did not resolve to an exact commit"
site_base_commit="$(git -C "${SITE_REPO}" rev-parse HEAD)"
[[ "${site_base_commit}" =~ ^[0-9a-f]{40}$ ]] || fail "site base did not resolve to an exact commit"
[[ -z "$(git -C "${SITE_REPO}" status --porcelain --untracked-files=all)" ]] ||
  fail "site checkout must be clean"

read -r generation current_alpha current_beta ga_status < <(
  python3 - "${RELEASE_INDEX}" <<'PY'
import json
import sys

value = json.load(open(sys.argv[1], encoding="utf-8"))
print(
    value["generation"],
    value["channels"]["alpha"],
    value["channels"]["beta"],
    value["ga_status"],
)
PY
)
case "${ga_status}" in
  preview) site_default_channel="beta" ;;
  active) site_default_channel="stable" ;;
  *) fail "release index has unsupported ga_status: ${ga_status}" ;;
esac

install_root="${SITE_REPO}/apps/sifr-site/public/install"
required_dispatchers=(index alpha beta)
if [[ "${ga_status}" == "active" ]]; then
  required_dispatchers+=(stable)
fi
for dispatcher in "${required_dispatchers[@]}"; do
  [[ -f "${install_root}/${dispatcher}" ]] ||
    fail "site dispatcher missing: ${install_root}/${dispatcher}"
  grep -q 'CHANNEL_METADATA_URL="https://github.com/sifr-lang/sifr/releases/download/channels/channels.json"' \
    "${install_root}/${dispatcher}" ||
    fail "site dispatcher drift: ${dispatcher} must resolve channels from GitHub"
done
[[ "$(extract_var "${install_root}/index" DEFAULT_CHANNEL)" == "${site_default_channel}" ]] ||
  fail "site dispatcher drift: index must default to ${site_default_channel}"
[[ "$(extract_var "${install_root}/alpha" DEFAULT_CHANNEL)" == "alpha" ]] ||
  fail "site dispatcher drift: alpha must default to alpha"
[[ "$(extract_var "${install_root}/beta" DEFAULT_CHANNEL)" == "beta" ]] ||
  fail "site dispatcher drift: beta must default to beta"
if [[ -f "${install_root}/stable" ]]; then
  grep -q 'CHANNEL_METADATA_URL="https://github.com/sifr-lang/sifr/releases/download/channels/channels.json"' \
    "${install_root}/stable" ||
    fail "site dispatcher drift: stable must resolve channels from GitHub"
  [[ "$(extract_var "${install_root}/stable" DEFAULT_CHANNEL)" == "stable" ]] ||
    fail "site dispatcher drift: stable must default to stable"
fi
[[ ! -e "${install_root}/channels.json" ]] ||
  fail "site checkout contains forbidden top-level channels.json shadow state"
[[ ! -d "${install_root}/metadata" && ! -d "${install_root}/versions" ]] ||
  fail "site checkout contains forbidden metadata or version shadow state"

index_sha256="$(sha256_file "${RELEASE_INDEX}")"
next_generation="$((generation + 1))"
new_alpha="${current_alpha}"
new_beta="${current_beta}"
if [[ "${CHANNEL}" == "alpha" ]]; then
  new_alpha="${VERSION}"
else
  new_beta="${VERSION}"
fi

cat <<EOF
release-publication-plan
schema_version=2
mutation_authority=.github/workflows/release-publication.yml
local_mutations=disabled
channel=${CHANNEL}
version=${VERSION}
source_commit=${source_sha}
site_repository=sifr-lang/sifr-website
site_base_commit=${site_base_commit}
current_generation=${generation}
current_index_sha256=${index_sha256}
proposed_generation=${next_generation}
ga_status=${ga_status}
current_alpha=${current_alpha}
current_beta=${current_beta}
proposed_alpha=${new_alpha}
proposed_beta=${new_beta}
site_default_channel=${site_default_channel}
version_asset_policy=write-once
channel_index_policy=replace-only
site_deployment=paired-after-index
EOF
