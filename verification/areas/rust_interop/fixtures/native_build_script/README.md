# native_build_script

This fixture family tracks build-script and native-link trust evidence for
`cc`, `bindgen`, `cxx`, and `zstd`.

- Positive evidence: `trusted_build_script_native_evidence` builds exact-pinned
  wrapper dependencies twice with locked/offline/frozen Cargo, compares their
  deterministic artifacts, and observes those artifacts plus a zstd
  encode/decode roundtrip through generated Sifr package glue.
- Negative evidence: `untrusted_native_link_rejected` independently removes
  the zstd wrapper's build-script and native-link permissions and requires
  `SIFR-RUST-TRUST-0001` before an armed build script can write its sentinel.
- Compatibility category: `supported` for this exact `cargo-probe` package and
  portable native-link envelope. Undeclared build scripts and links remain
  rejected; no general host-library fallback is implied.
