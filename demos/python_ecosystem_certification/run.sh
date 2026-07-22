#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT="${REPO_ROOT}/target/verification/areas/python_interop/ecosystem-certification-demo.json"

cd "${REPO_ROOT}"
uv run --project verification/areas/python_interop --locked python \
  verification/areas/python_interop/runner.py \
  --suite callback-examples \
  --suite buffer-examples \
  --suite arrow-examples \
  --suite dlpack-examples \
  --suite async-declaration-examples \
  --result-json "${REPORT}"

jq -er '
  .compiled_certification
  | select(
      .status == "complete"
      and .summary.passing == 7
      and .summary.compiled_evidence == 10
      and .summary.resource_zero_evidence == 4
    )
  | "Python ecosystem certification: status=\(.status):capabilities=\(.summary.passing):evidence=\(.summary.compiled_evidence):resources-zero=\(.summary.resource_zero_evidence)"
' "${REPORT}"
