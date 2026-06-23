#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-install-version-state.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.3"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
target="x86_64-unknown-linux-gnu"

make_target_specific_artifacts "${version}" "${artifact_dir}"
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

run_installer() {
  local install_dir="$1"
  shift
  SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_INSTALL_DIR="${install_dir}" \
    SIFR_NO_MODIFY_PATH=1 \
    sh "${installer}" --no-modify-path "$@"
}

fresh_install_root="${tmp_dir}/fresh"
fresh_install_dir="${fresh_install_root}/bin"
fresh_output="$(run_installer "${fresh_install_dir}")"
[[ "${fresh_output}" == *"Installing Sifr ${version}"* ]] || {
  echo "fresh install did not report installation" >&2
  echo "${fresh_output}" >&2
  exit 1
}
grep -F "\"version\": \"${version}\"" "${fresh_install_root}/install.json" >/dev/null

same_output="$(run_installer "${fresh_install_dir}")"
[[ "${same_output}" == *"Sifr ${version} is already installed"* ]] || {
  echo "same-version install did not report already installed" >&2
  echo "${same_output}" >&2
  exit 1
}

force_same_output="$(run_installer "${fresh_install_dir}" --force)"
[[ "${force_same_output}" == *"Reinstalling Sifr ${version} at ${fresh_install_dir}/sifr"* ]] || {
  echo "same-version force install did not report reinstall" >&2
  echo "${force_same_output}" >&2
  exit 1
}

custom_manifest_install_dir="${tmp_dir}/custom-manifest/bin"
custom_manifest_dir="${tmp_dir}/custom-manifest/state"
custom_manifest_output="$(
  SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_INSTALL_DIR="${custom_manifest_install_dir}" \
    SIFR_INSTALL_MANIFEST_DIR="${custom_manifest_dir}" \
    SIFR_NO_MODIFY_PATH=1 \
    sh "${installer}" --no-modify-path
)"
[[ "${custom_manifest_output}" == *"Installing Sifr ${version}"* ]] || {
  echo "custom manifest install did not report installation" >&2
  echo "${custom_manifest_output}" >&2
  exit 1
}
grep -F "\"version\": \"${version}\"" "${custom_manifest_dir}/install.json" >/dev/null
[[ ! -e "${custom_manifest_install_dir}/install.json" ]] || {
  echo "installer ignored custom manifest directory" >&2
  exit 1
}

known_old_dir="${tmp_dir}/known-old/bin"
mkdir -p "${known_old_dir}"
make_mock_binary "${known_old_dir}/sifr" "old known fixture"
printf '%s\n' '{"name":"sifr","version":"0.1.0-beta.2"}' >"${tmp_dir}/known-old/install.json"
known_old_output="$(run_installer "${known_old_dir}")"
[[ "${known_old_output}" == *"Updating Sifr 0.1.0-beta.2 -> ${version}"* ]] || {
  echo "known older install did not report update" >&2
  echo "${known_old_output}" >&2
  exit 1
}

unknown_old_dir="${tmp_dir}/unknown-old/bin"
mkdir -p "${unknown_old_dir}"
mkdir "${unknown_old_dir}/sifr"
chmod 755 "${unknown_old_dir}/sifr"
unknown_old_output="$(run_installer "${unknown_old_dir}")"
[[ "${unknown_old_output}" == *"Updating existing Sifr installation (version unknown) -> ${version}"* ]] || {
  echo "unknown existing install did not report version-unknown update" >&2
  echo "${unknown_old_output}" >&2
  exit 1
}

newer_dir="${tmp_dir}/newer/bin"
mkdir -p "${newer_dir}"
make_mock_binary "${newer_dir}/sifr" "newer fixture"
cat >"${tmp_dir}/newer/install.json" <<'EOF'
{
  "name": "sifr",
  "version": "0.1.0-beta.4"
}
EOF
require_failure_contains \
  "installed Sifr 0.1.0-beta.4 is newer than requested ${version}; use --force to downgrade" \
  run_installer "${newer_dir}"

force_output="$(run_installer "${newer_dir}" --force)"
[[ "${force_output}" == *"Downgrading Sifr 0.1.0-beta.4 -> ${version} (--force)"* ]] || {
  echo "forced downgrade did not report downgrade" >&2
  echo "${force_output}" >&2
  exit 1
}
