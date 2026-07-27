# Rust Interop Certification 2 — Review Round 3

## Reviewer

Claude Opus 5 (`--effort medium`), read-only full working-tree review against
base commit `f76a99046`.

## Verdict

`NEEDS REVISION`

## Confirmed remediation

The reviewer independently reproduced runtime, codegen, driver, verification,
and Clippy validation. It confirmed the mapper diagnostic classifier, isolated
panic-hook reentrancy design, file caps, matrix inventories, evidence
provenance, generated Rust shape, cache identity, and capability-based scenario
name.

## Findings

1. **Medium — cross-feature panic-hook leak.** Generated CPU-offload glue still
   swapped and restored the global hook independently. It could permanently
   displace the once-installed interop forwarding hook and expose later bridge
   panic payloads on stderr.
2. **Medium — stale architecture declaration.** An earlier architecture example
   still used synchronous `panic=map_error` without the mandatory
   `RustPanicError` fallback member.
3. **Medium — stale async fixture corpus.** Fifteen positive/planned/example
   fixture sources still combined `async def` with the newly rejected
   `panic=map_error` policy.
4. **Low — phase tracker stale.** `plans/phases/39_rust_interop.md` still called
   generated panic wrapper emission and mapper fallback future work.
5. **Low — test proxy classifier.** Driver test support fabricated panic
   classification from a string heuristic instead of exercising the production
   nominal, alias-aware classifier.
6. **Low — unnecessary trust grant.** The runtime scenario granted and required
   unsafe-bridge trust although its bridge contains no unsafe Rust.
7. **Low — error ancestry guard.** Single-message bridge-error mapping applied
   to any one-field class instead of requiring Error ancestry.

## Required follow-up

- Converge CPU-offload and interop panic suppression on one reentrant runtime
  mechanism and add cross-feature regression coverage.
- Sweep all architecture and fixture declarations for the certified sync/async
  boundary.
- Update phase tracking, replace the test proxy with production classification,
  remove unnecessary trust, and narrow error mapping to error ancestry.
- Re-run all focused/generated/evidence/guardrail checks and another full Opus
  review.
