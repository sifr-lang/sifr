#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/validate_preview_publication_inputs.sh \
  --channel alpha|beta --operation preview|bootstrap-alpha|bootstrap-index \
  --version VERSION --bootstrap-alpha-version VERSION_OR_EMPTY \
  --source-commit COMMIT --repository OWNER/REPO \
  --site-repository OWNER/REPO --site-commit COMMIT --site-workflow FILE \
  --site-workflow-ref REF --site-ruleset-id ID \
  --site-ruleset-updated-at TIMESTAMP --site-workflow-sha256 SHA256
EOF
  exit 2
}

reject() {
  echo "preview-publication: $1" >&2
  exit 2
}

channel=""
operation=""
version=""
bootstrap_alpha_version=""
source_commit=""
repository=""
site_repository=""
site_commit=""
site_workflow=""
site_workflow_ref=""
site_ruleset_id=""
site_ruleset_updated_at=""
site_workflow_sha256=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel) channel="${2:-}"; shift 2 ;;
    --operation) operation="${2:-}"; shift 2 ;;
    --version) version="${2:-}"; shift 2 ;;
    --bootstrap-alpha-version) bootstrap_alpha_version="${2:-}"; shift 2 ;;
    --source-commit) source_commit="${2:-}"; shift 2 ;;
    --repository) repository="${2:-}"; shift 2 ;;
    --site-repository) site_repository="${2:-}"; shift 2 ;;
    --site-commit) site_commit="${2:-}"; shift 2 ;;
    --site-workflow) site_workflow="${2:-}"; shift 2 ;;
    --site-workflow-ref) site_workflow_ref="${2:-}"; shift 2 ;;
    --site-ruleset-id) site_ruleset_id="${2:-}"; shift 2 ;;
    --site-ruleset-updated-at) site_ruleset_updated_at="${2:-}"; shift 2 ;;
    --site-workflow-sha256) site_workflow_sha256="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ "${channel}" =~ ^(alpha|beta)$ ]] ||
  reject "channel must be alpha or beta"
[[ "${operation}" =~ ^(preview|bootstrap-alpha|bootstrap-index)$ ]] ||
  reject "unsupported governance operation"
[[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$ ]] ||
  reject "version must be a semver alpha or beta prerelease"
[[ "${BASH_REMATCH[1]}" = "${channel}" ]] ||
  reject "version and preview channel disagree"
[[ "${source_commit}" =~ ^[0-9a-f]{40}$ ]] ||
  reject "source_commit must be an exact commit"
[[ "${repository}" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] ||
  reject "repository must be OWNER/REPO"
[[ "${site_repository}" = "sifr-lang/sifr-website" ]] ||
  reject "site repository must be sifr-lang/sifr-website"
[[ "${site_commit}" =~ ^[0-9a-f]{40}$ ]] ||
  reject "site_base_commit must be an exact commit"
[[ "${site_workflow}" = "release-site.yml" ]] ||
  reject "site workflow must be release-site.yml"
[[ -n "${site_workflow_ref}" ]] || reject "site workflow ref is required"
[[ "${site_ruleset_id}" =~ ^[1-9][0-9]*$ ]] ||
  reject "site ruleset id must be positive"
[[ -n "${site_ruleset_updated_at}" ]] ||
  reject "site ruleset revision is required"
[[ "${site_workflow_sha256}" =~ ^[0-9a-f]{64}$ ]] ||
  reject "site workflow digest must be SHA-256"
[[ -n "${SITE_TOKEN:-}" ]] || reject "SITE_TOKEN is required"
case "${operation}" in
  preview)
    [[ -z "${bootstrap_alpha_version}" ]] || {
      echo "preview-publication: preview cannot name a bootstrap alpha" >&2
      exit 2
    }
    ;;
  bootstrap-alpha)
    [[ "${channel}" = "alpha" && -z "${bootstrap_alpha_version}" ]] || {
      echo "preview-publication: bootstrap-alpha input mismatch" >&2
      exit 2
    }
    ;;
  bootstrap-index)
    [[ "${channel}" = "beta" &&
      "${bootstrap_alpha_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$ ]] || {
      echo "preview-publication: bootstrap-index input mismatch" >&2
      exit 2
    }
    ;;
esac
[[ "$(git rev-parse HEAD)" = "${source_commit}" ]] || {
  echo "preview-publication: checkout does not equal source commit" >&2
  exit 2
}
git fetch --no-tags origin main:refs/remotes/origin/main
git merge-base --is-ancestor "${source_commit}" refs/remotes/origin/main || {
  echo "preview-publication: source must be reachable from protected main" >&2
  exit 2
}
GH_TOKEN="${SITE_TOKEN}" \
  scripts/distribution/verify_site_workflow_identity.sh \
    --repository "${site_repository}" \
    --ruleset-id "${site_ruleset_id}" \
    --ruleset-updated-at "${site_ruleset_updated_at}" \
    --workflow-ref "${site_workflow_ref}" \
    --site-commit "${site_commit}" \
    --workflow "${site_workflow}" \
    --workflow-sha256 "${site_workflow_sha256}"
existing_release_count="$(
  gh api --method GET --paginate --slurp \
    "repos/${repository}/releases" -f per_page=100 |
    jq --arg version "${version}" \
      '[.[][] | select(.tag_name == $version)] | length'
)"
[[ "${existing_release_count}" = "0" ]] || {
  echo "preview-publication: version release already exists" >&2
  exit 2
}
existing_tag_count="$(
  gh api "repos/${repository}/git/matching-refs/tags/${version}" |
    jq --arg ref "refs/tags/${version}" \
      '[.[] | select(.ref == $ref)] | length'
)"
[[ "${existing_tag_count}" = "0" ]] || {
  echo "preview-publication: version tag already exists" >&2
  exit 2
}
