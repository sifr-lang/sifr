I've reviewed the milestone_py_8 changes against the phase contract and the listed validation. Findings below, ordered by severity.

## Blockers

None.

## Non-blocking observations

1. **Contract case `capsule_without_destructor_rejected` is unverified.** `verification/python_interop/fixtures/pyarrow_capsule/arrow_capsule_contract.json:49-53` lists this negative case, but neither the Rust unit tests (`crates/sifr_runtime/src/python/arrow_ops.rs:330-503`) nor the source fixtures exercise it. `PyCapsule::new_with_value` always installs a destructor, so the rejection branch at `crates/sifr_runtime/src/python/arrow_ops.rs:203-209` is implemented but untested. Either add a fixture/test that creates a destructor-less capsule via `pyo3::ffi::PyCapsule_New(ptr, name, None)`, or trim the contract claim.

2. **Sifr-level type collapse vs. spec.** The phase contract (`plans/issues/active/ad-hoc-embedded-python-interop.md:286-291`) defines `py.ArrowArray`, `py.ArrowStream`, `py.ArrowSchema` as separate types. `lib/sifr/python.sifr:145-161` unifies them into one `ArrowCapsule` with a `kind: str` discriminator. Runtime kind is preserved, so safety holds, but a Sifr caller cannot statically distinguish a stream handle from a schema handle — type system can't catch passing the wrong one to `release_arrow`. Worth noting for py_9/py_11 follow-up; not blocking here because the milestone's exit criteria don't enumerate per-kind Sifr types.

3. **Zero-copy producer allowlist is undocumented.** `crates/sifr_runtime/src/python/arrow_ops.rs:263-268` hardcodes `pyarrow` and `polars` as proven-zero-copy. The choice matches the spec ("Polars Arrow stream export is the preferred dataframe zero-copy target"), but the function lacks a comment naming the criterion future maintainers should apply when adding a new producer (e.g. duckdb, vaex). A one-line `// Why:` comment would prevent drift.

4. **`release_arrow` runs without GIL attach.** `crates/sifr_runtime/src/python/arrow_ops.rs:68-83` removes the entry under the store lock, then drops `TrackedArrowCapsules` outside any `super::attach(...)`. This relies on PyO3's deferred-decref to eventually run the PyCapsule destructor (which invokes the Arrow C release callback) on the next GIL acquisition. This is the same pattern py_7 chose for `release_buffer` (`crates/sifr_runtime/src/python/buffer_ops.rs:95-110`), so it's an intentional inheritance, but the spec wording (`plans/issues/active/ad-hoc-embedded-python-interop.md:450-451` "deterministic close/release semantics") permits but doesn't promise synchronous in-thread destructor execution. Worth a project-wide note before py_9 (DLPack one-shot semantics may need stronger guarantees).

5. **`PyCapsule_GetDestructor` error path not cleared.** `crates/sifr_runtime/src/python/arrow_ops.rs:203` treats a `None` return as "no destructor". The CPython contract is that `None` may also mean an exception was set. After a successful `cast::<PyCapsule>`, that error path is effectively unreachable, but a defensive `PyErr::take(py)` would prevent a stale exception from leaking into the next Python operation if invariants ever shift.

6. **.sifr fixtures only exercise type-checking.** `arrow_capsule_roundtrip.sifr`, `arrow_capsule_zero_copy.sifr`, and `arrow_capsule_copy_possible.sifr` use `from_none()` as the producer, which has no `__arrow_c_array__`. They're validated only via `cargo run -- check`. Real pyarrow/polars/pandas runtime behavior is covered exclusively by the Rust unit tests with synthetic exporter classes (`pyarrow.lib`, `pandas.core.frame`). That matches the validation list provided, but the milestone-exit claim "Polars and pyarrow zero-copy paths are verified" rests on `producer_info`'s `__module__` heuristic, not on a real polars/pyarrow capsule. Acceptable as a scaffold ahead of py_11 certification — flagging so the gap is explicit.

7. **Pillow Arrow path neither tested nor whitelisted.** Spec lists Pillow Arrow images in scope (`plans/issues/active/ad-hoc-embedded-python-interop.md:508-511`). The implementation correctly defaults Pillow to `copy_possible = true`, which is safe, but neither the contract JSON nor any test asserts this. Add a `pillow_image_is_marked_copy_possible` case to the contract for completeness when py_11 wires real Pillow into the matrix.

reviewer satisfied: no blockers
