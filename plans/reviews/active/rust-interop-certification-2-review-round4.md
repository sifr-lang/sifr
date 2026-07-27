# Rust Interop Certification 2 — Review Round 4

## Reviewer

Claude Opus 5 (`--effort medium`), read-only full working-tree review against
base commit `f76a99046`.

## Verdict

`NEEDS REVISION`

## Confirmed remediation

The reviewer confirmed round-3 findings F2-F6 closed and F7 functionally
closed. It reproduced the focused runtime/codegen checks, all Rust interop
verification scripts, file-size guardrails, inventory counts, stale-example
sweeps, minimal scenario trust, and production-classifier coverage.

## Findings

1. **Medium/high — Rayon fan-out stderr leak.** CPU-offload setup used the
   shared boundary on the calling worker, but parallel per-item catches still
   used `std::panic::catch_unwind` on Rayon threads whose thread-local
   suppression depth was zero. Caught user panic payloads could therefore print
   to stderr.
2. **Low — runtime evidence did not assert stderr privacy.** The generated
   runtime scenario asserted exact stdout but did not assert that private
   target/mapper panic payloads were absent from stderr.
3. **Low — dead error-mapping arm.** The single-message Error arm was subsumed
   by the general declared-error mapping arm and its helper no longer changed
   behavior.

## Required follow-up

- Route each Rayon per-item catch through the shared silent runtime boundary
  and add cross-thread behavioral coverage.
- Assert generated scenario stderr excludes private panic payloads.
- Remove the dead mapping arm/helper, rerun full validation, and repeat Opus
  review.
