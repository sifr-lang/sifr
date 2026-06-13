#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel)"

while IFS= read -r -d '' path; do
  rel="${path#"${REPO_ROOT}/"}"
  if git -C "${REPO_ROOT}" ls-files --error-unmatch -- "${rel}" >/dev/null 2>&1; then
    git -C "${REPO_ROOT}" rm -f -- "${rel}"
  else
    rm -f -- "${path}"
  fi
done < <(
  find \
    "${REPO_ROOT}/issues" \
    "${REPO_ROOT}/issues/archive" \
    "${REPO_ROOT}/plans/reviews/active" \
    "${REPO_ROOT}/plans/reviews/archive" \
    -maxdepth 1 \
    -type f \
    -name '*.claude.log' \
    -print0 | sort -z
)

"${SCRIPT_DIR}/archive_issues.sh"
"${SCRIPT_DIR}/archive_reviews.sh"
