# Review: INT-1 SifrInt Function Return Boundaries Tracker Pass 1

## Verdict

Satisfied.

## Findings

None.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1829 (`gh pr view 1829` → `MERGED 2026-05-06T18:43:48Z`).

### Review History (line 404)

```
- [x] INT-1 `SifrInt` function return boundary review pass 1 satisfied with non-blocking broader function-boundary follow-ups: `reviews/integer-model-int-1-sifrint-function-return-boundaries-review-pass-1.md`.
```

- File path resolves on disk (176 lines, present).
- Wording "satisfied with non-blocking broader function-boundary follow-ups" matches the pass-1 verdict ("Satisfied with non-blocking suggestions" — N1 closure leak, N2 parametrized-fn call-site asymmetry, N3 missing save/restore, N4 test coverage, N5 carry-forwards).
- Position after the augassign pass-2 entry (line 403) and before the INT-2A entries (line 405) preserves the chronological ordering.
- Single-pass entry pattern is consistent with prior INT-1 slices that landed in one review pass (#1817, #1819, #1821, #1823, #1827).

### Sub-item closure (line 441)

```
- [x] Module-level `-> int` functions whose returns transitively depend on exact-int helpers, locals, or promoted zero-argument helper calls now return generated Rust `SifrInt`; return statements are value-coerced and downstream zero-argument call sites retype exact-int locals/arithmetic, preserving shapes like `value: int = returned_big_limit()` and `returned_big_limit() + 1`; review is satisfied and quick validation is passing: PR #1829.
```

Truthfulness checks against the implementation diff (which I reviewed in pass-1):

- "**Module-level**" — load-bearing qualifier. The pre-scan at [function_emitter.rs:131](crates/sifr_codegen/src/function_emitter.rs:131) iterates `module.functions` only; class methods and nested function declarations are not visited. The bullet's "Module-level" precisely scopes this.
- "**`-> int` functions whose returns transitively depend on exact-int helpers, locals, or promoted zero-argument helper calls**" — accurate enumeration of the three sources `hir_expr_needs_sifr_int_storage` recognizes (`Name → module_sifr_int_bindings`, `Name → forced_locals`, and `Call{args: [], func ∈ function_sifr_int_returns}`). The phrase "transitively" correctly describes the fixed-point loop.
- "now return generated Rust `SifrInt`" — accurate. [lower_function_return_type](crates/sifr_codegen/src/function_emitter.rs:434) returns `RustType::Named("SifrInt")` when the function is in `sifr_int_function_returns`.
- "**return statements are value-coerced**" — accurate. The Return arm at [expr_render_helpers.rs:565](crates/sifr_codegen/src/expr_render_helpers.rs:565) routes through `coerce_expr_to_sifr_int_value` when `current_sifr_int_return == true`.
- "**downstream zero-argument call sites retype exact-int locals/arithmetic**" — load-bearing precision. The "zero-argument" qualifier reflects [is_sifr_int_returning_function_call](crates/sifr_codegen/src/expr_render_helpers.rs:1406) and the new arm in [hir_expr_needs_sifr_int_storage](crates/sifr_codegen/src/function_emitter.rs:836) both gating on `args.is_empty()`. Refuses to imply call-site recognition for parametrized calls.
- "preserving shapes like `value: int = returned_big_limit()` and `returned_big_limit() + 1`" — both shapes are pinned by the e2e fixture's `returned_big`/`returned_plus_one` lines and round-trip at runtime.
- "review is satisfied and quick validation is passing: PR #1829" — verified.

No overclaim:
- "Module-level" rules out class methods.
- "Zero-argument" rules out call sites with arguments (which pass-1 N2 noted is still broken — though pre-PR was also broken).
- Doesn't claim parameter migration.
- Doesn't claim closure-body coverage (pass-1 N1 leak remains an open concern, captured in the new follow-up).

### Open follow-up (line 442)

Old (from PR #1828's tracker):
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support, and **function argument/return boundaries** still need uniform `SifrInt` lowering instead of legacy `i64`."

New:
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support, **closure-body return coercion must not inherit promoted outer-function state**, and **function arguments/non-zero-argument call sites** still need uniform `SifrInt` lowering instead of legacy `i64`."

Diff effects:

- **Inserted** "**closure-body return coercion must not inherit promoted outer-function state**" — this is precisely pass-1 N1's finding. The wording is durable (frames the contract: closures shouldn't inherit the outer's flag) and matches the pass-1 review's reproducer.
- **Modified** "function argument/return boundaries" → "**function arguments/non-zero-argument call sites**" — semantically careful refinement:
  - Removes the broad "return boundaries" framing because zero-arg module-level returns are now closed by #1829.
  - Adds "non-zero-argument call sites" to capture pass-1 N2 (parametrized-fn call site `result: int = f(arg)` still fails because the pre-scan's `args.is_empty()` check excludes it).
  - Keeps "function arguments" for the broader parameter migration that hasn't started.
- **Carried forward verbatim**: "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage" and "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support".

Cross-referenced against pass-1 N1–N5:

| Pass-1 finding | Tracker phrase | Captured? |
|---|---|---|
| **N1** Closure body Returns inherit `current_sifr_int_return` from promoted outer | "closure-body return coercion must not inherit promoted outer-function state" | ✓ |
| **N2** Parametrized-fn call-site asymmetry | "function arguments/non-zero-argument call sites" | ✓ |
| **N3** Missing save/restore in 3 emitter paths (currently benign) | (not captured) | Defensive future-proofing; reasonable to leave at review-file level |
| **N4** Unit test coverage gaps | (not captured) | Code-shape/test hardening; reasonable to omit |
| **N5** Other carry-forwards | All carried forward | ✓ |

All user-facing remaining gaps are tracked. N3 and N4 are correctly left at the review-file level — consistent with prior tracker patterns leaving non-user-facing polish out of the milestone bullet.

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 432 — correct, because the new follow-up at line 442 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — Wave 1 → Wave 1B → oversized-module-int → use-sites-direct → SifrInt-local-comparisons → SifrInt-local-value-semantics → plain-assignment-targets → augmented-assignment-targets → **function-return-boundaries (this PR)** → broader-migration follow-up. Implementation order preserved.
- **PR linkage** — `gh pr view 1829` returns merged 2026-05-06T18:43:48Z with title "Promote exact-int function returns to SifrInt". Branch's `git log` shows the implementation merge commit `f11294e1` immediately preceded by tracker-only commit `789adf4e`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1827 and the pass-1 implementation review of #1829. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +2/-1 lines on a single file. No edits to architecture/roadmap docs, code, tests, or fixtures. Consistent with prior tracker-only PRs (#1818, #1820, #1822, #1824, #1826, #1828).

## Notes

(Non-blocking observations only.)

- **N1 — The new "closure-body return coercion" phrasing is durable and precise.** It frames the gap as a contract violation ("must not inherit") rather than a mechanism description, which survives a future refactor of how `current_sifr_int_return` is plumbed. This is the right level of abstraction for a tracker bullet.

- **N2 — The "function arguments/non-zero-argument call sites" wording bundles two distinct concerns** (parameter migration + call-site recognition with args). They're related — both will likely be unblocked by the same broader-migration milestone — so bundling is reasonable at tracker granularity. If a future slice closes one but not the other, the bullet will need to be split, but that's a normal evolution.

- **N3 — Pass-1 N3 (defensive save/restore in `function_like_lowering`/`class_emitter`/`class_method_emitter`) and N4 (unit test coverage gaps)** stay at the review-file level rather than being promoted to tracker bullets. Consistent with how prior tracker PRs (#1818, #1820, #1822, #1824, #1826, #1828) treated similar polish items — non-user-facing hardening lives in review files; tracker bullets enumerate user-visible failure shapes.

- **N4 — Single-pass review pattern.** PR #1829 landed in one review pass (Satisfied with non-blocking suggestions), unlike PR #1825 (two passes after a B1 blocker) and PR #1827 (two passes after optional N3/N4 hardening). The history pattern accommodates both: dual-pass slices have two adjacent entries, single-pass slices have one. This entry follows the single-pass pattern of #1817, #1819, #1821, and #1823.

- **N5 — Carry-forward open items unchanged.** Lexical shadowing, legacy-emission, fallible `//`/`%` — all stay tracked under the open INT-1 follow-up.
