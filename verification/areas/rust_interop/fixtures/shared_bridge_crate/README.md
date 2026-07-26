# shared_bridge_crate

Shared bridge crates are ordinary direct Cargo dependencies with a
package-boundary restriction: shared crates must not import package-specific
`crate::__sifr_bridge` modules.

- Positive `cargo-probe` evidence:
  `test_build_shared_bridge_crate_positive_cargo_probe` consumes the checked-in
  Cargo workspace, compiles and runs both the positive evidence source and the
  scenario source, links the shared crate, and observes `bytes` and `str`
  values crossing the boundary.
- Negative `cargo-probe` evidence:
  `test_check_shared_bridge_crate_negative_cargo_probe` checks the checked-in
  negative source, negative-only trust manifest, and rejected shared-crate Rust
  source; it observes `SIFR-RUST-RESOLVE-0001` for
  `crate::__sifr_bridge::*` before Cargo execution.

Both tests belong to the blocking `sifr_driver_generated_builds` crate-test
suite selected by the merge, nightly, and release profiles in full mode.
Reproduce either side with:

```bash
cargo test -p sifr_driver --lib test_build_shared_bridge_crate_positive_cargo_probe -- --ignored --test-threads=1
cargo test -p sifr_driver --lib test_check_shared_bridge_crate_negative_cargo_probe -- --ignored --test-threads=1
```
