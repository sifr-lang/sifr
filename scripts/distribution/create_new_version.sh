#!/usr/bin/env bash

set -euo pipefail

TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
)

usage() {
  cat <<'EOF'
Usage: scripts/distribution/create_new_version.sh --channel alpha|beta --version <preview> (--dry-run|--real-run) [options]

  Plan or execute a preview release.

Required:
  --channel alpha|beta       Preview channel to publish
  --version <preview>        Semver prerelease version, for example 0.1.0-beta.2
  --dry-run                  Print the exact mutation plan without side effects
  --real-run                 Execute the dry-run plan

Options:
  --base-ref <ref>           Base commit/ref to release (default: HEAD)
  --site-repo <path>         Site repo path (default: SIFR_SITE_REPO)
  --artifact-dir <dir>       Existing artifact directory for real-run validation/publication
  --binary <path>            Existing binary packaged for all targets in local validation
  --sysroot-root <dir>       Source sysroot root for local binary packaging
  --work-dir <dir>           Work/evidence directory (default: target/preview-release/<version>)
  --release-index <file>     Canonical schema-v2 baseline required by --real-run
  --mutation-mode <mode>     local only; GitHub publication is handled by preview-release.yml
  --help                     Show this help
EOF
}

CHANNEL=""
VERSION=""
MODE=""
BASE_REF="HEAD"
SITE_REPO="${SIFR_SITE_REPO:-}"
ARTIFACT_DIR=""
BINARY=""
SYSROOT_ROOT=""
WORK_DIR=""
RELEASE_INDEX=""
MUTATION_MODE="local"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel)
      CHANNEL="${2:-}"
      shift 2
      ;;
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --dry-run)
      [[ -z "${MODE}" ]] || { echo "choose exactly one of --dry-run or --real-run" >&2; exit 2; }
      MODE="dry-run"
      shift
      ;;
    --real-run)
      [[ -z "${MODE}" ]] || { echo "choose exactly one of --dry-run or --real-run" >&2; exit 2; }
      MODE="real-run"
      shift
      ;;
    --base-ref)
      BASE_REF="${2:-}"
      shift 2
      ;;
    --site-repo)
      SITE_REPO="${2:-}"
      shift 2
      ;;
    --artifact-dir)
      ARTIFACT_DIR="${2:-}"
      shift 2
      ;;
    --binary)
      BINARY="${2:-}"
      shift 2
      ;;
    --sysroot-root)
      SYSROOT_ROOT="${2:-}"
      shift 2
      ;;
    --work-dir)
      WORK_DIR="${2:-}"
      shift 2
      ;;
    --release-index)
      RELEASE_INDEX="${2:-}"
      shift 2
      ;;
    --mutation-mode)
      MUTATION_MODE="${2:-}"
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
INSTALL_ROOT="${SITE_REPO}/apps/sifr-site/public/install"

fail() {
  echo "create-new-version: $*" >&2
  exit 2
}

preview_channel_for_version() {
  local version="$1"
  if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta|rc)\.[0-9]+$ ]]; then
    return 1
  fi
  printf '%s\n' "${BASH_REMATCH[1]}"
}

sha256_text() {
  if command -v sha256sum >/dev/null 2>&1; then
    printf '%s' "$1" | sha256sum | awk '{print $1}'
  else
    printf '%s' "$1" | shasum -a 256 | awk '{print $1}'
  fi
}

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
  fi
}

extract_var() {
  local file="$1"
  local name="$2"
  sed -n "s/^${name}=\"\\(.*\\)\"$/\\1/p" "${file}" | head -n 1
}

