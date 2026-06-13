---
name: talk-to-claude
description: Talk to Claude asynchronously about a specific task by asking it to write its response into a file, then wait for that file to appear.
---

## Start Claude Conversation

Choose a target file under `./reviews`, then run Claude CLI in the background and redirect its output there.

Use an absolute path derived from `./reviews` so the output location is exact.

If you need the wait helper, set `TALK_TO_CLAUDE_PROJECT` to the local
`talk-to-claude` checkout. Do not hard-code a personal absolute path.

Authoritative launch command:

```bash
PWD_NOW="$(pwd)"
mkdir -p "${PWD_NOW}/reviews"
TARGET_FILE="${PWD_NOW}/reviews/<conversation-file-name>.md"
LOG_FILE="${TARGET_FILE%.md}.claude.log"

nohup claude --dangerously-skip-permissions --setting-sources project --model claude-opus-4-7 --effort xhigh -p "$(cat <<PROMPT
${TOPIC_NAME}
PROMPT
)" >"${TARGET_FILE}" 2>"${LOG_FILE}" &
CLAUDE_PID=$!
wait "${CLAUDE_PID}"
CLAUDE_EXIT_CODE=$?
if [ "${CLAUDE_EXIT_CODE}" -ne 0 ] || [ ! -s "${TARGET_FILE}" ]; then
  echo "Claude reviewer failed or produced an empty output. See ${LOG_FILE}" >&2
  exit 1
fi
```

- Replace `<conversation-file-name>.md` with a concrete file name for the active topic.
- If the target file already exists, use a new filename with the same prefix and an incremented suffix.
- Keep the `wait "${CLAUDE_PID}"` in the same shell invocation; detached `nohup ... &` can leave empty artifacts in Codex exec sessions.
- Keep using the target file as the handoff artifact.
- Include the task, constraints, validation commands, and changed files Claude should inspect in the prompt.
- For review prompts, tell Claude not to modify files.

## Wait For Output File

Use the wait command below to block until Claude writes the target file:

```bash
PWD_NOW="$(pwd)"
: "${TALK_TO_CLAUDE_PROJECT:?Set TALK_TO_CLAUDE_PROJECT to the talk-to-claude checkout path}"
uv run --project "${TALK_TO_CLAUDE_PROJECT}" \
  python "${TALK_TO_CLAUDE_PROJECT}/wait_for_review.py" "${PWD_NOW}/reviews/<conversation-file-name>.md" \
  --timeout-seconds 2400 \
  --poll-seconds 10
```

- Max wait: 40 minutes.
- If the file is still unavailable at timeout, stop and report blocker state.

## Debugging

If Claude does not produce the file, inspect the per-conversation log:

```bash
PWD_NOW="$(pwd)"
sed -n '1,200p' "${PWD_NOW}/reviews/<conversation-file-name>.claude.log"
```

## After Claude Responds

Read the review file, separate actionable findings from non-blocking
suggestions, and act on the findings according to engineering judgment. If a
recommendation is not worth taking, explain why briefly.
