---
name: talk-to-claude
description: Talk to Claude asynchronously about a specific task by asking it to write its response into a file, then wait for that file to appear.
---

## Start Claude Conversation

Choose a target file under `./reviews`, then run Claude CLI in the background and redirect its output there.

Use an absolute path derived from `./reviews` so the output location is exact.

Authoritative launch command:

```bash
PWD_NOW="$(pwd)"
TARGET_FILE="${PWD_NOW}/reviews/<conversation-file-name>.md"
LOG_FILE="${TARGET_FILE%.md}.claude.log"

nohup claude -p "$(cat <<PROMPT
${TOPIC_NAME}
PROMPT
)" >"${TARGET_FILE}" 2>"${LOG_FILE}" &
```

- Replace `<conversation-file-name>.md` with a concrete file name for the active topic.
- If the target file already exists, use a new filename with the same prefix and an incremented suffix.
- The command prints the background job id. Keep using the target file as the handoff artifact.

## Wait For Output File

Use the wait command below to block until Claude writes the target file:

```bash
PWD_NOW="$(pwd)"
uv run --project /Users/yaseralnajjar/work/talk-to-claude \
  python /Users/yaseralnajjar/work/talk-to-claude/wait_for_review.py "${PWD_NOW}/reviews/<conversation-file-name>.md" \
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
