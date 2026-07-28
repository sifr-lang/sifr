# Rust interop certification 6 review — round 5

Date: 2026-07-28

Reviewer: Claude Opus 5 (`--effort medium`)

Reviewed head: `2ab9b244e`

Verdict: **NOT SATISFIED**

The reviewer independently reproduced every round-1 through round-4 scenario
and confirmed all prior findings closed, including handler binding moves
through loops, branches, and exception flow plus declaration-time capture
snapshots. Both mandatory generated packages, 1,823 focused unit tests,
Clippy, formatting, file-size and maintainability guards, all Rust-interop
checks, and documented inventory counts passed.

## New findings

1. **Medium:** an annotated local initialized from an attribute read or method
   call can be recorded as `Unknown` in the nested capture environment. A
   routine valid `str` capture is then falsely rejected as not clone-capable,
   exposing the internal `Unknown` type in `SIFR-RUST-CB-0001`.
2. **Low:** a handler that mutates a `nonlocal` capture requires Rust `FnMut`,
   but the retained bridge requires `Fn`. Direct and transitive sibling
   mutation pass Sifr checking and reach raw rustc `E0525`.

## Required outcome

Restore capture type fidelity for attribute- and method-derived locals, and
give genuinely unresolved captures an explicit contract diagnostic. Reject
direct and transitive mutating retained captures before Cargo with
`SIFR-RUST-CB-0001`, add lowering and generated-build evidence, qualify the
docs, run the authoritative gate, and repeat the exact Opus review.

## Resolution

Retained attachment refreshes an inference-time unknown capture from its
lowered lexical binding, preserving concrete types for locals initialized from
attribute reads and method calls. A capture that remains unresolved now fails
with an explicit unverifiable-type contract message instead of surfacing the
internal type as a clone error. Nested-function inference also records
actually mutated `nonlocal` captures. Validation rejects both direct and
sibling-transitive mutation as requiring `FnMut` where the retained bridge
requires `Fn`. Focused lowering tests cover both directions; the mandatory
positive generated package captures attribute- and method-derived strings, and
the mandatory negative package adds direct and transitive mutation.
