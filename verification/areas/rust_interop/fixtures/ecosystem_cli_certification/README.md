# ecosystem_cli_certification

This fixture family tracks representative CLI/tooling package probes for
`clap`, `tracing`, `tracing-subscriber`, and `anyhow`.

- Positive evidence: `cli_tooling_probe_coverage` remains planned for canonical
  package compile/probe coverage with `tracing-subscriber` `env-filter`.
- Negative evidence: `unsupported_anyhow_surface` remains planned for a fixture
  proving arbitrary `anyhow::Error` surfaces require an explicit bridge error
  mapping before they can become Sifr-facing API.
- Compatibility category: `future-owned-by-separate-phase`. CLI ecosystem
  certification is not listed as verified support.
