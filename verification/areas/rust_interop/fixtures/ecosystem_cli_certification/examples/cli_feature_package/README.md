# fixture: ecosystem_cli_certification
# scenario-example: cli_feature_package

This scenario models CLI ecosystem feature policy. The package keeps `clap`,
`tracing`, `tracing-subscriber`, and `anyhow` explicit in Cargo, and enables
`tracing-subscriber`'s `env-filter` feature.
