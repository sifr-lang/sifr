# Rust Interop Certification 4 Review — Round 9

Reviewer: agent (`--effort medium`)

Verdict: **NOT SATISFIED**

The reviewer verified the round-8 fixed-point glob propagation, exact
per-shape assertions, unrelated collection constructors, runtime evidence, and
all non-audit milestone claims.

Blocking findings:

1. Named and renamed uniform-path imports were not resolved against same-file
   module bindings, allowing runtime/task/type aliases to bypass the audit.
2. Clearly external `std::thread::*` was treated as an unresolved intra-crate
   glob, falsely rejecting `Builder::new()`.

Correction wave:

- Added named/renamed import collection and fixed-point propagation from
  same-file source modules.
- Recognized uniform paths that name declared in-file modules as intra-crate
  while leaving unknown external crate globs outside the conservative
  fallback.
- Tracked declared external modules so explicit intra-crate module globs can
  still fail closed when their source file is not available to the per-file
  resolver.
- Removed the earlier literal relative-name shortcut in favor of actual symbol
  propagation.
- Added exact tests for uniform runtime modules, renamed types, renamed runtime
  modules, uniform blocking functions, and `std::thread::*` with
  `Builder::new()`.

Round 10 is required.
