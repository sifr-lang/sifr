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
build_mock_preview_artifacts "${version}" "${artifact_dir}" "${binary}"
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
zshrc_source_count_before="$(grep -F -c '. "${HOME}/.sifr/env"' "${home_dir}/.zshrc")"

resolved="$(
  HOME="${home_dir}" PATH="/usr/bin:/bin" sh -c '. "${HOME}/.sifr/env"; command -v sifr'
)"
if [[ "${resolved}" != "${home_dir}/.sifr/bin/sifr" ]]; then
  echo "env script did not put installed sifr on PATH: ${resolved}" >&2
  exit 1
fi

if ! grep -F 'path fixture' "${home_dir}/.sifr/bin/sifr" >/dev/null; then
  echo "installed sifr payload did not match fixture" >&2
  exit 1
fi

rm -f "${home_dir}/.sifr/env"
mkdir -p "${home_dir}/.config/fish"
output="$(
  HOME="${home_dir}" \
    SHELL=/bin/zsh \
    PATH="${home_dir}/.sifr/bin:/usr/bin:/bin" \
    SIFR_TARGET="${target}" \
    SIFR_ARTIFACT_BASE_URL="file://${artifact_dir}" \
    sh "${installer}"
)"

if [[ "${output}" == *"configured Sifr PATH via"* ]]; then
  echo "installer reported PATH configuration while repairing an already-on-PATH install" >&2
  echo "--- output ---" >&2
  echo "${output}" >&2
  exit 1
fi

if [[ ! -f "${home_dir}/.sifr/env" ]]; then
  echo "installer did not repair missing managed env script when install dir was already on PATH" >&2
  echo "--- output ---" >&2
  echo "${output}" >&2
  exit 1
fi

if [[ ! -f "${home_dir}/.config/fish/conf.d/sifr.env.fish" ]]; then
  echo "installer did not repair missing fish env script when install dir was already on PATH" >&2
  exit 1
fi

zshrc_source_count_after="$(grep -F -c '. "${HOME}/.sifr/env"' "${home_dir}/.zshrc")"
if [[ "${zshrc_source_count_after}" != "${zshrc_source_count_before}" ]]; then
  echo "installer duplicated zsh profile hook while repairing an already-on-PATH install" >&2
  exit 1
fi

resolved="$(
  HOME="${home_dir}" PATH="/usr/bin:/bin" sh -c '. "${HOME}/.zshrc"; command -v sifr'
)"
if [[ "${resolved}" != "${home_dir}/.sifr/bin/sifr" ]]; then
  echo "repaired zsh profile did not resolve installed sifr: ${resolved}" >&2
  exit 1
fi
