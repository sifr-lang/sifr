# Rust Interop Certification 4 Review — Round 8

Reviewer: Claude Opus 5 (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified the round-7 generated module-re-export evidence and all
non-audit milestone claims.

Blocking findings:

1. The unresolved-glob fallback rejected every two-segment `Type::new()` call,
   causing false diagnostics for ordinary constructors such as
   `HashMap::new()`.
2. Renamed runtime/task modules and bare `block_in_place` exported through a
   glob still bypassed the literal qualifier fallback.
3. The combined regression test asserted only aggregate violation categories,
   so already-detected cases masked failures in individual import shapes.

Correction wave:

- Added fixed-point propagation of same-file glob-visible runtime, task, Tokio
  crate, blocking-function, and type-alias bindings.
- Limited unresolved-glob constructor rejection to paths whose owner is
  literally `Builder` or `Runtime`, while rejecting `block_in_place` with any
  qualifier under an unresolved glob.
- Added a no-violation regression for `std::collections::*` with
  `HashMap::new()`, `String::new()`, and `Vec::new()`.
- Added independent exact assertions for super/crate globs, named relative
  re-exports, renamed runtime/task/Tokio modules, type aliases, and bare
  blocking functions.

Round 9 is required.
