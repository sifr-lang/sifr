# ecosystem_backend_certification

This fixture family tracks backend and service-oriented ecosystem probes for
`axum`, `tower-http`, and `sqlx`.

- Positive evidence: `backend_probe_coverage` remains planned for canonical
  package compile/probe coverage with documented feature pins.
- Negative evidence: `sqlx_without_offline_artifacts` remains planned for a
  fixture proving query-macro packages require checked-in `.sqlx/` offline
  artifacts instead of ambient `DATABASE_URL` execution.
- Compatibility category: `future-owned-by-separate-phase`. Product-level web
  framework workflows and backend ecosystem certification are not listed as
  verified support.