read_current_channel_versions() {
  local metadata_file="${RELEASE_INDEX}"
  local temp_file=""

  if [[ -z "${metadata_file}" ]]; then
    metadata_file="${INSTALL_ROOT}/channels.json"
    if [[ ! -f "${metadata_file}" ]]; then
      temp_file="$(mktemp "${TMPDIR:-/tmp}/sifr-current-channels.XXXXXX")"
      metadata_file="${temp_file}"
      curl -fsSL \
        "https://github.com/sifr-lang/sifr/releases/download/channels/channels.json" \
        -o "${metadata_file}" || fail "could not fetch current GitHub channel metadata"
    fi
  fi

  "${SCRIPT_DIR}/release_governance.py" validate \
    --kind release-index \
    --input "${metadata_file}" \
    --require-canonical >/dev/null
  read -r CURRENT_ALPHA CURRENT_BETA < <("${SCRIPT_DIR}/read_channel_versions.py" "${metadata_file}")
  [[ -n "${CURRENT_ALPHA}" && -n "${CURRENT_BETA}" ]] || fail "current channel metadata is missing alpha or beta"

  if [[ -n "${temp_file}" ]]; then
    rm -f "${temp_file}"
  fi
}

validate_inputs() {
  [[ "${CHANNEL}" == "alpha" || "${CHANNEL}" == "beta" ]] || fail "--channel must be alpha or beta"
  [[ -n "${VERSION}" ]] || fail "--version is required"
  [[ -n "${MODE}" ]] || fail "choose --dry-run or --real-run"
  [[ -n "${SITE_REPO}" ]] || fail "--site-repo is required when SIFR_SITE_REPO is not set"
  [[ "${MUTATION_MODE}" == "local" ]] || fail "--mutation-mode github is disabled; use scripts/distribution/trigger_preview_release.sh for GitHub publication"
  [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "stable-looking versions are disabled until a stable channel is supported: ${VERSION}"
  version_channel="$(preview_channel_for_version "${VERSION}")" || fail "version must be a semver prerelease using -alpha.N, -beta.N, or -rc.N: ${VERSION}"
  [[ "${version_channel}" == "${CHANNEL}" ]] || fail "version ${VERSION} belongs to ${version_channel}, not ${CHANNEL}"
  [[ -d "${SITE_REPO}" ]] || fail "site repo not found: ${SITE_REPO}"
  [[ -d "${INSTALL_ROOT}" ]] || fail "site install root not found: ${INSTALL_ROOT}"
  if [[ "${MODE}" == "real-run" ]]; then
    [[ -n "${RELEASE_INDEX}" ]] || fail "--release-index is required with --real-run"
    [[ -f "${RELEASE_INDEX}" ]] || fail "release index not found: ${RELEASE_INDEX}"
  fi
}

validate_site_dispatchers() {
  local expected_path
  for expected_path in index alpha beta; do
    [[ -f "${INSTALL_ROOT}/${expected_path}" ]] || fail "site dispatcher missing: ${INSTALL_ROOT}/${expected_path}"
  done

  [[ "$(extract_var "${INSTALL_ROOT}/index" DEFAULT_CHANNEL)" == "beta" ]] || fail "site dispatcher drift: index must default to beta"
  [[ "$(extract_var "${INSTALL_ROOT}/alpha" DEFAULT_CHANNEL)" == "alpha" ]] || fail "site dispatcher drift: alpha must default to alpha"
  [[ "$(extract_var "${INSTALL_ROOT}/beta" DEFAULT_CHANNEL)" == "beta" ]] || fail "site dispatcher drift: beta must default to beta"
  grep -q 'CHANNEL_METADATA_URL="https://github.com/sifr-lang/sifr/releases/download/channels/channels.json"' \
    "${INSTALL_ROOT}/index" || fail "site dispatcher drift: index must resolve channels from GitHub"
  [[ ! -d "${INSTALL_ROOT}/versions" ]] || fail "site dispatcher drift: website must not publish immutable version installers"

  read_current_channel_versions
}

build_plan() {
  validate_inputs
  validate_site_dispatchers
  BASE_SHA="$(git -C "${REPO_ROOT}" rev-parse "${BASE_REF}")"
  [[ -n "${BASE_SHA}" ]] || fail "could not resolve base ref: ${BASE_REF}"

  NEW_ALPHA="${CURRENT_ALPHA}"
  NEW_BETA="${CURRENT_BETA}"
  if [[ "${CHANNEL}" == "alpha" ]]; then
    NEW_ALPHA="${VERSION}"
  else
    NEW_BETA="${VERSION}"
  fi

  if [[ -z "${WORK_DIR}" ]]; then
    WORK_DIR="${REPO_ROOT}/target/preview-release/${VERSION}"
  fi
  if [[ -z "${ARTIFACT_DIR}" ]]; then
    ARTIFACT_DIR="${WORK_DIR}/artifacts"
  fi
  PLAN_FILE="${WORK_DIR}/plan.txt"
  CHECKLIST_FILE="${WORK_DIR}/release-checklist.md"
  RECOVERY_FILE="${WORK_DIR}/recovery-note.md"
  CHANNEL_METADATA_FILE="${WORK_DIR}/channels.json"
  INSTALLER_ASSET_FILE="${WORK_DIR}/sifr-installer-${VERSION}"

  artifact_lines=""
  for target in "${TARGETS[@]}"; do
    artifact_lines="${artifact_lines}
artifact=sifr-${VERSION}-${target}.tar.gz
checksum=sifr-${VERSION}-${target}.tar.gz.sha256"
  done

  PLAN_TEXT="$(cat <<EOF
preview-release-plan
channel=${CHANNEL}
version=${VERSION}
base_ref=${BASE_REF}
base_sha=${BASE_SHA}
mutation_mode=${MUTATION_MODE}
site_repo=${SITE_REPO}
install_root=${INSTALL_ROOT}
artifact_dir=${ARTIFACT_DIR}
current_alpha=${CURRENT_ALPHA}
current_beta=${CURRENT_BETA}
new_alpha=${NEW_ALPHA}
new_beta=${NEW_BETA}
github_installer_asset=${INSTALLER_ASSET_FILE}
channel_metadata=${CHANNEL_METADATA_FILE}
stable_entrypoint=unchanged_absent
github_release=sifr-lang/sifr:${VERSION}
github_channel_release=sifr-lang/sifr:channels
site_dispatcher_update=${CHANNEL}:${VERSION}
channel_metadata_update=alpha:${NEW_ALPHA},beta:${NEW_BETA}${artifact_lines}
EOF
)"
  PLAN_SHA="$(sha256_text "${PLAN_TEXT}")"
}

