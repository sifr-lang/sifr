#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/distribution/run_incident_publication.sh \
  --operation rollback|incident-roll-forward --mode initial|resume \
  --repository OWNER/REPO --incident-root DIR --incident-commit COMMIT \
  --incident-path plans/releases/incidents/ID/stable-incident-request.json \
  --expected-request-sha256 SHA256 \
  [--candidate-root DIR --candidate-commit COMMIT \
   --candidate-path plans/releases/candidates/X.Y.Z \
   --expected-plan-sha256 SHA256 --source-root DIR] \
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
incident_root=""
incident_commit=""
incident_path=""
expected_request_sha256=""
candidate_root=""
candidate_commit=""
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
    --incident-root) incident_root="${2:-}"; shift 2 ;;
    --incident-commit) incident_commit="${2:-}"; shift 2 ;;
    --incident-path) incident_path="${2:-}"; shift 2 ;;
    --expected-request-sha256) expected_request_sha256="${2:-}"; shift 2 ;;
    --candidate-root) candidate_root="${2:-}"; shift 2 ;;
    --candidate-commit) candidate_commit="${2:-}"; shift 2 ;;
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

[[ "${operation}" =~ ^(rollback|incident-roll-forward)$ &&
  "${mode}" =~ ^(initial|resume)$ &&
  "${repository}" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ &&
  "${incident_commit}" =~ ^[0-9a-f]{40}$ &&
  "${incident_path}" =~ ^plans/releases/incidents/[a-z0-9][a-z0-9-]{2,63}/stable-incident-request[.]json$ &&
  "${expected_request_sha256}" =~ ^[0-9a-f]{64}$ &&
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
[[ -d "${incident_root}" && ! -L "${incident_root}" &&
  -f "${prepare_summary}" && ! -L "${prepare_summary}" ]] || usage
if [[ "${operation}" == "incident-roll-forward" ]]; then
  [[ -d "${candidate_root}" && ! -L "${candidate_root}" &&
    -d "${source_root}" && ! -L "${source_root}" &&
    "${candidate_commit}" =~ ^[0-9a-f]{40}$ &&
    "${candidate_path}" =~ ^plans/releases/candidates/[0-9]+\.[0-9]+\.[0-9]+$ &&
    "${expected_plan_sha256}" =~ ^[0-9a-f]{64}$ ]] || usage
else
  [[ -z "${candidate_root}${candidate_commit}${candidate_path}${expected_plan_sha256}${source_root}" ]] || usage
fi
[[ -n "${SITE_TOKEN:-}" ]] || {
  echo "incident-publication: SITE_TOKEN is required" >&2
  exit 2
}
site_token="${SITE_TOKEN}"
marketplace_pat="${VSCE_PAT:-}"
unset SITE_TOKEN VSCE_PAT
if [[ "${operation}" == "incident-roll-forward" ]]; then
  [[ -n "${VSCE_BIN:-}" && -x "${VSCE_BIN}" ]] || {
    echo "incident-publication: VSCE_BIN must be the pinned executable" >&2
    exit 2
  }
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d "${RUNNER_TEMP:-/tmp}/sifr-incident-publication.XXXXXX")"
trap 'rm -rf "${work}"' EXIT
test "$(git -C "${repo_root}" rev-parse HEAD)" = "${workflow_commit}" || {
  echo "incident-publication: governance checkout does not equal workflow commit" >&2
  exit 2
}
git -C "${repo_root}" fetch --no-tags origin main:refs/remotes/origin/main
test "$(git -C "${repo_root}" rev-parse refs/remotes/origin/main)" = \
  "${workflow_commit}" || {
  echo "incident-publication: mutation must run from protected main HEAD" >&2
  exit 2
}
for ancestry in "incident evidence:${incident_commit}"; do
  label="${ancestry%%:*}"
  commit="${ancestry#*:}"
  git -C "${repo_root}" merge-base --is-ancestor \
    "${commit}" refs/remotes/origin/main || {
    echo "incident-publication: ${label} must be merged into protected main" >&2
    exit 2
  }
done
if [[ "${operation}" == "incident-roll-forward" ]]; then
  candidate_source_commit="$(jq -er '.release_prepare.source.commit' "${prepare_summary}")"
  for ancestry in \
    "candidate evidence:${candidate_commit}" \
    "candidate source:${candidate_source_commit}"
  do
    label="${ancestry%%:*}"
    commit="${ancestry#*:}"
    git -C "${repo_root}" merge-base --is-ancestor \
      "${commit}" refs/remotes/origin/main || {
      echo "incident-publication: ${label} must be merged into protected main" >&2
      exit 2
    }
  done
fi

request="${incident_root}/${incident_path}"
withdrawal_evidence="$(dirname "${request}")/withdrawal-evidence.txt"
affected_version="$(jq -er '.affected.version' "${prepare_summary}")"
successor_version="$(jq -er '.successor.version' "${prepare_summary}")"
affected_candidate="${repo_root}/plans/releases/candidates/${affected_version}"
affected_plan="${affected_candidate}/stable-release-plan.json"
affected_qualification="${affected_candidate}/qualification-artifact-index.json"
if [[ "${operation}" == "rollback" ]]; then
  successor_candidate="${repo_root}/plans/releases/candidates/${successor_version}"
  successor_plan="${successor_candidate}/stable-release-plan.json"
  successor_qualification="${successor_candidate}/qualification-artifact-index.json"
  # Keep the newest approved site generator/dispatcher contract. The staged
  # facts and public docs smoke below must render the rollback target and
  # withdrawal directly from the realized index.
  site_plan="${affected_plan}"
else
  successor_candidate="${candidate_root}/${candidate_path}"
  successor_plan="${successor_candidate}/stable-release-plan.json"
  successor_qualification="${successor_candidate}/qualification-artifact-index.json"
  site_plan="${successor_plan}"
fi
publication_attempt="${run_id}-${run_attempt}"
incident_id="$(jq -er '.incident.incident_id' "${prepare_summary}")"

fetch_governance() {
  local destination="$1"
  mkdir "${destination}" "${destination}/history"
  gh release download channels \
    --repo "${repository}" --pattern channels.json --dir "${destination}"
  mv "${destination}/channels.json" "${destination}/current-channels.json"
  local release_id
  release_id="$(gh api "repos/${repository}/releases/tags/channels" --jq '.id')"
  while IFS= read -r snapshot; do
    [[ -n "${snapshot}" ]] || continue
    gh release download channels \
      --repo "${repository}" --pattern "${snapshot}" \
      --dir "${destination}/history"
  done < <(
    gh api --paginate --slurp \
      "repos/${repository}/releases/${release_id}/assets?per_page=100" |
      jq -er \
        '.[][] | .name
         | select(test("^channels-generation-[1-9][0-9]*[.]json$"))'
  )
}

upload_or_verify_governance() {
  local asset="$1"
  local allow_existing="${2:-false}"
  local release_id names
  release_id="$(gh api "repos/${repository}/releases/tags/channels" --jq '.id')"
  names="$(
    gh api --paginate --slurp \
      "repos/${repository}/releases/${release_id}/assets?per_page=100" |
      jq -er '.[][] | .name'
  )"
  if grep -Fxq "$(basename "${asset}")" <<<"${names}"; then
    [[ "${allow_existing}" == "true" ]] || {
      echo "incident-publication: governance asset requires explicit resume" >&2
      exit 2
    }
    local verify
    verify="$(mktemp -d)"
    gh release download channels \
      --repo "${repository}" --pattern "$(basename "${asset}")" --dir "${verify}"
    cmp "${asset}" "${verify}/$(basename "${asset}")" || {
      echo "incident-publication: governance asset bytes drifted" >&2
      exit 2
    }
  else
    gh release upload channels "${asset}" --repo "${repository}"
  fi
}

