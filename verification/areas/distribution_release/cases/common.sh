#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
DEFAULT_SITE_INSTALL_ROOT="${REPO_ROOT}/target/distribution_release/default-site-install"
SITE_INSTALL_ROOT="${SIFR_SITE_INSTALL_ROOT:-${DEFAULT_SITE_INSTALL_ROOT}}"
run_dispatcher() {
  local dispatcher="$1"
  shift
  SIFR_DISPATCH_TRACE=1 \
    sh "${SITE_INSTALL_ROOT}/${dispatcher}" "$@"
}

require_success_contains() {
  local expected="$1"
  shift
  local output
  output="$("$@" 2>&1)"
  if [[ "${output}" != *"${expected}"* ]]; then
    echo "expected output to contain: ${expected}" >&2
    echo "--- output ---" >&2
    echo "${output}" >&2
    exit 1
  fi
}

require_failure_contains() {
  local expected="$1"
  shift
  local output
  set +e
  output="$("$@" 2>&1)"
  local status=$?
  set -e
  if [[ ${status} -eq 0 ]]; then
    echo "expected command to fail" >&2
    echo "--- output ---" >&2
    echo "${output}" >&2
    exit 1
  fi
  if [[ "${output}" != *"${expected}"* ]]; then
    echo "expected failure output to contain: ${expected}" >&2
    echo "--- output ---" >&2
    echo "${output}" >&2
    exit 1
  fi
}

make_dispatcher_fixture() {
  local target_root="$1"
  mkdir -p "${target_root}"
  "${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
    --install-root "${target_root}" \
    --installer-release-base-url "file://${target_root}/github-releases" \
    --channel-metadata-url "file://${target_root}/channels.json" >/dev/null
}

generate_channel_metadata_fixture() {
  local out="$1"
  local alpha_version="$2"
  local beta_version="$3"
  local stable_version="${4:-}"
  local record_dir
  record_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-release-records.XXXXXX")"
  python3 - "${record_dir}" "${alpha_version}" "${beta_version}" "$(dirname "${out}")" "${stable_version}" <<'PY'
import hashlib
import json
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
fixture_root = pathlib.Path(sys.argv[4])
targets = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
)
versions = [("alpha", sys.argv[2]), ("beta", sys.argv[3])]
if sys.argv[5]:
    versions.append(("stable", sys.argv[5]))
for channel, version in versions:
    installer = fixture_root / "github-releases" / version / f"sifr-installer-{version}"
    installer_sha256 = (
        hashlib.sha256(installer.read_bytes()).hexdigest()
        if installer.is_file()
        else "d" * 64
    )
    release = {
        "channel": channel,
        "status": "active",
        "source_commit": "e" * 40,
        "installer_sha256": installer_sha256,
        "targets": {
            target: {
                "artifact_sha256": "b" * 64,
                "sysroot_content_sha256": "c" * 64,
            }
            for target in targets
        },
    }
    (root / f"{channel}.json").write_text(
        json.dumps({"version": version, "release": release}),
        encoding="utf-8",
    )
PY
  local metadata_args=(
    --out "${out}" \
    --generation 1 \
    --ga-status "$([[ -n "${stable_version}" ]] && echo active || echo preview)" \
    --alpha-release "${record_dir}/alpha.json" \
    --beta-release "${record_dir}/beta.json"
  )
  if [[ -n "${stable_version}" ]]; then
    metadata_args+=(--stable-release "${record_dir}/stable.json")
  fi
  "${REPO_ROOT}/scripts/distribution/generate_channel_metadata.sh" \
    "${metadata_args[@]}" >/dev/null
  rm -rf "${record_dir}"
}

make_release_index_fixture() {
  local out="$1"
  generate_channel_metadata_fixture \
    "${out}" \
    "0.1.0-alpha.1" \
    "0.1.0-beta.1"
}

