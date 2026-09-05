# Rust Interop Certification 4 Review — Round 11

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified the round-10 scoped unresolved-glob policy, same-file
fixed-point resolver, precision regressions, runtime evidence, inventories, and
guardrails.

Blocking findings:

1. Qualified self-type calls such as
   `<tokio::runtime::Builder>::new_multi_thread()` discarded the qself owner
   path and bypassed constructor detection.
2. Read or parse failures for `.rs` files under audited roots silently skipped
   the source policy.

Correction wave:

- Added qself-aware expression path extraction that resolves inherent
  constructors through direct Tokio types and same-file aliases.
- Added exact qualified-path cases for Tokio `Builder`, Tokio `Runtime`, a
  runtime type alias, and non-Tokio `std::thread::Builder`.
- Made audited-source read and parse failures emit an
  `unauditable bridge source` violation through `SIFR-RUST-ASYNC-0001`.
- Added a malformed-source regression for the fail-closed parsing path.

Round 12 is required.
