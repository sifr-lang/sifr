# cargo_locked_offline

This fixture family tracks `--locked`, `--offline`, and `--frozen` Cargo
behavior for Rust interop packages.

- Positive evidence: `locked_offline_cache_hit` executes `sifr check`,
  `sifr build`, and `sifr run` in every lock mode against exact-pinned
  `indexmap 2.14.0`, preserves the authoritative `Cargo.lock`, and observes a
  network-disabled frozen cache miss followed by a warm cache hit.
- Negative evidence: `lockfile_feature_drift_rejected` independently mutates
  the lockfile's presence, version, checksum, and source plus the requested
  registry feature set. Every mutation runs through Cargo `--frozen`, which
  denies network and lockfile writes, and maps to `SIFR-RUST-CARGO-0001`.
- Compatibility category: `supported` for the exact checked-in registry
  dependency and feature policy. This does not claim that an uncached package
  can resolve in offline or frozen mode.
