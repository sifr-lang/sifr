# Review: INT-1 SifrInt Lexical Shadowing Tracker Pass 1

## Verdict

Satisfied.

## Findings

No blocking findings.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1843 (`gh pr view 1843` → `MERGED 2026-05-06T21:17:28Z`).

### Review History (line 412)

```
- [x] INT-1 immediate lexical shadowing review satisfied with non-blocking nested-scope shadowing follow-up: `reviews/integer-model-int-1-sifrint-lexical-shadowing-review-pass-1b.md`.
```

- File path resolves on disk (153 lines, present).
- Wording "satisfied with non-blocking nested-scope shadowing follow-up" matches the pass-1b verdict (Satisfied with N1 noting that nested-scope shadowing remains open).
- The "**immediate**" prefix correctly scopes — distinguishes from any future "nested-scope" slice and matches pass-1b's N1 framing.
- Position after the function-parameter-boundary entries (lines 410–411) and before the INT-2A entries (line 413) preserves chronological ordering.
- The review file uses the `-pass-1b` naming convention (single review pass with retry suffix); the entry text correctly says "review satisfied" without invoking dual-pass framing.

### Sub-item closure (line 456)

```
- [x] Function-local and parameter bindings that shadow oversized exact-int module constants now suppress helper rewrites and SifrInt pre-scan promotion in their immediate function scope, preserving `BIG_LIMIT: int = 5` and `def f(BIG_LIMIT: int)` shadow cases while unshadowed module constants still lower through `SifrInt`; review is satisfied and quick validation is passing: PR #1843.
```

Truthfulness checks:

- "**Function-local and parameter bindings**" — accurate. The slice's [collect_function_local_shadow_names](crates/sifr_codegen/src/function_emitter.rs:1031) returns the union of locally-defined names and parameter names.
- "**shadow oversized exact-int module constants**" — accurate. The check is against `module_sifr_int_bindings` (module-level helpers that produce `SifrInt`).
- "**now suppress helper rewrites and SifrInt pre-scan promotion**" — accurate description of the two coordinated mechanisms:
  1. Rewriter: [rewrite_special_ident](crates/sifr_codegen/src/expr_render_helpers.rs:1289) early-returns when `local_binding_types.contains_key(&name)`.
  2. Pre-scan: [hir_expr_needs_sifr_int_storage](crates/sifr_codegen/src/function_emitter.rs:1232) Name arm now checks `!shadowed_module_bindings.contains(name)`.
- "**in their immediate function scope**" — load-bearing precision. This precisely matches pass-1b N1's note that nested-scope shadowing isn't addressed (a nested helper inside an outer-shadow function doesn't see the outer's shadow because `try_lower_structured_nested_function_stmt` clears `local_binding_types` for the inner emit).
- "preserving `BIG_LIMIT: int = 5` and `def f(BIG_LIMIT: int)` shadow cases" — two concrete examples matching the e2e fixture's `shadow_exact_module_constant_with_local` and `shadow_exact_module_constant_with_param`.
- "while unshadowed module constants still lower through `SifrInt`" — accurate. The fixture's other entries (`returned_big_limit()`, `BIG_LIMIT + 1`, etc.) continue to round-trip through the `__const_BIG_LIMIT()` helper.
- "review is satisfied and quick validation is passing: PR #1843" — verified merged.

No overclaim:
- "**immediate function scope**" excludes nested-scope shadow cases (which remain open per pass-1b N1).
- Two concrete examples cover both local-let and parameter-shadow shapes.
- Doesn't claim nested-scope coverage.

### Open follow-up (line 457)

