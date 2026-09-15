#!/usr/bin/env bash
# Provision the publisher dependencies from the exact candidate editor owner.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
npm_bin="$(bash "${repo_root}/editor_integrations/vscode/scripts/setup-npm.sh" "${RUNNER_TEMP:?}/sifr-npm")"
export PATH="${npm_bin}:${PATH}"
printf '%s\n' "${npm_bin}" >> "${GITHUB_PATH:?}"
python3 "${repo_root}/scripts/check_node_toolchain.py"
npm ci --ignore-scripts --include=dev --prefix "${repo_root}/editor_integrations/vscode"
