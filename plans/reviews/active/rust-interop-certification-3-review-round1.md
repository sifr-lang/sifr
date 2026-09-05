# Rust Interop `certification_3` Review — Round 1

Reviewer: agent (`medium`)

Scope: call-scoped callback runtime implementation and evidence. The unrelated
parallel `opaque_resource_matrix` category-only edit was explicitly excluded.

Result: **UNSATISFIED**

## Findings

1. **High:** new callback contract code failed the authoritative Clippy gate.
2. **High:** callback-originated panics could escape declarations using
   `trusted_no_panic` or `panic=abort` without a recoverable outer boundary.
3. **High:** nested callback containers inherited parameter position and were
   accidentally classified as bridge-supported although codegen adapts only a
   top-level `Callable`.
4. **Medium-high:** every probe failure on a callback-bearing signature was
   misclassified as a callback lifetime failure.
5. **Medium:** the async guard filtered to `Function` decorators and could miss
   the explicit `@rust.async(...)` form.
6. **Medium:** mutable-borrow callback argument conventions wrote only to
   converted temporaries.
7. **Medium:** negative evidence asserted a fixed diagnostic message instead of
   the concrete rustc lifetime/thread failure, and did not execute return or
   thread escape variants.
8. **Low-medium:** call-scoped detection duplicated a rendered Rust type-name
   substring check.
9. **Low:** callback adapter lowering retained a silent fallback and zip
   truncation path.
10. **Low:** inventory counts depended on the incomplete parallel opaque-row
    promotion.
11. **Low:** two touched source files had only four to six lines of headroom
    under the 900-line cap.
12. **Low:** positive evidence and the checked-in scenario main duplicated the
    same verification body.

The reviewer confirmed that the borrowed runtime type itself is lifetime-sound
and genuinely non-`Send`/non-`Sync`, and that the documented callback
`Result` display-error mapping matches the generated implementation.

## Required disposition before round 2

- Enforce a synchronous, recoverable ordinary-error plus `RustPanicError`
  boundary for call-scoped callbacks.
- Keep callback support top-level only and reject mutable-borrow callback
  arguments.
- Classify only concrete rustc lifetime/thread escape failures as
  `SIFR-RUST-CB-0001`.
- Execute and pin storage, returned-deferred-call, and unmanaged-thread
  negative variants.
- Replace rendered-string detection with a structured type kind, remove
  fallback/truncation, correct inventory prose, split near-cap modules, and
  remove avoidable fixture duplication.
