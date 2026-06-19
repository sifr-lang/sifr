I've reviewed the changes. Here are my findings.

## Blockers

**B1. Latent UB in `metadata_for_tensor` for 0-dim tensors with `shape == NULL`** — `crates/sifr_runtime/src/python/dlpack_ops.rs:219-222`

```rust
if dl_tensor.shape.is_null() && len > 0 {
    return Err(dlpack_error("DLPack tensor shape pointer is null"));
}
let shape = unsafe { slice_to_vec(dl_tensor.shape, len) };
```

When a producer hands us a 0-dim tensor with `ndim == 0` and `shape == NULL`, the guard does not fire (because `len > 0` is false), and we call `slice_to_vec(NULL, 0)` → `slice::from_raw_parts(NULL, 0)`. Per the documented safety contract of `slice::from_raw_parts`, `data must be non-null and aligned even for zero-length slices`. The DLPack spec permits `shape == NULL` for scalar tensors, so a spec-compliant producer can trigger this.

This conflicts with the milestone's "no user-triggerable runtime panics/crashes in Sifr-attributable paths" invariant (UB is strictly worse than a panic). The strides branch already handles `is_null() → Vec::new()` correctly; shape should mirror that — e.g.

```rust
let shape = if dl_tensor.shape.is_null() {
    Vec::new()
} else {
    unsafe { slice_to_vec(dl_tensor.shape, len) }
};
```

Real-world torch/numpy/tensorflow scalars don't currently set `shape == NULL` so this likely won't manifest in the listed positive fixtures, but it's a latent soundness hole on the DLPack import path and the fix is trivial.

## Non-blocking suggestions

**N1. Redundant negative-ndim check** — `dlpack_ops.rs:211-216`. `checked_i64_i32` already returns an error for `value < 0`, so the subsequent `if dl_tensor.ndim < 0 { ... }` is unreachable. Either remove the second check or replace `checked_i64_i32` with `i64::from(dl_tensor.ndim)` and keep the explicit message.

**N2. Empty review placeholder** — `plans/reviews/active/ad-hoc-embedded-python-interop-py9-review-1.md` is 0 bytes. If this is meant to capture this review, it should be populated; if it was created by accident, it should be removed before merge.

**N3. Sifr fixtures are type-check only** — `dlpack_tensor_roundtrip.sifr` and `dlpack_tensor_device_failure.sifr` both invoke `zero_copy_dlpack_tensor(from_none())`. `None` has no `__dlpack__`, so these can't exercise the export path at runtime — the contract's positive cases (`numpy_cpu_tensor_supported_dtype`, `torch_cpu_tensor_supported_dtype`, `tensorflow_cpu_tensor_supported_dtype`) and the `sync_required_device_rejected_in_zero_copy_helper` negative case live only in `dlpack_tensor_contract.json` as scaffolding. Runtime coverage for CPU dtypes, double-consume, invalid capsule name, unsupported dtype, and unsupported device is in `dlpack_ops.rs` Rust tests. This matches the py7/py8 pattern, but the gap between the contract document and what's actually executed is worth flagging — the package-certification gates (milestone_py_11) will need real numpy/torch/tf fixtures to discharge these claims.

**N4. Drop-without-GIL on `attach` failure** — `dlpack_ops.rs:135` (and identically in `arrow_ops.rs`, `buffer_ops.rs`). `release_dlpack` captures `entry` into `super::attach(|_py| drop(entry))`. If `attach` returns `NotInitialized` before invoking the closure, the captured `entry` is dropped here without the GIL, which runs `TrackedDlpackTensor::drop` → calls `deleter(tensor)` and drops `Py<PyAny>` for `_owner` / `_capsule` without the GIL attached. This is the same shape as the accepted py8 pattern, but py9 inherits it. Realistically unreachable (you wouldn't be releasing a tensor with no runtime), but if it ever fires it is unsafe. Acceptable for this milestone if explicitly inherited.

**N5. `stream_sync_required` is dead for accepted tensors** — `dlpack_ops.rs:245`. Since non-CPU devices are rejected upstream at line 110, the value is always `false` in returned metadata. The field is still useful as part of the data contract for when device support arrives, but the negative fixture name `sync_required_device_rejected_in_zero_copy_helper` is currently indistinguishable from `unsupported_device_rejected` — both produce `SifrPythonDlpackUnsupportedDevice`. Worth a comment in the contract JSON or a TODO in the phase doc clarifying that these collapse into the same path until device tensors are wired in.

**N6. Double-consume semantics depend on the producer** — `dlpack_ops.rs:147-162`. The rename to `used_dltensor` enforces one-shot consumption *of a given capsule*, which is exactly what the DLPack spec mandates. Real producers (torch/numpy) mint a fresh capsule per `__dlpack__()` call, so calling `obj.__dlpack__()` twice will succeed twice — and that is correct DLPack behavior. The Rust test asserts the per-capsule property correctly via a producer that returns the same global capsule each call. Not a defect, just worth keeping in mind when authoring the eventual torch/numpy fixtures so they don't accidentally expect double-call-on-the-same-producer to be rejected.

---

reviewer not satisfied: 1 blocker (B1).
