# Rust Interop Certification 2 — Review Round 5

## Reviewer

agent (`--effort medium`), read-only full working-tree review against
base commit `f76a99046`.

## Verdict

`NEEDS REVISION`

## Confirmed remediation

The reviewer confirmed every round-4 finding closed. It independently built
and ran parallel map/try-map/pool, CPU spawn, task-group, and join-set panic and
ordinary-error cases with zero panic output; reproduced 15 runtime interop
tests, 47 codegen Rust interop tests, 136 driver Rust interop tests, all 43
mandatory generated-build tests, the 10-variant Rust interop area, raw-code and
all maintainability/dependency guardrails.

## Findings

1. **Medium — stale passing fixture.**
   `direct_crate_matrix/positive/compatible_direct_signatures.sifr` still used
   the newly rejected wrapper-only `Result[str, RustPanicError]` shape.
2. **Medium — async containment over-claim.** Public async examples correctly
   removed mapper policies but described their `RustPanicError` union member as
   an active panic fallback even though async generated catch wrappers remain
   future-owned.
3. **Low — parallel hot-path contention.** Every Rayon item entered/exited the
   global hook-state mutex, adding avoidable synchronization for cheap work.
4. **Low — stderr diagnostic scope.** Mapper classification could combine the
   assertion marker from one compiler error with `mismatched types` from an
   unrelated error elsewhere in the same stderr.
5. **Informational — dependency registration.** The worker helper's
   `sifr_runtime` reference sits in a verbatim statement that import scanning
   does not inspect; dependency registration currently arrives indirectly.

## Required follow-up

- Give the direct UUID fixture distinct ordinary and panic error members.
- State accurately that async declarations reserve the type surface but do not
  yet certify generated panic containment.
- Remove per-item hook-state contention while retaining worker-thread
  suppression, scope mapper diagnostics to the relevant compiler block, and
  register the generated runtime dependency explicitly.
- Re-run full validation and repeat agent review.
