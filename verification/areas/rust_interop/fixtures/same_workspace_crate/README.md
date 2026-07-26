# same_workspace_crate

Same-workspace crates are ordinary Cargo dependencies. The compiler does
not invent workspace fallback resolution.

- Positive `cargo-probe` evidence:
  `test_build_same_workspace_crate_positive_cargo_probe` consumes the checked-in
  Cargo workspace, compiles and runs both the positive evidence source and the
  scenario source, links the explicit workspace path dependency, and checks
  the observed hashes.
- Negative `cargo-probe` evidence:
  `test_check_same_workspace_crate_negative_cargo_probe` checks the checked-in
  negative source with a checked-in Cargo layout where `workspace_hash` remains
  a workspace member but is not a dependency, then observes
  `SIFR-RUST-RESOLVE-0001` before a generated build can proceed.

Both tests belong to the blocking `sifr_driver_generated_builds` crate-test
suite selected by the merge, nightly, and release profiles in full mode.
Reproduce either side with:

```bash
cargo test -p sifr_driver --lib test_build_same_workspace_crate_positive_cargo_probe -- --ignored --test-threads=1
cargo test -p sifr_driver --lib test_check_same_workspace_crate_negative_cargo_probe -- --ignored --test-threads=1
```