verify_retained_version() {
  local version="$1"
  local plan="$2"
  local qualification="$3"
  local destination="$4"
  mkdir "${destination}" "${destination}/assets"
  gh api "repos/${repository}/releases/tags/${version}" |
    jq -c '{
      tagName: .tag_name,
      targetCommitish: .target_commitish,
      isDraft: .draft,
      isPrerelease: .prerelease
    }' >"${destination}/release.json"
  local tag_commit
  tag_commit="$(
    gh api "repos/${repository}/git/ref/tags/${version}" --jq '.object.sha'
  )"
  gh release download "${version}" \
    --repo "${repository}" --dir "${destination}/assets"
  python3 scripts/distribution/verify_retained_stable_release.py \
    --plan "${plan}" --qualification "${qualification}" \
    --assets "${destination}/assets" \
    --release-metadata "${destination}/release.json" \
    --tag-commit "${tag_commit}" --out "${destination}/asset-digests.json"
}

revalidate() {
  local governance="$1"
  local arguments=(
    --prepare-summary "${prepare_summary}"
    --expected-summary-sha256 "${expected_summary_sha256}"
    --operation "${operation}" --mode "${mode}"
    --governance-root "${repo_root}"
    --incident-root "${incident_root}"
    --incident-commit "${incident_commit}"
    --incident-path "${incident_path}"
    --expected-request-sha256 "${expected_request_sha256}"
    --live-index "${governance}/current-channels.json"
    --snapshot-root "${governance}/history"
  )
  if [[ "${operation}" == "incident-roll-forward" ]]; then
    arguments+=(
      --candidate-root "${candidate_root}"
      --candidate-commit "${candidate_commit}"
      --candidate-path "${candidate_path}"
      --expected-plan-sha256 "${expected_plan_sha256}"
      --source-root "${source_root}"
      --artifact-root "${work}/stable-assets"
    )
  fi
  python3 scripts/distribution/revalidate_incident_publication.py "${arguments[@]}"
}

