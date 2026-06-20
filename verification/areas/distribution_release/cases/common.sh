#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
DEFAULT_SITE_INSTALL_ROOT="${REPO_ROOT}/target/distribution_release/default-site-install"
SITE_INSTALL_ROOT="${SIFR_SITE_INSTALL_ROOT:-${DEFAULT_SITE_INSTALL_ROOT}}"
DISPATCH_RELEASE_BASE_URL="${SIFR_INSTALLER_RELEASE_BASE_URL:-file://${SITE_INSTALL_ROOT}/github-releases}"
DISPATCH_CHANNEL_METADATA_URL="${SIFR_CHANNEL_METADATA_URL:-file://${SITE_INSTALL_ROOT}/channels.json}"

run_dispatcher() {
  local dispatcher="$1"
  shift
  SIFR_INSTALLER_RELEASE_BASE_URL="${DISPATCH_RELEASE_BASE_URL}" \
    SIFR_CHANNEL_METADATA_URL="${DISPATCH_CHANNEL_METADATA_URL}" \
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
    --install-root "${target_root}" >/dev/null
}

make_mock_version_installers() {
  local target_root="$1"
  mkdir -p "${target_root}/github-releases/0.1.0-alpha.1" "${target_root}/github-releases/0.1.0-beta.1"
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
  chmod 755 \
    "${target_root}/github-releases/0.1.0-alpha.1/sifr-installer-0.1.0-alpha.1" \
    "${target_root}/github-releases/0.1.0-beta.1/sifr-installer-0.1.0-beta.1"
  "${REPO_ROOT}/scripts/distribution/generate_channel_metadata.sh" \
    --out "${target_root}/channels.json" \
    --alpha-version "0.1.0-alpha.1" \
    --beta-version "0.1.0-beta.1" >/dev/null
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
  DISPATCH_RELEASE_BASE_URL="file://${MOCK_DISPATCHER_TMP_DIR}/github-releases"
  DISPATCH_CHANNEL_METADATA_URL="file://${MOCK_DISPATCHER_TMP_DIR}/channels.json"
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
#!/usr/bin/env sh
set -eu
echo "${message}"
EOF
  chmod 755 "${path}"
}

make_target_specific_artifacts() {
  local version="$1"
  local artifact_dir="$2"
  local target
  mkdir -p "${artifact_dir}"
  for target in \
    aarch64-apple-darwin \
    x86_64-apple-darwin \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu
  do
    local tmp_dir archive_path
    tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-target-artifact.XXXXXX")"
    make_mock_binary "${tmp_dir}/sifr" "target=${target}"
    archive_path="${artifact_dir}/sifr-${version}-${target}.tar.gz"
    tar -C "${tmp_dir}" -czf "${archive_path}" sifr
    sha256_fixture_file "${archive_path}" >"${archive_path}.sha256"
    rm -rf "${tmp_dir}"
  done
}

make_self_update_install_root_fixture() {
  local install_root="$1"
  local alpha_version="${2:-0.1.0-alpha.4}"
  local beta_version="${3:-0.1.0-beta.7}"
  "${REPO_ROOT}/scripts/distribution/generate_dispatchers.sh" \
    --install-root "${install_root}" >/dev/null
  "${REPO_ROOT}/scripts/distribution/generate_channel_metadata.sh" \
    --out "${install_root}/channels.json" \
    --alpha-version "${alpha_version}" \
    --beta-version "${beta_version}" >/dev/null
}

make_site_repo_fixture() {
  local target_repo="$1"
  if [[ -z "${SIFR_SITE_INSTALL_ROOT:-}" ]]; then
    rm -rf "${SITE_INSTALL_ROOT}"
    make_self_update_install_root_fixture "${SITE_INSTALL_ROOT}"
  fi
  mkdir -p "${target_repo}/apps/sifr-site/public"
  cp -R "${SITE_INSTALL_ROOT}" "${target_repo}/apps/sifr-site/public/install"
  rm -f "${target_repo}/apps/sifr-site/public/install/metadata/channels.json"
  rmdir "${target_repo}/apps/sifr-site/public/install/metadata" 2>/dev/null || true
}
