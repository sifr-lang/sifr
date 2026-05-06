# Review: INT-1 SifrInt Nested Helper Return Propagation Tracker Pass 1

## Verdict

Satisfied.

## Findings

None.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1833 (`gh pr view 1833` → `MERGED 2026-05-06T19:21:40Z`).

### Review History (line 406)

```
- [x] INT-1 nested helper `SifrInt` return propagation review satisfied with non-blocking broader function-boundary follow-ups: `reviews/integer-model-int-1-sifrint-nested-helper-return-propagation-review-pass-1.md`.
```

- File path resolves on disk (165 lines, present).
- Wording "satisfied with non-blocking broader function-boundary follow-ups" matches the pass-1 verdict ("Satisfied with non-blocking suggestions" — N1 scope undersell, N2 captured-local-only gap, N3 recursive capture-arg gap, N4 save/restore polish, N5 unit test coverage, N6 carry-forwards).
- Position after the function-call-args entry (line 405) and before the INT-2A entries (line 407) preserves the chronological ordering.
- Single-pass entry pattern is consistent with the milestone's other single-pass slices.

### Sub-item closure (line 445)

```
- [x] Nested helpers whose annotated `-> int` returns transitively produce `SifrInt` through module exact-int sources or sibling/deeper nested helpers now participate in enclosing function return promotion, so outer functions returning those helper results lower to Rust `SifrInt`; review is satisfied and quick validation is passing: PR #1833.
```

Truthfulness checks:

- "**Nested helpers**" — accurate. The slice handles `HirStmt::NestedFunction` inside module-level functions via [collect_nested_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:794).
- "**annotated `-> int` returns**" — accurate. The pre-scan filters by `matches!(crate::resolve_alias_type_for_plain_call(&func.return_type), Type::Int)`.
- "**transitively produce `SifrInt`**" — accurate. The fixed-point loop in `collect_nested_sifr_int_function_returns` and the recursive `hir_function_returns_sifr_int` together handle transitive dependencies.
- "**through module exact-int sources or sibling/deeper nested helpers**" — load-bearing precision. This phrasing addresses the pass-1 review's N1 observation that the implementation actually supports deeper nesting (despite the slice description's conservative "direct nested helper" framing). Calling out "sibling/deeper" gives an honest description of what the slice delivers.
- "now participate in enclosing function return promotion" — accurate. The active set machinery makes nested helpers visible to the enclosing function's promotion analysis.
- "so outer functions returning those helper results lower to Rust `SifrInt`" — accurate. Concrete example matches the e2e fixture's `returned_big_from_nested_helper() -> SifrInt`.
- "review is satisfied and quick validation is passing: PR #1833" — verified merged.

No overclaim:
- Doesn't claim captured-local-only nested helpers (pass-1 N2).
- Doesn't claim recursive-capture argument propagation (pass-1 N3).
- Doesn't claim parameter migration.

### Open follow-up (line 446)

Old (from PR #1832's tracker):
> "...function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`, and **nested helpers whose own bodies naturally produce `SifrInt` still need propagation into enclosing function return promotion**."

New:
> "...function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`, and **captured-local-only nested helpers plus recursive nested helper capture parameters still need propagation through the broader function-boundary migration**."

Diff effects:

- **Removed** "nested helpers whose own bodies naturally produce `SifrInt` still need propagation into enclosing function return promotion" — correctly removed because PR #1833 closes this for the module-source case. ✓
- **Added** "**captured-local-only nested helpers plus recursive nested helper capture parameters still need propagation through the broader function-boundary migration**" — captures pass-1 N2 (closure body uses captured outer SifrInt local but helper isn't promoted) and pass-1 N3 (recursive nested helper's capture parameter is `i64` while the call site passes a `SifrInt`-returning helper). Bundling these under one bullet is reasonable because both will likely be unblocked by the same broader-migration milestone (closure capture handling + parameter lowering).
- **Carried forward verbatim**: "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage", "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support", and "function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`".

Cross-referenced against pass-1 N1-N6:

| Pass-1 finding | Tracker phrase | Captured? |
|---|---|---|
| **N1** Slice undersells (deeper nesting works) | "module exact-int sources or **sibling/deeper nested helpers**" (in closure bullet) | ✓ The closure bullet's wording aligns with actual capability. |
| **N2** Captured-local-only nested helpers don't promote | "**captured-local-only nested helpers** ... still need propagation" | ✓ |
| **N3** Recursive nested helpers with module-source captures break at capture-arg | "**recursive nested helper capture parameters** still need propagation" | ✓ |
| **N4** `try_lower_structured_nested_function_stmt` sets `current_sifr_int_return.set(false)` rather than promotion status | (not captured) | Defensive future-proofing carry-forward from #1831; currently no reachable failure. Reasonable to leave at review-file level. |
| **N5** Unit test coverage for `collect_nested_sifr_int_function_returns` is e2e-only | (not captured) | Test-hardening, not user-facing. Reasonable to omit. |
| **N6** Carry-forward open items | All carried forward verbatim | ✓ |

All user-facing remaining gaps are tracked. N4 and N5 correctly stay at the review-file level — consistent with the milestone's established pattern (#1818, #1820, #1822, #1824, #1826, #1828, #1830, #1832).

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 434 — correct, because the new follow-up at line 446 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — Wave 1 → Wave 1B → ... → augmented-assignment-targets → function-return-boundaries → function-call-args → **nested-helper-return-propagation (this PR)** → broader-migration follow-up. Implementation order preserved.
- **PR linkage** — `gh pr view 1833` returns merged 2026-05-06T19:21:40Z with title "Propagate nested helper SifrInt returns". Branch's `git log` shows implementation merge commit `2b1c3ea8` immediately preceded by tracker-only commit `e2f34680`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1831 and the pass-1 implementation review of #1833. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +2/-1 lines on a single file. Consistent with prior tracker-only PRs.

## Notes

(Non-blocking observations only.)

- **N1 — "sibling/deeper nested helpers" phrasing is a positive deviation.** The closure bullet acknowledges the recursion capability that pass-1 N1 flagged as "underselling". Most prior tracker bullets stayed conservative when the implementation went broader; this one picks up the broader scope. Future tracker writers might consider this pattern when slice descriptions undersell.

- **N2 — Bundling N2 and N3 under one phrase is reasonable at tracker granularity.** "Captured-local-only nested helpers" (N2 — outer's forced locals not propagated into nested helper analysis) and "recursive nested helper capture parameters" (N3 — LocalFn explicit `i64` capture types vs SifrInt-returning call args) are technically distinct mechanisms but both belong to the broader closure-capture / function-argument migration. If a future slice closes one but not the other, the bullet will need to be split — that's a normal evolution.

- **N3 — N4 (defensive save/restore set-to-false in `try_lower_structured_nested_function_stmt`) carries forward.** This was first flagged in pass-1 of #1831 and stayed at the review-file level there; it's now mentioned in this slice's review too. Currently no reachable failure construction. Worth keeping in mind if the LocalFn path ever becomes hot, but consistent with prior milestone practice to leave it at review-file granularity.

- **N4 — Single-pass review entry pattern.** PR #1833 landed in one review pass like #1817/#1819/#1821/#1823/#1829/#1831 (rather than the dual-pass pattern of #1825/#1827). The history accommodates both.

- **N5 — Carry-forward open items unchanged.** Lexical shadowing, legacy-emission, fallible `//`/`%`, function arguments / arg expressions that are already SifrInt, all stay tracked under the open INT-1 follow-up.
