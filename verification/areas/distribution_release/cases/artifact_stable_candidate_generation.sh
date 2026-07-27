#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-stable-candidate.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0"
binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/sifr-installer-${version}"
sysroot_root="${tmp_dir}/mock-sysroot"

make_mock_binary "${binary}" "sifr stable candidate fixture"
make_mock_sysroot_root "${sysroot_root}"
"${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" \
  --sysroot-root "${sysroot_root}" >/dev/null

"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

grep -F 'APP_VERSION="0.1.0"' "${installer}" >/dev/null
grep -F 'APP_CHANNEL="stable"' "${installer}" >/dev/null

for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu
do
  install_root="${tmp_dir}/install-${target}"
  SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_INSTALL_DIR="${install_root}/bin" \
    SIFR_INSTALL_MANIFEST_DIR="${install_root}" \
    SIFR_NO_MODIFY_PATH=1 \
    sh "${installer}" >/dev/null
  grep -F '"version": "0.1.0"' "${install_root}/install.json" >/dev/null
  grep -F '"channel": "stable"' "${install_root}/install.json" >/dev/null
  grep -F '"sifr-version" = "0.1.0"' "${install_root}/sysroot.toml" >/dev/null
  grep -F "\"target-triple\" = \"${target}\"" "${install_root}/sysroot.toml" >/dev/null
done

case "$(uname -s):$(uname -m)" in
  Darwin:arm64)
    host_target="aarch64-apple-darwin"
    host_builder="macos-15"
    ;;
  Darwin:x86_64)
    host_target="x86_64-apple-darwin"
    host_builder="macos-15-intel"
    ;;
  Linux:aarch64 | Linux:arm64)
    host_target="aarch64-unknown-linux-gnu"
    host_builder="ubuntu-24.04-arm"
    ;;
  Linux:x86_64)
    host_target="x86_64-unknown-linux-gnu"
    host_builder="ubuntu-24.04"
    ;;
  *)
    echo "unsupported stable qualification host" >&2
    exit 2
    ;;
esac

host_archive="${artifact_dir}/sifr-${version}-${host_target}.tar.gz"
printf '\377\376' >"${host_archive}.sha256"
if "${REPO_ROOT}/scripts/distribution/qualify_stable_target.py" \
  --archive "${host_archive}" \
  --version "${version}" \
  --target "${host_target}" \
  --builder "${host_builder}" \
  --source-commit "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" \
  --out-dir "${tmp_dir}/invalid-qualification" \
  >"${tmp_dir}/invalid-qualification.stdout" \
  2>"${tmp_dir}/invalid-qualification.stderr"
then
  echo "target qualifier accepted a non-UTF-8 checksum" >&2
  exit 1
fi
grep -F "archive checksum is not readable UTF-8" \
  "${tmp_dir}/invalid-qualification.stderr" >/dev/null
if grep -F "Traceback" "${tmp_dir}/invalid-qualification.stderr" >/dev/null; then
  echo "target qualifier leaked a traceback for non-UTF-8 evidence" >&2
  exit 1
fi

echo "stable candidate artifact generation: PASS"
