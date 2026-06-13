---
name: phase-closure-loop
description: End-to-end execution loop for ad-hoc phases in Sifr: per-wave implementation, validation, PR/merge, external review pass-1/pass-2, then wave/milestone/phase closure cycles with the same review loop, notifications, and final closure bookkeeping.
---

# Phase Closure Loop

Use this skill when closing an ad-hoc phase using the enforced review-driven loop.

## Inputs

- `PHASE_NAME`: human-readable phase slug (for prompts and status text)
- `PHASE_DOC`: phase doc path under `plans/issues/`
- `PHASE_EXEC_DOC`: execution ledger path under `plans/issues/`
- `REVIEW_PREFIX`: review file prefix, for example:
  - `phase-ad-hoc-ownership-aware-collection-lowering-and-clone-elision`

## Core Rules

- Work one part/wave at a time; do not skip ahead.
- Before each PR, confirm demo/fixture behavior and run `$(pwd)/scripts/run_all_tests.sh`.
- For each review pass, apply only valid findings, validate again, then PR+merge.
- Do not self-review when external reviewer app is required.
- Keep docs in sync (`plans/issues/`, `internal_docs/architecture.md`, `plans/roadmap.md`) as closure progresses.

## Wave Loop (for each wave/part)

1. Plan and checklist for the active wave.
2. Implement root-cause fix (no shim/fallback shortcuts).
3. Run targeted demos/fixtures for the wave.
4. Run full validation:
   - `$(pwd)/scripts/run_all_tests.sh`
5. Open PR and merge for implementation.
6. Trigger external review pass 1, wait for file, apply valid notes, revalidate, PR+merge.
7. Run:
   - `say "First review is done"`
8. Trigger external review pass 2 (production-grade), wait for file, apply valid notes, revalidate, PR+merge.
9. Run:
   - `say "Second review is done"`
10. Update execution/phase docs with artifacts, actions, validation evidence, and merged PR link.

## Closure Cycles (same review loop)

After all waves are done, run these in order:

1. Wave closure:
   - pass 1 (completion check) -> apply -> validate -> PR+merge
   - pass 2 (production-grade check) -> apply -> validate -> PR+merge
   - send Telegram closure update
2. Milestone closure:
   - pass 1 (completion check) -> apply -> validate -> PR+merge
   - pass 2 (production-grade check) -> apply -> validate -> PR+merge
   - send Telegram closure update
3. Phase closure:
   - pass 1 (completion check) -> apply -> validate -> PR+merge
   - pass 2 (production-grade check) -> apply -> validate -> PR+merge
   - finalize statuses and roadmap wording
   - send Telegram closure update

When phase is fully closed, run:
- `say "Now we review"`

## Reviewer Trigger

Use the [talk-to-claude-opus](../talk-to-claude-opus/SKILL.md) skill for all external review passes.

Adjust prompt scope for the active stage: `wave`, `milestone closure`, `phase closure`, `pass 1`, `pass 2`.

If the target review file already exists, create a new filename with the same prefix and incremented suffix.

## Telegram Status Command

```bash
: "${SEND_TO_TELEGRAM_PROJECT:?Set SEND_TO_TELEGRAM_PROJECT to the send-to-telegram checkout path}"
direnv exec "${SEND_TO_TELEGRAM_PROJECT}" uv run --project "${SEND_TO_TELEGRAM_PROJECT}" \
  python "${SEND_TO_TELEGRAM_PROJECT}/send_to_telegram.py" "$(cat <<'MESSAGE'
Status for phase ${PHASE_NAME}:
<short status summary>
MESSAGE
)"
```

Send at minimum:

- after wave closure completes
- after milestone closure completes
- after phase closure completes
- immediately on blocker/failure

## Completion Checklist

- All wave implementation PRs merged
- All wave pass-1/pass-2 review PRs merged
- Wave closure pass-1/pass-2 merged
- Milestone closure pass-1/pass-2 merged
- Phase closure pass-1/pass-2 merged
- Execution ledger checkboxes and status lines updated to final state
- Roadmap wording updated to reflect closure
- Telegram updates sent for wave/milestone/phase closure
- `say "Now we review"` executed
