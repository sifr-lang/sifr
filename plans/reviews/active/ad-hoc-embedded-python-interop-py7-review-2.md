Round 2 review

Verified both follow-up changes against `crates/sifr_runtime/src/python/buffer_ops.rs`:

**1. Reserve-before-increment ordering** — `store_buffer` (buffer_ops.rs:129-148) now does:
- L133: acquire mutex (`buffer_store()?`) — failure path: no increment.
- L134: `reserve_handle(&mut store)?` — failure path (overflow): no increment.
- L135: `update_object_count(1)?` — only runs after lock + handle reserved.
- L138: `store.buffers.insert(...)` — infallible HashMap insert.

The invariant holds: if `update_object_count(+1)` returns Ok, the `TrackedBuffer` is unconditionally stored (insert can't fail). If `update_object_count` itself errors, the parameter `buffer: PyBuffer<u8>` drops as a raw PyO3 buffer (no `TrackedBuffer::drop` decrement), so net `live_objects` delta is 0 — no leak. Address of round-1 observation #1.

**2. `copy_buffer_u8`-after-release assertion** — `buffer_double_release_is_deterministic_resource_error` (buffer_ops.rs:273-291) now calls `copy_buffer_u8((view.handle, view.token))` after `release_buffer` and asserts both `error.kind == "resource"` / `error.exception_type == "SifrPythonClosedBuffer"` *and* the same on `copy_error`. Addresses round-1 observation #3 — the closed-handle path is now exercised explicitly through both `release_buffer` and `copy_buffer_u8`.

**Adjacent diff sanity** — `crates/sifr_runtime/src/python.rs` only adds the `buffer_ops` mod/re-exports and widens `update_object_count`/`record_leaked_object` to `pub(super)`, which is the minimum visibility needed for the submodule. Nothing else was touched.

No regressions, no new blockers.

reviewer satisfied: no blockers
