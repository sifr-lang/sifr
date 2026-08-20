#!/usr/bin/env bash

set -euo pipefail

TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
  cat <<'EOF'
Usage: scripts/distribution/build_release_artifacts.sh --version <version> --output-dir <dir> [options]

Build or package toolchain artifacts for every governed release target.

Options:
  --version <version>       Stable or prerelease SemVer, for example 0.1.0 or 0.1.0-beta.1
  --output-dir <dir>        Directory where archives and .sha256 files are written
  --binary <path>           Existing sifr binary to package for all targets; intended for local validation fixtures
  --sysroot-root <dir>      Source sysroot root to package (default: current repository)
  --target <triple>         Package only one release target; can repeat
  --cargo-build             Build target binaries with cargo instead of using --binary
  --help                    Show this help
EOF
}

VERSION=""
OUTPUT_DIR=""
BINARY=""
CARGO_BUILD=0
SYSROOT_ROOT="$(pwd)"
SELECTED_TARGETS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="${2:-}"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="${2:-}"
      shift 2
      ;;
    --binary)
      BINARY="${2:-}"
      shift 2
      ;;
    --sysroot-root)
      SYSROOT_ROOT="${2:-}"
      shift 2
      ;;
    --target)
      SELECTED_TARGETS+=("${2:-}")
      shift 2
      ;;
    --cargo-build)
      CARGO_BUILD=1
      shift
      ;;
    --help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "${VERSION}" || -z "${OUTPUT_DIR}" ]]; then
  echo "--version and --output-dir are required" >&2
  usage >&2
  exit 2
fi

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ &&
      ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$ ]]; then
  echo "version must be stable SemVer or a prerelease using -alpha.N or -beta.N: ${VERSION}" >&2
  exit 2
fi

if [[ -n "${BINARY}" && "${CARGO_BUILD}" -eq 1 ]]; then
  echo "--binary and --cargo-build are mutually exclusive" >&2
  exit 2
fi

if [[ -z "${BINARY}" && "${CARGO_BUILD}" -eq 0 ]]; then
  echo "choose --binary for fixture packaging or --cargo-build for production builds" >&2
  exit 2
fi

if [[ -n "${BINARY}" && ! -f "${BINARY}" ]]; then
  echo "binary not found: ${BINARY}" >&2
  exit 2
fi

if [[ ! -d "${SYSROOT_ROOT}" ]]; then
  echo "sysroot root not found: ${SYSROOT_ROOT}" >&2
  exit 2
fi

if [[ "${#SELECTED_TARGETS[@]}" -eq 0 ]]; then
  SELECTED_TARGETS=("${TARGETS[@]}")
fi

for selected_target in "${SELECTED_TARGETS[@]}"; do
  found=0
  for supported_target in "${TARGETS[@]}"; do
    if [[ "${selected_target}" = "${supported_target}" ]]; then
      found=1
      break
    fi
  done
  if [[ "${found}" -ne 1 ]]; then
    echo "unsupported release target: ${selected_target}" >&2
    exit 2
  fi
done

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
  fi
}

sha256_stream() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | awk '{print $1}'
  else
    shasum -a 256 | awk '{print $1}'
  fi
}

release_rustflags() {
  local flags="${RUSTFLAGS:-}"
  local repo_root
  local sysroot_abs
  local target_dir_abs
  local cargo_home
  local rustup_home
  repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
  repo_root="$(cd "${repo_root}" && pwd)"
  sysroot_abs="$(cd "${SYSROOT_ROOT}" && pwd)"
  flags="${flags} --remap-path-prefix=${repo_root}=."
  if [[ "${sysroot_abs}" != "${repo_root}" ]]; then
    flags="${flags} --remap-path-prefix=${sysroot_abs}=."
  fi
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    target_dir_abs="$(mkdir -p "${CARGO_TARGET_DIR}" && cd "${CARGO_TARGET_DIR}" && pwd)"
    flags="${flags} --remap-path-prefix=${target_dir_abs}=cargo-target"
  fi
  cargo_home="${CARGO_HOME:-${HOME:-}/.cargo}"
  if [[ -n "${cargo_home}" && -d "${cargo_home}" ]]; then
    flags="${flags} --remap-path-prefix=${cargo_home}=cargo-home"
  fi
  rustup_home="${RUSTUP_HOME:-${HOME:-}/.rustup}"
  if [[ -n "${rustup_home}" && -d "${rustup_home}" ]]; then
    flags="${flags} --remap-path-prefix=${rustup_home}=rustup-home"
  fi
  printf '%s' "${flags# }"
}