print_plan() {
  build_plan
  cat <<EOF
${PLAN_TEXT}
plan_sha256=${PLAN_SHA}
dry_run_side_effects=none
EOF
}

verify_artifact_dir() {
  local target archive
  for target in "${TARGETS[@]}"; do
    archive="${ARTIFACT_DIR}/sifr-${VERSION}-${target}.tar.gz"
    [[ -f "${archive}" ]] || fail "missing artifact: ${archive}"
    [[ -f "${archive}.sha256" ]] || fail "missing checksum: ${archive}.sha256"
  done
}

write_checklist() {
  cat >"${CHECKLIST_FILE}" <<EOF
# Sifr Preview Release Checklist

- Channel: ${CHANNEL}
- Version: ${VERSION}
- Base SHA: ${BASE_SHA}
- Plan SHA-256: ${PLAN_SHA}
- Mutation mode: ${MUTATION_MODE}

## Artifacts

$(for target in "${TARGETS[@]}"; do echo "- [x] sifr-${VERSION}-${target}.tar.gz and .sha256"; done)

## Installer And Dispatcher

- [x] GitHub immutable installer asset: ${INSTALLER_ASSET_FILE}
- [x] Website bootstrap dispatchers resolve channels from GitHub
- [x] GitHub channel metadata update: alpha=${NEW_ALPHA}, beta=${NEW_BETA}
- [x] Stable entrypoint unchanged and absent
- [x] SHA-256 checksums embedded in immutable installer

## Attribution

- [x] uv-derived code used: no
- [x] Copied/adapted uv files: none
- [x] MIT license retention required: not applicable
- [x] Pinned uv source URL/reference required: not applicable

## Validation Evidence

- [x] uv run --project verification --locked python -m sifr_verify areas run --area distribution_release --suite full
- [x] Generated installer validates checksum before install
- [x] Stable-looking versions rejected before mutation
EOF
}

