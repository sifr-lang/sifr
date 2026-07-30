# fixture: ecosystem_backend_certification
# scenario-example: backend_feature_package

This scenario models backend ecosystem feature policy. The package pins `sqlx`
to `runtime-tokio-rustls`, `postgres`, and `macros` with default features off.
It executes an Axum server bound to `127.0.0.1:0`, observes a response header
installed by tower-http, and compiles a SQLx query macro exclusively from the
checked-in `.sqlx/` metadata under `SQLX_OFFLINE=true`. No external service,
`DATABASE_URL`, or registry access is part of the evidence.
