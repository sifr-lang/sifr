Verified against the actual tree. All claims check out:

- stdlib impls (`crates/sifr_stdlib/src/time.rs`): `sleep` early-returns on non-finite or non-positive inputs and uses `Duration::try_from_secs_f64`, which returns an error for out-of-range durations instead of panicking; `monotonic` uses a `LazyLock<Instant>` process baseline. Panic-free boundary test present.
- Declarations (`stdlib/_sifr/time.sifr`): `sleep` and `monotonic` `@rust` rows point at `sifr_stdlib.time.*`.
- Removed cleanly: `registry/time.rs` deleted, `mod time` plus `sleep`/`monotonic` arms gone from `registry.rs`, `intrinsic_time` removed from retained `lib.rs` and `collections_bytes_time.rs`. No dangling `lower_sleep`/`lower_monotonic`/`intrinsic_time`/`registry/time` references remain.
- Closure/manifest: `sleep` and `monotonic` added to `RETIRED_INTRINSICS`; manifest row flipped to `closing` with registry/exact-intrinsic entries dropped.
- Async untouched: the remaining time-related references are `task.sleep` async lowering and an unrelated `process.rs` poll sleep, both expected.
- Tests: old registry-lowering test removed; `sleep` and `monotonic` moved into the compiled-declaration ownership and codegen assertions.

READY
