# Review: INT-1 SifrInt Function Parameter Boundary Tracker Pass 1

## Verdict

Satisfied.

## Findings

No blocking findings.

The +3/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1841 across both review passes (`gh pr view 1841` → `MERGED 2026-05-06T20:53:44Z`).

### Review History (lines 410–411)

Two entries are added, following the dual-pass pattern established by PR #1826 (#1825) and PR #1828 (#1827):

```
- [x] INT-1 function parameter boundary review pass 1 completed with registered-local double-coercion blocker: `reviews/integer-model-int-1-sifrint-function-parameter-boundaries-review-pass-1.md`.
- [x] INT-1 function parameter boundary review pass 2 satisfied after addressing registered-local argument coercion: `reviews/integer-model-int-1-sifrint-function-parameter-boundaries-review-pass-2.md`.
```

- Both file paths resolve on disk (196 + 144 lines respectively).
- **Pass-1 wording** "completed with registered-local double-coercion blocker" precisely names the B1 finding from pass-1 (registered SifrInt local passed to promoted parameter emitted `SifrInt::from_i64(big.clone())` due to double-application of `coerce_expr_to_sifr_int_value` across `adapt_plain_call_args_with_signature_for_ir` and the FnCall arm in `rewrite_stdlib_constant_idents_in_expr`).
- **Pass-2 wording** "satisfied after addressing registered-local argument coercion" matches the pass-2 verdict (Satisfied: the one-line `Clone(expr) => self.is_sifr_int_expr(expr)` arm closed B1).
- Position after the local-nonrecursive-capture-body entry (line 409) and before the INT-2A entries (line 412) preserves chronological ordering.
- The dual-pass entry pattern is consistent with prior dual-pass slices in the milestone.

### Sub-item closure (line 454)

```
- [x] Module-level `int` parameter positions whose call sites receive `SifrInt`-shaped arguments now promote to Rust `SifrInt`, coerce exact and small call arguments consistently, and register promoted parameters as exact-int locals inside function bodies, preserving module helpers such as `echo_int_parameter(BIG_LIMIT)`, `echo_int_parameter(reusable_oversized_local)`, and mixed-position exact-int arguments; review is satisfied after two passes and quick validation is passing: PR #1841.
```

Truthfulness checks:

- "**Module-level**" — load-bearing precision. The slice's pre-scan walks `module.functions` only and tracks per-function-per-position promotion via `sifr_int_function_params: HashMap<String, HashSet<usize>>`. Class methods and nested helpers are correctly out of scope.
- "**`int` parameter positions whose call sites receive `SifrInt`-shaped arguments**" — accurate. The pre-scan's [collect_sifr_int_call_arg_function_params](crates/sifr_codegen/src/function_emitter.rs:1031-1062) walker checks per-arg via `hir_expr_needs_sifr_int_storage` and marks promoted positions per (function name, parameter index).
- "**promote to Rust `SifrInt`**" — accurate. `lower_module_function_param_type` returns `RustType::Named("SifrInt")` for promoted positions.
- "**coerce exact and small call arguments consistently**" — accurate. Both exact (module helper, registered local, BinOp result) and small (literal cast) arguments flow through `coerce_expr_to_sifr_int_value` at promoted positions, producing pass-through, `Clone`, or `from_i64` shapes as appropriate.
- "**register promoted parameters as exact-int locals inside function bodies**" — accurate. `register_function_scope_params` was updated to insert promoted parameter names into `sifr_int_local_bindings` so the body's BinOp/UnaryOp arms coerce parameter uses.
- **Three concrete examples** are diagnostically useful:
  1. `echo_int_parameter(BIG_LIMIT)` — module helper as arg.
  2. `echo_int_parameter(reusable_oversized_local)` — registered local as arg (this is the load-bearing pass-1 B1 reproducer that pass-2 fixed and the new fixture line at module_constants.sifr:107 pins).
  3. "mixed-position exact-int arguments" — covers `add_to_exact_parameter(BIG_LIMIT, 3)` (only `value` at idx 0 promoted) and `add_right_exact_parameter(3, BIG_LIMIT)` (only `value` at idx 1 promoted).
