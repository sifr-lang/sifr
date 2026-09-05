## M11g Python zero-copy - Round 2 verdict

### 1. Blocking findings

**None.** The release/cache-cleanup delta is safe and correctly implements round 1's recommendation (option a: always remove).

Traced the three release paths at `crates/sifr_stdlib/src/python.rs:454-463`, `502-511`, `538-547`:

- **Ordering preserved.** `python::release_*(key)` still runs first (Python-side resource), `remove_*_metadata(key)` second (local bookkeeping). No lock held across the runtime call -> no re-entrant deadlock possible.
- **Primary error surface preserved.** `release_result?;` short-circuits with the runtime error when release fails; only when release succeeded do we surface the mutex-poison error from `remove_*_metadata`. No double error hiding.
- **Cache leak eliminated.** `remove_*_metadata` now executes unconditionally as a statement before `?`, so a runtime release failure no longer leaves stale metadata entries. Since handles are monotonic (`buffer_ops.rs:161`, `arrow_ops.rs:280`, `dlpack_ops.rs:319`), even if remove itself failed we'd never see a collision on the same key.
- **No use-after-free from concurrent reads.** A racing `py_buffer_shape` between the runtime release and the cache remove sees the cached `Vec<i64>` (owned by stdlib, cloned at `buffer_u8` time - independent of Python-side lifetime).
- **Panic-free.** `HashMap::remove` is idempotent and infallible; `.lock()` is mapped through `metadata_error` (`sifr_stdlib/src/python.rs:693-701`); no `unwrap`/`expect`.
- **Behavior divergence on release failure is intentional.** After a failed release, accessors will now return "metadata not found" instead of returning stale shape/strides. This matches the round 1 recommendation and doesn't affect the public `sifr.python` API (accessors are only invoked at construction in `_buffer_from_raw` / `_arrow_from_raw` / `_dlpack_from_raw` at `stdlib/sifr/python.sifr:188-233`).

### 2. Validation
- `cargo fmt` - clean
- `git diff --check` - clean
- `cargo test -p sifr_stdlib --features python` - 1 test pass, 0 fail
- `cargo test -p sifr_driver python_zero_copy_helpers_codegen_through_sifr_stdlib` - pass

### 3. Non-blocking follow-ups (unchanged from round 1)

Still worth doing in a later PR, none merge-blocking:
1. Add a `From<PythonBufferMetadata> for BufferRaw`-style impl or a pinning test so the tuple layout at `sifr_stdlib/src/python.rs:549-589` fails to compile if the runtime struct fields shift.
2. `stdlib/sifr/python.sifr` at 895/900 lines - plan a responsibility split before the next Python-interop milestone.
3. Add a runtime fixture (`run`, not `check`) exercising the acquire -> accessor -> release loop end-to-end.
4. The static caches at `sifr_stdlib/src/python.rs:31-36` remain unbounded; with this delta they now only accumulate if the caller never invokes `py_release_*` at all (leaked handles, not leaked releases).

### 4. Final verdict

**PR-ready.** The delta cleanly closes round 1's cache-leak follow-up without introducing any lifecycle, error-reporting, or panic regressions. Migration remains mechanically clean and preserves the public `sifr.python` API surface.
