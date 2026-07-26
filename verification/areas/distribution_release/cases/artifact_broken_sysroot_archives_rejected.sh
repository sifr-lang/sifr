#!/usr/bin/env bash

set -euo pipefail

source "$(dirname "$0")/common.sh"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-broken-sysroot-archives.XXXXXX")"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT HUP INT TERM

version="0.1.0-beta.14"
target="x86_64-unknown-linux-gnu"

make_broken_archive() {
  local label="$1"
  local missing_path="$2"
  local root="${tmp_dir}/${label}/root"
  local archive_dir="${tmp_dir}/${label}/artifacts"
  local binary="${tmp_dir}/${label}/sifr"
  local archive="${archive_dir}/sifr-${version}-${target}.tar.gz"

  mkdir -p "${archive_dir}" "${root}"
  make_mock_binary "${binary}" "broken ${label}"
  make_mock_sysroot_root "${root}"
  "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
    --version "${version}" \
    --output-dir "${archive_dir}" \
    --binary "${binary}" \
    --sysroot-root "${root}" >/dev/null

  mkdir -p "${tmp_dir}/${label}/extract"
  tar -xzf "${archive}" -C "${tmp_dir}/${label}/extract"
  rm -rf "${tmp_dir}/${label}/extract/${missing_path}"
  local entries=()
  local entry
  for entry in bin Cargo.toml Cargo.lock sysroot.toml .cargo vendor crates lib; do
    if [[ -e "${tmp_dir}/${label}/extract/${entry}" ]]; then
      entries+=("${entry}")
    fi
  done
  COPYFILE_DISABLE=1 tar -C "${tmp_dir}/${label}/extract" -czf "${archive}" "${entries[@]}"
  sha256_fixture_file "${archive}" >"${archive}.sha256"
  printf '%s\n' "${archive_dir}"
}

make_bad_digest_archive() {
  local label="$1"
  local replacement="$2"
  local root="${tmp_dir}/${label}/root"
  local archive_dir="${tmp_dir}/${label}/artifacts"
  local binary="${tmp_dir}/${label}/sifr"
  local archive="${archive_dir}/sifr-${version}-${target}.tar.gz"

  mkdir -p "${archive_dir}" "${root}"
  make_mock_binary "${binary}" "broken ${label}"
  make_mock_sysroot_root "${root}"
  "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
    --version "${version}" \
    --output-dir "${archive_dir}" \
    --binary "${binary}" \
    --sysroot-root "${root}" >/dev/null

  mkdir -p "${tmp_dir}/${label}/extract"
  tar -xzf "${archive}" -C "${tmp_dir}/${label}/extract"
  python3 - "${tmp_dir}/${label}/extract/sysroot.toml" "${replacement}" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
replacement = sys.argv[2]
source = path.read_text()
path.write_text(re.sub(r'"sysroot-content-sha256" = "[0-9a-f]{64}"', f'"sysroot-content-sha256" = "{replacement}"', source))
PY
  COPYFILE_DISABLE=1 tar -C "${tmp_dir}/${label}/extract" -czf "${archive}" \
    bin Cargo.toml Cargo.lock sysroot.toml .cargo vendor crates lib
  sha256_fixture_file "${archive}" >"${archive}.sha256"
  printf '%s\n' "${archive_dir}"
}

for spec in \
  "manifest:sysroot.toml:missing required archive file: sysroot.toml" \
  "runtime:crates/sifr_runtime:missing required archive file: crates/sifr_runtime/Cargo.toml" \
  "stdlib:lib/sifr/stdlib/sifr:missing required archive directory: lib/sifr/stdlib/sifr" \
  "vendor:vendor:missing required archive directory: vendor" \
  "cargo_config:.cargo/config.toml:missing required archive file: .cargo/config.toml"
do
  IFS=: read -r label missing_path expected <<<"${spec}"
  artifact_dir="$(make_broken_archive "${label}" "${missing_path}")"
  require_failure_contains \
    "${expected}" \
    "${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
      --version "${version}" \
      --artifact-dir "${artifact_dir}" \
      --out "${tmp_dir}/${label}/installer.sh"
done

for spec in \
  "zero_digest:0000000000000000000000000000000000000000000000000000000000000000:must not be the zero placeholder" \
  "mismatched_digest:1111111111111111111111111111111111111111111111111111111111111111:sysroot-content-sha256 mismatch"
do
  IFS=: read -r label replacement expected <<<"${spec}"
  artifact_dir="$(make_bad_digest_archive "${label}" "${replacement}")"
  require_failure_contains \
    "${expected}" \
    "${REPO_ROOT}/scripts/distribution/generate_version_installer.sh" \
      --version "${version}" \
      --artifact-dir "${artifact_dir}" \
      --out "${tmp_dir}/${label}/installer.sh"
done
