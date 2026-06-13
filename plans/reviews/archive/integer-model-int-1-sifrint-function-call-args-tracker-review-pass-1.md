# Review: INT-1 SifrInt Function Call Args Tracker Pass 1

## Verdict

Satisfied.

## Findings

None.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1831 (`gh pr view 1831` → `MERGED 2026-05-06T18:58:54Z`).

### Review History (line 405)

```
- [x] INT-1 `SifrInt` function call arguments and closure return-state review satisfied: `reviews/integer-model-int-1-sifrint-function-call-args-and-closure-return-state-review-pass-1.md`.
```

- File path resolves on disk (155 lines, present).
- Wording "satisfied" matches the pass-1 verdict (bare "Satisfied" without a qualifier — all the non-blocking notes are carry-forwards or future-proofing items, none introduced by this slice). The bare qualifier is slightly less informative than other recent entries (e.g., line 404 uses "satisfied with non-blocking broader function-boundary follow-ups"), but it's accurate and not misleading.
- Position after the function-return-boundaries entry (line 404) and before the INT-2A entries (line 406) preserves the chronological ordering.
- Single-pass entry pattern is consistent with prior single-pass slices in the milestone.

### Sub-item closure (line 443)

```
- [x] Calls to promoted `SifrInt`-returning functions now retype receiving `int` locals/arithmetic even when the call has ordinary arguments, and nested closure bodies no longer inherit promoted outer-function return coercion state, preserving shapes like `result: int = make_big_with_arg(3)` and promoted outers that call small nested helpers; review is satisfied and quick validation is passing: PR #1831.
```

Truthfulness checks against the implementation:

