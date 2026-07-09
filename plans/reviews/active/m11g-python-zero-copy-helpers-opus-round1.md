## M11g review - Python zero-copy helper migration

### 1. Blocking findings

**None.** I traced every changed edge and could not construct a repro for a correctness, bridge-ABI, panic, or lifecycle regression. Specifically:

- **Bridge ABI is now list-free.** `BufferRaw`, `ArrowRaw`, `DlpackRaw` at `crates/sifr_stdlib/src/python.rs:14-29` are all scalars (`i64`/`bool`/`String`), matching the `_sifr.python` tuple shapes at `stdlib/_sifr/python.sifr:262-322` element-for-element. The three `*_raw` helpers at `stdlib/sifr/python.sifr:188-233` index the same positions the shim produces. Verified no phantom indices survived after column shifts (e.g. `raw[5]` for `copy_possible` at `stdlib/sifr/python.sifr:417, 435, 453`).
- **Handle/token key uniqueness holds.** All three runtime stores (`buffer_ops.rs:161`, `arrow_ops.rs:280`, `dlpack_ops.rs:319`) allocate handles via `checked_add(1)` from monotonically-increasing counters, and tokens are a hash of `(handle, nonce)` where nonce is also monotonic. Handle reuse across shim cache generations is not possible, so the "release on thread A racing acquisition on thread B under the same key" scenario cannot occur.
- **No panic paths.** Mutex poisoning is mapped to `PythonError` (`sifr_stdlib/src/python.rs:591, 606, 615, 625, 638, 647, 658, 672, 682`). `SifrIntBridge::to_i64_saturating` (`interop.rs:56`) does not panic. No `unwrap`/`expect` on data-dependent paths.
- **Retired intrinsics fully gone.** `grep -n "py_buffer_u8\|py_arrow\|py_dlpack" crates/sifr_retained_intrinsics/src/python.rs` returns nothing (confirmed empty via the diff). The registry lowerings in `crates/sifr_codegen/src/intrinsics/registry/python.rs` were removed cleanly, and the negative test at `registry_extended_tests.rs:244-284` asserts they no longer lower.
- **Cache ordering is safe.** `py_buffer_u8` writes the cache *before* returning the raw tuple (`sifr_stdlib/src/python.rs:418-421`), so the immediate `py_buffer_shape` / `py_buffer_strides` / `py_buffer_suboffsets` calls in `_buffer_from_raw` (`stdlib/sifr/python.sifr:196-198`) always see fresh metadata. Same for arrow (`463-491`) and dlpack (`509-517`).
- **Release ordering is safe.** `python::release_*` runs first, then `remove_*_metadata` (`sifr_stdlib/src/python.rs:458-460, 503-506, 537-540`). No shim lock is held across the Python call, so no re-entrant deadlock is possible.
- **Public `sifr.python` shape preserved.** `BufferView.shape/strides/suboffsets`, `ArrowCapsule.capsule_names`, `DlpackTensor.shape/strides` are still populated at construction time via the accessor calls in the `_*_from_raw` helpers.

### 2. Non-blocking risks / follow-ups

- **Cache leak on runtime release failure.** In `py_release_buffer`/`py_release_arrow`/`py_release_dlpack` (`sifr_stdlib/src/python.rs:454-461, 500-507, 534-541`) the `?` on `python::release_*` short-circuits before `remove_*_metadata` runs. Because handles are monotonic, the entry can never be reused, so it's a bounded per-process leak proportional to release failures. Consider either (a) always removing the cache regardless of runtime outcome, or (b) documenting the leak in `internal_docs/architecture.md`.
- **Shim tuple layout is not statically tied to `PythonBufferMetadata`.** If `crates/sifr_runtime/src/python/buffer_ops.rs`'s metadata struct field order or types shift, the tuple builders in `buffer_raw`/`arrow_raw`/`dlpack_raw` and the matching `_sifr.python` declarations must be updated in lockstep with no compile-time enforcement. A `#[test]` that pins field ordering (or a `From<PythonBufferMetadata> for BufferRaw` impl living in the runtime crate) would harden this.
- **Unbounded static cache.** `BUFFER_METADATA`/`ARROW_METADATA`/`DLPACK_METADATA` (`sifr_stdlib/src/python.rs:31-36`) are process-global `Mutex<HashMap>`s with no eviction beyond `remove` on release. Long-lived processes that leak releases will accumulate entries; combined with monotonic handles this is a slow memory drift, not a correctness issue.
- **File-size headroom.** `stdlib/sifr/python.sifr` is now 895/900 lines. Any further growth in this file will breach the guardrail - plan a responsibility split before the next Python-interop milestone.
- **`_buffer_from_raw` / `_arrow_from_raw` / `_dlpack_from_raw` rely on implicit Sifr `Result` propagation** (no `try/except` inside the helpers, though callers wrap the calls in `try/except`). The `check` and codegen tests you ran pass, so this compiles as intended. Worth confirming with a runtime fixture (`run`, not `check`) once the interop fixtures grow one - the currently listed validation is compile-time only.

### 3. Verdict

**PR-ready.** The migration is mechanically clean, preserves the public `sifr.python` API surface, and the shim's caching/release protocol is race-free under the existing monotonic-handle invariant. The follow-ups above are quality-of-implementation items, not merge blockers.
