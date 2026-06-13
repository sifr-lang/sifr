#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact-no-path.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.11"
binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
home_dir="${tmp_dir}/home"
target="x86_64-unknown-linux-gnu"

make_mock_binary "${binary}" "no path fixture"
"${REPO_ROOT}/scripts/distribution/build_preview_artifacts.sh" \
  --version "${version}" \
  --output-dir "${artifact_dir}" \
  --binary "${binary}" >/dev/null
"${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
  --version "${version}" \
  --artifact-dir "${artifact_dir}" \
  --out "${installer}" \
  --artifact-base-url "file://${artifact_dir}" >/dev/null

mkdir -p "${home_dir}"
output="$(
  HOME="${home_dir}" \
    SHELL=/bin/zsh \
    PATH="/usr/bin:/bin" \
    SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    SIFR_NO_MODIFY_PATH=1 \
    sh "${installer}" --no-modify-path
)"

if [[ "${output}" == *"configured Sifr PATH"* ]]; then
  echo "installer configured PATH despite opt-out" >&2
  echo "--- output ---" >&2
  echo "${output}" >&2
  exit 1
fi

if [[ -e "${home_dir}/.sifr/env" || -e "${home_dir}/.profile" || -e "${home_dir}/.zshrc" ]]; then
  echo "installer wrote shell profile files despite opt-out" >&2
  find "${home_dir}" -maxdepth 2 -type f -print >&2
  exit 1
fi

if [[ "$("${home_dir}/.sifr/bin/sifr")" != "no path fixture" ]]; then
  echo "installer did not install binary when PATH modification was disabled" >&2
  exit 1
fi
