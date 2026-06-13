# INT-1 Large Module `int` Constant SifrInt Codegen Tracker — Review Pass 1

**Verdict:** Satisfied. No blockers.

## Scope reviewed

Working-tree diff against `c40d13c5` on `int-1-large-module-int-codegen-tracker`:

- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)

Reference docs:
- [reviews/integer-model-int-1-large-module-int-codegen-review-pass-1.md](reviews/integer-model-int-1-large-module-int-codegen-review-pass-1.md) (the implementation review this tracker change closes)
- Merged implementation PR: https://github.com/sifr-lang/sifr/pull/1817 (corresponds to `c40d13c5 Wire oversized module int constants through SifrInt (#1817)` in `git log`)

## Brief vs. diff

The brief asked for three changes. All three are present and correct.

1. **Add the satisfied INT-1 oversized module int constant codegen review to Review History.** ✓ — line 396 inserts:

   ```
   - [x] INT-1 oversized module `int` constant codegen review satisfied: `reviews/integer-model-int-1-large-module-int-codegen-review-pass-1.md`.
   ```

   Format matches the surrounding INT-1 history rows (e.g. line 395 "INT-1 fixed-width conversion substrate wave review satisfied: …"), the path is exactly the on-disk filename (verified: 130-line file present), and the placement after the wave-1B fixed-width-conversion entry and before the INT-2A entries is consistent with the existing chronological/milestone ordering.

2. **Mark the module-level oversized int constant SifrInt codegen checklist item complete with PR #1817.** ✓ — line 427 flips:

   ```
   - [ ] Wire module-level `int` constants whose in-budget values exceed `i64` through `SifrInt` codegen, removing the current module-constant production panic path tracked by the INT-2B module const/fixed-width fallback cleanup review.
   ```

   to

   ```
   - [x] Module-level `int` constants whose in-budget values exceed `i64` now lower through `SifrInt` helper codegen, removing the current module-constant production panic path tracked by the INT-2B module const/fixed-width fallback cleanup review; review is satisfied and quick validation is passing: PR #1817.
   ```

   The new wording mirrors the closing-bullet template used for the rest of INT-2B (e.g. line 446 "…; review is satisfied and quick validation is passing: PR #1814."). The PR number is consistent with `git log --oneline -1 c40d13c5` (`Wire oversized module int constants through SifrInt (#1817)`).

3. **Preserve the non-blocking N1 follow-up as an explicit open INT-1 use-site/arithmetic migration bullet.** ✓ — line 428 adds:

   ```
   - [ ] Wire `int`-typed use sites and arithmetic that reference oversized `SifrInt` module-constant helpers, so expressions like `BIG_LIMIT + 1` no longer fall through to invalid legacy `i64` Rust during the broader `Type::Int` codegen migration.
   ```

   This is an accurate restatement of the pass-1 N1 finding (`__const_BIG_LIMIT() + (1 as i64)` → `cannot add i64 to SifrInt` at `rustc`). Scoping it to "use sites and arithmetic that reference oversized `SifrInt` module-constant helpers" is the precise, narrowest framing of the regression: the new helper exists only for the `i64::try_from` failure path, so the gap is specifically when users mix a small-int legacy `const` with the helper-call shape. Tying it to "the broader `Type::Int` codegen migration" matches the pass-4 N4 deferral note that this work belongs to a later INT-1 wave (the broader `int` arithmetic SifrInt wiring), so the bullet does not over-promise a near-term fix.

## Cross-checks

- **Top-level INT-1 line stays open.** Line 424 is still `- [ ] INT-1 runtime `SifrInt` and ownership semantics`, which is correct because the new follow-up sub-item at line 428 is unchecked. No spurious milestone-level closure.
- **Sub-item ordering inside INT-1.** Wave 1 (line 425) → Wave 1B (line 426) → oversized-module-int closure (line 427) → use-site/arithmetic follow-up (line 428). The bullets stay in implementation order. New bullet sits adjacent to its parent slice, which makes the dependency obvious to a future reviewer.
- **PR linkage.** PR #1817 in `git log --oneline -1 c40d13c5` corresponds to "Wire oversized module int constants through SifrInt (#1817)", matching the slice description.
- **Validation reference.** The brief reports `scripts/run_all_tests.sh --profile quick` passing (`report_signature=e1bf653aaa770517`, `wall_time=79.25s`). The first pass-1 implementation review recorded the same `report_signature=e1bf653aaa770517` at `wall_time=71.91s`, so the suite is reproducibly green on the merged tip with no new test deltas — consistent with a tracker-only change. Per AGENTS.md the quick-profile gate is the authoritative pre-PR validation, which this satisfies.
- **No collateral churn.** Diff is +3/-1 lines on a single file; no edits to `internal_docs/architecture.md`, `internal_docs/roadmap.md`, code, or tests. That matches expectations for a tracker-only PR closing a sub-item: AGENTS.md asks for doc updates "as applicable", and no architecture/roadmap claim depends on the new closure beyond what the issue tracker already records.

## Non-blocking observations

None. The pass-1 review surfaced six non-blocking items (N1–N6); the brief explicitly elects to preserve only N1 in the tracker, which is the right call — N2 (codegen dead `UnaryOp` branches) and N3 (bare-constant fold test) are fix-on-touch items, N4 (digit-budget invariant test) is INT-8 panic-shape territory, N5 (per-call `parse_decimal` caching) is INT-8 perf territory, and N6 (`RustLiteral::Str` style) is an inline cleanup. None warrant their own tracker bullets at this granularity, and surfacing them in the pass-1 review is sufficient for follow-up discoverability.

## Verdict

**Satisfied. No blockers.** The diff accurately reflects the merged implementation: the Review History entry points at the on-disk pass-1 review file with consistent formatting, the closed sub-item is checked with the correct PR number and a well-formed prose summary, the parent INT-1 milestone correctly stays open behind the new sub-item, and the new open bullet preserves the pass-1 N1 follow-up at the right scope (use-site/arithmetic mixing with the `SifrInt` helper, deferred to the broader `Type::Int` codegen migration). Quick validation reproduces the same `report_signature` as the implementation review, confirming the tracker change introduces no test or build deltas.
