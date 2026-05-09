# Review: INT-1 SifrInt Recursive Capture Params Tracker Pass 1

## Verdict

Satisfied.

## Findings

None.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1835 (`gh pr view 1835` → `MERGED 2026-05-06T19:43:54Z`).

### Review History (line 407)

```
- [x] INT-1 module-source recursive nested helper capture parameter review satisfied with non-blocking local-source capture follow-ups: `reviews/integer-model-int-1-sifrint-recursive-capture-params-review-pass-1.md`.
```

- File path resolves on disk (148 lines, present).
- Wording "satisfied with non-blocking local-source capture follow-ups" is **load-bearing precision** — it explicitly references the pass-1 N1 finding (local-source recursive capture body coercion gap) and signals to a future reader that closure was conditional on that follow-up. More informative than a bare "satisfied" qualifier.
- The "module-source" prefix correctly narrows what was fixed — the slice's primary target. This wording aligns with my pass-1 review's N1 observation that only the module-source branch of `recursive_capture_lowers_to_sifr_int` actually fires at the load-bearing line 281 insert site.
- Position after the nested-helper-return-propagation entry (line 406) and before the INT-2A entries (line 408) preserves the chronological ordering.
- Single-pass entry pattern is consistent with prior single-pass slices (#1817, #1819, #1821, #1823, #1829, #1831, #1833).

### Sub-item closure (line 447)

```
- [x] Recursive nested helper capture parameters for module exact-int sources now lower as Rust `SifrInt` and their hidden capture arguments use the exact-int value path, preserving recursive helpers that capture `BIG_LIMIT`; review is satisfied and quick validation is passing: PR #1835.
```

Truthfulness checks:

- "**Recursive nested helper capture parameters**" — accurate. The slice modifies `lower_recursive_capture_param_type` ([function_emitter.rs:184-189](crates/sifr_codegen/src/function_emitter.rs:184)) used in the recursive LocalFn path.
- "**for module exact-int sources**" — load-bearing precision matching the pass-1 N1 finding. This correctly narrows to the case that fully works (the only branch of the predicate that fires at line 281 because of the state-clear timing).
- "**now lower as Rust `SifrInt`**" — accurate. The new param-type lowering returns `SifrInt` for module-source captures.
- "their hidden capture arguments use the exact-int value path" — accurate. [lower_recursive_capture_arg_for_ir](crates/sifr_codegen/src/stmt_support_emitter.rs:5289) routes through `rewrite_stdlib_constant_idents_in_expr` + `coerce_expr_to_sifr_int_value`, which is the value-position SifrInt path established in PR #1825.
- "preserving recursive helpers that capture `BIG_LIMIT`" — concrete example matches the e2e fixture's `returned_big_from_recursive_nested_helper`.
- "review is satisfied and quick validation is passing: PR #1835" — verified merged 2026-05-06T19:43:54Z.

No overclaim:
- "for **module exact-int sources**" explicitly limits the scope.
- Doesn't claim local-source captures (which are correctly in the open follow-up).
- Doesn't claim full function parameter migration.

This is the right narrowing pattern — the closure bullet uses the precise scope qualifier that the pass-1 review surfaced.

### Open follow-up (line 448)

Old (from PR #1834's tracker):
> "...captured-local-only nested helpers plus **recursive nested helper capture parameters** still need propagation through the broader function-boundary migration."

New:
> "...captured-local-only nested helpers plus **local-source recursive capture body coercion** still need propagation through the broader function-boundary migration."

Diff effects:

- **Removed** "recursive nested helper capture parameters still need propagation" — correctly removed because PR #1835 closes this for module sources. ✓
- **Added** "**local-source recursive capture body coercion**" — captures pass-1 N1 (when the captured value is a forced/registered outer local rather than a module source, the parameter type is correctly lifted but the body's expressions on the captured name aren't coerced because the line 281 insert's predicate runs against cleared state). ✓
- **Carried forward verbatim**: "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage", "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support", "function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`", and "captured-local-only nested helpers".

Cross-referenced against pass-1 N1-N5:

| Pass-1 finding | Tracker phrase | Captured? |
|---|---|---|
| **N1** Local-source capture body-coercion gap | "local-source recursive capture body coercion still need propagation" | ✓ |
| **N2** Module-source captured parameter is dead code (Rust unused-variable warning) | (not captured) | Code-shape, not user-facing failure. Reasonable to leave at review-file level. |
| **N3** Visibility widening of three helpers | (not captured) | Implementation detail. |
| **N4** No focused unit tests for the new slice | (not captured) | Test-hardening, not user-facing. |
| **N5** Carry-forward open items | All carried forward verbatim | ✓ |

All user-facing findings are tracked. N2 (warning-level dead-code parameter), N3 (visibility), and N4 (test coverage) correctly stay at the review-file level — consistent with prior tracker patterns leaving non-user-facing polish out of milestone bullets.

The shift from "recursive nested helper capture parameters" to "local-source recursive capture body coercion" is semantically precise — it captures exactly what's still broken after this slice (parameter type is correctly lifted for all sources via line 321, but body coercion only fires for module sources via the cleared-state line 281 insert).

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 435 — correct, because the new follow-up at line 448 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — Wave 1 → Wave 1B → ... → nested-helper-return-propagation → **recursive-capture-params (this PR)** → broader-migration follow-up. Implementation order preserved.
- **PR linkage** — `gh pr view 1835` returns merged 2026-05-06T19:43:54Z with title "Promote recursive SifrInt captures". Branch's `git log` shows implementation merge commit `3999ce3e` immediately preceded by tracker-only commit `1ada849b`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1833 and the pass-1 implementation review of #1835. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +2/-1 lines on a single file. Consistent with prior tracker-only PRs.

## Notes

(Non-blocking observations only.)

- **N1 — The "module-source" qualifier in the closure bullet is the right narrowing.** It precisely captures what fully works after the slice and acknowledges the predicate's broader appearance vs. its actual delivery. Future tracker writers might consider this pattern when a slice's predicate is broader than the working scope: instead of letting the closure bullet inherit the predicate's framing, scope it to what actually works.

- **N2 — The "local-source recursive capture body coercion" wording** is precise and durable. It frames the remaining gap as a concrete codegen mechanism rather than abstract "broader migration", which makes the next slice's scope easier to define. Good signaling for the next slice author.

- **N3 — Pass-1 N2 (dead-code parameter for module-source captures)** stays at the review-file level. This is a Rust unused-variable warning shape that doesn't cause compile failures. Reasonable to omit from the tracker. If the milestone wants to clean this up, it would be a future polish slice — not a user-facing blocker.

- **N4 — Single-pass review entry pattern.** PR #1835 landed in one review pass like the majority of INT-1 slices (#1817, #1819, #1821, #1823, #1829, #1831, #1833). The history pattern accommodates both single-pass and dual-pass entries.

- **N5 — Carry-forward open items unchanged.** Lexical shadowing, legacy-emission, fallible `//`/`%`, function arguments / arg expressions that are already SifrInt, and captured-local-only nested helpers all stay tracked under the open INT-1 follow-up.
