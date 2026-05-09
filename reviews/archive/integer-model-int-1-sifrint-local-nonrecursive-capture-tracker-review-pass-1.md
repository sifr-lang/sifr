# Review: INT-1 SifrInt Local-Source Non-Recursive Capture Body Tracker Pass 1

**Verdict: Satisfied. No blockers.**

## Findings

None.

The +2/-1 tracker diff at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) faithfully reflects merged PR #1839 (`gh pr view 1839` → `MERGED 2026-05-06T20:26:21Z`).

### Review History (line 409)

```
- [x] INT-1 local-source non-recursive nested helper capture body review satisfied with non-blocking broader function-boundary follow-ups: `reviews/integer-model-int-1-sifrint-local-nonrecursive-capture-body-review-pass-1.md`.
```

- File path resolves on disk (157 lines, present).
- Wording "satisfied with non-blocking broader function-boundary follow-ups" matches the pass-1 verdict ("Satisfied with non-blocking suggestions" with N1 capture-detection narrowness, N2 line-316-318 redundancy on recursive path, N3 missing unit tests, N4 carry-forwards — all non-blocking).
- The "local-source non-recursive" prefix correctly distinguishes this slice from PR #1837's "local-source recursive" slice (line 408) and from PR #1835's "module-source recursive" slice (line 407). The naming maintains the milestone's matrix of (source × recursion) labels.
- Position after the local-recursive-capture-body entry (line 408) and before the INT-2A entries (line 410) preserves chronological ordering.
- Single-pass entry pattern is consistent with prior single-pass slices.

### Sub-item closure (line 451)

```
- [x] Non-recursive nested helper closures that capture outer locals already forced to `SifrInt` now propagate that exact-int state through nested return analysis and closure body lowering, preserving shapes like `big: int = BIG_LIMIT + 1` followed by a captured-local `helper()` call; review is satisfied and quick validation is passing: PR #1839.
```

Truthfulness checks:

- "**Non-recursive nested helper closures**" — load-bearing precision matching the slice's stated scope. Distinguishes from the recursive case (closed in PR #1837).
- "**capture outer locals already forced to `SifrInt`**" — accurate. The slice's [function_emitter.rs:233](crates/sifr_codegen/src/function_emitter.rs:233) uses `collect_sifr_int_captured_forced_locals(func, &outer_forced_locals)` which filters to outer's forced set.
- "**propagate that exact-int state through**" — accurate description of the two-fold mechanism the implementation delivers and the pass-1 review verified:
  1. **"nested return analysis"** — `hir_function_returns_sifr_int_with_extra_forced` is now fed `sifr_int_nested_capture_bindings` (union of recursive_captures and captured_forced_locals), so non-recursive helpers' SifrInt-detection sees captured outer SifrInt locals.
  2. **"closure body lowering"** — line 316-318 unconditionally extends `sifr_int_local_bindings` with captured-forced names so the BinOp arm coerces references in the closure body.
- "preserving shapes like `big: int = BIG_LIMIT + 1` followed by a captured-local `helper()` call" — concrete example matches the e2e fixture's `returned_big_from_local_nested_helper`.
- "review is satisfied and quick validation is passing: PR #1839" — verified merged.

The two-mechanism enumeration is appropriately scoped for the non-recursive case. PR #1837's recursive-case bullet enumerated four mechanisms (pre-scan, body lowering, recursive hidden capture arguments, enclosing function return promotion) because recursive helpers have additional surfaces. Non-recursive closures have fewer mechanisms (no LocalFn capture argument re-passing), so two is the right count.

No overclaim:
- "**Non-recursive**" explicitly scopes — recursive is correctly closed in the prior slice.
- "outer locals **already forced to `SifrInt`**" — matches the predicate's actual reach (forced-only per pass-1 N1).
- Doesn't claim function parameter migration.

### Open follow-up (line 452)

