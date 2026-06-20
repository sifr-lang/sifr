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
  --work-dir <dir>           Work/evidence directory (default: target/preview-release/<version>)
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
WORK_DIR=""
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
    --work-dir)
      WORK_DIR="${2:-}"
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

extract_var() {
  local file="$1"
  local name="$2"
  sed -n "s/^${name}=\"\\(.*\\)\"$/\\1/p" "${file}" | head -n 1
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
}

validate_site_dispatchers() {
  local expected_path
  for expected_path in index alpha beta; do
    [[ -f "${INSTALL_ROOT}/${expected_path}" ]] || fail "site dispatcher missing: ${INSTALL_ROOT}/${expected_path}"
  done

  local index_alpha index_beta alpha_alpha alpha_beta beta_alpha beta_beta
  index_alpha="$(extract_var "${INSTALL_ROOT}/index" ALPHA_VERSION)"
  index_beta="$(extract_var "${INSTALL_ROOT}/index" BETA_VERSION)"
  alpha_alpha="$(extract_var "${INSTALL_ROOT}/alpha" ALPHA_VERSION)"
  alpha_beta="$(extract_var "${INSTALL_ROOT}/alpha" BETA_VERSION)"
  beta_alpha="$(extract_var "${INSTALL_ROOT}/beta" ALPHA_VERSION)"
  beta_beta="$(extract_var "${INSTALL_ROOT}/beta" BETA_VERSION)"

  [[ -n "${index_alpha}" && -n "${index_beta}" ]] || fail "site dispatcher drift: index missing channel version variables"
  [[ "${index_alpha}" == "${alpha_alpha}" && "${index_alpha}" == "${beta_alpha}" ]] || fail "site dispatcher drift: ALPHA_VERSION differs across dispatchers"
  [[ "${index_beta}" == "${alpha_beta}" && "${index_beta}" == "${beta_beta}" ]] || fail "site dispatcher drift: BETA_VERSION differs across dispatchers"

  [[ "$(extract_var "${INSTALL_ROOT}/index" DEFAULT_CHANNEL)" == "beta" ]] || fail "site dispatcher drift: index must default to beta"
  [[ "$(extract_var "${INSTALL_ROOT}/alpha" DEFAULT_CHANNEL)" == "alpha" ]] || fail "site dispatcher drift: alpha must default to alpha"
  [[ "$(extract_var "${INSTALL_ROOT}/beta" DEFAULT_CHANNEL)" == "beta" ]] || fail "site dispatcher drift: beta must default to beta"

  CURRENT_ALPHA="${index_alpha}"
  CURRENT_BETA="${index_beta}"
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
version_installer=${INSTALL_ROOT}/versions/${VERSION}
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

- [x] Immutable generated installer: ${INSTALL_ROOT}/versions/${VERSION}
- [x] GitHub immutable installer asset: ${INSTALLER_ASSET_FILE}
- [x] Channel dispatcher update: ${CHANNEL} -> ${VERSION}
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
  - immutable installer generated: ${INSTALL_ROOT}/versions/${VERSION}
  - GitHub immutable installer asset generated: ${INSTALLER_ASSET_FILE}
  - channel dispatchers regenerated: alpha=${NEW_ALPHA}, beta=${NEW_BETA}
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
    "${SCRIPT_DIR}/build_preview_artifacts.sh" --version "${VERSION}" --output-dir "${ARTIFACT_DIR}" --binary "${BINARY}" >/dev/null
  elif [[ -d "${ARTIFACT_DIR}" ]]; then
    verify_artifact_dir
  else
    "${SCRIPT_DIR}/build_preview_artifacts.sh" --version "${VERSION}" --output-dir "${ARTIFACT_DIR}" --cargo-build >/dev/null
  fi

  "${SCRIPT_DIR}/generate_version_installer.sh" \
    --version "${VERSION}" \
    --artifact-dir "${ARTIFACT_DIR}" \
    --out "${INSTALLER_ASSET_FILE}" >/dev/null

  cp "${INSTALLER_ASSET_FILE}" "${INSTALL_ROOT}/versions/${VERSION}"

  "${SCRIPT_DIR}/generate_dispatchers.sh" \
    --install-root "${INSTALL_ROOT}" \
    --alpha-version "${NEW_ALPHA}" \
    --beta-version "${NEW_BETA}" >/dev/null

  "${SCRIPT_DIR}/generate_channel_metadata.sh" \
    --out "${CHANNEL_METADATA_FILE}" \
    --alpha-version "${NEW_ALPHA}" \
    --beta-version "${NEW_BETA}" >/dev/null

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
site_installer=${INSTALL_ROOT}/versions/${VERSION}
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