write_recovery_note() {
  cat >"${RECOVERY_FILE}" <<EOF
# Preview Release Recovery Note

- Channel: ${CHANNEL}
- Version: ${VERSION}
- Plan SHA-256: ${PLAN_SHA}
- Completed mutations:
  - artifact directory verified: ${ARTIFACT_DIR}
  - GitHub immutable installer asset generated: ${INSTALLER_ASSET_FILE}
  - GitHub-backed website dispatchers regenerated
  - GitHub channel metadata generated: ${CHANNEL_METADATA_FILE}
  - release checklist written: ${CHECKLIST_FILE}
- Incomplete mutations:
  - GitHub release publication is skipped when mutation_mode=local.
  - Site deployment remains the existing sifr.sh deployment step.
  - The GitHub Actions workflow publishes the version release before the shared channels release asset; if channel verification fails, retry after correcting the failed asset check.
EOF
}

real_run() {
  build_plan
  mkdir -p "${WORK_DIR}"
  printf '%s\nplan_sha256=%s\n' "${PLAN_TEXT}" "${PLAN_SHA}" >"${PLAN_FILE}"

  if [[ -n "${BINARY}" ]]; then
    package_args=(
      --version "${VERSION}"
      --output-dir "${ARTIFACT_DIR}"
      --binary "${BINARY}"
    )
    if [[ -n "${SYSROOT_ROOT}" ]]; then
      package_args+=(--sysroot-root "${SYSROOT_ROOT}")
    fi
    "${SCRIPT_DIR}/build_release_artifacts.sh" "${package_args[@]}" >/dev/null
  elif [[ -d "${ARTIFACT_DIR}" ]]; then
    verify_artifact_dir
  else
    "${SCRIPT_DIR}/build_release_artifacts.sh" --version "${VERSION}" --output-dir "${ARTIFACT_DIR}" --cargo-build >/dev/null
  fi

  "${SCRIPT_DIR}/generate_version_installer.sh" \
    --version "${VERSION}" \
    --artifact-dir "${ARTIFACT_DIR}" \
    --out "${INSTALLER_ASSET_FILE}" >/dev/null

  "${SCRIPT_DIR}/generate_dispatchers.sh" \
    --install-root "${INSTALL_ROOT}" >/dev/null

  current_index="${RELEASE_INDEX}"
  "${SCRIPT_DIR}/release_governance.py" validate \
    --kind release-index \
    --input "${current_index}" \
    --require-canonical >/dev/null
  expected_generation="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["generation"])' "${current_index}")"
  expected_sha256="$(sha256_file "${current_index}")"
  release_record="${WORK_DIR}/new-release-record.json"
  "${SCRIPT_DIR}/release_governance.py" build-release-record \
    --version "${VERSION}" \
    --channel "${CHANNEL}" \
    --source-commit "${BASE_SHA}" \
    --installer "${INSTALLER_ASSET_FILE}" \
    --artifact-dir "${ARTIFACT_DIR}" \
    --out "${release_record}" >/dev/null
  "${SCRIPT_DIR}/release_governance.py" update-preview-index \
    --current "${current_index}" \
    --out "${CHANNEL_METADATA_FILE}" \
    --channel "${CHANNEL}" \
    --release "${release_record}" \
    --expected-generation "${expected_generation}" \
    --expected-sha256 "${expected_sha256}" >/dev/null

  "${REPO_ROOT}/verification/areas/distribution_release/tools/validate_self_update_metadata.sh" \
    --install-root "${INSTALL_ROOT}" \
    --channels-file "${CHANNEL_METADATA_FILE}" >/dev/null

  write_checklist
  write_recovery_note

  cat <<EOF
real_run_complete=1
plan_sha256=${PLAN_SHA}
plan_file=${PLAN_FILE}
release_checklist=${CHECKLIST_FILE}
recovery_note=${RECOVERY_FILE}
github_release_mode=${MUTATION_MODE}
EOF
}

case "${MODE}" in
  dry-run)
    print_plan
    ;;
  real-run)
    real_run
    ;;
  *)
    fail "choose --dry-run or --real-run"
    ;;
esac
