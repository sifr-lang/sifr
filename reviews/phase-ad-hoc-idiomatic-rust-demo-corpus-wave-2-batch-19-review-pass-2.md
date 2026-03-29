## Review Summary

Pass 2 completed via the same per-file external review path as pass 1. No actionable issues were reported for any of the three companions.

## `demos/env/idiomatic.rs`

No actionable findings.

Non-blocking notes:
- `assert_bool_vector_eq` could be inlined into `assert_eq!`, but the current helper is harmless.
- `getenv_opt` is a small alias over `env_get`, which is acceptable for parity with the demo surface.

## `demos/regex/idiomatic.rs`

No actionable findings.

Non-blocking notes:
- The catch-all error branch in `collect_primary_actual` intentionally collapses failures into boolean outcomes for the demo harness.
- The local regex flag constants remain a demo-local convention rather than a crate-driven type surface.

## `demos/regex_and_filesystem/idiomatic.rs`

No actionable findings.

Non-blocking notes:
- Non-UTF-8 filenames are filtered out by `to_str()` rather than surfaced, which is acceptable for the current demo assumptions.
- The `/tmp` demo path remains environment-specific by design and matches the paired Sifr demo behavior in this workspace.
