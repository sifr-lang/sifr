#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo "usage: verify_site_workflow_identity.sh --repository OWNER/REPO --ruleset-id ID --ruleset-updated-at TIMESTAMP --workflow-ref REF --site-commit SHA --workflow PATH --workflow-sha256 SHA256" >&2
  exit 2
}

repository=""
ruleset_id=""
ruleset_updated_at=""
workflow_ref=""
site_commit=""
workflow=""
workflow_sha256=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --ruleset-id) ruleset_id="${2:-}"; shift 2 ;;
    --ruleset-updated-at) ruleset_updated_at="${2:-}"; shift 2 ;;
    --workflow-ref) workflow_ref="${2:-}"; shift 2 ;;
    --site-commit) site_commit="${2:-}"; shift 2 ;;
    --workflow) workflow="${2:-}"; shift 2 ;;
    --workflow-sha256) workflow_sha256="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ -n "${GH_TOKEN:-}" && -n "${repository}" &&
  "${ruleset_id}" =~ ^[1-9][0-9]*$ &&
  -n "${ruleset_updated_at}" && -n "${workflow_ref}" &&
  "${site_commit}" =~ ^[0-9a-f]{40}$ && -n "${workflow}" &&
  "${workflow_sha256}" =~ ^[0-9a-f]{64}$ ]] || usage

site_ruleset="$(
  gh api \
    -H "Time-Zone: UTC" \
    "repos/${repository}/rulesets/${ruleset_id}"
)"
if ! jq -e \
  --arg ref "refs/tags/${workflow_ref}" \
  --arg updated_at "${ruleset_updated_at}" \
  --argjson id "${ruleset_id}" \
  '.id == $id
   and .target == "tag"
   and .enforcement == "active"
   and .updated_at == $updated_at
   and .conditions.ref_name.include == [$ref]
   and .conditions.ref_name.exclude == []
   and (.bypass_actors // []) == []
   and (.current_user_can_bypass // "never") == "never"
   and ([.rules[].type] | sort) == ["deletion", "update"]' \
  <<<"${site_ruleset}" >/dev/null
then
  echo "site-workflow-identity: immutable tag ruleset is not active and exact" >&2
  exit 2
fi

site_dispatch_sha="$(
  gh api \
    "repos/${repository}/git/ref/tags/${workflow_ref}" \
    --jq '.object.sha'
)"
if [[ "${site_dispatch_sha}" != "${site_commit}" ]]; then
  echo "site-workflow-identity: protected workflow tag moved" >&2
  exit 2
fi

site_workflow="$(mktemp)"
gh api \
  -H "Accept: application/vnd.github.raw+json" \
  "repos/${repository}/contents/.github/workflows/${workflow}?ref=${site_commit}" \
  >"${site_workflow}"
actual_sha256="$(sha256sum "${site_workflow}" | awk '{print $1}')"
if [[ "${actual_sha256}" != "${workflow_sha256}" ]]; then
  echo "site-workflow-identity: pinned workflow bytes do not match the reviewed contract" >&2
  exit 2
fi
