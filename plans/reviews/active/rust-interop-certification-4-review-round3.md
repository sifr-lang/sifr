# Rust Interop Certification 4 Review — Round 3

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer confirmed all round-1 and round-2 runtime, trust, root-union,
order-independence, caching, predicate, provenance, mutation-test, count, and
guardrail fixes. It independently ran 149 focused tests, all three generated
async reqwest tests, hostile-proxy execution, Clippy, formatting, and area
checks.

Blocking findings:

1. `tokio::task::block_in_place` detection did not recognize a renamed Tokio
   crate or imported `tokio::task` module.
2. A direct `tokio::runtime::Builder` import made a fully qualified
   `std::thread::Builder::new()` look like a Tokio constructor.
3. Local `block_on` collection covered only file-level functions, so a valid
   helper inside an inline module was rejected.

Correction wave:

- Added Tokio task-module aliases and reused renamed Tokio crate roots for
  `block_in_place` detection.
- Limited imported runtime-type constructor matching to a bare two-segment
  `Alias::constructor` path, preventing qualified unrelated builders from
  inheriting the alias.
- Collected local functions recursively by inline-module path and tracked the
  active module during expression visitation.
- Added regression coverage containing both an imported Tokio `Builder` and
  `std::thread::Builder`, a local module-scoped `block_on`, imported task-module
  calls, and renamed-Tokio task calls.

Round 4 is required.