Old (from PR #1838's tracker):
> "...function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`, and **captured-local-only non-recursive nested helpers still need propagation through the broader function-boundary migration**."

New:
> "...function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`."

Diff effects:

- **Removed** "captured-local-only non-recursive nested helpers still need propagation through the broader function-boundary migration" — correctly removed because PR #1839 closes this. ✓
- **Carried forward verbatim**:
  - "lexical shadowing and legacy-emission paths need scope-safe exact-int coverage"
  - "unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support"
  - "function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`"

Cross-referenced against pass-1 N1-N4:

| Pass-1 finding | Tracker phrase | Captured? |
|---|---|---|
| **N1** Captured-forced detection only checks `outer_forced_locals` (narrower than `recursive_capture_lowers_to_sifr_int`'s three-source check) | (not captured) | In practice forced and registered are correlated; hard to construct a real reproducer. Reasonable to leave at review-file level. |
| **N2** Line 316-318 extend redundant for recursive path (HashSet dedupes) | (not captured) | Code-shape redundancy, harmless. Reasonable to leave at review-file level. |
| **N3** No focused unit tests for the new mechanism | (not captured) | Test-hardening, not user-facing. |
| **N4** Carry-forward open items | All carried forward verbatim | ✓ |

All user-facing remaining gaps are tracked. N1, N2, and N3 correctly stay at the review-file level — consistent with the milestone's established pattern of leaving non-user-facing polish out of tracker bullets.

### Notable structural observation: the capture/closure series is now fully closed

After this slice, the open INT-1 follow-up contains **no remaining capture or closure items**. The four closure/capture series sub-items are all closed:

| Sub-slice                                          | PR    | Status |
|----------------------------------------------------|-------|--------|
| Module-source recursive nested helper capture      | #1835 | ✓ closed |
| Local-source recursive nested helper capture body  | #1837 | ✓ closed |
| Local-source non-recursive nested helper capture body | #1839 | ✓ closed |
| Module-source non-recursive nested helper return propagation | #1833 | ✓ closed |

The remaining open items (lexical shadowing, legacy-emission paths, unsupported AugAssign / fallible `//` and `%`, function argument expressions) are genuinely non-capture concerns belonging to other parts of the broader function-boundary migration. This is appropriate signaling that the closure/capture sub-phase of INT-1 has converged.

### Cross-checks

- **Top-level INT-1 line stays `[ ]`** at line 439 — correct, because the new follow-up at line 452 is unchecked. No spurious milestone closure.
- **Sub-item ordering** — preserved: previous slices → recursive-capture-params (#1835) → local-recursive-capture-body (#1837) → **local-nonrecursive-capture-body (this PR, #1839)** → broader-migration follow-up. Implementation order maintained.
- **PR linkage** — `gh pr view 1839` returns `MERGED 2026-05-06T20:26:21Z` with title "Propagate local SifrInt captures into closures". Branch's `git log` shows implementation merge commit `ef63f4c5` (squashed) immediately preceded by tracker-only commit `62c07b75`. PR number consistent.
- **Validation reproduction** — `report_signature=e1bf653aaa770517` is identical to the signatures recorded across #1817–#1837 and the pass-1 implementation review of #1839. Tracker-only diff preserves the signature.
- **No collateral churn** — diff is +2/-1 lines on a single file. Consistent with prior tracker-only PRs (#1818, #1820, #1822, #1824, #1826, #1828, #1830, #1832, #1834, #1836, #1838).

## Notes

(Non-blocking observations only.)

- **N1 — The closing bullet's two-mechanism enumeration is appropriately scoped.** Listing "nested return analysis and closure body lowering" maps to the implementation review's two-fold mechanism. PR #1837's recursive-case bullet had four mechanisms; this slice's non-recursive case correctly has fewer because closures have fewer surfaces (no recursive LocalFn capture argument re-passing).

- **N2 — The "non-recursive" qualifier in the closing bullet is precise.** Combined with PR #1837's "recursive" qualifier, the two adjacent bullets clearly distinguish the (recursive vs non-recursive) × (local vs module) matrix that INT-1 has been working through.

- **N3 — Pass-1 N1 (narrower captured-forced detection)** stays at the review-file level. In practice, forced and registered states are correlated for SifrInt locals, so the narrower detection is a theoretical concern without a current reproducer. Reasonable to leave at review granularity until a real shape surfaces.

- **N4 — Pass-1 N2 (line 316-318 redundancy on recursive path)** stays at the review-file level. HashSet dedupes, so the redundancy is harmless. Pure code-shape polish.

- **N5 — Single-pass review entry pattern.** PR #1839 landed in one review pass like the majority of INT-1 slices (#1817, #1819, #1821, #1823, #1829, #1831, #1833, #1835, #1837). The history pattern accommodates both single-pass and dual-pass entries.

- **N6 — Capture/closure sub-phase converged.** After this slice, the open INT-1 follow-up has no remaining capture or closure items. The remaining work (lexical shadowing, legacy-emission, AugAssign/fallible arithmetic, function argument expressions) is genuinely non-capture territory. The next slice will likely be in one of those areas, signaling INT-1's progression beyond the closure-capture sub-phase.
