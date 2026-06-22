# fixture: ecosystem_backend_certification
# scenario-example: backend_feature_package

This scenario models backend ecosystem feature policy. The package pins `sqlx`
to `runtime-tokio-rustls`, `postgres`, and `macros` with default features off,
and keeps `axum` and `tower-http` as explicit Cargo path dependencies.