- "**Calls to promoted `SifrInt`-returning functions now retype receiving `int` locals/arithmetic even when the call has ordinary arguments**" — accurate. The slice drops the `args.is_empty()` guard in both [hir_expr_needs_sifr_int_storage](crates/sifr_codegen/src/function_emitter.rs:836) and [is_sifr_int_expr](crates/sifr_codegen/src/expr_render_helpers.rs:1385). The "even when the call has ordinary arguments" qualifier precisely names the asymmetry the slice closes (pass-1 N2 of #1829).
- "**nested closure bodies no longer inherit promoted outer-function return coercion state**" — accurate. The Closure/ClosureBlock arms in [rewrite_stdlib_constant_idents_in_expr](crates/sifr_codegen/src/expr_render_helpers.rs:399-431) save/clear/restore `current_sifr_int_return`, addressing pass-1 N1 of #1829.
- "preserving shapes like `result: int = make_big_with_arg(3)`" — concrete example for the call-with-args case; matches the e2e fixture's `returned_big_with_offset(3)` line.
- "and promoted outers that call small nested helpers" — concrete example for the closure isolation case; matches the e2e fixture's `returned_big_with_nested_small() -> int` (with internal `def small_inner() -> int: return 42`).
- "review is satisfied and quick validation is passing: PR #1831" — verified merged.

No overclaim: the bullet does not claim parameter migration, fallible `//`/`%` support, lexical shadowing fix, or nested-helper-naturally-SifrInt → outer promotion. All explicitly deferred concerns stay in the open follow-up below.

### Open follow-up (line 444)

Old (from PR #1830's tracker):
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support, **closure-body return coercion must not inherit promoted outer-function state**, and **function arguments/non-zero-argument call sites** still need uniform `SifrInt` lowering instead of legacy `i64`."

New:
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support, **function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`**, and **nested helpers whose own bodies naturally produce `SifrInt` still need propagation into enclosing function return promotion**."

Diff effects:

- **Removed** "closure-body return coercion must not inherit promoted outer-function state" — correctly removed because PR #1831 closes this. ✓
- **Removed** "non-zero-argument call sites still need uniform SifrInt lowering" — correctly removed because PR #1831 closes the call-site-with-args recognition. ✓
- **Modified** "function arguments/non-zero-argument call sites still need uniform SifrInt lowering" → "**function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`**" — surgical refinement. The "non-zero-argument call sites" half is now closed, leaving the "function arguments" half: specifically, when an argument expression has SifrInt-shape (e.g., a registered SifrInt local or a promoted helper call) but the function's parameter is still legacy-lowered as `i64`. This is the explicit deferral noted in the slice description ("argument expressions that themselves require SifrInt may remain future work") and confirmed by the pass-1 review's section-4 reproducer.
- **Added** "**nested helpers whose own bodies naturally produce `SifrInt` still need propagation into enclosing function return promotion**" — captures the pass-1 review's N-pass2-2 finding (the inverse of pass-1 N1: a nested helper inside a non-promoted outer that naturally returns SifrInt should propagate up to promote the outer's signature). The wording is precise and durable.
- **Carried forward verbatim**: "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage" and "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support".

Cross-referenced against pass-1 N-pass2-1 through N-pass2-5:

| Pass-1 finding | Tracker phrase | Captured? |
|---|---|---|
| **N-pass2-1** Defensive save/restore missing in `function_like_lowering`, `class_emitter`, `class_method_emitter` (carry-forward from #1829's pass-1 N3) | (not captured) | Future-proofing, currently benign. Reasonable to leave at review-file level. |
| **N-pass2-2** Nested helper naturally SifrInt → outer promotion gap | "nested helpers whose own bodies naturally produce `SifrInt` still need propagation into enclosing function return promotion" | ✓ |
| **N-pass2-3** Single-expr Closure save/restore is defensive | (not captured) | Code-shape, not user-facing. |
| **N-pass2-4** Single-expr Closure not unit-tested | (not captured) | Test-hardening, not user-facing. |
| **N-pass2-5** Other carry-forwards | All carried forward verbatim | ✓ |

All user-facing remaining gaps are tracked. N-pass2-1, N-pass2-3, and N-pass2-4 correctly stay at the review-file level — consistent with the milestone's established pattern of leaving non-user-facing polish out of tracker bullets (see PR #1818, #1820, #1822, #1824, #1826, #1828, #1830).

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 433 — correct, because the new follow-up at line 444 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — preserved: previous slices → augmented-assignment-targets → function-return-boundaries → **function-call-args (this PR)** → broader-migration follow-up. Each completed sub-item sits before the open work that depends on it.
- **PR linkage** — `gh pr view 1831` returns merged 2026-05-06T18:58:54Z with title "Handle SifrInt function calls with args". Branch's `git log` shows implementation commit `b2f0e42e` immediately preceded by tracker-only commit `124edc76`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1829 and the pass-1 implementation review of #1831. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +2/-1 lines on a single file. No edits to architecture/roadmap docs, code, tests, or fixtures. Consistent with prior tracker-only PRs (#1818, #1820, #1822, #1824, #1826, #1828, #1830).

## Notes

(Non-blocking observations only.)

- **N1 — The "satisfied" history qualifier is bare.** Recent INT-1 history entries used phrases like "satisfied with non-blocking broader function-boundary follow-ups" (line 404) or "satisfied with optional test-hardening notes" (line 402). The new entry uses just "satisfied". This is technically accurate — the pass-1 review's verdict was "Satisfied" without a qualifier — but it's slightly less informative. A reader scanning the history won't know there are non-blocking notes in the linked review. Optional polish: future tracker writers might consider adding "with non-blocking broader migration follow-ups" or similar to maintain consistency, but the current wording isn't misleading.

- **N2 — The "function argument expressions that are already `SifrInt`" wording** is precise and durable. It correctly narrows the open follow-up to the specific shape that's still broken: arg expressions with SifrInt-shape (registered locals, promoted helper calls) being passed to legacy-lowered `i64` parameters. This is more useful than the prior "function arguments/non-zero-argument call sites" bundle because it points at the exact failure mode that needs the broader parameter-migration work.

- **N3 — The "nested helpers whose own bodies naturally produce `SifrInt`" wording** correctly captures pass-1 N-pass2-2 — the inverse direction of the closure leak that #1831 fixed. PR #1831 stops the *outer* flag from leaking *into* the closure; pass-1 N-pass2-2 is about the *closure*'s natural SifrInt shape not propagating *up* to the outer. Both directions need broader migration to fully close.

- **N4 — Pass-1 N-pass2-1 (defensive save/restore in 3 other emitter paths)** stays at the review-file level. This is a carry-forward from #1829's pass-1 N3, so it's been at the review level for two consecutive trackers now. Worth noting if the milestone wants to track it explicitly, but consistent with prior precedent to omit.

- **N5 — Single-pass review entry pattern.** PR #1831 landed in one review pass like #1817/#1819/#1821/#1823/#1829 (rather than the dual-pass pattern of #1825/#1827). The history pattern accommodates both.
