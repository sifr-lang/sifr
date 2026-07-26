#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-candidate.XXXXXX")"
RESULT_ROOT="${REPO_ROOT}/target/verification/stable-candidate-demo-$$"
cleanup() {
  rm -rf "${DEMO_DIR}" "${RESULT_ROOT}"
}
trap cleanup EXIT HUP INT TERM

if [[ -n "$(git -C "${REPO_ROOT}" status --porcelain --untracked-files=all)" ]]; then
  echo "stable candidate qualification demo requires a clean source checkout" >&2
  exit 2
fi

host_target="$(
  PYTHONPATH="${REPO_ROOT}/verification/areas/distribution_release" python3 - <<'PY'
from governance.common import BUILDERS
from pathlib import Path
import importlib.util

path = Path("scripts/distribution/qualify_stable_target.py").resolve()
spec = importlib.util.spec_from_file_location("qualify_stable_target", path)
if spec is None or spec.loader is None:
    raise SystemExit("could not load target qualifier")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
target = module.current_host_target()
if target not in BUILDERS:
    raise SystemExit(f"unsupported host target: {target}")
print(target)
PY
)"
source_commit="$(git -C "${REPO_ROOT}" rev-parse HEAD)"
builder="$(
  PYTHONPATH="${REPO_ROOT}/verification/areas/distribution_release" \
    python3 - "${host_target}" <<'PY'
from governance.common import BUILDERS
import sys

print(BUILDERS[sys.argv[1]])
PY
)"

mkdir -p "${DEMO_DIR}/dist" "${DEMO_DIR}/qualification"
"${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
  --version 0.1.0 \
  --output-dir "${DEMO_DIR}/dist" \
  --cargo-build \
  --target "${host_target}"

archive="${DEMO_DIR}/dist/sifr-0.1.0-${host_target}.tar.gz"
"${REPO_ROOT}/scripts/distribution/qualify_stable_target.py" \
  --archive "${archive}" \
  --version 0.1.0 \
  --target "${host_target}" \
  --builder "${builder}" \
  --source-commit "${source_commit}" \
  --out-dir "${DEMO_DIR}/qualification"

PYTHONPATH="${REPO_ROOT}/verification/areas/distribution_release" \
  python3 - \
  "${REPO_ROOT}" \
  "${DEMO_DIR}" \
  "${RESULT_ROOT}" \
  "${archive}" \
  "${DEMO_DIR}/qualification" <<'PY'
from pathlib import Path
import subprocess
import sys

from governance.qualification_fixture import build_evidence_bundle

repo_root = Path(sys.argv[1])
demo_root = Path(sys.argv[2])
result_root = Path(sys.argv[3])
bundle = build_evidence_bundle(
    source_root=repo_root,
    evidence_root=demo_root / "evidence",
    result_root=result_root,
    host_archive=Path(sys.argv[4]),
    host_qualification_dir=Path(sys.argv[5]),
)
command = [
    sys.executable,
    str(repo_root / "scripts/distribution/release_governance.py"),
    "plan-stable-release",
    "--spec", str(bundle["plan_spec"]),
    "--source-root", str(bundle["source_root"]),
    "--source-ref", str(bundle["source_ref"]),
    "--live-index", str(bundle["active_index"]),
    "--release-report", str(bundle["release_report"]),
    "--qualification-index", str(bundle["qualification_index"]),
    "--artifact-root", str(bundle["artifact_root"]),
    "--stable-support-claims", str(bundle["stable_support_claims"]),
    "--rust-validation-report", str(bundle["rust_validation_report"]),
    "--documentation-report", str(bundle["documentation_report"]),
    "--release-notes", str(bundle["release_notes"]),
    "--out", str(demo_root / "stable-release-plan.json"),
]
subprocess.run(command, cwd=repo_root, check=True)
PY

python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" \
  validate \
  --kind release-plan \
  --input "${DEMO_DIR}/stable-release-plan.json" \
  --require-canonical

echo "Stable candidate qualification demo: PASS"
echo "  source=${source_commit}"
echo "  host_target=${host_target}"
echo "  artifact install/sifr --version/sifr check/sifr self version=pass"
echo "  planner=canonical, schema-complete, unapproved, non-mutating"
python3 -m json.tool "${DEMO_DIR}/stable-release-plan.json"
