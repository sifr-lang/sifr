# Rust Interop `certification_3` Review — Round 2

Reviewer: agent (`medium`)

Result: **UNSATISFIED**

The first round-2 process was externally terminated before returning a verdict.
The completed replacement round inspected the revised diff without rerunning
the already-recorded long generated-build tests.

## Confirmed round-1 resolutions

agent confirmed the structured call-scoped type kind, top-level-only contract,
recoverable panic boundary, precise escape classifier, explicit async
rejection, mutable inner-argument rejection, storage/return/thread/signature
negative coverage, removal of fallback/truncation, corrected inventory, and
responsibility-based source splits.

## Remaining findings

1. **Medium:** `panic=abort` with an otherwise valid
   `Result[T, E | RustPanicError]` could still pass callback validation even
   though an abort profile cannot unwind a Sifr callback panic.
2. **Low-medium:** the now-rejected mutable inner callback argument still had
   contradictory temporary-mutation codegen.
3. **Low:** `mut callback: Callable[...]` on the outer Rust parameter silently
   downgraded the mutable-borrow convention.
4. **Low:** the checked-in scenario satisfied its panic token through README
   prose while executing only the safe mode.
5. **Low:** the owned Rust argument requirement for callback invocations was
   not documented.
6. **Low:** the probe split left the 886-line parent with the diagnostic
   classification responsibility still in place.

## Required disposition before round 3

- Reject `panic=abort` regardless of representable return type and add a
  focused contract test.
- Remove dead mutable-temporary codegen and reject mutable outer callback
  parameters.
- Execute safe, ordinary-error, and panic modes in the checked-in scenario.
- Document owned callback bridge arguments.
- Move the complete probe diagnostic classification chain into its focused
  sibling module.