if [[ "${operation}" == "incident-roll-forward" ]]; then
  python3 scripts/distribution/fetch_qualification_artifacts.py \
    --qualification-index "${successor_qualification}" \
    --repository "${repository}" \
    --expected-source-commit "$(jq -er '.source_commit' "${successor_plan}")" \
    --out "${work}/stable-assets"
fi
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

verify_retained_version \
  "${affected_version}" "${affected_plan}" "${affected_qualification}" \
  "${work}/affected-release"
affected_installer="$(
  jq -er '.artifacts[] | select(.id == "installer") | .name' \
    "${affected_qualification}"
)"
for root in "${work}/working-client" "${work}/broken-client"; do
  mkdir "${root}"
  GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  HOME="${root}" SIFR_INSTALL_DIR="${root}/bin" \
  SIFR_SYSROOT_INSTALL_DIR="${root}" SIFR_NO_MODIFY_PATH=1 \
    sh "${work}/affected-release/assets/${affected_installer}"
done

mkdir "${work}/dispatchers"
scripts/distribution/generate_dispatchers.sh \
  --install-root "${work}/dispatchers" --default-channel stable
if [[ "${operation}" == "incident-roll-forward" ]]; then
  jq -cS '.release_prepare' "${prepare_summary}" >"${work}/release-prepare.json"
  python3 scripts/distribution/materialize_stable_publication.py stage \
    --prepare-summary "${work}/release-prepare.json" \
    --qualification-index "${successor_qualification}" \
    --artifact-root "${work}/stable-assets" \
    --plan "${successor_plan}" --dispatchers "${work}/dispatchers" \
    --out "${work}/stable-staged"
  python3 scripts/distribution/publish_stable_release.py \
    --repository "${repository}" --version "${successor_version}" \
    --source-commit "$(jq -er '.source.commit' "${work}/release-prepare.json")" \
    --mode "${mode}" --assets "${work}/stable-staged/release-assets" \
    --notes "${successor_candidate}/release-notes.md" \
    --out "${work}/successor-assets.json"
  vsix_name="$(jq -er '.artifacts.vsix.name' "${work}/release-prepare.json")"
  vsix_container="$(
    jq -er '.artifacts.vsix.workflow_artifact_name' "${work}/release-prepare.json"
  )"
  GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="${marketplace_pat}" \
    scripts/distribution/publish_marketplace_extension.sh \
    --package "${work}/stable-assets/${vsix_container}/${vsix_name}" \
    --publisher "$(jq -er '.marketplace.publisher' "${work}/release-prepare.json")" \
    --extension "$(jq -er '.marketplace.extension' "${work}/release-prepare.json")" \
    --version "$(jq -er '.marketplace.version' "${work}/release-prepare.json")" \
    --expected-sha256 "$(jq -er '.marketplace.vsix_sha256' "${work}/release-prepare.json")" \
    --verified-out "${work}/marketplace.vsix"
