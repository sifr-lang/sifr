# fixture: cargo_locked_offline
# scenario-example: locked_offline_cache

This scenario models the future owned verification package for `--locked`,
`--offline`, and `--frozen` Cargo behavior. The dependency graph is fully local
and represented by `Cargo.lock`, so the fixture can prove that Sifr preserves
locked resolution without fetching or changing features.

The paired negative evidence changes the requested feature set without updating
the lockfile and must surface `SIFR-RUST-CARGO-0001`.
