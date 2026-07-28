#!/usr/bin/env bash

set -euo pipefail

usage() {
  echo "usage: run_incident_public_recovery.sh --operation rollback|incident-roll-forward --affected-version X.Y.Z --successor-version X.Y.Z --working-root DIR --broken-root DIR --stable-dispatcher PATH --out PATH" >&2
  exit 2
}

operation=""
affected_version=""
successor_version=""
working_root=""
broken_root=""
stable_dispatcher=""
out=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --operation) operation="${2:-}"; shift 2 ;;
    --affected-version) affected_version="${2:-}"; shift 2 ;;
    --successor-version) successor_version="${2:-}"; shift 2 ;;
    --working-root) working_root="${2:-}"; shift 2 ;;
    --broken-root) broken_root="${2:-}"; shift 2 ;;
    --stable-dispatcher) stable_dispatcher="${2:-}"; shift 2 ;;
    --out) out="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done
[[ "${operation}" =~ ^(rollback|incident-roll-forward)$ &&
  "${affected_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ &&
  "${successor_version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ &&
  "${affected_version}" != "${successor_version}" &&
  -x "${working_root}/bin/sifr" &&
  -x "${broken_root}/bin/sifr" &&
  -f "${stable_dispatcher}" && ! -L "${stable_dispatcher}" &&
  ! -e "${out}" && ! -L "${out}" ]] || usage
unset GH_TOKEN SITE_TOKEN VSCE_PAT

run_working() {
  GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  HOME="${working_root}" \
  SIFR_INSTALL_DIR="${working_root}/bin" \
  SIFR_SYSROOT_INSTALL_DIR="${working_root}" \
  SIFR_NO_MODIFY_PATH=1 \
    "${working_root}/bin/sifr" self update --channel stable "$@"
}
run_out_of_band() {
  GH_TOKEN="" SITE_TOKEN="" VSCE_PAT="" \
  HOME="${broken_root}" \
  SIFR_INSTALL_DIR="${broken_root}/bin" \
  SIFR_SYSROOT_INSTALL_DIR="${broken_root}" \
  SIFR_NO_MODIFY_PATH=1 \
    sh "${stable_dispatcher}" "$@"
}

if [[ "${operation}" == "rollback" ]]; then
  if run_working >"${working_root}/without-force.txt" 2>&1; then
    echo "incident-public-recovery: affected downgrade succeeded without --force" >&2
    exit 2
  fi
  grep -F -- "--force" "${working_root}/without-force.txt" >/dev/null || {
    echo "incident-public-recovery: downgrade rejection omitted recovery command" >&2
    exit 2
  }
  run_working --force >"${working_root}/with-force.txt"
  rm "${broken_root}/bin/sifr"
  run_out_of_band --force >"${broken_root}/out-of-band.txt"
else
  run_working >"${working_root}/roll-forward.txt"
  rm "${broken_root}/bin/sifr"
  run_out_of_band >"${broken_root}/out-of-band.txt"
fi

for root in "${working_root}" "${broken_root}"; do
  test "$("${root}/bin/sifr" --version)" = "sifr ${successor_version}" || {
    echo "incident-public-recovery: recovered binary version drifted" >&2
    exit 2
  }
  scripts/distribution/release_governance.py validate \
    --kind install-receipt \
    --input "${root}/install.json"
  jq -e \
    --arg version "${successor_version}" \
    '.version == $version and .channel == "stable"' \
    "${root}/install.json" >/dev/null || {
    echo "incident-public-recovery: recovered receipt version drifted" >&2
    exit 2
  }
done

jq -cnS \
  --arg operation "${operation}" \
  --arg affected "${affected_version}" \
  --arg successor "${successor_version}" \
  '{
    schema_version: 2,
    operation: $operation,
    affected_version: $affected,
    successor_version: $successor,
    working_client: "pass",
    out_of_band: "pass"
  }' >"${out}"
