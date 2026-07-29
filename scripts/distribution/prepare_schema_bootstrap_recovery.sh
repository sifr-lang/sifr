#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF' >&2
usage: prepare_schema_bootstrap_recovery.sh \
  --repository OWNER/REPO --original-run-id ID --original-run-attempt N \
  --failed-site-run-id ID --alpha-version VERSION --beta-version VERSION \
  --source-commit COMMIT --site-commit COMMIT \
  --prepare-summary PATH --mutation-approval-decision PATH \
  --expected-prepare-summary-sha256 SHA256 \
  --expected-index-sha256 SHA256 --expected-plan-sha256 SHA256 \
  --expected-site-facts-sha256 SHA256 --work-dir DIR --summary-out PATH
EOF
  exit 2
}

repository=""
original_run_id=""
original_run_attempt=""
failed_site_run_id=""
alpha_version=""
beta_version=""
source_commit=""
site_commit=""
prepare_summary=""
mutation_approval_decision=""
expected_prepare_summary_sha256=""
expected_index_sha256=""
expected_plan_sha256=""
expected_site_facts_sha256=""
work_dir=""
summary_out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --original-run-id) original_run_id="${2:-}"; shift 2 ;;
    --original-run-attempt) original_run_attempt="${2:-}"; shift 2 ;;
    --failed-site-run-id) failed_site_run_id="${2:-}"; shift 2 ;;
    --alpha-version) alpha_version="${2:-}"; shift 2 ;;
    --beta-version) beta_version="${2:-}"; shift 2 ;;
    --source-commit) source_commit="${2:-}"; shift 2 ;;
    --site-commit) site_commit="${2:-}"; shift 2 ;;
    --prepare-summary) prepare_summary="${2:-}"; shift 2 ;;
    --mutation-approval-decision) mutation_approval_decision="${2:-}"; shift 2 ;;
    --expected-prepare-summary-sha256)
      expected_prepare_summary_sha256="${2:-}"
      shift 2
      ;;
    --expected-index-sha256) expected_index_sha256="${2:-}"; shift 2 ;;
    --expected-plan-sha256) expected_plan_sha256="${2:-}"; shift 2 ;;
    --expected-site-facts-sha256)
      expected_site_facts_sha256="${2:-}"
      shift 2
      ;;
    --work-dir) work_dir="${2:-}"; shift 2 ;;
    --summary-out) summary_out="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ "${repository}" = "sifr-lang/sifr" &&
  "${original_run_id}" =~ ^[1-9][0-9]*$ &&
  "${original_run_attempt}" =~ ^[1-9][0-9]*$ &&
  "${failed_site_run_id}" =~ ^[1-9][0-9]*$ &&
  "${alpha_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$ &&
  "${beta_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-beta\.[0-9]+$ &&
  "${source_commit}" =~ ^[0-9a-f]{40}$ &&
  "${site_commit}" =~ ^[0-9a-f]{40}$ &&
  "${expected_prepare_summary_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${expected_index_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${expected_plan_sha256}" =~ ^[0-9a-f]{64}$ &&
  "${expected_site_facts_sha256}" =~ ^[0-9a-f]{64}$ &&
  -n "${prepare_summary}" && -n "${mutation_approval_decision}" &&
  -n "${work_dir}" && -n "${summary_out}" ]] || usage
test -f "${prepare_summary}" && ! test -L "${prepare_summary}" || usage
test -f "${mutation_approval_decision}" &&
  ! test -L "${mutation_approval_decision}" || usage
! test -e "${work_dir}" && ! test -L "${work_dir}" || usage
! test -e "${summary_out}" && ! test -L "${summary_out}" || usage

test "$(sha256sum "${prepare_summary}" | awk '{print $1}')" = \
  "${expected_prepare_summary_sha256}" || {
  echo "schema-bootstrap-recovery: original prepare summary digest drifted" >&2
  exit 2
}
jq -e \
  --arg alpha "${alpha_version}" \
  --arg beta "${beta_version}" \
  --arg source "${source_commit}" \
  '
    keys == [
      "assets",
      "bootstrap_alpha_evidence_sha256",
      "bootstrap_alpha_version",
      "channel",
      "current_index_sha256",
      "operation",
      "schema_version",
      "source_commit",
      "version"
    ]
    and .schema_version == 2
    and .operation == "bootstrap-index"
    and .channel == "beta"
    and .version == $beta
    and .source_commit == $source
    and .bootstrap_alpha_version == $alpha
    and (.bootstrap_alpha_evidence_sha256 | test("^[0-9a-f]{64}$"))
    and .current_index_sha256 ==
      "71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef"
    and (.assets | type == "object" and length == 9)
  ' "${prepare_summary}" >/dev/null || {
  echo "schema-bootstrap-recovery: original prepare summary contract drifted" >&2
  exit 2
}
jq -e '
  keys == ["approval_policy", "approvers"]
  and (.approvers | type == "array" and length > 0)
  and (.approval_policy.mode |
    . == "distinct-reviewer" or . == "single-maintainer-waiver")
  and (.approval_policy.waiver_sha256 |
    . == "none" or test("^[0-9a-f]{64}$"))
