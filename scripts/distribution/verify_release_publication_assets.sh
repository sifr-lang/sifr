#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo "usage: verify_release_publication_assets.sh --version X.Y.Z-CHANNEL.N --assets DIR --prepare-summary PATH" >&2
  exit 2
}

version=""
assets=""
prepare_summary=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) version="${2:-}"; shift 2 ;;
    --assets) assets="${2:-}"; shift 2 ;;
    --prepare-summary) prepare_summary="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done
[[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-(alpha|beta)\.[0-9]+$ &&
  -d "${assets}" && -f "${prepare_summary}" ]] || usage

for target in \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu \
  aarch64-unknown-linux-gnu
do
  archive="${assets}/sifr-${version}-${target}.tar.gz"
  checksum="${archive}.sha256"
  test -f "${archive}" || {
    echo "release-publication-assets: missing ${archive}" >&2
    exit 2
  }
  test -f "${checksum}" || {
    echo "release-publication-assets: missing ${checksum}" >&2
    exit 2
  }
  expected="$(tr -d '[:space:]' <"${checksum}")"
  actual="$(sha256sum "${archive}" | awk '{print $1}')"
  test "${expected}" = "${actual}" || {
    echo "release-publication-assets: checksum mismatch for ${archive}" >&2
    exit 2
  }
  scripts/distribution/verify_release_archive.py \
    "${archive}" \
    --version "${version}" \
    --target "${target}"
done

scripts/distribution/generate_version_installer.sh \
  --version "${version}" \
  --artifact-dir "${assets}" \
  --out "${assets}/sifr-installer-${version}"
expected_names="$(
  {
    for target in \
      aarch64-apple-darwin \
      x86_64-apple-darwin \
      x86_64-unknown-linux-gnu \
      aarch64-unknown-linux-gnu
    do
      echo "sifr-${version}-${target}.tar.gz"
      echo "sifr-${version}-${target}.tar.gz.sha256"
    done
    echo "sifr-installer-${version}"
  } | LC_ALL=C sort
)"
actual_names="$(
  find "${assets}" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; |
    LC_ALL=C sort
)"
if [[ "${actual_names}" != "${expected_names}" ]]; then
  echo "release-publication-assets: asset set is incomplete or unexpected" >&2
  diff -u <(printf '%s\n' "${expected_names}") \
    <(printf '%s\n' "${actual_names}") || true
  exit 2
fi

publish_asset_digests="$(
  find "${assets}" -mindepth 1 -maxdepth 1 -type f -print0 |
    LC_ALL=C sort -z |
    xargs -0 sha256sum |
    jq -Rn --arg prefix "${assets}/" '
      [
        inputs
        | capture("^(?<sha>[0-9a-f]{64})  (?<path>.+)$")
        | {key:(.path | ltrimstr($prefix)), value:.sha}
      ]
      | from_entries
    '
)"
jq -e \
  --argjson assets "${publish_asset_digests}" \
  '.assets == $assets' "${prepare_summary}" >/dev/null || {
  echo "release-publication-assets: bytes differ from read-only prepare" >&2
  exit 2
}
