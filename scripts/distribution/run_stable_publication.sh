#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/run_stable_publication.sh \
  --operation ga-activation|normal --mode initial|resume \
  --repository OWNER/REPO --evidence-root DIR --evidence-commit COMMIT \
  --candidate-path plans/releases/candidates/X.Y.Z \
  --expected-plan-sha256 SHA256 --source-root DIR \
  --prepare-summary PATH --expected-summary-sha256 SHA256 \
  --workflow-ref REF --workflow-commit COMMIT \
  --run-id ID --run-attempt N --initiator LOGIN \
  --site-repository OWNER/REPO --site-workflow FILE --site-workflow-ref REF \
  --site-ruleset-id ID --site-ruleset-updated-at TIMESTAMP \
  --site-workflow-sha256 SHA256
EOF
  exit 2
}

operation=""
mode=""
repository=""
evidence_root=""
evidence_commit=""
candidate_path=""
expected_plan_sha256=""
source_root=""
prepare_summary=""
expected_summary_sha256=""
workflow_ref=""
workflow_commit=""
run_id=""
run_attempt=""
initiator=""
site_repository=""
site_workflow=""
site_workflow_ref=""
site_ruleset_id=""
site_ruleset_updated_at=""
site_workflow_sha256=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --operation) operation="${2:-}"; shift 2 ;;
    --mode) mode="${2:-}"; shift 2 ;;
    --repository) repository="${2:-}"; shift 2 ;;
    --evidence-root) evidence_root="${2:-}"; shift 2 ;;
    --evidence-commit) evidence_commit="${2:-}"; shift 2 ;;
    --candidate-path) candidate_path="${2:-}"; shift 2 ;;
    --expected-plan-sha256) expected_plan_sha256="${2:-}"; shift 2 ;;
    --source-root) source_root="${2:-}"; shift 2 ;;
    --prepare-summary) prepare_summary="${2:-}"; shift 2 ;;
    --expected-summary-sha256) expected_summary_sha256="${2:-}"; shift 2 ;;
    --workflow-ref) workflow_ref="${2:-}"; shift 2 ;;
    --workflow-commit) workflow_commit="${2:-}"; shift 2 ;;
    --run-id) run_id="${2:-}"; shift 2 ;;
    --run-attempt) run_attempt="${2:-}"; shift 2 ;;
    --initiator) initiator="${2:-}"; shift 2 ;;
    --site-repository) site_repository="${2:-}"; shift 2 ;;
    --site-workflow) site_workflow="${2:-}"; shift 2 ;;
    --site-workflow-ref) site_workflow_ref="${2:-}"; shift 2 ;;
    --site-ruleset-id) site_ruleset_id="${2:-}"; shift 2 ;;
    --site-ruleset-updated-at) site_ruleset_updated_at="${2:-}"; shift 2 ;;
    --site-workflow-sha256) site_workflow_sha256="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ "${operation}" =~ ^(ga-activation|normal)$ &&
  "${mode}" =~ ^(initial|resume)$ &&
  "${repository}" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ &&
  "${evidence_commit}" =~ ^[0-9a-f]{40}$ &&
  "${candidate_path}" =~ ^plans/releases/candidates/[0-9]+\.[0-9]+\.[0-9]+$ &&
  "${expected_plan_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${expected_summary_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${workflow_ref}" = "refs/heads/main" &&
  "${workflow_commit}" =~ ^[0-9a-f]{40}$ &&
  "${run_id}" =~ ^[1-9][0-9]*$ &&
  "${run_attempt}" =~ ^[1-9][0-9]*$ &&
  -n "${initiator}" &&
  "${site_repository}" = "sifr-lang/sifr-website" &&
  "${site_workflow}" = "release-site.yml" &&
  -n "${site_workflow_ref}" &&
  "${site_ruleset_id}" =~ ^[1-9][0-9]*$ &&
  -n "${site_ruleset_updated_at}" &&
  "${site_workflow_sha256}" =~ ^[0-9a-f]{64}$ ]] || usage
for path in "${evidence_root}" "${source_root}"; do
  [[ -d "${path}" && ! -L "${path}" ]] || usage
