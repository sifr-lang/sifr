# Rust Interop `certification_3` Review — Round 3

Reviewer: agent (`medium`)

Result: **UNSATISFIED**

## Confirmed round-2 resolutions

agent confirmed explicit `panic=abort` rejection, removal of contradictory
mutable-inner codegen, scenario execution of safe/error/panic modes, owned
argument documentation, and the completed probe diagnostic split.

## Remaining findings

1. **Medium:** abort policy was checked only on the first declaration for a
   target, so sibling decorator ordering could bypass it.
2. **Medium:** an ambient selected Cargo `panic = "abort"` profile could bypass
   the source-policy check.
3. **Low-medium:** source `mut callback: Callable[...]` normalized ownership to
   `Own`, erasing mutability before the callback validator; the synthetic
   contract mutation test did not prove the source path.

## Required disposition before round 4

- Aggregate effective abort policy across every declaration for a canonical
  target.
- Reject any call-scoped callback target whose selected release Cargo profile
  uses abort.
- Preserve owned-mutability in the structured bridge parameter convention and
  reject it from a real source-level `mut callback` test.
