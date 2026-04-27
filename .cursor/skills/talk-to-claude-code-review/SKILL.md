---
name: talk-to-claude-code-review
description: Talk to Claude Code asynchronously for review or second-opinion workflows by sending a prompt with send_to_claude_code.py, asking Claude to write its response into reviews/, then waiting for the review file.
---

# Talk To Claude Code Review

Use this skill when the user asks to run the Claude review flow through Claude Code, especially for code reviews, second opinions, or validation notes that should be saved as a file in `reviews/`.

## Output Location

Choose a target Markdown file under `./reviews`.

Use an absolute path in the prompt so Claude has an exact output location:

```bash
PWD_NOW="$(pwd)"
mkdir -p "${PWD_NOW}/reviews"
TARGET_FILE="${PWD_NOW}/reviews/<review-file-name>.md"
```

- Replace `<review-file-name>.md` with a concrete filename for the active topic.
- If the target file already exists, use a new filename with the same prefix and an incremented suffix.

## Start Claude Code

Authoritative launch command:

```bash
uv run python /Users/yaseralnajjar/work/talk-to-claude/send_to_claude_code.py "review-prompt"
```

For real use, replace `review-prompt` with a prompt that includes:

- the task Claude should review
- the exact `TARGET_FILE` path under `reviews/`
- a requirement that Claude writes the complete response to that file
- any constraints, validation commands, or changed files Claude should inspect

Example:

```bash
PWD_NOW="$(pwd)"
mkdir -p "${PWD_NOW}/reviews"
TARGET_FILE="${PWD_NOW}/reviews/<review-file-name>.md"

uv run python /Users/yaseralnajjar/work/talk-to-claude/send_to_claude_code.py "$(cat <<PROMPT
Review the current uncommitted changes in this repository.

Focus on correctness bugs, behavioral regressions, missing tests, and whether the implementation matches the requested scope.

Write the complete review to:
${TARGET_FILE}

Do not modify files.
PROMPT
)"
```

## Wait For Output File

Use the wait helper to block until Claude writes the target file:

```bash
uv run --project /Users/yaseralnajjar/work/talk-to-claude \
  python /Users/yaseralnajjar/work/talk-to-claude/wait_for_review.py "${TARGET_FILE}" \
  --timeout-seconds 2400 \
  --poll-seconds 10
```

- Max wait: 40 minutes.
- If the file is still unavailable at timeout, report the blocker and do not invent Claude's response.

## Sub-Agent Pattern

If the user explicitly asks to use a sub-agent for this flow, spawn it with model `gpt-5.4-mini` and give it the same command-based workflow:

1. Tell the sub-agent the repository path and the target file under `reviews/`.
2. Tell it to run:
   ```bash
   uv run python /Users/yaseralnajjar/work/talk-to-claude/send_to_claude_code.py "review-prompt"
   ```
   with the real review prompt substituted for `review-prompt`.
3. Tell it to wait for the target file with `wait_for_review.py`.
4. Tell it to report the review file path and summarize actionable findings.

Use `gpt-5.4-mini` for this Claude Code review sub-agent pattern unless the user explicitly overrides it.

## After Claude Responds

Read the review file, separate actionable findings from non-blocking suggestions, and act on the findings according to engineering judgment. If a recommendation is not worth taking, explain why briefly.