done
[[ -f "${prepare_summary}" && ! -L "${prepare_summary}" ]] || usage
[[ -n "${SITE_TOKEN:-}" ]] || {
  echo "stable-publication: SITE_TOKEN is required" >&2
  exit 2
}
site_token="${SITE_TOKEN}"
marketplace_pat="${VSCE_PAT:-}"
unset SITE_TOKEN VSCE_PAT
[[ -n "${VSCE_BIN:-}" && -x "${VSCE_BIN}" ]] || {
  echo "stable-publication: VSCE_BIN must be the pinned executable" >&2
  exit 2
}

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d "${RUNNER_TEMP:-/tmp}/sifr-stable-publication.XXXXXX")"
trap 'rm -rf "${work}"' EXIT
test "$(git -C "${repo_root}" rev-parse HEAD)" = "${workflow_commit}" || {
  echo "stable-publication: governance checkout does not equal workflow commit" >&2
  exit 2
}
git -C "${repo_root}" fetch --no-tags origin main:refs/remotes/origin/main
test "$(git -C "${repo_root}" rev-parse refs/remotes/origin/main)" = \
  "${workflow_commit}" || {
  echo "stable-publication: stable mutation must run from protected main HEAD" >&2
  exit 2
}
candidate_source_commit="$(jq -er '.source.commit' "${prepare_summary}")"
for ancestry in \
  "candidate source:${candidate_source_commit}" \
  "candidate evidence:${evidence_commit}"
do
  label="${ancestry%%:*}"
  commit="${ancestry#*:}"
  git -C "${repo_root}" merge-base --is-ancestor \
    "${commit}" refs/remotes/origin/main || {
    echo "stable-publication: ${label} commit must be merged into protected main" >&2
    exit 2
  }
done
candidate="${evidence_root}/${candidate_path}"
plan="${candidate}/stable-release-plan.json"
qualification="${candidate}/qualification-artifact-index.json"
notes="${candidate}/release-notes.md"
version="${candidate_path##*/}"
publication_attempt="${run_id}-${run_attempt}"

fetch_governance() {
  local destination="$1"
  mkdir "${destination}" "${destination}/history"
  gh release download channels \
    --repo "${repository}" \
    --pattern channels.json \
    --dir "${destination}"
  mv "${destination}/channels.json" "${destination}/current-channels.json"
  local channels_release_id
  channels_release_id="$(
    gh api "repos/${repository}/releases/tags/channels" --jq '.id'
  )"
  while IFS= read -r snapshot; do
    [[ -n "${snapshot}" ]] || continue
    gh release download channels \
      --repo "${repository}" \
      --pattern "${snapshot}" \
      --dir "${destination}/history"
  done < <(
    gh api --paginate --slurp \
      "repos/${repository}/releases/${channels_release_id}/assets?per_page=100" |
      jq -er \
        '.[][] | .name
         | select(test("^channels-generation-[1-9][0-9]*[.]json$"))'
  )
}

revalidate() {
  local governance="$1"
  python3 scripts/distribution/revalidate_stable_publication.py \
    --prepare-summary "${prepare_summary}" \
    --expected-summary-sha256 "${expected_summary_sha256}" \
    --operation "${operation}" \
    --mode "${mode}" \
    --evidence-root "${evidence_root}" \
    --evidence-commit "${evidence_commit}" \
    --candidate-path "${candidate_path}" \
    --expected-plan-sha256 "${expected_plan_sha256}" \
    --source-root "${source_root}" \
    --live-index "${governance}/current-channels.json" \
    --snapshot-root "${governance}/history" \
    --artifact-root "${work}/stable-assets"
}

upload_or_verify_governance() {
  local asset="$1"
  local names
  local channels_release_id
  channels_release_id="$(
    gh api "repos/${repository}/releases/tags/channels" --jq '.id'
  )"
  names="$(
    gh api --paginate --slurp \
      "repos/${repository}/releases/${channels_release_id}/assets?per_page=100" |
      jq -er '.[][] | .name'
  )"
  if grep -Fxq "$(basename "${asset}")" <<<"${names}"; then
    local verify
    verify="$(mktemp -d)"
    gh release download channels \
      --repo "${repository}" \
      --pattern "$(basename "${asset}")" \
      --dir "${verify}"
    cmp "${asset}" "${verify}/$(basename "${asset}")" || {
      echo "stable-publication: governance asset bytes drifted: $(basename "${asset}")" >&2
      exit 2
    }
  else
    gh release upload channels "${asset}" --repo "${repository}"
  fi
}

