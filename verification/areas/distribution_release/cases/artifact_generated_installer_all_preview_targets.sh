#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact-all-targets.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
sysroot_root="${tmp_dir}/mock-sysroot"
version="0.1.0-beta.7"

make_mock_binary "${binary}" "sifr all-target fixture"
make_mock_sysroot_root "${sysroot_root}"
"${REPO_ROOT}/scripts/distribution/build_preview_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" \
  --sysroot-root "${sysroot_root}" >/dev/null

"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu
do
  install_root="${tmp_dir}/install-${target}"
  install_dir="${install_root}/bin"
  SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_INSTALL_DIR="${install_dir}" \
    sh "${installer}" >/dev/null
  if ! grep -F 'sifr all-target fixture' "${install_dir}/sifr" >/dev/null; then
    echo "unexpected installed binary content for ${target}" >&2
    exit 1
  fi
  test -f "${install_root}/sysroot.toml" || { echo "missing installed sysroot manifest" >&2; exit 1; }
  test -f "${install_root}/.cargo/config.toml" || { echo "missing installed cargo config" >&2; exit 1; }
  test -d "${install_root}/vendor" || { echo "missing installed vendor" >&2; exit 1; }
  test -f "${install_root}/crates/sifr_runtime/Cargo.toml" || { echo "missing installed runtime crate" >&2; exit 1; }
  test -f "${install_root}/crates/sifr_stdlib/Cargo.toml" || { echo "missing installed stdlib crate" >&2; exit 1; }
  test -d "${install_root}/lib/sifr/stdlib/sifr" || { echo "missing installed public stdlib" >&2; exit 1; }
  test -d "${install_root}/lib/sifr/stdlib/_sifr" || { echo "missing installed private stdlib" >&2; exit 1; }
done
