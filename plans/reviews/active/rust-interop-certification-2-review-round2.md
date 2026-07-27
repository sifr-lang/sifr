# Rust Interop Certification 2 — Review Round 2

## Reviewer

Claude Opus 5 (`--effort medium`), read-only full working-tree review against
base commit `f76a99046`.

## Verdict

`NEEDS REVISION`

## Confirmed remediation

The reviewer confirmed that all eight round-1 findings were structurally fixed
and independently reproduced the focused compiler/driver tests, both ignored
generated-build integration tests, and the Rust interop verification area.
It also confirmed panic containment, mapper-panic fallback, ordinary `Err`
preservation, mapper probing in the mapper crate's Cargo context, cache
identity, generated Rust validity, evidence provenance, matrix consistency,
inventory counts, scope discipline, and the capability-based
`panic_wrapper_runtime` scenario name.

## Findings

1. **Blocking — stale async examples.** Public examples in
   `docs/rust-interop.mdx` and `docs/guides/interop/reqwest.mdx` still combined
   `async def` with `panic=map_error`, even though this milestone intentionally
   rejects async mapper policies until async wrapper execution is certified.
2. **Blocking — stale architecture example.**
   `internal_docs/rust_interop_architecture.md` still described `map_error` as
   a substitute for a declared `RustPanicError` fallback and showed a now
   rejected example.
3. **Lower severity — mapper diagnostic classification.** A missing mapper
   target could be reported as a signature mismatch because mapper-probe
   diagnostics used a broad stderr heuristic.
4. **Lower severity — nested-call hook deadlock.** Activating
   `catch_rust_panic` in generated glue exposed a latent non-reentrant global
   panic-hook mutex, which could deadlock once nested callback execution
   becomes reachable.
5. **Observation — exact file cap.** `crates/sifr_driver/src/build/rust_interop.rs`
   is exactly 900 lines and must not grow further without decomposition.

## Required follow-up

- Update every stale public/internal example to match the certified synchronous
  mapper boundary and explicit async fallback contract.
- Distinguish missing mapper paths from invalid mapper signatures.
- Make panic-hook containment safe for nested use, or record and enforce a
  certified boundary that prevents nested entry.
- Re-run focused, generated runtime/negative, evidence, guardrail, and reviewer
  validation.