python3 scripts/distribution/fetch_qualification_artifacts.py \
  --qualification-index "${qualification}" \
  --repository "${repository}" \
  --expected-source-commit "$(jq -er '.source_commit' "${plan}")" \
  --out "${work}/stable-assets"
fetch_governance "${work}/governance-initial"
revalidate "${work}/governance-initial"

gh api "repos/${repository}/actions/runs/${run_id}/approvals" \
  >"${work}/approvals.json"
approvers="$(
  scripts/distribution/release_governance.py resolve-publication-approvers \
    --approvals "${work}/approvals.json" \
    --initiator "${initiator}" \
    --environment stable-release
)"
approver="$(jq -er '.[0]' <<<"${approvers}")"

GH_TOKEN="${site_token}" \
  scripts/distribution/verify_site_workflow_identity.sh \
    --repository "${site_repository}" \
    --ruleset-id "${site_ruleset_id}" \
    --ruleset-updated-at "${site_ruleset_updated_at}" \
    --workflow-ref "${site_workflow_ref}" \
    --site-commit "$(jq -er '.site.base_commit' "${prepare_summary}")" \
    --workflow "${site_workflow}" \
    --workflow-sha256 "${site_workflow_sha256}"

mkdir "${work}/dispatchers"
scripts/distribution/generate_dispatchers.sh \
  --install-root "${work}/dispatchers" \
  --default-channel stable
python3 scripts/distribution/materialize_stable_publication.py stage \
  --prepare-summary "${prepare_summary}" \
  --qualification-index "${qualification}" \
  --artifact-root "${work}/stable-assets" \
  --plan "${plan}" \
  --dispatchers "${work}/dispatchers" \
  --out "${work}/staged"

python3 scripts/distribution/publish_stable_release.py \
  --repository "${repository}" \
  --version "${version}" \
  --source-commit "$(jq -er '.source.commit' "${prepare_summary}")" \
  --mode "${mode}" \
  --assets "${work}/staged/release-assets" \
  --notes "${notes}" \
  --out "${work}/published-assets.json"
vsix_name="$(jq -er '.artifacts.vsix.name' "${prepare_summary}")"
vsix_container="$(jq -er '.artifacts.vsix.workflow_artifact_name' "${prepare_summary}")"
GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="${marketplace_pat}" \
  scripts/distribution/publish_marketplace_extension.sh \
  --package "${work}/stable-assets/${vsix_container}/${vsix_name}" \
  --publisher "$(jq -er '.marketplace.publisher' "${prepare_summary}")" \
  --extension "$(jq -er '.marketplace.extension' "${prepare_summary}")" \
  --version "$(jq -er '.marketplace.version' "${prepare_summary}")" \
  --expected-sha256 "$(jq -er '.marketplace.vsix_sha256' "${prepare_summary}")" \
  --verified-out "${work}/marketplace.vsix"

fetch_governance "${work}/governance-before-index"
revalidate "${work}/governance-before-index"
publication_state="$(jq -er '.publication_state' "${prepare_summary}")"
proposed_generation="$(jq -er '.mutation.proposed_index.generation' "${prepare_summary}")"
proposed_sha256="$(jq -er '.mutation.proposed_index_sha256' "${prepare_summary}")"
if [[ "${publication_state}" == "pending" ]]; then
  snapshot="${work}/channels-generation-${proposed_generation}.json"
  cp "${work}/staged/channels.json" "${snapshot}"
  upload_or_verify_governance "${snapshot}"
  live_after_snapshot="$(mktemp -d)"
  gh release download channels \
    --repo "${repository}" \
    --pattern channels.json \
    --dir "${live_after_snapshot}"
  cmp \
    "${work}/governance-before-index/current-channels.json" \
    "${live_after_snapshot}/channels.json" || {
    echo "stable-publication: release index changed after generation reservation" >&2
    exit 2
  }
  gh release upload channels "${work}/staged/channels.json" \
    --repo "${repository}" \
    --clobber
elif [[ "${publication_state}" != "activated" ]]; then
  echo "stable-publication: unsupported publication state" >&2
  exit 2
fi
activated="$(mktemp -d)"
gh release download channels \
  --repo "${repository}" \
  --pattern channels.json \
  --dir "${activated}"
test "$(sha256sum "${activated}/channels.json" | awk '{print $1}')" = \
  "${proposed_sha256}" || {
  echo "stable-publication: activated release-index bytes drifted" >&2
  exit 2
}