Old (from PR #1842's tracker):
> "Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: **lexical shadowing and legacy-emission paths need scope-safe exact-int coverage**, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support."

New:
> "Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: **nested helper lexical shadowing for outer locals that shadow exact-int module constants still needs scope-safe coverage**, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support."

Diff effects:
- **Modified** "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage" → "**nested helper lexical shadowing for outer locals that shadow exact-int module constants still needs scope-safe coverage**" — narrows the lexical-shadowing residual to precisely what's still open: pass-1b N1's nested-scope gap. The new wording is more diagnostically useful (concretely names the failure shape).
- **Carried forward verbatim**: "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support".

The narrowed residual list is accurate and complete relative to pass-1b N1–N5 findings:

| Pass-1b finding | Tracker phrase | Captured? |
|---|---|---|
| **N1** Nested-scope shadowing not addressed (outer's shadow doesn't propagate to inner closure body) | "nested helper lexical shadowing for outer locals that shadow exact-int module constants still needs scope-safe coverage" | ✓ |
| **N2** Rewriter's early-return shadows ALL special-ident handling (correct behavior) | (not captured) | Not a residual gap; observation only. |
| **N3** Call-order convention dependency | (not captured) | Implementation detail, not user-facing. |
| **N4** Carry-forward open items | "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support" | ✓ |
| **N5** Validation notes adequate | (not captured) | Process observation. |

All user-facing residual gaps are tracked. N2, N3, and N5 correctly stay at the review-file level — consistent with prior tracker patterns leaving non-user-facing polish out of milestone bullets.

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 444 — correct, because the new follow-up at line 457 is unchecked.
- **Sub-item ordering** — preserved: previous slices → function-parameter-boundaries (#1841) → **lexical-shadowing (this PR, #1843)** → broader-migration follow-up. Implementation order maintained.
- **PR linkage** — `gh pr view 1843` returns `MERGED 2026-05-06T21:17:28Z` with title "Respect local shadows for SifrInt module constants". Branch's `git log` shows implementation merge commit `34999e29` immediately preceded by tracker-only commit `99fe76f9`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1841 and the pass-1b implementation review of #1843. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +2/-1 lines on a single file. Consistent with prior tracker-only PRs.
- **Validation notes adequate** — `git diff --check` plus `scripts/run_all_tests.sh --profile quick` (with matching `report_signature`) is the canonical AGENTS.md gate for tracker-only PRs.

## Notes

(Non-blocking observations only.)

### N1 — The "and legacy-emission paths" portion of the prior bullet is dropped

The prior open-follow-up bullet (PR #1842's tracker) said "lexical shadowing **and legacy-emission paths** need scope-safe exact-int coverage" — bundling two concerns under one item. The new bullet keeps only the nested-helper lexical shadowing portion and drops the "legacy-emission paths" mention.

This is plausibly correct because:
- The slice's two coordinated mechanisms (rewriter early-return + pre-scan threading) update the two emission-relevant paths historically grouped under "scope-safe exact-int coverage".
- No specific reproducer for "legacy-emission paths" has been flagged in any recent review.
- The function-boundary closure series (#1829 through #1841) has converted many former emission paths to flow through the structured rewriter.

But because no slice explicitly recorded "legacy-emission paths" as closed (it was always a vague carry-forward), dropping it here is a soft tracker-hygiene transition rather than a tracked closure. If future work uncovers a specific legacy-emission path that bypasses SifrInt coercion, it should be added back as a residual gap. Worth flagging so a future maintainer doesn't assume the term was rigorously closed.

This isn't a blocker — the milestone-closure milestone's overall hygiene check can verify whether any concrete legacy-emission path remains.

### N2 — The "immediate function scope" qualifier is precise

This is the same load-bearing pattern used in PR #1838's "module-source recursive" qualifier and PR #1840's "non-recursive" qualifier — narrowing the closing bullet to precisely what was delivered while signaling the residual scope clearly. Future tracker writers can continue this pattern.

### N3 — Single-pass review entry pattern

PR #1843 landed in one review pass (the file is named `review-pass-1b` because of an internal retry naming convention, not because there were two distinct passes). The history entry correctly uses the single-pass format ("review satisfied") rather than the dual-pass format used for #1825 and #1827. ✓

### N4 — Convergence signal

After this slice, the open INT-1 follow-up has only **two remaining items**:
1. Nested helper lexical shadowing for outer locals that shadow exact-int module constants.
2. Unsupported augmented assignment / fallible `//` and `%`.

Both are narrow, specific gaps. Combined with the function-boundary sub-phase being fully closed (per #1842's tracker observation), INT-1 is now very close to closure. The next slice will likely target one of these two residuals, or a milestone-closure review will assess whether the remaining work is small enough to bundle.

### N5 — Validation notes adequate for tracker-only PR

`git diff --check` and `scripts/run_all_tests.sh --profile quick` are the canonical gates per AGENTS.md. The matching `report_signature` confirms no test deltas elsewhere. The cited `wall_time=75.44s` is in the normal range for tracker-only PRs.