make_mock_version_installers() {
  local target_root="$1"
  mkdir -p \
    "${target_root}/github-releases/0.1.0-alpha.1" \
    "${target_root}/github-releases/0.1.0-beta.1" \
    "${target_root}/github-releases/0.1.0"
  cat >"${target_root}/github-releases/0.1.0-alpha.1/sifr-installer-0.1.0-alpha.1" <<'EOF'
#!/usr/bin/env sh
set -eu
echo "sifr mock generated installer version=0.1.0-alpha.1"
EOF
  cat >"${target_root}/github-releases/0.1.0-beta.1/sifr-installer-0.1.0-beta.1" <<'EOF'
#!/usr/bin/env sh
set -eu
echo "sifr mock generated installer version=0.1.0-beta.1"
EOF
  cat >"${target_root}/github-releases/0.1.0/sifr-installer-0.1.0" <<'EOF'
#!/usr/bin/env sh
set -eu
echo "sifr mock generated installer version=0.1.0"
EOF
  chmod 755 \
    "${target_root}/github-releases/0.1.0-alpha.1/sifr-installer-0.1.0-alpha.1" \
    "${target_root}/github-releases/0.1.0-beta.1/sifr-installer-0.1.0-beta.1" \
    "${target_root}/github-releases/0.1.0/sifr-installer-0.1.0"
  generate_channel_metadata_fixture \
    "${target_root}/channels.json" \
    "0.1.0-alpha.1" \
    "0.1.0-beta.1" \
    "0.1.0"
}

use_mock_dispatcher_fixture() {
  MOCK_DISPATCHER_TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sifr-dispatcher-fixture.XXXXXX")"
  cleanup_mock_dispatcher_fixture() {
    rm -rf "${MOCK_DISPATCHER_TMP_DIR}"
  }
  trap cleanup_mock_dispatcher_fixture EXIT HUP INT TERM
  make_dispatcher_fixture "${MOCK_DISPATCHER_TMP_DIR}"
  make_mock_version_installers "${MOCK_DISPATCHER_TMP_DIR}"
  SITE_INSTALL_ROOT="${MOCK_DISPATCHER_TMP_DIR}"
}

sha256_fixture_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
  fi
}

make_mock_binary() {
  local path="$1"
  local message="$2"
  cat >"${path}" <<EOF
#!/bin/sh
set -eu
printf '%s\n' "${message}"
exit 0
EOF
  chmod 755 "${path}"
}

make_mock_sysroot_root() {
  local root="$1"
  mkdir -p \
    "${root}/.cargo" \
    "${root}/crates/sifr_runtime/src" \
    "${root}/crates/sifr_structural_identity/src" \
    "${root}/crates/sifr_stdlib/src" \
    "${root}/stdlib/sifr" \
    "${root}/stdlib/_sifr" \
    "${root}/vendor/mock-crate"
  cat >"${root}/Cargo.toml" <<'EOF'
[workspace]
members = [
  "crates/sifr_runtime",
  "crates/sifr_structural_identity",
  "crates/sifr_stdlib",
]
resolver = "2"
EOF
  cat >"${root}/Cargo.lock" <<'EOF'
# This file is automatically @generated by Cargo.
version = 4
EOF
  cat >"${root}/sysroot.toml" <<'EOF'
"schema-version" = 1
"sifr-version" = "0.0.0-fixture"
"target-triple" = "fixture"
"built-by-compiler-commit" = "fixture"
"sysroot-content-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"cargo-lock-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
EOF
  cat >"${root}/.cargo/config.toml" <<'EOF'
# fixture source config; artifact packaging writes the installed vendor config
EOF
  cat >"${root}/crates/sifr_runtime/Cargo.toml" <<'EOF'
[package]
name = "sifr_runtime"
version = "0.0.0-fixture"
edition = "2021"
EOF
  printf '%s\n' 'pub fn fixture() {}' >"${root}/crates/sifr_runtime/src/lib.rs"
  cat >"${root}/crates/sifr_structural_identity/Cargo.toml" <<'EOF'
[package]
name = "sifr_structural_identity"
version = "0.0.0-fixture"
edition = "2021"
EOF
  printf '%s\n' 'pub fn fixture() {}' >"${root}/crates/sifr_structural_identity/src/lib.rs"
  cat >"${root}/crates/sifr_stdlib/Cargo.toml" <<'EOF'
[package]
name = "sifr_stdlib"
version = "0.0.0-fixture"
edition = "2021"
EOF
  printf '%s\n' 'pub fn fixture() {}' >"${root}/crates/sifr_stdlib/src/lib.rs"
  printf '%s\n' '# fixture public stdlib' >"${root}/stdlib/sifr/__init__.sifr"
  printf '%s\n' '# fixture private stdlib' >"${root}/stdlib/_sifr/runtime.sifr"
  printf '%s\n' '{"files":{}}' >"${root}/vendor/mock-crate/.cargo-checksum.json"
}

