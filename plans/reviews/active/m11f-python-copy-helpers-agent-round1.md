# M11f Python Copy Helpers - agent Review, Round 1

## Scope

Migration slice for Python copy helper leaves (22 functions) from
compiler-retained intrinsic dispatch to `stdlib/_sifr/python.sifr` +
`sifr_stdlib::python`:

- `py_copy_list_{bool,int,i32,u8,float,str,bytes}`
- `py_copy_tuple_{bool,int,i32,u8,float,str,bytes}`
- `py_copy_dict_str_{bool,int,i32,u8,float,str,bytes}`
- `py_copy_record_fields`

Private declaration for `py_copy_record_fields` was reshaped from
`list[tuple[str, tuple[int, int]]]` to `list[tuple[int, int]]`; the public
wrapper repairs handles with the caller-supplied field names.

## Assessment

Change is mechanical and consistent with prior M11 slices (M11a-M11e). The
diff cleanly removes compiler-side dispatch, signature registry, and manifest
rows, and routes all 22 leaves through checked `_sifr.python` declarations
backed by `sifr_stdlib::python` shims. Coverage is preserved by moving the
copy-helper assertions out of
`lowers_python_intrinsics_with_runtime_feature_metadata` into two new focused
tests (`python_copy_helpers_are_owned_by_compiled_stdlib_declarations`,
`python_copy_helpers_codegen_through_sifr_stdlib`).

Bridge-type audit confirms shim return types match direct Rust interop
expectations for each leaf:

- `list[T]` -> `Vec<T-bridged>` (Int -> `SifrIntBridge`).
- `dict[str, V]` -> `sifr_runtime::interop::IndexMap<String, V-bridged>`.
- `list[tuple[int, int]]` -> `Vec<(i64, i64)>` (tuple-item Int is `i64`, not
  `SifrIntBridge`, so `py_copy_record_fields` returning `Vec<ObjectRaw>` is
  correct).

Runtime ownership is preserved: `sifr_runtime::python::copy_*` and
`copy_record_fields` remain the sole CPython substrate; the shim only bridges
`SifrIntBridge` <-> `i64` and `HashMap` <-> `IndexMap`.

## Findings

### Low - `_record_fields_from_handles` silently truncates on length mismatch

`stdlib/sifr/python.sifr:230-237` uses
`while index < len(keys) and index < len(raw_values)` and never checks that
the two lengths agree. Per the current runtime contract
(`sifr_runtime::python::copy_record_fields` returns exactly `fields.len()`
handles or `Err`), this can never trigger. But if the runtime contract ever
regresses to a partial return, the wrapper would silently drop or misalign
values instead of surfacing a mismatch. Not blocking - this is the direct
consequence of the "length-guarded while loop" fix noted in the task context,
and Sifr's typed indexed access requires the length guard. Worth a one-line
comment naming the runtime contract, or an explicit mismatch error in a
follow-up.

### Low - Dict insertion-order is dropped in the runtime, not restored by the shim

The stdlib shim converts `HashMap<String, T>` -> `IndexMap<String, T>`
(`crates/sifr_stdlib/src/python.rs:375-383`), which reads like an intent to
preserve Python's insertion order. But
`sifr_runtime::python::copy_dict_str`
(`crates/sifr_runtime/src/python/object_ops.rs:625-648`) already collects
into a `HashMap`, so ordering is lost before the shim runs. This is
pre-existing behavior - M11f faithfully replicates the prior intrinsic
lowering - but it means `sifr.python.copy_dict_str_*` does not honor
Python 3.7+ dict order. Non-blocking for this slice; belongs in a runtime
follow-up that changes `copy_dict_str` to return `IndexMap` (or a
`Vec<(String, T)>`) directly.

### Low - `stdlib/sifr/python.sifr` is 889/900 lines

Only 11 lines of headroom before the file-size guardrail fires. Remaining
M11 slices (buffer, DLPack, Arrow, callbacks, context managers, coroutine
completions) will need to add public wrappers, and even one more
`@blocking_io` copy-helper stanza would breach the cap. Not a blocker for
M11f, but the next slice will likely need a responsibility-based split
(e.g., `python/copy.sifr`, `python/callbacks.sifr`). Flagging so the split
is planned rather than reactive.

## Verification

Spot-checked:

- No stale `py_copy_*` references remain in compiler codegen registry,
  retained intrinsics, preamble, or manifest (only `py_copy_buffer_u8` is
  intentionally retained for a later slice).
- Retired-name guard in `scripts/check_stdlib_migration_closure.py` covers
  all 22 leaves.
- New tests assert both directions: not lowered by intrinsic dispatch, and
  the generated `_sifr.python` Rust code contains
  `sifr_stdlib::python::py_copy_*(` for each leaf, plus the
  `intrinsic_names` set no longer includes these names.
- Public `copy_list_*`, `copy_tuple_*`, `copy_dict_str_*`, and
  `copy_record_fields` signatures in `stdlib/sifr/python.sifr` are unchanged.

## Verdict

**PR-ready.** No blocking findings. The three low-severity items above are
notes for follow-up, not gates for this slice.
