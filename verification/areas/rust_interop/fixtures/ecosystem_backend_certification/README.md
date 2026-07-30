# ecosystem_backend_certification

This fixture family tracks backend and service-oriented ecosystem probes for
`axum`, `tower-http`, and `sqlx`.

- Positive evidence: `backend_probe_coverage` builds an exact-pinned generated
  package, executes an Axum loopback on `127.0.0.1:0`, observes a response
  header installed by tower-http, compiles a SQLx query macro from checked-in
  `.sqlx/` metadata, and shuts the server down deterministically.
- Negative evidence: `sqlx_without_offline_artifacts` independently removes
  and stale-mutates the query metadata. The fixture supplies no SQLx offline
  environment override: Sifr forces `SQLX_OFFLINE=true`, removes inherited
  `DATABASE_URL`, includes the complete metadata directory in cache identity,
  reports `SIFR-RUST-CARGO-0001`, and never connects to the armed database
  sentinel.
- Compatibility category: `supported-through-bridge`. This exact bridge and
  crate graph are certified; product-level web framework workflows and
  arbitrary framework APIs remain outside this claim.
