## Review Summary

**Round 1 blocker (B1) verified resolved.**

The UB at `slice::from_raw_parts(NULL, 0)` for 0-dim DLPack tensors is gone. At `dlpack_ops.rs:215-227`:

```rust
if dl_tensor.shape.is_null() && len > 0 {
    return Err(dlpack_error("DLPack tensor shape pointer is null"));
}
let shape = if dl_tensor.shape.is_null() {
    Vec::new()
} else {
    unsafe { slice_to_vec(dl_tensor.shape, len) }
};
let strides = if dl_tensor.strides.is_null() {
    Vec::new()
} else {
    unsafe { slice_to_vec(dl_tensor.shape, len)... }
};
```

The NULL-with-len>0 error path is preserved; NULL-with-len==0 takes the safe `Vec::new()` branch and never dereferences. The new test `dlpack_scalar_tensor_allows_null_shape_pointer` at `dlpack_ops.rs:455-471` exercises exactly this path with `ndim: 0, shape: null_mut(), strides: null_mut()` and asserts `dimensions == 0` and empty shape/strides, then releases successfully (deleter count goes to 1).

**Other listed fixes verified:**

- **N1 (redundant negative-ndim check)** — removed at `dlpack_ops.rs:212-214`. `checked_i64_i32` handles negatives; the follow-up `usize::try_from` is now the only width-narrowing check (defensive but cheap).
- **N4 (drop-without-GIL on attach failure)** — `release_dlpack` (lines 122-138) now performs the `store.tensors.remove(&handle)` and `drop(entry)` *inside* the `super::attach(|_py| { ... })` closure. If attach fails, the closure never runs, the entry stays in the store, and no `TrackedDlpackTensor::drop` runs without the GIL. The closure has no Python-object captures.

**Lifecycle re-checked:**

- `metadata_for_tensor` validates `tensor.is_null()` (line 207) before any deref. The downstream `unsafe { (*tensor).deleter }` at `consume_capsule:167` is unreachable when `tensor` is null because `metadata_for_tensor` errors out first.
- Drop order in `TrackedDlpackTensor`: explicit `Drop::drop` calls deleter first (with GIL via attach), then fields drop in declaration order — `_owner: Py<PyAny>`, `_capsule: Py<PyAny>` — also with GIL held. Capsule rename to `used_dltensor` means the producer's destructor won't double-call the deleter when the capsule's refcount eventually hits zero.
- `store_tensor` increments `update_object_count(1)` exactly once on successful insert; `TrackedDlpackTensor::drop` decrements by 1. Balanced.

**Non-blocking observations (carried forward, not regressions):**

- Sifr fixtures (`dlpack_tensor_roundtrip.sifr`, `dlpack_tensor_device_failure.sifr`) still pass `from_none()` and exercise no real exporter — runtime coverage lives in the Rust tests; the contract's numpy/torch/tf rows are scaffolding for milestone_py_11. Same as round 1 N3.
- `sync_required_device_rejected_in_zero_copy_helper` and `unsupported_device_rejected` collapse to the same path/exception. Same as round 1 N5.

reviewer satisfied: no blockers