make_target_specific_artifacts() {
  local version="$1"
  local artifact_dir="$2"
  local target
  local sysroot_root
  mkdir -p "${artifact_dir}"
  sysroot_root="${artifact_dir}/mock-sysroot"
  make_mock_sysroot_root "${sysroot_root}"
  for target in \
    aarch64-apple-darwin \
    x86_64-apple-darwin \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu
  do
    local tmp_dir
    tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-target-artifact.XXXXXX")"
    make_mock_binary "${tmp_dir}/sifr" "target=${target}"
    "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
      --version "${version}" \
      --output-dir "${artifact_dir}" \
      --binary "${tmp_dir}/sifr" \
      --sysroot-root "${sysroot_root}" \
      --target "${target}" >/dev/null
    rm -rf "${tmp_dir}"
  done
  rm -rf "${sysroot_root}"
}

build_mock_preview_artifacts() {
  local version="$1"
  local artifact_dir="$2"
  local binary="$3"
  shift 3
  local sysroot_root
  local args=()
  sysroot_root="$(mktemp -d "${TMPDIR:-/tmp}/sifr-mock-sysroot.XXXXXX")"
  make_mock_sysroot_root "${sysroot_root}"
  while [[ $# -gt 0 ]]; do
    args+=("$1")
    shift
  done
  "${REPO_ROOT}/scripts/distribution/build_release_artifacts.sh" \
    --version "${version}" \
    --output-dir "${artifact_dir}" \
    --binary "${binary}" \
    --sysroot-root "${sysroot_root}" \
    "${args[@]}" >/dev/null
  rm -rf "${sysroot_root}"
}

make_self_update_install_root_fixture() {
  local install_root="$1"
  local alpha_version="${2:-0.1.0-alpha.4}"
  local beta_version="${3:-0.1.0-beta.7}"
  local default_channel="${4:-stable}"
  "${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
    --install-root "${install_root}" \
    --default-channel "${default_channel}" >/dev/null
  generate_channel_metadata_fixture \
    "${install_root}/channels.json" \
    "${alpha_version}" \
    "${beta_version}"
}

make_site_repo_fixture() {
  local target_repo="$1"
  local default_channel="${2:-beta}"
  local install_root="${target_repo}/apps/sifr-site/public/install"
  mkdir -p "${target_repo}/apps/sifr-site/public"
  "${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
    --install-root "${install_root}" \
    --default-channel "${default_channel}" >/dev/null
  if [[ "${default_channel}" == "beta" ]]; then
    rm "${install_root}/stable"
  fi
  git -C "${target_repo}" init -q
  git -C "${target_repo}" config user.name "Sifr Test"
  git -C "${target_repo}" config user.email "test@sifr.invalid"
  git -C "${target_repo}" add apps/sifr-site/public/install
  git -C "${target_repo}" commit -qm "fixture: add production dispatchers"
}
