#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact-path.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.10"
binary="${tmp_dir}/sifr"
artifact_dir="${tmp_dir}/artifacts"
installer="${tmp_dir}/installer.sh"
home_dir="${tmp_dir}/home"
target="x86_64-unknown-linux-gnu"

make_mock_binary "${binary}" "path fixture"
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
    sh "${installer}"
)"

if [[ "${output}" != *"configured Sifr PATH via ${home_dir}/.sifr/env"* ]]; then
  echo "installer did not report PATH configuration" >&2
  echo "--- output ---" >&2
  echo "${output}" >&2
  exit 1
fi

grep -F '. "${HOME}/.sifr/env"' "${home_dir}/.profile" >/dev/null
grep -F '. "${HOME}/.sifr/env"' "${home_dir}/.zshrc" >/dev/null

resolved="$(
  HOME="${home_dir}" PATH="/usr/bin:/bin" sh -c '. "${HOME}/.sifr/env"; command -v sifr'
)"
if [[ "${resolved}" != "${home_dir}/.sifr/bin/sifr" ]]; then
  echo "env script did not put installed sifr on PATH: ${resolved}" >&2
  exit 1
fi

output="$(
  HOME="${home_dir}" PATH="/usr/bin:/bin" sh -c '. "${HOME}/.sifr/env"; sifr'
)"
if [[ "${output}" != "path fixture" ]]; then
  echo "installed sifr was not runnable through configured PATH: ${output}" >&2
  exit 1
fi
