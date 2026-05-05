# milestone_diag_10 slice 1 review: recovery limit summaries

Reviewer is satisfied.

## Key structural changes in this slice

- `SIFR-INTERNAL-0002` activated (Reserved -> Active) as a structured note diagnostic.
- Grouping key changed from rendered `message` to `message_template` + `dedupe_args`, refining how similar diagnostics are clustered.
- Per-group cap (`MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5`) now emits a `SIFR-INTERNAL-0002` summary note instead of a mutated clone.
- Top-level cap (`MAX_TOP_LEVEL_DIAGNOSTICS = 50`) also emits a `SIFR-INTERNAL-0002` summary note instead of silent truncation.
- New `RecoveryGroupKey` struct, `recovery_dedupe_args`, and `diagnostic_arg_key` helper for the refined grouping.
- Test fixtures adjusted (`zzz_distinct_*`, `aaa_repeated`) and test expectations updated to account for the new grouping semantics.

## Test Assertions

- `canonical[5]` now checks for `SIFR-INTERNAL-0002` (`3 additional diagnostics omitted by recovery cap (similar-diagnostic group)`).
- `canonical[49]` now checks for `SIFR-INTERNAL-0002` (`11 additional diagnostics omitted by recovery cap (top-level diagnostic stream)`).
- Summary line updated to `48 error(s), 2 note(s)` reflecting the two summary notes.

No blocking issues found.
