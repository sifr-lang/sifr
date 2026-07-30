# ecosystem_backend_certification

This fixture family tracks backend and service-oriented ecosystem probes for
`axum`, `tower-http`, and `sqlx`.

- Positive evidence: `backend_probe_coverage` builds an exact-pinned generated
  package, executes an Axum loopback on `127.0.0.1:0`, observes a response
  header installed by tower-http, compiles a SQLx query macro from checked-in
  `.sqlx/` metadata, and shuts the server down deterministically.
- Negative evidence: `sqlx_without_offline_artifacts` independently removes
  and stale-mutates the query metadata. The fixture supplies no SQLx offline
  environment override. The test places an armed loopback `DATABASE_URL` in
  the backend package `.env`, which SQLx reads across path-dependency builds.
  Sifr forces `SQLX_OFFLINE=true`, removes inherited `DATABASE_URL`, includes
  package/workspace metadata roots for every backend in cache identity, and
  reaches Cargo for the valid control without connecting to the sentinel.
  Missing/stale metadata reports `SIFR-RUST-CARGO-0001` before Cargo starts.
- Compatibility category: `supported-through-bridge`. This exact bridge and
  crate graph are certified; product-level web framework workflows and
  arbitrary framework APIs remain outside this claim.
