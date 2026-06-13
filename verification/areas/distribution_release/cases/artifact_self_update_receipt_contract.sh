#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-self-update-receipt.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.12"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
install_dir="${tmp_dir}/managed/bin"
target="x86_64-unknown-linux-gnu"

make_target_specific_artifacts "${version}" "${artifact_dir}"
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

SIFR_TARGET="${target}" \
  SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
  SIFR_INSTALL_DIR="${install_dir}" \
  SIFR_NO_MODIFY_PATH=1 \
  sh "${installer}" --no-modify-path >/dev/null

receipt_path="${install_dir}/install.json"
schema_path="${REPO_ROOT}/verification/areas/distribution_release/schemas/self_update_install_receipt.schema.json"

python3 - "${receipt_path}" "${schema_path}" "${install_dir}" "${version}" "${target}" <<'PY'
import json
import pathlib
import sys

receipt = json.loads(pathlib.Path(sys.argv[1]).read_text())
schema = json.loads(pathlib.Path(sys.argv[2]).read_text())
install_dir_text = sys.argv[3]
install_dir = pathlib.Path(install_dir_text)
version = sys.argv[4]
target = sys.argv[5]
expected_keys = schema["required"]

if list(receipt.keys()) != expected_keys:
    raise SystemExit(f"receipt field order drifted: {list(receipt.keys())}")
if set(receipt) != set(expected_keys):
    raise SystemExit(f"receipt fields do not match schema: {sorted(receipt)}")
if receipt["schema_version"] != 1:
    raise SystemExit("schema_version must be 1")
if receipt["name"] != "sifr":
    raise SystemExit("name must be sifr")
if receipt["version"] != version:
    raise SystemExit("version drifted")
if receipt["channel"] != "beta":
    raise SystemExit("channel was not derived from version")
if receipt["target"] != target:
    raise SystemExit("target drifted")
if receipt["install_dir"] != install_dir_text:
    raise SystemExit("install_dir drifted")
if receipt["binary_path"] != str((install_dir / "sifr").resolve()):
    raise SystemExit(f"binary_path was not canonicalized: {receipt['binary_path']}")
if receipt["artifact"] != f"sifr-{version}-{target}.tar.gz":
    raise SystemExit("artifact drifted")
if receipt["modify_path"] is not False:
    raise SystemExit("modify_path must reflect SIFR_NO_MODIFY_PATH")
PY

if compgen -G "${install_dir}/.install.json.*" >/dev/null; then
  echo "temporary manifest file was not atomically renamed away" >&2
  exit 1
fi

if [[ -e "${install_dir}/.sifr-update.lock" ]]; then
  echo "installer lock was not released" >&2
  exit 1
fi

external_lock_install_dir="${tmp_dir}/external-lock/bin"
mkdir -p "${external_lock_install_dir}"
mkdir "${external_lock_install_dir}/.sifr-update.lock"
SIFR_TARGET="${target}" \
  SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
  SIFR_INSTALL_DIR="${external_lock_install_dir}" \
  SIFR_NO_MODIFY_PATH=1 \
  SIFR_INSTALL_LOCK_HELD=1 \
  sh "${installer}" --no-modify-path >/dev/null

if [[ ! -d "${external_lock_install_dir}/.sifr-update.lock" ]]; then
  echo "installer removed a caller-owned self-update lock" >&2
  exit 1
fi
rm -rf "${external_lock_install_dir}/.sifr-update.lock"

if [[ ! -x "${external_lock_install_dir}/sifr" ]]; then
  echo "installer did not install while caller-owned lock was held" >&2
  exit 1
fi

path_install_dir="${tmp_dir}/path-managed/bin"
home_dir="${tmp_dir}/home"
mkdir -p "${home_dir}"
HOME="${home_dir}" \
  SHELL=/bin/sh \
  PATH="/usr/bin:/bin" \
  SIFR_TARGET="${target}" \
  SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
  SIFR_INSTALL_DIR="${path_install_dir}" \
  sh "${installer}" >/dev/null

python3 - "${path_install_dir}/install.json" <<'PY'
import json
import pathlib
import sys

receipt = json.loads(pathlib.Path(sys.argv[1]).read_text())
if receipt["modify_path"] is not True:
    raise SystemExit("modify_path must be true when PATH modification is not disabled")
PY
