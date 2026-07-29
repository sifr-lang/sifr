# fixture: cargo_locked_offline
# scenario-example: locked_offline_cache

This scenario certifies `--locked`, `--offline`, and `--frozen` Cargo behavior
through a real exact-pinned registry dependency:
`indexmap = { version = "=2.14.0", default-features = false }`. The wrapper
executes `IndexMap::<String, u32>::new()` so feature or dependency drift changes
the compiled graph rather than inert fixture metadata.

The positive test first observes a cache miss and then a cache hit for the same
frozen build. It executes `sifr check`, `sifr build`, and `sifr run` with each
lock mode while preserving the checked-in `Cargo.lock`. Cargo resolution is
network-disabled for the frozen path; all registry sources must already be
cached or vendored.

The paired negative test independently removes or mutates the lockfile, changes
its selected version, checksum, or source, and changes the requested feature
set without updating the lockfile. Each `--frozen` attempt denies network and
must surface `SIFR-RUST-CARGO-0001` without creating or changing `Cargo.lock`.
