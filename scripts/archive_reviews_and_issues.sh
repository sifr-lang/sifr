#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel)"

find_roots=(
  "${REPO_ROOT}/plans/issues/active"
  "${REPO_ROOT}/plans/issues/completed"
  "${REPO_ROOT}/plans/issues/archive"
  "${REPO_ROOT}/plans/reviews/active"
  "${REPO_ROOT}/plans/reviews/archive"
)
existing_roots=()
for root in "${find_roots[@]}"; do
  if [[ -d "${root}" ]]; then
    existing_roots+=("${root}")
  fi
done

if [[ "${#existing_roots[@]}" -gt 0 ]]; then
  while IFS= read -r -d '' path; do
    rel="${path#"${REPO_ROOT}/"}"
    if git -C "${REPO_ROOT}" ls-files --error-unmatch -- "${rel}" >/dev/null 2>&1; then
      git -C "${REPO_ROOT}" rm -f -- "${rel}"
    else
      rm -f -- "${path}"
    fi
  done < <(
    find "${existing_roots[@]}" \
      -maxdepth 1 \
      -type f \
      -name '*.claude.log' \
      -print0 | sort -z
  )
fi

"${SCRIPT_DIR}/archive_issues.sh"
"${SCRIPT_DIR}/archive_reviews.sh"
