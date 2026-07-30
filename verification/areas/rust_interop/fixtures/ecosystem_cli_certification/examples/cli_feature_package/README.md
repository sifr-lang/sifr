# fixture: ecosystem_cli_certification
# scenario-example: cli_feature_package

This exact-pinned scenario executes `clap`, `tracing`,
`tracing-subscriber`'s `env-filter`, and an internal `anyhow` context chain
through a package-local bridge. The bridge exposes only `CliErrorBridge`.
The sibling `anyhow_surface` crate deliberately exposes `anyhow::Error`
directly so the negative evidence can prove that unadapted surface is rejected.
