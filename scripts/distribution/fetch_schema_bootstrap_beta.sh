#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF' >&2
usage: fetch_schema_bootstrap_beta.sh \
  --repository OWNER/REPO --version X.Y.Z-beta.N --source-commit COMMIT \
  --index PATH --assets DIR --record PATH
EOF
  exit 2
}

repository=""
version=""
source_commit=""
index=""
assets=""
record=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --version) version="${2:-}"; shift 2 ;;
    --source-commit) source_commit="${2:-}"; shift 2 ;;
    --index) index="${2:-}"; shift 2 ;;
    --assets) assets="${2:-}"; shift 2 ;;
    --record) record="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ "${repository}" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ &&
  "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-beta\.[0-9]+$ &&
  "${source_commit}" =~ ^[0-9a-f]{40}$ &&
  -n "${index}" && -n "${assets}" && -n "${record}" ]] || usage
test -f "${index}" && ! test -L "${index}" || usage
! test -e "${assets}" && ! test -L "${assets}" || usage
! test -e "${record}" && ! test -L "${record}" || usage

scripts/distribution/release_governance.py validate \
  --kind release-index \
  --input "${index}" \
  --require-canonical
test "$(
  gh api "repos/${repository}/git/ref/tags/${version}" --jq '.object.sha'
)" = "${source_commit}" || {
  echo "schema-bootstrap-beta: tag source mismatch" >&2
  exit 2
}

mkdir "${assets}"
gh release download "${version}" --repo "${repository}" --dir "${assets}"
expected_names="$(
  {
    echo "sifr-installer-${version}"
    for target in \
      aarch64-apple-darwin \
      aarch64-unknown-linux-gnu \
      x86_64-apple-darwin \
      x86_64-unknown-linux-gnu
    do
      echo "sifr-${version}-${target}.tar.gz"
      echo "sifr-${version}-${target}.tar.gz.sha256"
    done
  } | LC_ALL=C sort
)"
actual_names="$(
  find "${assets}" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; |
    LC_ALL=C sort
)"
test "${actual_names}" = "${expected_names}" || {
  echo "schema-bootstrap-beta: immutable asset set drifted" >&2
  exit 2
}
for target in \
  aarch64-apple-darwin \
  aarch64-unknown-linux-gnu \
  x86_64-apple-darwin \
  x86_64-unknown-linux-gnu
do
  archive="${assets}/sifr-${version}-${target}.tar.gz"
  checksum="${archive}.sha256"
  test "$(tr -d '[:space:]' <"${checksum}")" = "$(
    sha256sum "${archive}" | awk '{print $1}'
  )" || {
    echo "schema-bootstrap-beta: checksum drifted for ${target}" >&2
    exit 2
  }
done

scripts/distribution/release_governance.py build-release-record \
  --version "${version}" \
  --channel beta \
  --source-commit "${source_commit}" \
  --installer "${assets}/sifr-installer-${version}" \
  --artifact-dir "${assets}" \
  --out "${record}"
jq -e \
  --arg version "${version}" \
  --slurpfile wrapper "${record}" \
  '
    .schema_version == 2
    and .generation == 1
    and .ga_status == "preview"
    and .channels.beta == $version
    and .releases[$version] == $wrapper[0].release
  ' "${index}" >/dev/null || {
  echo "schema-bootstrap-beta: public release disagrees with generation 1" >&2
  exit 2
}
