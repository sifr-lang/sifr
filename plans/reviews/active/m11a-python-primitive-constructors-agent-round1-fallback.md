I've completed a thorough review. Here are my findings.

## Verdict: **PR-ready** - no blockers

### Scope compliance [ok]
Exactly the six constructors (`py_from_none`/`bool`/`int`/`float`/`str`/`bytes`) were migrated, nothing more:
- Removed from the intrinsic lowering (`crates/sifr_codegen/src/intrinsics/registry/python.rs:35-40`, `:533-591`), the retained type signatures (`crates/sifr_retained_intrinsics/src/python.rs:272-310`), and the manifest `exact_intrinsics` (`internal_docs/stdlib_retained_compiler_intrinsics.toml:264-269`).
- Added as `@rust(sifr_stdlib.python.*)` declarations in `stdlib/_sifr/python.sifr:26-47` and implemented in `crates/sifr_stdlib/src/python.rs:5-36`.
- All other `py_*` intrinsics remain retained (verified in the registry match arms, `retained_intrinsics`, and the toml surface at `:263-296`). No dangling references to the removed `lower_py_from_*` fns.

### PythonError fields preserved [ok]
- All five fields (`message`, `kind`, `exception_type`, `traceback`, `context`) are intact. The class definition simply moved from `python_core.sifr` -> `_sifr/python.sifr:4-24` and is re-exported through `python_core` (`stdlib/sifr/python_core.sifr:4`), so `sifr.python`'s import chain still resolves.
- The new `python_error_expr` (`rust_interop_error_mapping.rs:142-166`) maps all five fields identically to the old `map_python_error` (old moved the `String`s, new `.to_string()`-clones them - semantically equivalent). Runtime `sifr_runtime::python::PythonError` (`object_ops.rs:36-42`) exposes matching public fields, so the generated `map_err` closure binds cleanly.

### Signature & semantic fidelity [ok]
- Return type `Result[tuple[int,int], PythonError]` matches the old intrinsic's `object_handle()` = `Type::Tuple([Int,Int])`, and `ObjectHandle = (i64,i64)`. The unchanged consumer code in `stdlib/sifr/python.sifr:571-616` still typechecks.
- `py_from_int`: emitted wrapper takes `i64` and round-trips `SifrIntBridge::from(value).to_i64_saturating()` - lossless for any `i64`, so behavior is identical to the old `from_int(value)` path. `to_i64_saturating` is the established int-bridge convention (net.rs, random.rs).
- New `Type::Tuple` bridge support (`rust_interop_bridge_contract.rs:647-691`) and `error_classes`/`is_error_type` detection are correct and defensively cover parent-metadata-stripped cases.

### Compilation verified [ok]
- `cargo check -p sifr_stdlib --features python` is clean.
- End-to-end `build` of a program using `from_int`/`from_str`/`from_none` compiled the generated bridge Rust successfully; it only stopped at the environment-level embedded-CPython native-link trust gate (`SIFR-RUST-TRUST-0001` for `python3.13`), which is pre-existing and unrelated to this diff. Generated wrappers (`sifr_stdlib::python::py_from_int(SifrIntBridge::from(value))`, `py_from_str(&String -> &str)`, etc.) are well-typed.

Tests (stateless codegen, direct, plan, extended registry), the migration-closure script, and the plan doc are all updated consistently.
