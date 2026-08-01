---
name: talk-to-claude-opus
description: Send a read-only prompt to Claude Opus and return its response through an atomic temporary file.
---

# Talk to Claude Opus

Run one request with the prompt supplied by the calling workflow.

```bash
CLAUDE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/sifr-claude.XXXXXX")"
RESPONSE_TMP="${CLAUDE_DIR}/response.tmp"
RESPONSE_FILE="${CLAUDE_DIR}/response.md"
LOG_FILE="${CLAUDE_DIR}/claude.log"
TIMEOUT_FILE="${CLAUDE_DIR}/timed-out"

claude \
  --permission-mode plan \
  --setting-sources project \
  --model claude-opus-5 \
  --effort medium \
  --no-session-persistence \
  -p "$(cat <<'PROMPT'
<prompt>
PROMPT
)" >"${RESPONSE_TMP}" 2>"${LOG_FILE}" &
CLAUDE_PID=$!

(
  sleep 2400
  touch "${TIMEOUT_FILE}"
  kill -TERM "${CLAUDE_PID}" 2>/dev/null
) &
WATCHDOG_PID=$!

wait "${CLAUDE_PID}"
CLAUDE_STATUS=$?
kill "${WATCHDOG_PID}" 2>/dev/null || true
wait "${WATCHDOG_PID}" 2>/dev/null || true

if [ -f "${TIMEOUT_FILE}" ]; then
  rm -f "${RESPONSE_TMP}"
  echo "Claude timed out after 40 minutes. See ${LOG_FILE}" >&2
  exit 1
fi

if [ "${CLAUDE_STATUS}" -ne 0 ] || [ ! -s "${RESPONSE_TMP}" ]; then
  rm -f "${RESPONSE_TMP}"
  echo "Claude failed or produced empty output. See ${LOG_FILE}" >&2
  exit 1
fi

mv "${RESPONSE_TMP}" "${RESPONSE_FILE}"
printf 'CLAUDE_RESPONSE_FILE=%s\n' "${RESPONSE_FILE}"
```

The temporary file keeps incomplete output hidden.

The atomic rename makes file existence a completion signal.

The response and log remain outside the Git tree.

Do not poll for the output file.

Use the command completion notification.

Read `CLAUDE_RESPONSE_FILE` after the command succeeds.
