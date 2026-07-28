#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/dispatch_stable_site_publication.sh \
  --repository OWNER/REPO --workflow FILE --workflow-ref REF \
  --ruleset-id ID --ruleset-updated-at TIMESTAMP --workflow-sha256 SHA256 \
  --source-commit COMMIT --site-commit COMMIT --release-plan-sha256 SHA256 \
  --publication-attempt ID --generation N --index-sha256 SHA256 \
  --default-channel beta|stable \
  --dispatcher-index-sha256 SHA256 --dispatcher-stable-sha256 SHA256 \
  --dispatcher-alpha-sha256 SHA256 --dispatcher-beta-sha256 SHA256 \
  --publication-facts-sha256 SHA256 \
  --stable-site-facts-sha256 SHA256|none --result-out PATH
EOF
  exit 2
}

repository=""
workflow=""
workflow_ref=""
ruleset_id=""
ruleset_updated_at=""
workflow_sha256=""
source_commit=""
site_commit=""
release_plan_sha256=""
publication_attempt=""
generation=""
index_sha256=""
default_channel=""
dispatcher_index_sha256=""
dispatcher_stable_sha256=""
dispatcher_alpha_sha256=""
dispatcher_beta_sha256=""
publication_facts_sha256=""
stable_site_facts_sha256=""
result_out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --workflow) workflow="${2:-}"; shift 2 ;;
    --workflow-ref) workflow_ref="${2:-}"; shift 2 ;;
    --ruleset-id) ruleset_id="${2:-}"; shift 2 ;;
    --ruleset-updated-at) ruleset_updated_at="${2:-}"; shift 2 ;;
    --workflow-sha256) workflow_sha256="${2:-}"; shift 2 ;;
    --source-commit) source_commit="${2:-}"; shift 2 ;;
    --site-commit) site_commit="${2:-}"; shift 2 ;;
    --release-plan-sha256) release_plan_sha256="${2:-}"; shift 2 ;;
    --publication-attempt) publication_attempt="${2:-}"; shift 2 ;;
    --generation) generation="${2:-}"; shift 2 ;;
    --index-sha256) index_sha256="${2:-}"; shift 2 ;;
    --default-channel) default_channel="${2:-}"; shift 2 ;;
    --dispatcher-index-sha256) dispatcher_index_sha256="${2:-}"; shift 2 ;;
    --dispatcher-stable-sha256) dispatcher_stable_sha256="${2:-}"; shift 2 ;;
    --dispatcher-alpha-sha256) dispatcher_alpha_sha256="${2:-}"; shift 2 ;;
    --dispatcher-beta-sha256) dispatcher_beta_sha256="${2:-}"; shift 2 ;;
    --publication-facts-sha256) publication_facts_sha256="${2:-}"; shift 2 ;;
    --stable-site-facts-sha256) stable_site_facts_sha256="${2:-}"; shift 2 ;;
    --result-out) result_out="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

channel_facts_valid=false
if [[ "${default_channel}" == "stable" &&
  "${stable_site_facts_sha256}" =~ ^[0-9a-f]{64}$ ]]; then
  channel_facts_valid=true
elif [[ "${default_channel}" == "beta" &&
  "${stable_site_facts_sha256}" == "none" ]]; then
  channel_facts_valid=true
fi
[[ "${repository}" = "sifr-lang/sifr-website" &&
  "${workflow}" = "release-site.yml" &&
  -n "${workflow_ref}" &&
  "${ruleset_id}" =~ ^[1-9][0-9]*$ &&
  -n "${ruleset_updated_at}" &&
  "${workflow_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${source_commit}" =~ ^[0-9a-f]{40}$ &&
  "${site_commit}" =~ ^[0-9a-f]{40}$ &&
  "${release_plan_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${publication_attempt}" =~ ^[1-9][0-9]*-[1-9][0-9]*$ &&
  "${generation}" =~ ^[1-9][0-9]*$ &&
  "${index_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${default_channel}" =~ ^(beta|stable)$ &&
  "${dispatcher_index_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${dispatcher_stable_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${dispatcher_alpha_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${dispatcher_beta_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${publication_facts_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${channel_facts_valid}" == "true" &&
  ! -e "${result_out}" && ! -L "${result_out}" ]] || usage
[[ -n "${SITE_TOKEN:-}" ]] || {
  echo "site-publication: SITE_TOKEN is required" >&2
  exit 2
}
site_token="${SITE_TOKEN}"
unset SITE_TOKEN

GH_TOKEN="${site_token}" \
  scripts/distribution/verify_site_workflow_identity.sh \
    --repository "${repository}" \
    --ruleset-id "${ruleset_id}" \
    --ruleset-updated-at "${ruleset_updated_at}" \
    --workflow-ref "${workflow_ref}" \
    --site-commit "${site_commit}" \
    --workflow "${workflow}" \
    --workflow-sha256 "${workflow_sha256}"
dispatched_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -n \
  --arg ref "${workflow_ref}" \
  --arg source "${source_commit}" \
  --arg plan "${release_plan_sha256}" \
  --arg attempt "${publication_attempt}" \
  --arg generation "${generation}" \
  --arg index "${index_sha256}" \
  --arg site "${site_commit}" \
  --arg dispatcher_index "${dispatcher_index_sha256}" \
  --arg dispatcher_stable "${dispatcher_stable_sha256}" \
  --arg dispatcher_alpha "${dispatcher_alpha_sha256}" \
  --arg dispatcher_beta "${dispatcher_beta_sha256}" \
  --arg facts "${publication_facts_sha256}" \
  --arg stable_facts "${stable_site_facts_sha256}" \
  --arg default_channel "${default_channel}" \
  '{
    ref: $ref,
    inputs: {
      sifr_source_commit: $source,
      release_plan_sha256: $plan,
      publication_attempt: $attempt,
      release_index_generation: $generation,
      release_index_sha256: $index,
      site_base_commit: $site,
      dispatcher_index_sha256: $dispatcher_index,
      dispatcher_stable_sha256: $dispatcher_stable,
      dispatcher_alpha_sha256: $dispatcher_alpha,
      dispatcher_beta_sha256: $dispatcher_beta,
      dispatcher_default_channel: $default_channel,
      publication_facts_sha256: $facts,
      stable_site_facts_sha256: $stable_facts
    }
  }' |
  GH_TOKEN="${site_token}" gh api \
    --method POST \
    "repos/${repository}/actions/workflows/${workflow}/dispatches" \
    --input -
GH_TOKEN="${site_token}" \
  scripts/distribution/poll_site_release_run.sh \
    --repository "${repository}" \
    --workflow "${workflow}" \
    --title "Sifr site release ${publication_attempt}" \
    --sha "${site_commit}" \
    --dispatched-at "${dispatched_at}" \
    --deadline-seconds 1200 \
    --result-out "${result_out}"
