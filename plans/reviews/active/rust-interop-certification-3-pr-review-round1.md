# Rust Interop Certification 3 PR Review Round 1

PR: [#3033](https://github.com/sifr-lang/sifr/pull/3033)

Reviewer: Claude Opus 5, medium effort

Verdict: `NOT SATISFIED`

The exact committed diff at `d1287e97f` was reviewed with the unrelated
unstaged `opaque_resource_matrix` promotion excluded.

Actionable findings:

1. High: the generic `CallScopedCallbackBridge::new` constructor did not give
   rustc enough expected-type information to infer closure tuple parameters
   before generated `int` and composite conversions invoked methods on them.
2. Medium: generated-build evidence compiled only a `(String,)` callback and
   did not cover exact integers, nested collections, optionals, or
   multi-argument conversion.
3. Medium: Sifr assertion payload and location information inside a callback
   is suppressed by the silent outer Rust panic boundary, but this limitation
   was not documented.

Non-blocking observations:

- negative evidence intentionally pins concrete rustc reason text and will
  require maintenance when the toolchain changes;
- exact-integer conversion had a redundant special case in
  `callback_handler_arg`.
