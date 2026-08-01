---
name: phase-closure-loop
description: Close one bounded Sifr phase item with stable scope, reusable evidence, external review, and explicit terminal states.
---

# Phase Closure Loop

Use this skill to implement or close one phase item.

One session owns one item, wave, PR, or closure task.

Do not use one session to finish a complete multi-item phase.

## Input

- `PHASE_DOC`: Phase document under `plans/issues/`.

If required information is absent, ask for clarification before you continue.

## Terminal States

Every session must end in one of these states:

- `DONE`: The item is merged and its records are complete.
- `BLOCKED_EXTERNAL`: An external requirement prevents progress.
- `PAUSED_FOR_RESOURCES`: A required machine resource is unavailable.
- `NEEDS_RESCOPING`: New work exceeds the approved scope.
- `SUPERSEDED_BY_EXTERNAL_MERGE`: Another actor merged or replaced the candidate.

Record the evidence, owner, resume condition, and next action for each non-`DONE` state.

Stop after you record a terminal state.

## Step 1: Establish Ownership

Verify that this session owns the worktree, branch, Git index, and candidate.

Verify that no other process can switch the branch or create a commit.

Use private target, report, review, and temporary directories.

Do not share these directories with another session.

If another actor owns the item, use `BLOCKED_EXTERNAL`.

If another actor merged the candidate, use `SUPERSEDED_BY_EXTERNAL_MERGE`.

## Step 2: Freeze the Scope

Write the numbered acceptance criteria in `PHASE_DOC`.

Write the non-goals in `PHASE_DOC`.

Record `BASE_SHA`, `ALLOWED_PATHS`, and `ITEM_ID`.

Classify each dependency as:

- `implement`: This item owns the dependency.
- `consume-only`: Another item owns the dependency.
- `wait`: Work cannot start before the dependency is ready.

Do not modify a `consume-only` dependency.

If a new requirement changes the scope, use `NEEDS_RESCOPING`.

Get explicit user approval before you add the requirement.

## Step 3: Run the Preflight

Verify these conditions before you modify files:

- The branch and base commit are correct.
- The worktree has no unrelated changes.
- The worktree has enough disk space for the required gate.
- No competing Sifr gate uses the host.
- Each dependency has one owner.
- Required credentials and permissions are available.
- Required waivers remain valid through the estimated completion date.
- Recovery procedures exist before an irreversible operation.

If a resource is unavailable, use `PAUSED_FOR_RESOURCES`.

If a credential or permission is unavailable, use `BLOCKED_EXTERNAL`.

Do not start an irreversible operation after a failed preflight.

## Step 4: Implement One Item

Create a checklist for the active item.

Implement the root-cause correction.

Do not add a fallback path unless the user requests one.

Do not implement work from a different item.

Run the smallest relevant demos, fixtures, and tests after each change.

Load the `sifr-demo-authoring` skill before you change `demos/`.

Record the candidate SHA after the checklist is complete.

## Step 5: Classify Unexpected Failures

Classify each unexpected failure as:

- A regression from the candidate.
- An existing failure that is in scope.
- A pre-existing failure that is out of scope.
- An infrastructure or resource failure.
- A dependency that another owner controls.

Correct a regression from the candidate.

Correct an existing in-scope failure.

Record an out-of-scope failure and its owning issue.

Do not absorb an out-of-scope failure into this item.

Use a terminal state when an external failure prevents progress.

## Step 6: Run Candidate Validation

Select validation from the changed files.

For compiler, runtime, workflow, schema, or lockfile changes:

1. Run targeted tests.
2. Run `scripts/run_all_tests.sh --profile create-pr`.
3. Record the candidate SHA and validation inputs.

For fixture-only changes, run the affected fixture suites.

For documentation-only changes, run documentation and guardrail checks.

For review-record-only changes, run mechanical documentation checks.

Do not run a full compiler gate for a review-record-only change.

Reuse evidence when implementation and validation inputs are unchanged.

Do not repeat a failed performance gate on an unchanged candidate without new evidence.

Use `PAUSED_FOR_RESOURCES` when host contention invalidates a performance result.

