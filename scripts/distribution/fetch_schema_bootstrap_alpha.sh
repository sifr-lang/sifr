#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo "usage: fetch_schema_bootstrap_alpha.sh --repository OWNER/REPO --version X.Y.Z-alpha.N --evidence PATH --assets DIR --record PATH" >&2
  exit 2
}

repository=""
version=""
evidence=""
assets=""
record=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repository) repository="${2:-}"; shift 2 ;;
    --version) version="${2:-}"; shift 2 ;;
    --evidence) evidence="${2:-}"; shift 2 ;;
    --assets) assets="${2:-}"; shift 2 ;;
    --record) record="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

[[ -n "${repository}" && -n "${version}" && -n "${evidence}" &&
  -n "${assets}" && -n "${record}" ]] || usage
[[ "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+-alpha\.[0-9]+$ ]] || usage
test -f "${evidence}" || {
  echo "schema-bootstrap-alpha: missing evidence: ${evidence}" >&2
  exit 2
}
scripts/distribution/release_governance.py validate \
  --kind schema-bootstrap-evidence \
  --input "${evidence}" \
  --require-canonical
test "$(
  jq -r '.stage + ":" + .alpha.version' "${evidence}"
)" = "alpha-assets:${version}" || {
  echo "schema-bootstrap-alpha: evidence identity mismatch" >&2
  exit 2
}
alpha_source="$(jq -r '.alpha.source_commit' "${evidence}")"
test "$(
  gh api "repos/${repository}/git/ref/tags/${version}" --jq '.object.sha'
)" = "${alpha_source}" || {
  echo "schema-bootstrap-alpha: tag source mismatch" >&2
  exit 2
}
mkdir -p "${assets}"
gh release download "${version}" --repo "${repository}" --dir "${assets}"
actual_names="$(
  find "${assets}" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; |
    LC_ALL=C sort
)"
expected_names="$(jq -r '.alpha.published_assets | keys[]' "${evidence}" | LC_ALL=C sort)"
test "${actual_names}" = "${expected_names}" || {
  echo "schema-bootstrap-alpha: immutable asset set drifted" >&2
  exit 2
}
while IFS=$'\t' read -r name expected; do
  test "$(sha256sum "${assets}/${name}" | awk '{print $1}')" = "${expected}" || {
    echo "schema-bootstrap-alpha: asset digest drifted: ${name}" >&2
    exit 2
  }
done < <(
  jq -r '.alpha.published_assets | to_entries[] | [.key, .value] | @tsv' \
    "${evidence}"
)
scripts/distribution/release_governance.py build-release-record \
  --version "${version}" \
  --channel alpha \
  --source-commit "${alpha_source}" \
  --installer "${assets}/sifr-installer-${version}" \
  --artifact-dir "${assets}" \
  --out "${record}"
test "$(sha256sum "${record}" | awk '{print $1}')" = "$(
  jq -r '.alpha.release_record_sha256' "${evidence}"
)" || {
  echo "schema-bootstrap-alpha: release record is not reproducible" >&2
  exit 2
}