site_base_commit="$(jq -er '.site.base_commit' "${prepare_summary}")"
for dispatcher in index stable alpha beta; do
  digest="$(sha256sum "${work}/dispatchers/${dispatcher}" | awk '{print $1}')"
  printf -v "dispatcher_${dispatcher}_sha256" '%s' "${digest}"
done
python3 scripts/distribution/generate_site_publication_facts.py \
  --out "${work}/site-publication-facts.json" \
  --source-commit "$(jq -er '.source.commit' "${prepare_summary}")" \
  --site-base-commit "${site_base_commit}" \
  --release-plan-sha256 "${expected_plan_sha256}" \
  --publication-attempt "${publication_attempt}" \
  --release-index-generation "${proposed_generation}" \
  --release-index-sha256 "${proposed_sha256}" \
  --dispatcher-default-channel stable \
  --dispatcher-index-sha256 "${dispatcher_index_sha256}" \
  --dispatcher-stable-sha256 "${dispatcher_stable_sha256}" \
  --dispatcher-alpha-sha256 "${dispatcher_alpha_sha256}" \
  --dispatcher-beta-sha256 "${dispatcher_beta_sha256}"
publication_facts_sha256="$(
  sha256sum "${work}/site-publication-facts.json" | awk '{print $1}'
)"

GH_TOKEN="${site_token}" \
  scripts/distribution/verify_site_workflow_identity.sh \
    --repository "${site_repository}" \
    --ruleset-id "${site_ruleset_id}" \
    --ruleset-updated-at "${site_ruleset_updated_at}" \
    --workflow-ref "${site_workflow_ref}" \
    --site-commit "${site_base_commit}" \
    --workflow "${site_workflow}" \
    --workflow-sha256 "${site_workflow_sha256}"
dispatched_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -n \
  --arg ref "${site_workflow_ref}" \
  --arg source "$(jq -er '.source.commit' "${prepare_summary}")" \
  --arg plan "${expected_plan_sha256}" \
  --arg attempt "${publication_attempt}" \
  --arg generation "${proposed_generation}" \
  --arg index "${proposed_sha256}" \
  --arg site "${site_base_commit}" \
  --arg dispatcher_index "${dispatcher_index_sha256}" \
  --arg dispatcher_stable "${dispatcher_stable_sha256}" \
  --arg dispatcher_alpha "${dispatcher_alpha_sha256}" \
  --arg dispatcher_beta "${dispatcher_beta_sha256}" \
  --arg facts "${publication_facts_sha256}" \
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
      dispatcher_default_channel: "stable",
      publication_facts_sha256: $facts
    }
  }' |
  GH_TOKEN="${site_token}" gh api \
    --method POST \
    "repos/${site_repository}/actions/workflows/${site_workflow}/dispatches" \
    --input -
GH_TOKEN="${site_token}" \
  scripts/distribution/poll_site_release_run.sh \
    --repository "${site_repository}" \
    --workflow "${site_workflow}" \
    --title "Sifr site release ${publication_attempt}" \
    --sha "${site_base_commit}" \
    --dispatched-at "${dispatched_at}" \
    --deadline-seconds 1200 \
    --result-out "${work}/site-run.json"

GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  scripts/distribution/run_stable_public_smoke.sh \
  --repository "${repository}" \
  --version "${version}" \
  --index "${work}/staged/channels.json" \
  --dispatchers "${work}/dispatchers" \
  --asset-digests "${work}/published-assets.json" \
  --marketplace-vsix "${work}/marketplace.vsix" \
  --out "${work}/stable-smoke"
python3 scripts/distribution/materialize_stable_publication.py signoff \
  --prepare-summary "${prepare_summary}" \
  --release-assets "${work}/staged/release-assets" \
  --site-facts "${work}/staged/stable-site-release-facts.json" \
  --site-run "${work}/site-run.json" \
  --smoke "${work}/stable-smoke" \
  --run-id "${run_id}" \
  --approver "${approver}" \
  --out "${work}/stable-release-signoff-${version}-attempt-${publication_attempt}.json"
cp \
  "${work}/staged/stable-site-release-facts.json" \
  "${work}/stable-site-release-facts-generation-${proposed_generation}.json"
upload_or_verify_governance \
  "${work}/stable-site-release-facts-generation-${proposed_generation}.json"
upload_or_verify_governance \
  "${work}/stable-release-signoff-${version}-attempt-${publication_attempt}.json"