Record successful validation against the candidate SHA.

## Step 7: Run External Review

Open one draft implementation PR for the validated candidate.

Use the [talk-to-claude-opus](../talk-to-claude-opus/SKILL.md) skill.

Ask the reviewer to inspect the exact base and candidate SHAs.

Give the reviewer the acceptance criteria and changed paths.

Tell the reviewer not to modify files.

Tell the reviewer to classify each finding as:

- A blocking regression.
- A blocking in-scope omission.
- A pre-existing issue.
- An infrastructure issue.
- A non-blocking suggestion.

Only blocking regressions and blocking omissions can prevent approval.

Keep the final review evidence outside the reviewed Git tree.

Publish the evidence as a PR review, check, or immutable external artifact.

Key the evidence by the candidate SHA.

Do not commit the final review into the candidate that it approves.

Record reviewer approval against the candidate SHA.

## Step 8: Process Review Findings

Apply all valid blocking findings in one batch.

Run targeted validation after the batch.

Run another external review only when code, tests, fixtures, workflows, schemas, or lockfiles change.

Do not reopen approval for review records or status text.

Convert non-blocking suggestions into follow-up items.

If a second review finds a new mechanism-level defect, use `NEEDS_RESCOPING`.

If the same finding returns twice, stop and request adjudication.

A failed reviewer request is not a review pass.

Retry one reviewer transport failure.

After the second transport failure, use `BLOCKED_EXTERNAL`.

## Step 9: Prepare the PR

Verify that the candidate SHA still matches the validated and reviewed SHA.

Verify that no relevant base change invalidates the evidence.

If relevant base code changed, update the base and repeat the affected steps.

Do not invalidate evidence for an unrelated base change.

Do not open separate PRs for each review record.

## Step 10: Run the Merge Gate

Run `scripts/run_all_tests.sh` once on the final implementation candidate.

Record the command, SHA, and result.

Reuse the result after documentation-only updates.

If the merge gate changes files, repeat the affected validation and review steps.

Do not merge before the final candidate has successful validation and reviewer approval.

## Step 11: Merge and Record the Result

Merge the PR after the final candidate has successful validation and reviewer approval.

Update `PHASE_DOC` with:

- The merged PR link.
- The final candidate SHA.
- The validation evidence.
- The external review evidence.
- Deferred follow-up items.

Update `internal_docs/architecture.md` only when architecture changed.

Update `plans/roadmap.md` only when roadmap status changed.

Do not run another external review for these record-only updates.

Set the session state to `DONE` after all required records are complete.

## Step 12: End the Session

Create a handoff with:

- The terminal state.
- The branch, worktree, base SHA, and candidate SHA.
- The merged or open PR.
- The validation evidence.
- The review evidence.
- Dirty or untracked files.
- External blockers.
- Deferred findings.
- The exact next action.
- Commands that the next session must not repeat.

Stop after the handoff.

Start a new session for the next item or wave.

## Phase Closure

Close the phase only after all items are `DONE`.

Reuse item-level validation and review evidence.

Do not repeat wave, milestone, and phase reviews when code, tests, fixtures, workflows, schemas, and lockfiles are unchanged.

Run mechanical documentation checks for closure-only changes.

If phase closure changes code, tests, fixtures, workflows, schemas, or lockfiles, create a new bounded item.

Use the normal item workflow for that new item.

## Completion Checklist

- The scope and non-goals are recorded.
- One owner controls the worktree and candidate.
- The PR is merged.
- The final candidate passed the required validation.
- External review approved the final candidate.
- Final review evidence is outside the reviewed Git tree.
- The phase document contains the PR and evidence.
- Follow-up items contain all non-blocking suggestions.
- The worktree has no unexplained artifacts.
- The handoff contains the terminal state and next action.

## Prohibited Patterns

- Do not review a commit that only archives its own approval.
- Do not rerun full validation after review-record-only changes.
- Do not absorb unrelated gate failures.
- Do not modify another owner’s worktree or dependency.
- Do not let another process mutate the branch during validation.
- Do not merge before the final candidate has validation and review evidence.
- Do not continue after you record a terminal state.