' "${mutation_approval_decision}" >/dev/null || {
  echo "schema-bootstrap-recovery: mutation approval decision drifted" >&2
  exit 2
}

mkdir "${work_dir}"
mkdir \
  "${work_dir}/channels" \
  "${work_dir}/dispatchers" \
  "${work_dir}/release-records"
gh release download channels \
  --repo "${repository}" \
  --pattern channels.json \
  --pattern channels-generation-1.json \
  --pattern "schema-v2-bootstrap-alpha-${alpha_version}.json" \
  --dir "${work_dir}/channels"
index="${work_dir}/channels/channels.json"
snapshot="${work_dir}/channels/channels-generation-1.json"
alpha_evidence="$(
  printf '%s/channels/schema-v2-bootstrap-alpha-%s.json' \
    "${work_dir}" "${alpha_version}"
)"
scripts/distribution/release_governance.py validate \
  --kind release-index --input "${index}" --require-canonical
cmp "${index}" "${snapshot}" || {
  echo "schema-bootstrap-recovery: generation 1 snapshot and live index drifted" >&2
  exit 2
}
test "$(sha256sum "${index}" | awk '{print $1}')" = \
  "${expected_index_sha256}" || {
  echo "schema-bootstrap-recovery: generation 1 digest drifted" >&2
  exit 2
}
test "$(sha256sum "${alpha_evidence}" | awk '{print $1}')" = "$(
  jq -r '.bootstrap_alpha_evidence_sha256' "${prepare_summary}"
)" || {
  echo "schema-bootstrap-recovery: alpha evidence digest drifted" >&2
  exit 2
}

scripts/distribution/fetch_schema_bootstrap_alpha.sh \
  --repository "${repository}" \
  --version "${alpha_version}" \
  --evidence "${alpha_evidence}" \
  --assets "${work_dir}/alpha-assets" \
  --record "${work_dir}/release-records/alpha.json"
scripts/distribution/fetch_schema_bootstrap_beta.sh \
  --repository "${repository}" \
  --version "${beta_version}" \
  --source-commit "${source_commit}" \
  --index "${index}" \
  --assets "${work_dir}/beta-assets" \
  --record "${work_dir}/release-records/beta.json"
beta_asset_digests="$(
  find "${work_dir}/beta-assets" -mindepth 1 -maxdepth 1 -type f -print0 |
    LC_ALL=C sort -z |
    xargs -0 sha256sum |
    jq -Rn '
      [inputs | capture("^(?<sha>[0-9a-f]{64})  .*/(?<name>[^/]+)$")]
      | map({key:.name, value:.sha})
      | from_entries
    '
)"
jq -e \
  --argjson published "${beta_asset_digests}" \
  '.assets == $published' "${prepare_summary}" >/dev/null || {
  echo "schema-bootstrap-recovery: published beta assets drifted from prepare" >&2
  exit 2
}

scripts/distribution/generate_dispatchers.sh \
  --install-root "${work_dir}/dispatchers" \
  --default-channel beta
publication_attempt="${original_run_id}-${original_run_attempt}"
plan="${work_dir}/publication-plan.json"
jq -cnS \
  --arg channel beta \
  --arg version "${beta_version}" \
  --arg source "${source_commit}" \
  --arg site "${site_commit}" \
  --arg current_sha \
    "71b3243925670f56dc510b8f45b6614a622f58097a0fea9492f61d20dc4bf9ef" \
  --argjson current_generation 0 \
  --argjson proposed_generation 1 \
  --arg attempt "${publication_attempt}" \
  --arg site_default beta \
  --arg alpha "${alpha_version}" \
  '{
    schema_version: 2,
    operation: "schema-epoch-bootstrap",
    channel: $channel,
    version: $version,
    source_commit: $source,
    site_base_commit: $site,
    current_index: {
      generation: $current_generation,
      sha256: $current_sha
    },
    proposed_generation: $proposed_generation,
    publication_attempt: $attempt,
    site_default_channel: $site_default,
    bootstrap_alpha_version: $alpha
  }' >"${plan}"
