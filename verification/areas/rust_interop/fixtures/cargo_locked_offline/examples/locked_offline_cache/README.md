# fixture: cargo_locked_offline
# scenario-example: locked_offline_cache

This scenario certifies `--locked`, `--offline`, and `--frozen` Cargo behavior
through a real exact-pinned registry dependency:
`indexmap = { version = "=2.14.0", default-features = false }`. The wrapper
executes
`indexmap::IndexMap::<String, u32, RandomState>::with_hasher(RandomState::default())`
with the wrapper's `std::collections::hash_map::RandomState`. This is the same
empty-map/default-hasher construction as `new()` when IndexMap's `std` feature is
enabled, but it preserves this scenario's disabled default features. The wrapper
inserts every byte and folds IndexMap's insertion-ordered values; exact hash and
order regression checks retain that real compiled dependency use. The policy
checker checks the constructor and insertion/fold directly in Rust source, so
README prose alone cannot satisfy the dependency-use proof.

The positive test first observes a cache miss and then a cache hit for the same
frozen build. It executes `sifr check`, `sifr build`, and `sifr run` with each
lock mode while preserving the checked-in `Cargo.lock`. Cargo resolution is
network-disabled for the frozen path; all registry sources must already be
cached or vendored.

The paired negative test independently removes or mutates the lockfile, changes
its selected version, checksum, or source, and changes the requested feature
set without updating the lockfile. Each `--frozen` attempt denies network and
must surface `SIFR-RUST-CARGO-0001` without creating or changing `Cargo.lock`.
