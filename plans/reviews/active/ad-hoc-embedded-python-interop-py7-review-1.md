I have enough to provide a complete review. The implementation is straightforward and consistent with prior milestones (py3/py4/py5/py6) patterns, with proper integration of buffer handles into the existing `live_objects` resource diagnostics surface.

# Review findings — milestone_py_7 `Py_buffer` zero-copy core

## Contract coverage (verified)

- `PyObject_GetBuffer`/`PyBuffer_Release` via PyO3's `PyBuffer<u8>` — `crates/sifr_runtime/src/python/buffer_ops.rs:64-78`, `48-56`. PyO3's `PyBuffer::drop` re-acquires the GIL internally for `PyBuffer_Release`, satisfying the GIL discipline rule in the phase contract.
- Tracks owner / pointer-metadata / length / readonly / itemsize / format / ndim / shape / strides / suboffsets / contiguity — `buffer_ops.rs:112-127`. All required fields are present.
- Handle/token store with deterministic double-release / use-after-release reporting `SifrPythonClosedBuffer` — `buffer_ops.rs:150-159`, `203-211`. Tokens use a process-random `RandomState` seed via `hash_one`, mirroring the object store's pattern, so use-after-release across reused handle ids is statistically detectable. Handles are monotonic and never reused (`buffer_ops.rs:161-176`), so detection is in fact exact.
- Participation in the shared `live_objects` count for `resource_diagnostics`/`validate_shutdown` — `buffer_ops.rs:133` (`update_object_count(1)`) and `buffer_ops.rs:48-56` (`-1` on `TrackedBuffer::drop`). The runtime test at `buffer_ops.rs:255-260` asserts `live_objects: 2` for one bytes object + one buffer view, then `0` after release+close. Matches the phase note.
- Zero-copy vs copy split exposed at the Sifr surface (`zero_copy_as_u8`, `zero_copy_as_writable_u8`, `copy_buffer_bytes`, `copy_as_bytes`, `release_buffer`) — `lib/sifr/python.sifr:261-305`. Fixtures `py_buffer_roundtrip.sifr`, `py_buffer_memoryview.sifr`, and `py_buffer_readonly_failure.sifr` exercise the positive and the readonly-rejection negative paths; runner additions in `verification/python_interop/runner/run.py:55-76` enforce all four artifacts.
- Negative-path coverage of "non-buffer exporter", "double release", and "wrong dtype / writable on readonly" via the JSON contract and the three Rust unit tests `buffer_view_tracks_metadata_copy_and_release`, `buffer_double_release_is_deterministic_resource_error`, `buffer_rejects_wrong_dtype_and_readonly_writable_request`.
- Codegen lowering of all three new intrinsics + shared metadata feature gate — `crates/sifr_codegen/src/intrinsics/registry/python.rs:237-281` and assertions in `registry_extended_tests.rs:127-145`.

## Non-blocking observations

1. **Counter accounting on improbable failure paths** — `crates/sifr_runtime/src/python/buffer_ops.rs:129-148`. `update_object_count(+1)` runs *before* `buffer_store()` and `reserve_handle`. If either fails after the increment (mutex poison or `i64` overflow on `next_handle`/`next_nonce`), the raw `PyBuffer<u8>` is dropped without going through `TrackedBuffer::drop`, so `live_objects` is leaked by 1 and shutdown is blocked forever. Both failure modes are essentially unreachable in practice (≈2^63 buffer allocations or a poisoned mutex), so this is informational; if you want belt-and-braces, swap the order so `update_object_count(+1)` happens only after `reserve_handle` succeeds.

2. **`require_writable` is checked post-acquisition, not via `PyBUF_WRITABLE`** — `buffer_ops.rs:64-73`. PyO3's `PyBuffer::<u8>::get` doesn't request `PyBUF_WRITABLE`, so we always acquire whatever view the exporter offers and then reject if `buffer.readonly()`. That correctly rejects bytes-style readonly producers (covered by `py_buffer_readonly_failure.sifr` and the unit test), and it correctly accepts producers that natively return a writable view (e.g., `bytearray`). It is *not* equivalent to requesting `PyBUF_WRITABLE` from CPython, but since no actual mutation API is exposed in py7 (only `copy_buffer_u8`/`release_buffer`), the difference isn't observable yet. Worth a note for whichever future milestone adds a writable byte-poke API.

3. **No standalone Rust unit test for use-after-release via `copy_buffer_u8`** — the JSON contract `numpy_buffer/py_buffer_contract.json:43-49` names this case but only the double-release Rust test exercises the closed-handle path. Both paths flow through the same `lookup_buffer` filter (`buffer_ops.rs:150-159`), so behavior is covered transitively; a one-line `copy_buffer_u8` call after release would make coverage explicit, but isn't required.

4. **Phase doc** — `plans/issues/active/ad-hoc-embedded-python-interop.md` is touched only to mark py6 as merged (`#2671`); the py7 entry stays `[ ]`, matching the project convention of ticking the box at merge time, not pre-PR.

## Severity summary

| Severity | Count |
| --- | --- |
| Blocker | 0 |
| Non-blocking observations | 4 |

reviewer satisfied: no blockers
