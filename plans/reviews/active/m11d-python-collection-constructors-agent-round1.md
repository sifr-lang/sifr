Review complete. I've verified the migration against the diff, the ABI bridge (via codegen contract + rustc probe), the retained/compiler boundary, and config consistency.

## Code-review verdict: **PR-ready**

No blockers found. The migration is minimal, correct, and faithfully mirrors the established M11a/b/c stdlib-migration pattern.

### Findings

**Correctness - verified clean**
- ABI bridge is sound. Borrowed `list[str]` -> `&[String]` and `list[tuple[int,int]]` -> `&[(i64,i64)]` per `rust_interop_bridge_contract.rs`, matching the impls in `crates/sifr_stdlib/src/python.rs:169,177`. `ObjectRaw` and runtime `ObjectHandle` are the same alias `(i64,i64)`, so `python::from_list(&[ObjectRaw])`/`from_dict_str(&[(&str,ObjectRaw)])` line up exactly. Three pre-existing precedents (`json_dump_tokens`, `http_build_cookie_header`, `url_query_build`) confirm the `list[str]->&[String]` mapping. This is rustc-enforced by the cargo-check signature probe, which fires on the `emit`/`build` you already ran on `primitive_roundtrip.sifr`.
- Key/value ordering is preserved: `_keys_from_keyed_objects` and `_handles_from_keyed_objects` both iterate the same `values` in order, and `keyed_object_handles` re-zips positionally.
- No-panic guarantee holds: the length-mismatch guard returns `PythonError` instead of letting `zip` silently truncate. (In practice unreachable from the public API since both lists derive from one `values`; correctly defensive for direct private-decl calls.)

**Retained/compiler boundary - no drift**
- The four constructors are removed from all three retention surfaces (codegen registry match arms + helpers, `sifr_retained_intrinsics` type table, retained TOML `exact_intrinsics`) and added to the closure script's `RETIRED_INTRINSICS`. No dangling references to the deleted `lower_py_from_*`/`*_object_handles_expr`/`lower_*_constructor` helpers remain.
- Deferred M11 slices stay retained: `py_call`/`py_call_attr`, all `py_copy_*` helpers, buffers, Arrow/DLPack, context/coroutine, and callback echoes are untouched in both the registry and the type table. `_keyed_handles_from_objects` is still live (used by `call`/`call_attr`), so no orphaned helper.

**Public API - no regression**
- `sifr.python` still exports `from_list`/`from_tuple`/`from_dict_str`/`from_record` with identical `Object`/`list[tuple[str,Object]]` signatures; only the private wrapper bodies changed. `from_list`/`from_tuple` bodies are correctly untouched (same private-decl shape); only the keyed constructors split into `keys`/`values`.

**Tests - adequate**
- Both new tests match the M11c object-core precedent: one asserts the four no longer lower via the compiler registry (`registry_extended_tests.rs`), one asserts they generate `sifr_stdlib::python::py_from_*(` calls and are absent from private intrinsic names (`stateless_python_codegen_tests.rs`).

### Non-blocking nits (optional, no action required to merge)
1. `crates/sifr_stdlib/src/python.rs:193` - the new `kind: "invalid_argument"` introduces an underscore-style kind value inconsistent with the existing runtime python kinds (`conversion`/`resource`/`runtime`/`trust`/`zero-copy`). Cosmetic only; the branch is defensive/unreachable from public wrappers.
2. The length-mismatch error path in `keyed_object_handles` has no direct unit test - acceptable given it's unreachable from the public API, but a small `sifr_stdlib` test would lock the no-panic contract if a future slice exposes the private decl.
3. `stdlib/sifr/python.sifr` is now 878/900 lines - under the cap but tight; upcoming M11 python slices adding wrappers here will likely force a responsibility-based split of this module.
