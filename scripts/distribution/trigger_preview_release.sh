#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/trigger_preview_release.sh --channel alpha|beta --version <preview> [options]

Trigger the GitHub Actions preview-release workflow.

Required:
  --channel alpha|beta       Preview channel to publish
  --version <preview>        Semver prerelease version, for example 0.1.0-beta.5

Options:
  --base-ref <ref>           Ref to build and release (default: main)
  --workflow-ref <ref>       Ref containing the workflow file (default: --base-ref)
  --repo <owner/repo>        GitHub repository (default: detected from origin)
  --workflow <file>          Workflow file name (default: preview-release.yml)
  --watch                    Watch the dispatched run until completion (default)
  --no-watch                 Dispatch only
  --dry-run                  Print the dispatch command without running it
  --help                     Show this help
EOF
}

CHANNEL=""
VERSION=""
BASE_REF="main"
WORKFLOW_REF=""
REPO=""
WORKFLOW="preview-release.yml"
WATCH=1
DRY_RUN=0

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
    --base-ref)
      BASE_REF="${2:-}"
      shift 2
      ;;
    --workflow-ref)
      WORKFLOW_REF="${2:-}"
      shift 2
      ;;
    --repo)
      REPO="${2:-}"
      shift 2
      ;;
    --workflow)
      WORKFLOW="${2:-}"
      shift 2
      ;;
    --watch)
      WATCH=1
      shift
      ;;
    --no-watch)
      WATCH=0
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
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
  echo "trigger-preview-release: $*" >&2
  exit 2
}

preview_channel_for_version() {
  local version="$1"
  if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta|rc)\.[0-9]+$ ]]; then
    return 1
  fi
  printf '%s\n' "${BASH_REMATCH[1]}"
}

quote_arg() {
  printf "%q" "$1"
}

print_command() {
  local arg
  for arg in "$@"; do
    quote_arg "${arg}"
    printf ' '
  done
  printf '\n'
}

detect_repo() {
  local remote_url
  remote_url="$(git config --get remote.origin.url || true)"
  [[ -n "${remote_url}" ]] || return 1

  case "${remote_url}" in
    git@github.com:*)
      remote_url="${remote_url#git@github.com:}"
      ;;
    https://github.com/*)
      remote_url="${remote_url#https://github.com/}"
      ;;
    *)
      return 1
      ;;
  esac

  remote_url="${remote_url%.git}"
  [[ "${remote_url}" == */* ]] || return 1
  printf '%s\n' "${remote_url}"
}

find_dispatched_run() {
  local title="Preview release ${VERSION} (${CHANNEL})"
  gh run list \
    --repo "${REPO}" \
    --workflow "${WORKFLOW}" \
    --event workflow_dispatch \
    --limit 20 \
    --json databaseId,displayTitle,url \
    --jq ".[] | select(.displayTitle == \"${title}\") | [.databaseId, .url] | @tsv" \
    | head -n 1
}

[[ "${CHANNEL}" == "alpha" || "${CHANNEL}" == "beta" ]] || fail "--channel must be alpha or beta"
[[ -n "${VERSION}" ]] || fail "--version is required"
[[ -n "${BASE_REF}" ]] || fail "--base-ref must not be empty"
if [[ -z "${WORKFLOW_REF}" ]]; then
  WORKFLOW_REF="${BASE_REF}"
fi
[[ -n "${WORKFLOW_REF}" ]] || fail "--workflow-ref must not be empty"

if [[ "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  fail "stable-looking versions are disabled until a stable channel is supported: ${VERSION}"
fi

version_channel="$(preview_channel_for_version "${VERSION}")" || \
  fail "version must be a semver prerelease using -alpha.N, -beta.N, or -rc.N: ${VERSION}"
[[ "${version_channel}" == "${CHANNEL}" ]] || fail "version ${VERSION} belongs to ${version_channel}, not ${CHANNEL}"

command -v gh >/dev/null 2>&1 || fail "gh is required"

if [[ -z "${REPO}" ]]; then
  REPO="$(detect_repo)" || fail "--repo is required when remote.origin.url is not a GitHub repository"
fi

dispatch_command=(
  gh workflow run "${WORKFLOW}"
  --repo "${REPO}"
  --ref "${WORKFLOW_REF}"
  -f "channel=${CHANNEL}"
  -f "version=${VERSION}"
  -f "base_ref=${BASE_REF}"
)

if [[ "${DRY_RUN}" -eq 1 ]]; then
  printf 'dry_run=1\n'
  printf 'dispatch_command='
  print_command "${dispatch_command[@]}"
  exit 0
fi

"${dispatch_command[@]}"

cat <<EOF
dispatched=1
repo=${REPO}
workflow=${WORKFLOW}
channel=${CHANNEL}
version=${VERSION}
base_ref=${BASE_REF}
workflow_ref=${WORKFLOW_REF}
EOF

if [[ "${WATCH}" -eq 0 ]]; then
  exit 0
fi

run_info=""
for _ in {1..12}; do
  run_info="$(find_dispatched_run || true)"
  if [[ -n "${run_info}" ]]; then
    break
  fi
  sleep 5
done

[[ -n "${run_info}" ]] || fail "workflow was dispatched, but the run was not found in GitHub run listings"

run_id="${run_info%%$'\t'*}"
run_url="${run_info#*$'\t'}"
printf 'run_id=%s\n' "${run_id}"
printf 'run_url=%s\n' "${run_url}"

gh run watch "${run_id}" --repo "${REPO}" --exit-status
