#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
prompt="${*:-say hi}"
session_id="${CLAUDE_SESSION_ID:-}"
resume_command="${CLAUDE_RESUME_COMMAND:-/desktop}"

if [[ -z "${session_id}" ]]; then
  printf 'CLAUDE_SESSION_ID is required\n' >&2
  printf 'Example: CLAUDE_SESSION_ID="$(uuidgen | tr '\''[:upper:]'\'' '\''[:lower:]'\'')" bash %s "say hi"\n' "${script_dir}/claude_resume_to_desktop.sh" >&2
  exit 1
fi

handoff_log="${CLAUDE_HANDOFF_LOG:-${TMPDIR:-/tmp}/claude-resume-to-desktop-${session_id}.log}"

printf 'Claude session: %s\n' "${session_id}" >&2

claude -p --session-id "${session_id}" "${prompt}"

export CLAUDE_SESSION_ID="${session_id}"
export CLAUDE_RESUME_COMMAND="${resume_command}"

nohup /usr/bin/expect "${script_dir}/claude_resume_to_desktop.expect" >"${handoff_log}" 2>&1 &
handoff_pid=$!

sleep 1

if ! kill -0 "${handoff_pid}" 2>/dev/null; then
  printf 'Desktop handoff failed to start. See log: %s\n' "${handoff_log}" >&2
  exit 1
fi

printf 'Desktop handoff pid: %s\n' "${handoff_pid}" >&2
printf 'Desktop handoff log: %s\n' "${handoff_log}" >&2
