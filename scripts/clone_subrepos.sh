#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Format: local_path|remote_url|branch
SUBREPOS=(
  "audits/leetcode|https://github.com/sifr-lang/leetcode.git|main"
)

ensure_clean_subrepo() {
  local path="$1"

  if [[ -n "$(git -C "${path}" status --porcelain)" ]]; then
    echo "Refusing to update dirty sub-repository: ${path}" >&2
    echo "Commit, stash, or discard changes inside ${path} first." >&2
    exit 1
  fi
}

ensure_expected_remote() {
  local path="$1"
  local expected_remote="$2"
  local ssh_remote="$3"
  local actual_remote

  actual_remote="$(git -C "${path}" remote get-url origin)"
  if [[ "${actual_remote}" != "${expected_remote}" && "${actual_remote}" != "${ssh_remote}" ]]; then
    echo "Unexpected origin for ${path}: ${actual_remote}" >&2
    echo "Expected ${expected_remote} or ${ssh_remote}." >&2
    exit 1
  fi
}

clone_or_update() {
  local path="$1"
  local remote="$2"
  local branch="$3"
  local absolute_path="${REPO_ROOT}/${path}"
  local ssh_remote="git@github.com:sifr-lang/${remote##*/}"

  if [[ ! -e "${absolute_path}" ]]; then
    mkdir -p "$(dirname "${absolute_path}")"
    echo "Cloning ${remote} into ${path}"
    git clone --branch "${branch}" "${remote}" "${absolute_path}"
    return
  fi

  if [[ ! -d "${absolute_path}/.git" ]]; then
    local non_generated_file

    non_generated_file="$(
      find "${absolute_path}" -type f \
        ! -name "*.pyc" \
        ! -name ".DS_Store" \
        -print -quit
    )"
    if [[ -z "${non_generated_file}" ]]; then
      rm -rf "${absolute_path}"
      mkdir -p "$(dirname "${absolute_path}")"
      echo "Cloning ${remote} into ${path}"
      git clone --branch "${branch}" "${remote}" "${absolute_path}"
      return
    fi

    echo "${path} exists but is not a git repository." >&2
    exit 1
  fi

  ensure_expected_remote "${absolute_path}" "${remote}" "${ssh_remote}"
  ensure_clean_subrepo "${absolute_path}"

  echo "Updating ${path}"
  git -C "${absolute_path}" fetch origin "${branch}"
  git -C "${absolute_path}" checkout "${branch}"
  git -C "${absolute_path}" pull --ff-only origin "${branch}"
}

for entry in "${SUBREPOS[@]}"; do
  IFS="|" read -r path remote branch <<< "${entry}"
  clone_or_update "${path}" "${remote}" "${branch}"
done
