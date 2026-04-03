## Review Summary

Pass 1 completed via per-file external review prompts after the larger batch prompt path proved unreliable in this workspace. No actionable issues were reported for any of the three companions.

## `demos/env/idiomatic.rs`

No actionable findings.

Non-blocking notes:
- `env_get` uses `then(...).flatten()` instead of a more direct conditional shape, but the behavior is correct.
- `assert_bool_vector_eq` is a thin wrapper over `assert_eq!`, but it does not affect semantics.

## `demos/regex/idiomatic.rs`

No actionable findings.

Non-blocking notes:
- The local flag constants use `i64`, which is unconventional relative to the underlying crate, but harmless in this demo-local surface.
- The error path in `collect_primary_actual` intentionally collapses failures into the boolean result vector instead of surfacing details.

## `demos/regex_and_filesystem/idiomatic.rs`

No actionable findings.

Non-blocking notes:
- `main` returns `Box<dyn std::error::Error>` while the inner closures use concrete `RegexError` and `IOError` types; this is acceptable for the demo.
- `rglob` returns traversal order rather than a separately resorted result vector, but the demo only asserts count, not order.
