#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

python3 "${REPO_ROOT}/scripts/distribution/qualify_stable_editor.py" --self-test
python3 "${REPO_ROOT}/scripts/distribution/qualify_stable_documentation.py" --self-test
grep -F '"--candidate-smoke"' \
  "${REPO_ROOT}/scripts/distribution/qualify_stable_editor.py" >/dev/null
grep -F 'timeout=120' \
  "${REPO_ROOT}/scripts/distribution/qualify_stable_editor.py" >/dev/null
python3 "${REPO_ROOT}/scripts/distribution/render_stable_release_docs.py" \
  --facts \
  "${REPO_ROOT}/verification/areas/documentation/fixtures/stable_site_release_facts.json" \
  --document "${REPO_ROOT}/docs/releases/stable.mdx" \
  --check

report_validator="${REPO_ROOT}/verification/areas/distribution_release/governance/planner.py"
editor_validator="${REPO_ROOT}/verification/areas/distribution_release/governance/editor_qualification.py"
for field in \
  candidate_version \
  rollback_version \
  candidate_target \
  candidate_binary_sha256 \
  target_report_sha256 \
  vsix_package_smoke \
  lsp_smoke \
  marketplace_publish_plan
do
  grep -F "\"${field}\"" "${report_validator}" "${editor_validator}" >/dev/null || {
    echo "editor qualification report omitted ${field}" >&2
    exit 1
  }
done

echo "stable editor qualification contract: PASS"
