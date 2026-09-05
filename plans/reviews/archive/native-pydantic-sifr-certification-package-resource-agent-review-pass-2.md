# Native Pydantic-Sifr Package Resource Certification — agent Pass 2

Exact base: `00ea8867569884943413f7809414839c8992db97`

Exact candidate: `d6133a976b70fdccf196938698ca55db2e64efb2`

## Verdict

`SATISFIED`

## Closed blockers

- Bridge-local aliases now share `Rc<RefCell<ResourceState>>` lifecycle state,
  so the negative path observes access rejection after close and stable second
  close behavior.
- Lowering now rejects direct construction of a Rust-opaque resource with
  stable `SIFR-RUST-TYPE-0001` ownership and a focused unit test.
- The generated-build test asserts exactly two negative diagnostics with exact
  code ownership instead of lexical matching.
- Compiler-rejection mutations now live in the one registered negative
  evidence source. The unregistered file and its false headers are gone.
- Public and durable docs now describe only executable evidence.

The reviewer also confirmed genuine close observation, the deliberate shape
mismatch probe, panic redaction, generated bridge signature compatibility,
mandatory merge-profile provenance, scenario lock validity, all recomputed
inventory counts, the 900-line guardrail, and the absence of compatibility,
fallback, legacy, or versioned active paths.

## Non-blocking hardening selected for the final candidate

- Cover or rule out constructor calls through a Sifr type alias.
- Keep the fixture-local Rust bridge rustfmt-clean and restore module ordering.
- Use native validation to prove the return-only structural type parameter path.
