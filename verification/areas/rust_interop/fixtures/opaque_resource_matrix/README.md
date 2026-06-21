# opaque_resource_matrix

This fixture family tracks resource-shaped opaque handles for `reqwest`,
`rusqlite`, `tokio-postgres`, and `redis`.

- Positive evidence: `resource_close_aclose_matrix` remains planned for runtime
  fixtures proving close/aclose, cleanup, and service-local behavior with
  explicit loopback or local service configuration.
- Negative evidence: `invalid_resource_aliasing` remains planned for fixtures
  proving invalid borrowing, aliasing, and cleanup contracts fail before the
  row is listed as verified support.
- Compatibility category: `future-owned-by-separate-phase`. Basic opaque handle
  contracts and state transitions are verified; full resource ecosystem
  certification is not listed as verified support.
