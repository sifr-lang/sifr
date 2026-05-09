# Review: INT-1 SifrInt Local-Source Recursive Capture Body Tracker Pass 1

**Verdict: Satisfied. No blockers.**

## Findings

None.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1837 (`gh pr view 1837` → `MERGED 2026-05-06T20:04:38Z`).

### Review History (line 408)

```
- [x] INT-1 local-source recursive nested helper capture body review satisfied with non-blocking non-recursive capture follow-ups: `reviews/integer-model-int-1-sifrint-local-recursive-capture-body-review-pass-1.md`.
```

- File path resolves on disk (175 lines, present).
- Wording "satisfied with non-blocking non-recursive capture follow-ups" is **load-bearing precision** — directly references the pass-1 N-pass1-1 finding that the non-recursive captured-local case stays broken (signaling the residual scope to a future reader).
- The "local-source" prefix correctly distinguishes this slice from the prior "module-source" slice (PR #1835). The "recursive nested helper capture body" naming also matches PR #1837's commit message and review file naming.
- Position after the recursive-capture-params entry (line 407) and before the INT-2A entries (line 409) preserves chronological ordering.
- Single-pass entry pattern is consistent with prior single-pass slices.

### Sub-item closure (line 449)

```
- [x] Recursive nested helpers that capture outer locals already forced to `SifrInt` now propagate that exact-int state through nested return pre-scan, helper body lowering, recursive hidden capture arguments, and enclosing function return promotion, preserving shapes like `big: int = BIG_LIMIT + 1` followed by `return helper(2)`; review is satisfied and quick validation is passing: PR #1837.
```

Truthfulness checks:

- "**Recursive**" — load-bearing precision matching the slice's stated scope. Distinguishes from non-recursive (still open).
- "**capture outer locals already forced to `SifrInt`**" — accurate. The slice's [collect_sifr_int_captured_forced_locals](crates/sifr_codegen/src/function_emitter.rs:965-986) filters to `outer_forced_locals`, exactly this contract.
- "**propagate that exact-int state through**" — accurate description of the four-fold mechanism the implementation delivers and the pass-1 review verified in section 1:
  1. **"nested return pre-scan"** — `hir_function_returns_sifr_int_with_extra_forced` injects captured-forced into the helper's SifrInt-detection.
  2. **"helper body lowering"** — line 302 insert into inner `sifr_int_local_bindings` so the BinOp arm coerces references to the captured local.
  3. **"recursive hidden capture arguments"** — `lower_recursive_capture_arg_for_ir` (from PR #1835) routes through `coerce_expr_to_sifr_int_value` for registered locals → `big.clone()`.
  4. **"enclosing function return promotion"** — outer's `return helper(...)` is recognized as SifrInt-returning via the enriched `sifr_int_function_returns`, promoting outer's signature.
- "preserving shapes like `big: int = BIG_LIMIT + 1` followed by `return helper(2)`" — concrete example matches the e2e fixture's `returned_big_from_local_recursive_nested_helper`.
- "review is satisfied and quick validation is passing: PR #1837" — verified merged.

No overclaim:
- "**Recursive**" explicitly scopes — non-recursive captured-local case remains in the open follow-up below.
- "outer locals **already forced to `SifrInt`**" — matches the predicate's actual reach.
- Doesn't claim non-recursive captured-local fix.
- Doesn't claim full function parameter migration.

### Open follow-up (line 450)

Old (from PR #1836's tracker):
> "...captured-local-only nested helpers plus **local-source recursive capture body coercion** still need propagation through the broader function-boundary migration."

New:
> "...captured-local-only **non-recursive** nested helpers still need propagation through the broader function-boundary migration."

Diff effects:

- **Removed** "local-source recursive capture body coercion" — correctly removed because PR #1837 closes this. ✓
- **Modified** "captured-local-only nested helpers" → "captured-local-only **non-recursive** nested helpers" — the "non-recursive" qualifier precisely scopes the remaining gap. The recursive captured-local case is now closed; what remains is the non-recursive case (per pass-1 N-pass1-1).
- **Carried forward verbatim**: "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage", "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support", and "function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`".

The shift from "captured-local-only nested helpers plus local-source recursive capture body coercion" to "captured-local-only non-recursive nested helpers" is **semantically precise**:

- Before: two distinct gaps (non-recursive captured-local + recursive captured-local body coercion) bundled awkwardly.
- After: one gap (non-recursive captured-local), since the recursive variant is now closed.

This is exactly the right structural update.

Cross-referenced against pass-1 N-pass1-1 through N-pass1-4:

| Pass-1 finding | Tracker phrase | Captured? |
|---|---|---|
| **N-pass1-1** Non-recursive captured-local case stays broken | "captured-local-only **non-recursive** nested helpers still need propagation" | ✓ |
| **N-pass1-2** Module-source captured parameter dead code (carry-forward from PR #1835 N2) | (not captured) | Rust unused-variable warning shape, not user-facing failure. Reasonable to leave at review-file level. |
| **N-pass1-3** No focused unit tests | (not captured) | Test-hardening, not user-facing. |
| **N-pass1-4** Carry-forward open items | All carried forward verbatim | ✓ |

All user-facing remaining gaps are tracked. N-pass1-2 and N-pass1-3 correctly stay at the review-file level — consistent with the milestone's established pattern.

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 437 — correct, because the new follow-up at line 450 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — preserved: previous slices → recursive-capture-params (#1835) → **local-recursive-capture-body (this PR, #1837)** → broader-migration follow-up. Each completed sub-item sits before the open work that depends on it.
- **PR linkage** — `gh pr view 1837` returns `MERGED 2026-05-06T20:04:38Z` with title "Propagate local SifrInt recursive captures". Branch's `git log` shows implementation merge commit `49766ae4` immediately preceded by tracker-only commit `435a314b`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1835 and the pass-1 implementation review of #1837. Tracker-only diff preserves the signature, as expected.
- **No collateral churn** — diff is +2/-1 lines on a single file. No edits to architecture/roadmap docs, design doc, code, tests, or fixtures. Consistent with prior tracker-only PRs (#1818, #1820, #1822, #1824, #1826, #1828, #1830, #1832, #1834, #1836).

## Notes

(Non-blocking observations only.)

- **N1 — The closing bullet's four-fold mechanism enumeration is informative.** Listing "nested return pre-scan, helper body lowering, recursive hidden capture arguments, and enclosing function return promotion" maps directly to the pass-1 review's section 1 invariants. This makes the bullet diagnostically useful — a future regression would be locatable to one of these four mechanisms.

- **N2 — The "non-recursive" qualifier in the open follow-up is precise and durable.** It scopes the remaining gap to exactly what's left (non-recursive captured-local), which makes the next slice's scope easier to define. Future tracker writers might consider this pattern when a slice closes one variant but leaves a sibling variant open.

- **N3 — Pass-1 N-pass1-1's secondary observation is implicitly captured.** The pass-1 review noted that for non-recursive captured-local, the slice's pre-scan promotes outer's signature to `-> SifrInt` even though the closure body fails — creating a divergence between pre-scan promotion and lowering coverage. The open follow-up's "captured-local-only non-recursive" wording covers both halves of this gap (the closure body coercion AND the now-misleading outer signature promotion). One bullet suffices at tracker granularity. If a future slice closes only one half, the bullet can be split.

- **N4 — Pass-1 N-pass1-2 (dead-code captured parameter for module-source recursive helpers)** stays at the review-file level. It was already carried forward from PR #1835's pass-1 N2 and remains a Rust unused-variable warning shape that doesn't cause compile failures. Reasonable to leave at review-file granularity.

- **N5 — Single-pass review entry pattern.** PR #1837 landed in one review pass like the majority of INT-1 slices (#1817, #1819, #1821, #1823, #1829, #1831, #1833, #1835). The history pattern accommodates both single-pass and dual-pass entries.
