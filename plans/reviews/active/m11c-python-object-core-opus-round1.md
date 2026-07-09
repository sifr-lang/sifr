I've completed a thorough review of all changed files and traced the migration through the runtime signatures, public stdlib consumers, and guard scripts. Here's my verdict.

## Code Review: M11c Python object-core migration

**Verdict: PR-ready.** No blockers.

### Findings

**Correctness - all clean.** The five leaves are migrated with exact type fidelity:

| Leaf | `.sifr` return | stdlib Rust | Runtime signature |
|------|---------------|-------------|-------------------|
| `py_import_module(name)` | `Result[tuple[int,int], PythonError]` | `Result<ObjectRaw, PythonError>` | `import_module(&str) -> Result<ObjectHandle,...>` |
| `py_get_attr/py_get_item_str` | `tuple[int,int]` | `ObjectRaw` | `(ObjectHandle,&str)->ObjectHandle` |
| `py_close` | `None` | `()` | `close_object(ObjectHandle)->()` |
| `py_resource_diagnostics` | `tuple[bool,int,int]` | `(bool,i64,i64)` | struct `{initialized:bool, live_objects:i64, leaked_objects:i64}` |

`ObjectRaw = (i64,i64) = ObjectHandle`, and the diagnostics struct fields are exactly `bool/i64/i64`, so the tuple construction type-checks precisely (confirmed compilable by `cargo test -p sifr_stdlib --features python`). Argument marshaling (`SifrIntBridge` -> `object_handle` -> `to_i64_saturating`, `str` -> `&str`) matches the M11b `py_to_*` precedent.

**No panic risk.** Saturating int conversion, `Result` propagation throughout, no `unwrap`/`expect`; the pre-existing `i64::try_from` overflow guard in `resource_diagnostics()` is preserved (returns `PythonError`, not a panic).

**Boundary - no drift.** The 5 names are removed consistently from (a) `registry/python.rs` match arms + lowerers, (b) `sifr_retained_intrinsics/python.rs` signatures, (c) the TOML `exact_intrinsics`, and added to (d) `_sifr/python.sifr` `@rust` decls + `sifr_stdlib/python.rs`. A repo-wide grep of non-test crate source found zero leftover references. The `_sifr.python` surface correctly stays `state = "retained"` (it still owns `py_call`, callbacks, buffers, Arrow/DLPack, context, coroutine, constructors/copy helpers) with `registry_files`/`direct_runtime_roots` still valid.

**Scope - respected exactly.** `py_call`/`py_call_attr`, all collection constructors/copy helpers, context/coroutine, buffers, Arrow/DLPack, and callbacks remain retained. Public wiring is intact: `stdlib/sifr/python.sifr` (unchanged) imports these from `_sifr.python` and its usages still type-check (validated by the fixture `check`).

**Tests - adequate.** New `python_object_core_is_owned_by_compiled_stdlib_declarations` (registry returns `None`) and `python_object_core_codegen_through_sifr_stdlib` (private `_sifr.python` routes through `sifr_stdlib::python::*`, names absent from `intrinsic_names`) mirror the M11b extractor tests. The trimmed `lowers_python_intrinsics_with_runtime_feature_metadata` correctly moves the `PythonError` error-mapping assertion onto the still-retained `py_call` path.

### Minor observations (non-blocking)

1. **`check_stdlib_migration_closure.py` additions are documentation, not enforcement.** `_active_intrinsic_names` scans only `registry.rs`, where Python intrinsics dispatch via the `starts_with("py_")` prefix - individual `py_*` names never appear there, so `active_intrinsics & RETIRED_INTRINSICS` can never trip for these. Real reintroduction protection lives in `check_stdlib_native_intrinsic_allowlist.py` (which scans `registry/python.rs` against the TOML). This is consistent with the existing `py_from_*`/`py_to_*` entries, so acceptable - just be aware the closure entry is cosmetic for `py_` names.
2. **Redundant assertion** in `stateless_python_codegen_tests.rs`: the standalone `contains("sifr_stdlib::python::py_get_attr(")` duplicates the loop check above it. Harmless.
3. The `SifrIntBridge::from(handle/token)` assertions aren't object-core-specific (the migrated `py_to_*` functions already emit them), so they don't uniquely prove object-core marshaling - the stdlib type-checking is what actually guarantees it. Not a gap, just noting the assertion is broad.

None of these warrant a change. The migration is correct, scoped, and well-tested.
