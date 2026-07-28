#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-release-governance.XXXXXX")"
cleanup() {
  rm -rf "${DEMO_DIR}"
}
trap cleanup EXIT HUP INT TERM

python3 - "${REPO_ROOT}" "${DEMO_DIR}/plan-spec.json" <<'PY'
import sys
from pathlib import Path

repo_root = Path(sys.argv[1])
sys.path.insert(0, str(repo_root / "verification" / "areas" / "distribution_release"))

from governance.common import write_canonical_json
from governance.selftest import valid_plan

write_canonical_json(Path(sys.argv[2]), valid_plan())
PY

python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" \
  generate-release-plan \
  --spec "${DEMO_DIR}/plan-spec.json" \
  --out "${DEMO_DIR}/stable-release-plan.json"

python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" \
  validate \
  --kind release-plan \
  --input "${DEMO_DIR}/stable-release-plan.json" \
  --require-canonical

echo "Canonical, non-mutating stable release governance plan:"
python3 -m json.tool "${DEMO_DIR}/stable-release-plan.json"

# Record the distinct protected reviewer contract without using credentials.
cat >"${DEMO_DIR}/approvals.json" <<'JSON'
[{
  "state": "approved",
  "environments": [{"name": "stable-release"}],
  "user": {"login": "release-reviewer"}
}]
JSON
echo
echo "Protected approval evidence (initiator release-operator is excluded):"
python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" \
  resolve-publication-approvers \
  --approvals "${DEMO_DIR}/approvals.json" \
  --initiator release-operator \
  --environment stable-release

# Exercise the credential-free GA, rollback, and first-GA recovery orchestration.
echo
echo "Protected credential-free GA and incident drills:"
(
  cd "${REPO_ROOT}"
  env -u CLOUDFLARE_API_TOKEN -u GH_TOKEN -u GITHUB_TOKEN -u VSCE_PAT \
    -u SIFR_SITE_TOKEN -u SIFR_WEBSITE_ACTIONS_TOKEN \
    python3 -m \
    verification.areas.distribution_release.governance.protected_drill_selftest
)

# Exercise the production adapters against local immutable-release, Gallery,
# installer/update, site-correlation, and incident evidence fixtures.
echo
echo "Stable install/update, Marketplace, and incident publication adapters:"
(
  cd "${REPO_ROOT}"
  python3 -m \
    verification.areas.distribution_release.governance.stable_publish_selftest
  python3 -m \
    verification.areas.distribution_release.governance.stable_public_smoke_selftest
  python3 -m \
    verification.areas.distribution_release.governance.incident_publication_selftest
  python3 -m \
    verification.areas.distribution_release.governance.incident_public_recovery_selftest
)

# Show the exact protected production entrypoint without dispatching or exposing
# credentials. A real operator substitutes the merged evidence identities.
grep -F -- "- incident-roll-forward" \
  "${REPO_ROOT}/.github/workflows/release-publication.yml" >/dev/null
echo
echo "Protected roll-forward dispatch shape (not executed by this demo):"
echo "gh workflow run release-publication.yml --ref main \\"
echo "  -f governance_mode=incident-roll-forward -f publication_mode=initial \\"
echo "  -f evidence_commit=<merged-candidate-commit> \\"
echo "  -f candidate_path=plans/releases/candidates/<version> \\"
echo "  -f expected_plan_sha256=<sha256> \\"
echo "  -f incident_commit=<merged-incident-commit> \\"
echo "  -f incident_path=plans/releases/incidents/<incident-id>/stable-incident-request.json \\"
echo "  -f expected_request_sha256=<sha256>"
