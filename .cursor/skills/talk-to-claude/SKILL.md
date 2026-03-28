---
name: talk-to-another-agent
description: Talk to another agent about a specific topic.
---

# Talk to Another Agent

Use this skill when you need to talk to another agent.

## Talk to Another Agent Command

```bash
PWD_NOW="$(pwd)"; uv run --project /Users/yaseralnajjar/work/talk-to-claude \
  python /Users/yaseralnajjar/work/talk-to-claude/send_to_claude_code.py "$(cat <<PROMPT
${TOPIC_NAME}
Write the output into ${PWD_NOW}/tmp/<conversation-file-name>.md
PROMPT
)"
```

Adjust prompt scope for the active topic.

If the target conversation file already exists, create a new filename with the same prefix and incremented suffix.

## Wait for Conversation Output Command (authoritative)

```bash
PWD_NOW="$(pwd)"; uv run --project /Users/yaseralnajjar/work/talk-to-claude \
  python /Users/yaseralnajjar/work/talk-to-claude/wait_for_review.py "${PWD_NOW}/tmp/<conversation-file-name>.md" \
  --timeout-seconds 2400 \
  --poll-seconds 10
```

- Max wait: 40 minutes.
- If file is still unavailable at timeout, stop and report blocker state.
