# Rust Interop Certification 4 Review — Round 12

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified the round-11 qself and fail-closed parsing corrections,
the runtime and trust evidence, inventories, and all guardrails.

Blocking finding:

- Async probes materialized borrowed arguments from divergent `loop {}` values,
  allowing their inferred lifetimes to become `'static`. A static-only async
  Rust target therefore passed the probe even though generated glue could not
  supply its required lifetime.

Correction wave:

- Replaced divergent local argument values with parameters on an uncalled
  probe function.
- Bound borrowed and mutable-borrowed parameter references to an explicit
  caller-chosen probe lifetime while leaving owned parameters unchanged.
- Added a focused negative Cargo-probe test proving an async
  `&'static str` target is rejected with `SIFR-RUST-TYPE-0001`.
- Updated the architecture probe example to show the caller-lifetime-bound
  shape.

Round 13 is required.
