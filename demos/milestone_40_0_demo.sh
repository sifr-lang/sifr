#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sifr-milestone-40-0.XXXXXX")"
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

echo "Canonical, non-mutating milestone-40.0 release plan:"
python3 -m json.tool "${DEMO_DIR}/stable-release-plan.json"
