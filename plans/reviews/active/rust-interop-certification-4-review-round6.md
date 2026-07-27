# Rust Interop Certification 4 Review — Round 6

Reviewer: Claude Opus 5 (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified the round-5 import pre-pass, supported glob and grouped
`self` forms, generated negative mutation, runtime evidence, inventories, and
guardrails.

Blocking finding:

- Intra-crate `super`/`crate` glob and named re-export forms could hide Tokio
  runtime constructors. A multi-thread runtime could then execute work without
  any syntactic `block_on`, and direct Tokio `type` aliases were also missed.

Correction wave:

- Made unresolved relative and non-Tokio globs fail closed for bare
  `Builder`/`Runtime` constructors.
- Recognized relative `Builder`/`Runtime`, runtime/task module, and blocking
  function re-exports.
- Added an order-independent type-alias pass, including alias chains.
- Added regression coverage for `super::*`, `crate::*`,
  `super::Builder as B`, a direct Tokio type alias, and a multi-thread runtime
  using `spawn`/`shutdown_timeout` without `block_on`.
- Documented the conservative unresolved-glob behavior in internal, public,
  and fixture documentation.

Round 7 is required.