sysroot_content_sha256() {
  local root="$1"
  (
    cd "${root}"
    find Cargo.toml Cargo.lock .cargo/config.toml crates lib vendor -type f -print |
      LC_ALL=C sort |
      while IFS= read -r path; do
        printf '%s\n' "${path}"
        sha256_file "${path}"
      done
  ) | sha256_stream
}

require_sysroot_asset() {
  local relative="$1"
  if [[ ! -e "${SYSROOT_ROOT}/${relative}" ]]; then
    echo "sysroot root ${SYSROOT_ROOT} is missing ${relative}" >&2
    exit 2
  fi
}

copy_sysroot_dir() {
  local relative="$1"
  local destination="$2"
  require_sysroot_asset "${relative}"
  mkdir -p "$(dirname "${destination}")"
  cp -R "${SYSROOT_ROOT}/${relative}" "${destination}"
}

copy_sysroot_file() {
  local relative="$1"
  local destination="$2"
  require_sysroot_asset "${relative}"
  mkdir -p "$(dirname "${destination}")"
  cp "${SYSROOT_ROOT}/${relative}" "${destination}"
}

write_installed_cargo_config() {
  local root="$1"
  mkdir -p "${root}/.cargo"
  cat >"${root}/.cargo/config.toml" <<'EOF'
[source.crates-io]
replace-with = "sifr-vendor"

[source.sifr-vendor]
directory = "vendor"
EOF
}

write_sysroot_manifest() {
  local root="$1"
  local target="$2"
  local cargo_lock_sha
  local sysroot_sha
  local commit
  cargo_lock_sha="$(sha256_file "${root}/Cargo.lock")"
  sysroot_sha="$(sysroot_content_sha256 "${root}")"
  commit="$(git -C "${SYSROOT_ROOT}" rev-parse --verify HEAD 2>/dev/null || printf '%s' unknown)"
  cat >"${root}/sysroot.toml" <<EOF
"schema-version" = 1
"sifr-version" = "${VERSION}"
"target-triple" = "${target}"
"built-by-compiler-commit" = "${commit}"
"sysroot-content-sha256" = "${sysroot_sha}"
"cargo-lock-sha256" = "${cargo_lock_sha}"
EOF
}

package_toolchain() {
  local target="$1"
  local binary_path="$2"
  local archive_name="sifr-${VERSION}-${target}.tar.gz"
  local archive_path="${OUTPUT_DIR}/${archive_name}"
  local tmp_dir
  local package_root

  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact.XXXXXX")"
  cleanup_package() {
    rm -rf "${tmp_dir}"
  }
  trap cleanup_package RETURN

  package_root="${tmp_dir}/package-root"
  mkdir -p "${package_root}/bin" "${package_root}/crates" "${package_root}/lib/sifr/stdlib"
  cp "${binary_path}" "${package_root}/bin/sifr"
  chmod 755 "${package_root}/bin/sifr"

  copy_sysroot_file "Cargo.toml" "${package_root}/Cargo.toml"
  copy_sysroot_file "Cargo.lock" "${package_root}/Cargo.lock"
  copy_sysroot_dir "crates/sifr_runtime" "${package_root}/crates/sifr_runtime"
  copy_sysroot_dir "crates/sifr_structural_identity" "${package_root}/crates/sifr_structural_identity"
  copy_sysroot_dir "crates/sifr_stdlib" "${package_root}/crates/sifr_stdlib"
  copy_sysroot_dir "stdlib/sifr" "${package_root}/lib/sifr/stdlib/sifr"
  copy_sysroot_dir "stdlib/_sifr" "${package_root}/lib/sifr/stdlib/_sifr"
  copy_sysroot_dir "vendor" "${package_root}/vendor"
  write_installed_cargo_config "${package_root}"
  write_sysroot_manifest "${package_root}" "${target}"

  COPYFILE_DISABLE=1 tar -C "${package_root}" -czf "${archive_path}" \
    bin Cargo.toml Cargo.lock sysroot.toml .cargo vendor crates lib
  python3 "${SCRIPT_DIR}/verify_release_archive.py" \
    "${archive_path}" \
    --version "${VERSION}" \
    --target "${target}"
  sha256_file "${archive_path}" >"${archive_path}.sha256"
  echo "${archive_name}"
}

mkdir -p "${OUTPUT_DIR}"

for target in "${SELECTED_TARGETS[@]}"; do
  if [[ "${CARGO_BUILD}" -eq 1 ]]; then
    RUSTFLAGS="$(release_rustflags)" SIFR_RELEASE_VERSION="${VERSION}" cargo build --locked --release -p sifr --target "${target}"
    binary_path="${CARGO_TARGET_DIR:-target}/${target}/release/sifr"
  else
    binary_path="${BINARY}"
  fi
  package_toolchain "${target}" "${binary_path}"
done
