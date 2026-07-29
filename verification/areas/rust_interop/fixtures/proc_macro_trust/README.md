# proc_macro_trust

This fixture family tracks proc-macro and codegen trust evidence for
`serde_derive` and `prost-build`.

- Positive evidence: `trusted_proc_macro` builds exact-pinned wrapper
  dependencies twice with locked/offline/frozen Cargo, compares deterministic
  prost-build output, and runtime-observes the wrapper-macro execution,
  separately labeled upstream `serde_derive` compilation, and compiled schema
  marker through generated Sifr package glue.
- Negative evidence: `untrusted_proc_macro_rejected_pre_execution` positively
  controls both armed sentinels, then independently removes proc-macro and
  build-script permission and requires kind-specific
  `SIFR-RUST-TRUST-0001` diagnostics while both sentinels remain absent.
- Compatibility category: `supported` for this exact `cargo-probe` package.
  Undeclared direct build-time code remains rejected; trusting these wrappers
  does not grant arbitrary procedural macro or build-script execution.
