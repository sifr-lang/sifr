# Rust Interop Certification 4 Review — Round 4

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer confirmed the runtime-reuse, cancellation-cleanup, offline
loopback, native-link trust, async-probe lifetime, diagnostic-ordering,
inventory, and guardrail claims. It independently ran 149 focused unit tests,
the three ignored generated async-reqwest tests, the Rust interop area checks,
Clippy, formatting, and both source guardrails.

Blocking findings:

1. Qualified calls to demonstrably local `block_on` helpers were rejected
   because the local-function exemption applied only to bare calls.
2. The architecture document incorrectly suggested Cargo probing mitigated
   macro-generated runtime construction even though macros are outside the
   source audit and probing does not constrain their bodies.

Correction wave:

- Added source-file module-path derivation and path-aware local-function
  resolution for relative, `self`, `super`, and `crate` call paths.
- Added regression coverage for `helpers::block_on()`, `self::block_on()`, and
  `crate::bridges::local_helper::block_on()`.
- Corrected the architecture claim: macro-expanded and
  attribute-macro-generated runtime construction is not detected by Cargo
  probing and remains governed only by the declared package trust contract.

Round 5 is required.