test "$(sha256sum "${plan}" | awk '{print $1}')" = \
  "${expected_plan_sha256}" || {
  echo "schema-bootstrap-recovery: original release plan is not reproducible" >&2
  exit 2
}

dispatcher_index_sha256="$(
  sha256sum "${work_dir}/dispatchers/index" | awk '{print $1}'
)"
dispatcher_stable_sha256="$(
  sha256sum "${work_dir}/dispatchers/stable" | awk '{print $1}'
)"
dispatcher_alpha_sha256="$(
  sha256sum "${work_dir}/dispatchers/alpha" | awk '{print $1}'
)"
dispatcher_beta_sha256="$(
  sha256sum "${work_dir}/dispatchers/beta" | awk '{print $1}'
)"
site_facts="${work_dir}/site-publication-facts.json"
scripts/distribution/generate_site_publication_facts.py \
  --out "${site_facts}" \
  --source-commit "${source_commit}" \
  --site-base-commit "${site_commit}" \
  --release-plan-sha256 "${expected_plan_sha256}" \
  --publication-attempt "${publication_attempt}" \
  --release-index-generation 1 \
  --release-index-sha256 "${expected_index_sha256}" \
  --dispatcher-default-channel beta \
  --dispatcher-index-sha256 "${dispatcher_index_sha256}" \
  --dispatcher-stable-sha256 "${dispatcher_stable_sha256}" \
  --dispatcher-alpha-sha256 "${dispatcher_alpha_sha256}" \
  --dispatcher-beta-sha256 "${dispatcher_beta_sha256}"
test "$(sha256sum "${site_facts}" | awk '{print $1}')" = \
  "${expected_site_facts_sha256}" || {
  echo "schema-bootstrap-recovery: site publication facts are not reproducible" >&2
  exit 2
}

jq -cnS \
  --arg source "${source_commit}" \
  --arg site "${site_commit}" \
  --arg alpha "${alpha_version}" \
  --arg beta "${beta_version}" \
  --arg prepare "${expected_prepare_summary_sha256}" \
  --arg index "${expected_index_sha256}" \
  --arg plan "${expected_plan_sha256}" \
  --arg site_facts "${expected_site_facts_sha256}" \
  --arg dispatcher_index "${dispatcher_index_sha256}" \
  --arg dispatcher_stable "${dispatcher_stable_sha256}" \
  --arg dispatcher_alpha "${dispatcher_alpha_sha256}" \
  --arg dispatcher_beta "${dispatcher_beta_sha256}" \
  --arg alpha_evidence_sha256 "$(
    sha256sum "${alpha_evidence}" | awk '{print $1}'
  )" \
  --arg alpha_record_sha256 "$(
    sha256sum "${work_dir}/release-records/alpha.json" | awk '{print $1}'
  )" \
  --arg beta_record_sha256 "$(
    sha256sum "${work_dir}/release-records/beta.json" | awk '{print $1}'
  )" \
  --argjson original_run_id "${original_run_id}" \
  --argjson original_run_attempt "${original_run_attempt}" \
  --argjson failed_site_run_id "${failed_site_run_id}" \
  --slurpfile approval "${mutation_approval_decision}" \
  '{
    schema_version: 2,
    operation: "schema-bootstrap-index-recovery",
    original_run_id: $original_run_id,
    original_run_attempt: $original_run_attempt,
    failed_site_run_id: $failed_site_run_id,
    source_commit: $source,
    site_base_commit: $site,
    alpha_version: $alpha,
    beta_version: $beta,
    original_prepare_summary_sha256: $prepare,
    index: {generation: 1, sha256: $index},
    release_plan_sha256: $plan,
    site_publication_facts_sha256: $site_facts,
    dispatchers: {
      index: $dispatcher_index,
      stable: $dispatcher_stable,
      alpha: $dispatcher_alpha,
      beta: $dispatcher_beta
    },
    alpha_evidence_sha256: $alpha_evidence_sha256,
    release_records: {
      alpha: $alpha_record_sha256,
      beta: $beta_record_sha256
    },
    mutation_approval: $approval[0]
  }' >"${summary_out}"
