#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-self-json-parity.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.12"
target="$(rustc -vV | sed -n 's/^host: //p')"
case "${target}" in
  aarch64-apple-darwin|x86_64-apple-darwin|aarch64-unknown-linux-gnu|x86_64-unknown-linux-gnu) ;;
  *) echo "unsupported parity-test host: ${target}" >&2; exit 2 ;;
esac

build_root="${REPO_ROOT}/target/phase40-self-json-parity"
SIFR_RELEASE_VERSION="${version}" \
  CARGO_TARGET_DIR="${build_root}" \
  cargo build -q --locked -p sifr --bin sifr

install_root="${tmp_dir}/managed"
install_dir="${install_root}/bin"
binary_path="${install_dir}/sifr"
mkdir -p "${install_dir}"
cp "${build_root}/debug/sifr" "${binary_path}"
chmod 755 "${binary_path}"
cat >"${install_root}/sysroot.toml" <<EOF
schema-version = 1
sifr-version = "${version}"
target-triple = "${target}"
sysroot-content-sha256 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
EOF

python3 - "${install_root}/install.json" "${install_dir}" "${binary_path}" "${install_root}" "${version}" "${target}" <<'PY'
import json
import pathlib
import sys

out, install_dir, binary_path, install_root, version, target = sys.argv[1:]
payload = {
    "schema_version": 2,
    "name": "sifr",
    "version": version,
    "channel": "beta",
    "target": target,
    "install_dir": install_dir,
    "binary_path": binary_path,
    "sysroot_path": install_root,
    "sysroot_schema_version": 1,
    "sysroot_sifr_version": version,
    "sysroot_target_triple": target,
    "sysroot_content_sha256": "a" * 64,
    "artifact": f"sifr-{version}-{target}.tar.gz",
    "modify_path": False,
}
pathlib.Path(out).write_text(json.dumps(payload, sort_keys=True, separators=(",", ":")) + "\n")
PY

SIFR_INSTALL_MANIFEST_DIR="${install_root}" \
  "${binary_path}" self version --format json >"${tmp_dir}/self-version.json"
python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" validate \
  --kind self-version \
  --input "${tmp_dir}/self-version.json" >/dev/null

SIFR_INSTALL_MANIFEST_DIR="${install_root}" \
  "${binary_path}" self update \
    --version 0.1.0-beta.13 \
    --dry-run \
    --format json >"${tmp_dir}/self-update-plan.json"
python3 "${REPO_ROOT}/scripts/distribution/release_governance.py" validate \
  --kind self-update-plan \
  --input "${tmp_dir}/self-update-plan.json" >/dev/null