- "**review is satisfied after two passes**" — explicitly references the dual-pass history. Same load-bearing wording pattern as prior dual-pass tracker bullets (#1826's "satisfied after pass 2 closed the value-position alias blocker", #1828's "satisfied after addressing optional test-hardening notes").
- "quick validation is passing: PR #1841" — verified merged 2026-05-06T20:53:44Z.

No overclaim:
- "Module-level" excludes class methods and nested helpers.
- The example list correctly includes the load-bearing registered-local case.
- Doesn't claim full broader migration.

### Open follow-up (line 455)

Old (from PR #1840's tracker):
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support, and **function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`**."

New:
> "...lexical shadowing and legacy-emission paths need scope-safe exact-int coverage, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support."

Diff effects:
- **Removed** "function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`" — correctly removed because PR #1841 closes this. ✓
- **Carried forward verbatim**:
  - "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage"
  - "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support"

The narrowed residual list is accurate and complete:

| Pass-1/Pass-2 finding | Tracker phrase | Captured? |
|---|---|---|
| Pass-1 B1 (registered-local double-coercion) | (closed by PR #1841 pass-2) | ✓ Closed in implementation |
| Pass-1 N1–N6 (pre-scan structure, per-position promotion, body coercion, prior behavior preserved, carry-forwards, no focused unit tests) | All implementation/test polish; carry-forwards captured | ✓ |
| Pass-2 N1–N6 (minimum-cost fix, Clone arm pairing, no focused unit test for Clone, carry-forwards, pass-pattern, validation) | Implementation-detail observations; carry-forwards captured | ✓ |

All user-facing remaining gaps are tracked. Implementation-detail and process observations correctly stay at the review-file level — consistent with the milestone's established pattern of leaving non-user-facing polish out of tracker bullets (#1818, #1820, #1822, #1824, #1826, #1828, #1830, #1832, #1834, #1836, #1838, #1840).

### Notable structural observation: function-boundary sub-phase fully closed

After this slice, the open INT-1 follow-up has only **two remaining items**:
1. Lexical shadowing and legacy-emission paths.
2. Unsupported augmented assignment / fallible `//` and `%`.

Both are **non-function-boundary** concerns. The function-boundary sub-phase is now fully closed across seven coordinated sub-slices:

| Sub-slice                                                  | PR    |
|------------------------------------------------------------|-------|
| Function returns                                           | #1829 |
| Call args + closure return-state isolation                 | #1831 |
| Nested helper returns                                      | #1833 |
| Recursive nested helper module-source captures             | #1835 |
| Recursive nested helper local-source captures              | #1837 |
| Non-recursive nested helper local-source captures          | #1839 |
| **Function parameter boundaries (this slice)**             | #1841 |

This is significant milestone progression — INT-1 is converging toward closure. Worth noting in any milestone-closure review.

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 442 — correct, because the new follow-up at line 455 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — preserved: previous slices → local-nonrecursive-capture-body (#1839) → **function-parameter-boundaries (this PR, #1841)** → broader-migration follow-up. Implementation order maintained.
- **PR linkage** — `gh pr view 1841` returns `MERGED 2026-05-06T20:53:44Z` with title "Promote SifrInt function parameters". Branch's `git log` shows implementation merge commit `fe4b3f2b` immediately preceded by tracker-only commit `8b623bdb`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1839 and the pass-1/pass-2 implementation reviews of #1841. Tracker-only diff preserves the signature, as expected. The cited `wall_time=105.32s` is slightly higher than typical (mid-60s to mid-70s in earlier trackers) but within normal variance for cache state.
- **No collateral churn** — diff is +3/-1 lines on a single file (two new history rows, one closing bullet, one modified open follow-up). No edits to architecture/roadmap docs, design doc, code, tests, or fixtures. Consistent with prior dual-pass tracker PRs (#1826 for #1825, #1828 for #1827).

## Notes

(Non-blocking observations only.)

- **N1 — The dual-pass entry pattern is consistent.** Two adjacent history rows (one for pass-1 with blocker, one for pass-2 satisfied) match the format from PRs #1825/#1826 and #1827/#1828. The "completed with [blocker description]" / "satisfied after [resolution description]" pairing reads cleanly.

- **N2 — The closing bullet's three-example list is unusually detailed.** Most prior tracker bullets list one or two concrete shapes; this slice lists three (`echo_int_parameter(BIG_LIMIT)`, `echo_int_parameter(reusable_oversized_local)`, "mixed-position exact-int arguments"). The extra detail is justified because the slice covers three distinct argument shapes (module helper, registered local — load-bearing for the B1 fix, and per-position selectivity), so the example list is diagnostically useful for future maintainers.

- **N3 — "satisfied after two passes" is the right precision marker.** It signals that closure was conditional on the pass-2 B1 fix without dragging the pass-1 blocker into the closing bullet's prose. Same load-bearing wording pattern as PR #1828's "satisfied after addressing optional test-hardening notes".

- **N4 — Function-boundary sub-phase converged.** Seven sub-slices over PRs #1829–#1841 close the full function-boundary surface (returns, parameters, captures × recursion × source). The next milestone-closure review should highlight this convergence. The remaining open items are genuinely separate concerns (shadowing/legacy emission, fallible arithmetic).

- **N5 — Validation notes are adequate for a tracker-only PR.** `git diff --check` and `scripts/run_all_tests.sh --profile quick` cover the canonical AGENTS.md gates. The `report_signature` matching all prior milestone PRs confirms no test deltas.

- **N6 — Pass-1 N1–N6 (implementation-detail observations) and Pass-2 N1–N6 (code-shape, test, process observations)** stay at the review-file level. Consistent with prior tracker patterns. None are user-facing failure shapes that would warrant tracker-bullet promotion.
