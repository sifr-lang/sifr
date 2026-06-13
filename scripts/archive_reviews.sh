#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel)"
SOURCE_DIR="${REPO_ROOT}/plans/reviews/active"
ARCHIVE_DIR="${REPO_ROOT}/plans/reviews/archive"

if [[ ! -d "${SOURCE_DIR}" ]]; then
  echo "missing reviews directory: ${SOURCE_DIR}" >&2
  exit 1
fi

mkdir -p "${ARCHIVE_DIR}"

while IFS= read -r -d '' path; do
  name="$(basename "${path}")"
  dest="${ARCHIVE_DIR}/${name}"
  if [[ -e "${dest}" ]]; then
    echo "archive destination already exists: ${dest}" >&2
    exit 1
  fi
  git -C "${REPO_ROOT}" mv "plans/reviews/active/${name}" "plans/reviews/archive/${name}"
done < <(find "${SOURCE_DIR}" -maxdepth 1 -type f -print0 | sort -z)
