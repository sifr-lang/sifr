# native_build_script

This fixture family tracks build-script and native-link trust evidence for
`cc`, `bindgen`, `cxx`, and `zstd`.

- Positive evidence: `trusted_build_script_native_evidence` remains planned for
  a fixture proving trusted build-script output and native links are recorded
  before final artifact acceptance.
- Negative evidence: `untrusted_native_link_rejected` remains planned for a
  fixture proving untrusted native links fail with `SIFR-RUST-TRUST-*`.
- Compatibility category: `future-owned-by-separate-phase`. Trust policy
  plumbing is implemented, but representative native-link ecosystem
  certification is not listed as verified support.
