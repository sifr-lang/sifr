# opaque_resource_matrix

This fixture family tracks resource-shaped opaque handles for `reqwest`,
`rusqlite`, `tokio-postgres`, and `redis`.

- Positive evidence: `resource_close_aclose_matrix` is executed by
  `test_build_opaque_resource_lifecycle_runtime`. Generated glue transfers an
  opaque handle through borrowed operation and owned `close=async_close`
  member bridges on the current-thread runtime, performs HTTP, SQLite, RESP, and PostgreSQL
  operations, closes the handle twice with a stable
  `closed`/`already-closed` result, removes the temporary database, exercises
  the runtime poison guard, and observes zero live harness-owned tracked tasks.
- Negative evidence: `invalid_resource_aliasing` is executed by
  `test_build_opaque_resource_alias_rejection_runtime`. Its distinct generated
  bridge path first operates on all four live resources, closes the original,
  retries through a bridge-local shared alias, and classifies the resulting
  `resource-state=closed` rejection. No Sifr-level clone policy is claimed.
- Compatibility category: `supported-through-bridge`. The locked
  `resource_lifecycle_runtime` scenario uses only ephemeral loopback listeners,
  a unique temporary SQLite path, bounded operations, and checked-in offline
  Cargo resolution. Redis library-metadata `CLIENT SETINFO` is disabled, so the
  RESP harness covers only the exercised connection and `PING` frames.
