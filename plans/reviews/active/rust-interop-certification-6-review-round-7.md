# Rust interop certification 6 review — round 7

Date: 2026-07-28

Reviewer: Claude Opus 5 (`--effort medium`)

Reviewed head: `f4403d144`

Verdict: **NOT SATISFIED**

The reviewer independently confirmed every round-1 through round-6 finding
closed, ran the real fixture, compatibility, tier, claim, stale-draft, and full
Rust-interop area gates, re-ran both mandatory generated packages, and
reproduced the documented lifecycle and ownership behavior.

## New findings

1. **High:** capture and mutation collection stopped at a function declared
   inside the retained handler. Mutation used only in that inner function
   passed `sifr check` and reached rustc `E0525`; `NonSend` state read only
   there passed both check and build because the marker has no Rust backstop.
   Direct and sibling-helper variants both escaped.
2. **Medium:** the name-only mutating-method list treated `RwLock.write()` as
   requiring `FnMut`, falsely rejecting the synchronization wrapper explicitly
   sanctioned by the share-safety contract.
3. **Medium:** a walrus assignment to a declared `nonlocal` emitted a shadowing
   Rust `let` and silently left the outer binding unchanged.
4. **Low:** public and internal claims did not qualify the two gaps above.

## Resolution

Nested-function capture collection now propagates free names through each
inner function while removing that function's parameters and local bindings.
Mutation collection descends through the same scope boundaries with a
scope-filtered map of actual capture types, so direct and sibling-transitive
inner functions cannot hide mutation or `NonSend` state. Shadowing regressions
prove same-named parameters and locals do not create false captures.

Mutating-method classification now delegates to the receiver-type-aware
collection contract. `RwLock.write()` therefore remains an `Fn` operation,
while typed list, dict, set, and buffer mutation remains rejected. The
mandatory positive package compiles a retained `RwLock.write()` handler, and
the mandatory negative package adds direct and sibling variants for both inner
mutation and inner `NonSend` reads.

Walrus lowering explicitly rejects rebinding a declared `nonlocal` with
`SIFR-FLOW-0003`, preventing the prior wrong-answer Rust emission. Focused
tests and the docs pin all three outcomes.

Post-remediation validation also caught and closed a recursive-helper
inference regression: unresolved function parameters retain conservative
collection-method inference, while callback captures and other resolved
bindings use exact receiver types. The affected lowering, codegen, and driver
crate suites, the full Rust-interop area, workspace Clippy, formatting,
maintainability, file-size, and diff checks all pass.
