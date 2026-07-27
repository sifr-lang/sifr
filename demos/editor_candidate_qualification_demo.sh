#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
work_root="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-editor.XXXXXX")"
trap 'rm -rf "${work_root}"' EXIT

source_commit="$(git -C "${repo_root}" rev-parse HEAD)"
report="${work_root}/qualification-editor.json"
fixture="${work_root}/editor_fixture.sifr"
test_dir="${work_root}/candidate_tests"
test_fixture="${test_dir}/test_editor_candidate.sifr"

if [[ -n "$(git -C "${repo_root}" status --porcelain --untracked-files=all)" ]]; then
  echo "stable editor release demo requires a clean source checkout" >&2
  exit 2
fi

host_target="$(
  PYTHONPATH="${repo_root}/verification/areas/distribution_release" \
    python3 - <<'PY'
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
builder="$(
  PYTHONPATH="${repo_root}/verification/areas/distribution_release" \
    python3 - "${host_target}" <<'PY'
from governance.common import BUILDERS
import sys

print(BUILDERS[sys.argv[1]])
PY
)"

mkdir -p "${work_root}/dist" "${work_root}/target-qualification"
"${repo_root}/scripts/distribution/build_release_artifacts.sh" \
  --version 0.1.0 \
  --output-dir "${work_root}/dist" \
  --cargo-build \
  --target "${host_target}"
archive="${work_root}/dist/sifr-0.1.0-${host_target}.tar.gz"
"${repo_root}/scripts/distribution/qualify_stable_target.py" \
  --archive "${archive}" \
  --version 0.1.0 \
  --target "${host_target}" \
  --builder "${builder}" \
  --source-commit "${source_commit}" \
  --out-dir "${work_root}/target-qualification"
mkdir -p "${work_root}/installed-candidate"
tar -xzf "${archive}" -C "${work_root}/installed-candidate"
candidate_binary="${work_root}/installed-candidate/bin/sifr"
target_report="${work_root}/target-qualification/qualification-${host_target}.json"
test "$("${candidate_binary}" --version)" = "sifr 0.1.0"

npm ci --prefix "${repo_root}/editor_integrations/vscode"
for script in lint typecheck test test:extension package; do
  npm run --prefix "${repo_root}/editor_integrations/vscode" "${script}"
done

vsix=("${repo_root}"/editor_integrations/vscode/dist/*.vsix)
test "${#vsix[@]}" -eq 1
python3 "${repo_root}/scripts/distribution/qualify_stable_editor.py" \
  --source-root "${repo_root}" \
  --source-commit "${source_commit}" \
  --candidate-version 0.1.0 \
  --rollback-version none \
  --candidate-binary "${candidate_binary}" \
  --target-report "${target_report}" \
  --vsix "${vsix[0]}" \
  --out "${report}"

cat >"${fixture}" <<'SIFR'
def add(left: int, right: int) -> int:
    return left + right

def main() -> None:
    print(add(20, 22))
SIFR
mkdir -p "${test_dir}"
cat >"${test_fixture}" <<'SIFR'
@test
def test_addition() -> None:
    assert 20 + 22 == 42
SIFR

(
  cd "${work_root}"
  "${candidate_binary}" check "${fixture}"
  "${candidate_binary}" fmt "${fixture}"
  "${candidate_binary}" fmt --check "${fixture}"
  "${candidate_binary}" lint "${fixture}"
  "${candidate_binary}" test "${test_dir}"
)

python3 - "${report}" <<'PY'
import json
import sys
from pathlib import Path

report = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["candidate_version"] == "0.1.0"
assert report["package_version"] == "0.2.0"
assert report["compiler_compatibility"] == ">=0.1.0,<0.2.0"
assert report["vsix_package_smoke"] == "pass"
assert report["lsp_smoke"] == "pass"
assert report["marketplace_publish_plan"]["rebuild"] is False
assert report["marketplace_publish_plan"]["status"] == "planned"
print("editor candidate qualification demo: PASS")
PY
