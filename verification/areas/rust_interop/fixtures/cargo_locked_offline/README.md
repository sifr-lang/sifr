# cargo_locked_offline

This fixture family tracks `--locked`, `--offline`, and `--frozen` Cargo
behavior for Rust interop packages.

- Positive evidence: `locked_offline_cache_hit` remains planned for a fixture
  proving a fully cached package can build through Sifr without network access.
- Negative evidence: `lockfile_feature_drift_rejected` remains planned for a
  fixture proving lockfile, feature, and frozen-mode drift produce
  `SIFR-RUST-CARGO-*` diagnostics instead of silent resolution changes.
- Compatibility category: `future-owned-by-separate-phase`. Cargo metadata and
  trust contracts are implemented, but locked/offline ecosystem certification is
  not listed as verified support.