else
  verify_retained_version \
    "${successor_version}" "${successor_plan}" "${successor_qualification}" \
    "${work}/successor-release"
  cp "${work}/successor-release/asset-digests.json" \
    "${work}/successor-assets.json"
  editor_report="$(
    jq -er '.artifacts[] | select(.id == "editor-qualification-report") | .name' \
      "${affected_qualification}"
  )"
  editor_report="${work}/affected-release/assets/${editor_report}"
  marketplace_publisher="$(
    jq -er '.marketplace_publish_plan.publisher' "${editor_report}"
  )"
  marketplace_extension="$(
    jq -er '.marketplace_publish_plan.extension' "${editor_report}"
  )"
  marketplace_version="$(jq -er '.vscode.version' "${affected_plan}")"
  [[ "${marketplace_publisher}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ &&
    "${marketplace_extension}" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || {
    echo "incident-publication: retained Marketplace identity is unsafe" >&2
    exit 2
  }
  jq -e \
    --arg version "${marketplace_version}" \
    --arg digest "$(jq -er '.vscode.vsix_sha256' "${affected_plan}")" \
    '.package_version == $version
     and .vsix_sha256 == $digest
     and .marketplace_publish_plan.version == $version
     and .marketplace_publish_plan.vsix_sha256 == $digest' \
    "${editor_report}" >/dev/null || {
    echo "incident-publication: retained Marketplace evidence drifted" >&2
    exit 2
  }
  curl -fsSL --connect-timeout 10 --max-time 120 \
    "https://${marketplace_publisher}.gallery.vsassets.io/_apis/public/gallery/publisher/${marketplace_publisher}/extension/${marketplace_extension}/${marketplace_version}/assetbyname/Microsoft.VisualStudio.Services.VSIXPackage" \
    -o "${work}/marketplace.vsix"
  python3 scripts/distribution/verify_marketplace_vsix.py \
    --vsix "${work}/marketplace.vsix" \
    --expected-sha256 "$(jq -er '.vscode.vsix_sha256' "${affected_plan}")" \
    --publisher "${marketplace_publisher}" \
    --extension "${marketplace_extension}" \
    --version "${marketplace_version}" \
    --compiler-version "${successor_version}"
fi

python3 scripts/distribution/materialize_incident_publication.py stage \
  --prepare-summary "${prepare_summary}" \
  --successor-plan "${successor_plan}" --site-plan "${site_plan}" \
  --dispatchers "${work}/dispatchers" --out "${work}/incident-staged"

request_asset="${work}/stable-incident-request-${incident_id}-${expected_request_sha256:0:16}.json"
cp "${request}" "${request_asset}"
allow_existing="$([[ "${mode}" == "resume" ]] && echo true || echo false)"
upload_or_verify_governance \
  "${request_asset}" "${allow_existing}"

fetch_governance "${work}/governance-before-index"
revalidate "${work}/governance-before-index"
publication_state="$(jq -er '.publication_state' "${prepare_summary}")"
proposed_generation="$(jq -er '.mutation.proposed_index.generation' "${prepare_summary}")"
proposed_sha256="$(jq -er '.mutation.proposed_index_sha256' "${prepare_summary}")"
if [[ "${publication_state}" == "pending" ]]; then
  snapshot="${work}/channels-generation-${proposed_generation}.json"
  cp "${work}/incident-staged/channels.json" "${snapshot}"
  upload_or_verify_governance "${snapshot}"
  live_after_snapshot="$(mktemp -d)"
  gh release download channels \
    --repo "${repository}" --pattern channels.json --dir "${live_after_snapshot}"
  cmp \
    "${work}/governance-before-index/current-channels.json" \
    "${live_after_snapshot}/channels.json" || {
    echo "incident-publication: index changed after generation reservation" >&2
    exit 2
  }
  gh release upload channels "${work}/incident-staged/channels.json" \
    --repo "${repository}" --clobber
elif [[ "${publication_state}" != "activated" ]]; then
  echo "incident-publication: unsupported publication state" >&2
  exit 2
fi
activated="$(mktemp -d)"
gh release download channels \
  --repo "${repository}" --pattern channels.json --dir "${activated}"
test "$(sha256sum "${activated}/channels.json" | awk '{print $1}')" = \
  "${proposed_sha256}" || {
  echo "incident-publication: activated index bytes drifted" >&2
  exit 2
}

site_base_commit="$(jq -er '.site.base_commit' "${prepare_summary}")"
source_commit="$(jq -er '.source_commit' "${successor_plan}")"
for dispatcher in index stable alpha beta; do
  digest="$(sha256sum "${work}/dispatchers/${dispatcher}" | awk '{print $1}')"
  printf -v "dispatcher_${dispatcher}_sha256" '%s' "${digest}"
done
python3 scripts/distribution/generate_site_publication_facts.py \
  --out "${work}/site-publication-facts.json" \
  --source-commit "${source_commit}" --site-base-commit "${site_base_commit}" \
  --release-plan-sha256 "$(jq -er '.successor.plan_sha256' "${prepare_summary}")" \
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
stable_site_facts_sha256="$(
  sha256sum \
    "${work}/incident-staged/stable-site-release-facts.json" |
    awk '{print $1}'
)"
SITE_TOKEN="${site_token}" \
  scripts/distribution/dispatch_stable_site_publication.sh \
  --repository "${site_repository}" --workflow "${site_workflow}" \
  --workflow-ref "${site_workflow_ref}" --ruleset-id "${site_ruleset_id}" \
  --ruleset-updated-at "${site_ruleset_updated_at}" \
  --workflow-sha256 "${site_workflow_sha256}" \
  --source-commit "${source_commit}" --site-commit "${site_base_commit}" \
  --release-plan-sha256 "$(jq -er '.successor.plan_sha256' "${prepare_summary}")" \
  --publication-attempt "${publication_attempt}" \
  --generation "${proposed_generation}" --index-sha256 "${proposed_sha256}" \
  --default-channel stable \
  --dispatcher-index-sha256 "${dispatcher_index_sha256}" \
  --dispatcher-stable-sha256 "${dispatcher_stable_sha256}" \
  --dispatcher-alpha-sha256 "${dispatcher_alpha_sha256}" \
  --dispatcher-beta-sha256 "${dispatcher_beta_sha256}" \
  --publication-facts-sha256 "${publication_facts_sha256}" \
  --stable-site-facts-sha256 "${stable_site_facts_sha256}" \
  --result-out "${work}/site-run.json"

GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  scripts/distribution/run_stable_public_smoke.sh \
  --repository "${repository}" --version "${successor_version}" \
  --index "${work}/incident-staged/channels.json" \
  --dispatchers "${work}/dispatchers" \
  --site-facts "${work}/incident-staged/stable-site-release-facts.json" \
  --asset-digests "${work}/successor-assets.json" \
  --marketplace-vsix "${work}/marketplace.vsix" \
  --out "${work}/stable-smoke"

release_signoff_arguments=()
if [[ "${operation}" == "incident-roll-forward" ]]; then
  release_signoff="${work}/stable-release-signoff-${successor_version}-attempt-${publication_attempt}.json"
  python3 scripts/distribution/materialize_stable_publication.py signoff \
    --prepare-summary "${work}/release-prepare.json" \
    --release-assets "${work}/stable-staged/release-assets" \
    --site-facts "${work}/incident-staged/stable-site-release-facts.json" \
    --site-run "${work}/site-run.json" --smoke "${work}/stable-smoke" \
    --run-id "${run_id}" --approver "${approver}" --out "${release_signoff}"
  release_signoff_arguments=(--release-signoff "${release_signoff}")
fi
mkdir "${work}/incident-smoke"
cp "${work}/stable-smoke"/* "${work}/incident-smoke/"
GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  scripts/distribution/run_incident_public_recovery.sh \
  --operation "${operation}" --affected-version "${affected_version}" \
  --successor-version "${successor_version}" \
  --working-root "${work}/working-client" \
  --broken-root "${work}/broken-client" \
  --stable-dispatcher "${work}/dispatchers/stable" \
  --out "${work}/incident-smoke/incident-recovery.json"
incident_signoff="${work}/stable-incident-signoff-${incident_id}-attempt-${publication_attempt}.json"
python3 scripts/distribution/materialize_incident_publication.py signoff \
  --prepare-summary "${prepare_summary}" --request "${request}" \
  --withdrawal-evidence "${withdrawal_evidence}" \
  --site-facts "${work}/incident-staged/stable-site-release-facts.json" \
  --site-run "${work}/site-run.json" --smoke "${work}/incident-smoke" \
  --run-id "${run_id}" --approver "${approver}" \
  "${release_signoff_arguments[@]}" --out "${incident_signoff}"
cp \
  "${work}/incident-staged/stable-site-release-facts.json" \
  "${work}/stable-site-release-facts-generation-${proposed_generation}.json"
upload_or_verify_governance \
  "${work}/stable-site-release-facts-generation-${proposed_generation}.json" \
  "${allow_existing}"
if [[ "${operation}" == "incident-roll-forward" ]]; then
  upload_or_verify_governance "${release_signoff}"
fi
upload_or_verify_governance "${incident_signoff}"
