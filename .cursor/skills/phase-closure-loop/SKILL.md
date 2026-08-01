---
name: phase-closure-loop
description: Close one bounded Sifr phase item without scope drift, repeated evidence, or recursive review.
---

# Phase Closure Loop

Use this skill with one phase document under `plans/issues/`.

Treat the phase document as the source of truth.

## Core Rules

- Work on one unfinished item or closure task.
- Keep all changes inside that item's scope.
- Do not modify another owner's dependency or worktree.
- Stop after the item is merged, externally blocked, or needs new scope.
- Start a new session for the next item.

## Execute the Item

1. Read the item, scope, dependencies, and acceptance criteria from the phase document.
2. Verify that this session owns the worktree, branch, Git index, and temporary paths.
3. Verify credentials and recovery steps before an irreversible external action.
4. Implement the item under the rules in `AGENTS.md`.
5. Run targeted tests and the required validation from `AGENTS.md`.

For documentation-only or review-record-only changes, run only relevant documentation checks.

Reuse validation evidence when implementation and validation inputs are unchanged.

Do not repeat a failed performance gate on an unchanged candidate without new evidence.

## Handle Unexpected Failures

- Correct regressions from the current item.
- Correct existing failures only when they are inside the item scope.
- Record out-of-scope failures in their owning issue.
- Do not absorb unrelated failures into the current item.
- If an external failure blocks the item, record it and stop.

## Review

1. Open one draft implementation PR.
2. Use the [talk-to-claude-opus](../talk-to-claude-opus/SKILL.md) skill.
3. Give Claude the review prompt below.

The prompt must include:

- The exact base and candidate SHAs.
- The changed paths.
- The item scope and acceptance criteria.
- Existing validation evidence.
- Prior blocking findings for a remediation review.

Tell Claude not to modify files.

Tell Claude not to create new requirements.

Tell Claude not to repeat broad validation that existing evidence covers.

Require this response:

```text
Verdict: SATISFIED | NOT SATISFIED

Blocking findings:
- regression | in-scope omission: file and line, criterion, reason, correction

Follow-up findings:
- pre-existing issue | infrastructure issue | suggestion: reason
```

Only regressions and in-scope omissions can block approval.

Convert follow-up findings into separate work.

Apply valid blocking findings in one batch.

Repeat review only when code, tests, fixtures, workflows, schemas, or lockfiles change.

If a second review finds a new mechanism-level defect, stop and rescope the item.

If the same finding returns twice, stop and request adjudication.

A timeout, API error, empty response, or incomplete response is not a review pass.

After the initial request fails, retry up to two times with new temporary directories.

If all three requests fail, record the blocker and stop.

Do not create numbered review artifacts for failed requests.

Publish final review evidence outside the reviewed Git tree.

Key the review evidence by candidate SHA.

Do not commit the final review into the commit that it approves.

## Merge

Verify that validation and reviewer approval cover the same final candidate SHA.

If relevant implementation files change, repeat the affected validation and review.

If relevant base code changes, update the base and repeat the affected work.

Do not invalidate evidence for an unrelated base change.

Run the merge gate once on the final implementation candidate.

Do not rerun the merge gate after documentation-only or review-record-only changes.

Merge the PR after validation and reviewer approval succeed.

Update the phase document with:

- The merged PR.
- The final candidate SHA.
- The validation evidence.
- The review evidence.
- Deferred follow-up work.

Do not run another external review for this record-only update.

## End the Session

Record:

- The current state.
- The branch, candidate SHA, and PR.
- The validation and review evidence.
- The blocker, if one exists.
- The exact next action.

Stop after you record this handoff.

## Close the Phase

Close the phase after all items are merged.

Reuse item-level validation and review evidence.

Do not repeat wave, milestone, or phase reviews when implementation files are unchanged.

Run relevant documentation checks for closure-only changes.

If closure changes implementation files, treat it as a new item.
