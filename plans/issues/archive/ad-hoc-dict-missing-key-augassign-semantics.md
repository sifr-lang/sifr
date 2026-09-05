# Ad Hoc Issue: Dict Missing-Key Augassign Semantics

## Status

Complete on 2026-08-08 via implementation
[PR #3108](https://github.com/sifr-lang/sifr/pull/3108). The final reviewed
candidate was `b341b47f9f8e81baa0d7403979eb6551886e5568`; it merged as
`d54fd5f3b1fc8efbc4cde81479159a62073686c0`. Record-only archival and tracker
updates are carried by [PR #3109](https://github.com/sifr-lang/sifr/pull/3109).

This non-blocking correctness follow-up was discovered while diagnosing
[`ad-hoc-algorithmic-full-corpus-preexisting-failures.md`](../active/ad-hoc-algorithmic-full-corpus-preexisting-failures.md)
on 2026-07-29.

## Problem

An explicitly typed plain dictionary could compile a missing-key augmented
subscript assignment into a conditional update that silently did nothing:

```python
values: dict[int, int] = {}
values[1] += 1
```

The approved Python-compatible behavior is a checked `KeyError`. Silently
returning a wrong value violated Sifr's correctness guarantee.

## Completed Scope

- Plain-dictionary subscript augassign now carries its checked missing-key
  effect through HIR and generated Rust.
- A missing key returns `KeyError` through the active `try` result surface;
  outside a matching checked-error surface, lowering emits
  `SIFR-RESULT-0001`.
- Exact-key presence facts from membership guards and prior subscript writes
  suppress the checked effect, preserving statically proven-present updates.
- An explicitly typed `dict` initialized from `defaultdict(int)` retains the
  defaultdict semantic alias and factory-insertion behavior.
- Generated mapping lookups consistently borrow their key argument.

## Acceptance Criteria

- [x] Present-key plain-dict augassign preserves its result.
- [x] Missing-key plain-dict augassign returns the approved checked error and
      never silently no-ops or panics.
- [x] Annotated-`defaultdict` initialization cannot erase missing-key semantics
      and silently select the plain-dict conditional-update path.
- [x] Existing `defaultdict(int)` augassign continues inserting its factory
      default.
- [x] Focused compiler and build/run e2e tests cover the approved paths.
- [x] Local validation and iterative review are satisfied.

## Validation Evidence

- Focused lowering: 5 passed; complete `sifr_lowering`: 962 passed, 1 ignored.
- Focused augmented-subscript codegen: 13 passed; focused subscript assignment:
  19 passed; complete `sifr_codegen`: 969 passed.
- Native present/missing-key fixtures and the exact fail diagnostic fixture
  passed.
- The pinned `0438_find_all_anagrams_in_a_string` regression fixture checks
  successfully after exact-key flow suppression was added.
- Workspace Clippy, rustfmt, file-size, HIR maintainability, and diff-hygiene
  checks passed.
- The create-PR profile passed every selected functional area, including all
  19 Python-interop variants. Its Python aggregate took 639.010 seconds against
  the host-sensitive 600-second budget under concurrent Cargo contention.
- The final merge profile passed coverage, core, diagnostics, and all earlier
  selected checks, then one CPython differential fixture hit its fixed
  240-second Sifr timeout under concurrent worktree compilation. An immediate
  isolated replay passed both selected variants and all four cases; the timed
  out `bounded_int_arithmetic` case completed in 36.3 seconds. This known host
  variance is recorded in
  [`adhoc_performance_budget_host_variance.md`](../active/adhoc_performance_budget_host_variance.md);
  no threshold, baseline, waiver, or implementation source changed.

## Review Evidence

- [agent round 1](https://github.com/sifr-lang/sifr/pull/3108#issuecomment-5225915101)
  rejected candidate `5742d265a5abdb6a35d1c0460147c58950424bf0`
  because unconditional missing-key effects regressed the guarded pinned
  LeetCode 0438 fixture.
- Commit `b341b47f9f8e81baa0d7403979eb6551886e5568` added exact-key membership and
  prior-write flow suppression plus focused regressions.
- [agent round 2](https://github.com/sifr-lang/sifr/pull/3108#issuecomment-5225915170)
  reviewed the complete original-base-to-final-candidate implementation and
  returned `SATISFIED` with no blocking findings.
- The [final validation note](https://github.com/sifr-lang/sifr/pull/3108#issuecomment-5225972801)
  binds the merge-lane host-variance result and isolated replay to the exact
  reviewed SHA.

## Deferred Follow-up Work

Pre-existing mapping mutation and missing-key gaps found during review are
separately owned by
[`ad-hoc-mapping-missing-key-mutation-followups.md`](../active/ad-hoc-mapping-missing-key-mutation-followups.md).
They were not introduced by PR #3108 and did not change the satisfied verdict.
