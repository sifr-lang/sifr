# ecosystem_cli_certification

This fixture family certifies an exact-pinned package-local bridge for
`clap 4.6.6`, `tracing 0.1.44`, `tracing-subscriber 0.3.23`, and
`anyhow 1.0.104`.

- Positive evidence: `cli_tooling_probe_coverage` parses a real `clap`
  command, observes a filtered `tracing` event through
  `tracing-subscriber`'s `env-filter` feature, and maps an internal `anyhow`
  context chain into the declared `CliError`.
- Negative evidence: `unsupported_anyhow_surface` proves an unadapted function
  returning `anyhow::Error` is rejected with `SIFR-RUST-TYPE-0001`.
- Compatibility category: `supported-through-bridge`. This is exact generated
  package evidence, not a claim that arbitrary crate APIs or `anyhow::Error`
  values cross the Sifr boundary directly.
