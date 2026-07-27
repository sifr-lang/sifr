# Rust Interop Certification 4 Review — Round 5

Reviewer: Claude Opus 5 (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer confirmed the round-4 local-path and macro-trust corrections, the
runtime harness, generated negative ordering, inventories, and guardrails.

Blocking finding:

- Runtime and blocking aliases were collected during expression traversal,
  making the audit depend on `use` declaration order. Runtime/task glob
  imports and grouped `self` imports were also not recorded, allowing ordinary
  Rust import forms to bypass `SIFR-RUST-ASYNC-0001`.

Correction wave:

- Replaced traversal-time alias mutation with a module-scoped import pre-pass,
  so legal use-after-item declarations have the same policy result as
  declarations at the top of the module.
- Added explicit support for Tokio crate/runtime/task glob imports and grouped
  `self`/`self as alias` imports.
- Added regression coverage for use-after-item aliases, nested-module aliases,
  runtime/task/Tokio globs, grouped runtime `self`, grouped Tokio
  `self as alias`, and cross-module alias isolation.
- Changed the generated-build negative bridge to use a runtime glob declared
  after the function, binding the order-independent glob rejection to the
  merge suite.

Round 6 is required.
