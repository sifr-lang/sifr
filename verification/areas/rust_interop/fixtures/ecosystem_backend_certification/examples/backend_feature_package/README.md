# fixture: ecosystem_backend_certification
# scenario-example: backend_feature_package

This scenario models backend ecosystem feature policy. The package pins SQLx
0.9.0 with default features off. It selects `runtime-tokio` and
`tls-rustls-ring-webpki` separately. It also selects `postgres` and `macros`.
The removed combined runtime and TLS feature is not accepted.

The scenario executes an Axum server bound to `127.0.0.1:0`. It observes a
response header installed by tower-http. It compiles a SQLx query macro only
from the checked-in `.sqlx/` metadata. Sifr forces `SQLX_OFFLINE=true`. The
fixture does not set that variable. No external service, inherited
`DATABASE_URL`, or registry access is part of the evidence.
