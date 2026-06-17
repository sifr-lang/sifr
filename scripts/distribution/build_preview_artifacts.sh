#!/usr/bin/env bash

set -euo pipefail

TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
)

usage() {
  cat <<'EOF'
Usage: scripts/distribution/build_preview_artifacts.sh --version <preview> --output-dir <dir> [options]

Build or package preview artifacts for every preview release target.

Options:
  --version <preview>       Semver prerelease version, for example 0.1.0-beta.1
  --output-dir <dir>        Directory where archives and .sha256 files are written
  --binary <path>           Existing sifr binary to package for all targets; intended for local validation fixtures
  --cargo-build             Build target binaries with cargo instead of using --binary
  --help                    Show this help
EOF
}

VERSION=""
OUTPUT_DIR=""
BINARY=""
CARGO_BUILD=0

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

if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta|rc)\.[0-9]+$ ]]; then
  echo "version must be a semver prerelease using -alpha.N, -beta.N, or -rc.N: ${VERSION}" >&2
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

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  else
    shasum -a 256 "${path}" | awk '{print $1}'
  fi
}

package_binary() {
  local target="$1"
  local binary_path="$2"
  local archive_name="sifr-${VERSION}-${target}.tar.gz"
  local archive_path="${OUTPUT_DIR}/${archive_name}"
  local tmp_dir

  tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/sifr-artifact.XXXXXX")"
  cleanup_package() {
    rm -rf "${tmp_dir}"
  }
  trap cleanup_package RETURN

  cp "${binary_path}" "${tmp_dir}/sifr"
  chmod 755 "${tmp_dir}/sifr"
  tar -C "${tmp_dir}" -czf "${archive_path}" sifr
  sha256_file "${archive_path}" >"${archive_path}.sha256"
  echo "${archive_name}"
}

mkdir -p "${OUTPUT_DIR}"

for target in "${TARGETS[@]}"; do
  if [[ "${CARGO_BUILD}" -eq 1 ]]; then
    SIFR_RELEASE_VERSION="${VERSION}" cargo build --release -p sifr --target "${target}"
    binary_path="target/${target}/release/sifr"
  else
    binary_path="${BINARY}"
  fi
  package_binary "${target}" "${binary_path}"
done
