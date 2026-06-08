I've audited the diff, the local validation report/log, prior M5 merge-ledger PRs (#2406, #2408), and the prior review files that govern the established convention.

---

**FAIL**

**Required fix**

1. **Revert the change at `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:447`** — restore the line to `- M5 resource nullcontext foundation: in progress.`.

   The "Implementation PRs" status block has an established M5 merge-ledger convention from the two immediately preceding ledger PRs:
   - PR #2406 (signal ledger, `4001d206a`) explicitly left line 445 as `M5 signal value-model foundation: in progress.` after #2405 merged. The accepted review (`reviews/...m5-signal-ledger-review-pass-1.md:6`) recorded this as scope discipline.
   - PR #2408 (warnings ledger, `8e3d5f59d`) explicitly left line 446 as `M5 warnings global-filter rejection: in progress.` after #2407 merged. The accepted review (`reviews/...m5-warnings-ledger-review-pass-1.md:20`) verified: *"Status section (line 446) keeps `M5 warnings global-filter rejection: in progress.` untouched."*

   This branch breaks that convention by promoting line 447 to a URL while the parallel-status sibling lines 445 and 446 — whose implementation PRs (#2405, #2407) have also merged with similarly partial sub-surfaces — remain `in progress.`. Result: the status section becomes internally inconsistent, and the M5 resource row visually claims a different completion grade than the signal/warnings rows even though all three are foundation-only slices with explicit follow-up scope (value-carrying nullcontext, ExitStack/AsyncExitStack, closing/aclosing, async cleanup, cancellation cleanup reports) still open per the implementation entry at lines 604-609. Reverting line 447 matches the established convention; the new `M5 resource nullcontext foundation merge ledger:` subsection at 626-629 already records the merge correctly.

**What's correct (no fix needed)**

- Merge ledger subsection at 626-629 accurately records PR #2409, merge SHA `5001b0985838a240a7adeb01adf6fa343970cb36` (verified against `git show 5001b0985`), and date 2026-06-08.
- Validation metrics quoted in the ledger match the report on disk:
  - `time.real_seconds=221.95` ✓ → `221.95s`
  - `e2e.cache_hits=28`, `e2e.group_count=32` ✓ → `28/32`
  - `report_signature=6dd646fdf4fc2cb4` ✓ (from `.log`)
  - `platform-golden summary pass=6 skip=1` ✓
  - `116 pass tests completed (116 passed, 0 failed)` ✓
  - Both advisories (`warm wall-time budget exceeded`, `warm-cache hit rate below advisory target`) ✓
  - Lane-step enumeration matches the report's `lane_steps`.
- Merge-ledger subsection scope is honest: it does not overclaim ExitStack/AsyncExitStack, value-carrying nullcontext, closing/aclosing, or async cleanup beyond the no-value nullcontext slice.

**Non-blocking observation**

- `reviews/ad-hoc-production-concurrency-runtime-m5-resource-ledger-review-pass-1.md` is currently 0 bytes (untracked). Both prior ledger PRs added a populated `*-ledger-review-pass-1.md` review artifact AND a corresponding sub-bullet under their merge-ledger subsection referencing it (see lines 579 and 602 for the prior pattern). The pass-1 review file will need to be populated and a matching `- reviews/...m5-resource-ledger-review-pass-1.md: PASS; ...` bullet appended under the new merge-ledger subsection (after the revert above) before merging the PR — consistent with how the signal and warnings ledger PRs landed. Flagging as non-blocker because that step is normally done after this review pass, not before.
