Based on my review, I traced the key round-2 fixes against the current code:

1. **Package bridge authority carries exact inventoried modules**: `PythonBridgeTargetAuthority.modules: BTreeSet<String>` (mod_context.rs:428-431) is populated from `package.modules.iter().map(|module| module.module.clone())` (python_bridges.rs:83-90). The lowering gate at python_interop.rs:597-612 rejects targets when no prefix `target.segments[1..end].join(".")` is in `authority.modules`, and for `bridge.func` (no module prefix) the range `2..2` is empty, so the check emits `PYIMP_INVALID_TARGET`. Test `bridge_target_requires_an_inventoried_module` confirms this.

2. **SIFR-PYIMP-0003 emission moved out of Display**: `PythonRuntimeError::ReservedBridgeCollision`'s Display (python.rs:178-181) is now bare text without the code. The generated bootstrap match arm (python_runtime.rs:222-224) uses `DiagnosticCode::PYIMP_RESERVED_BRIDGE_COLLISION.code()` to prefix the eprintln! output, with a test asserting `"SIFR-PYIMP-0003: reserved Python bridge namespace collision"` in the rendered main.rs. The specific arm matches before the generic Err arm, so the code prefix is emitted exclusively from the compiler-owned bootstrap.

3. **Active diagnostic catalog + baseline coverage**: `PYIMP_RESERVED_BRIDGE_COLLISION` is declared, listed in `ACTIVE_DIAGNOSTIC_CODES`, has a registry entry with owner `sifr_runtime::python::bridge_loader`, docs file `SIFR-PYIMP-0003.md`, catalog entry, and a baseline coverage deferral entry pointing at the bridge_loader test with removal_target "add the M6 generated-binary reserved sys.modules collision baseline fixture" — matching the stated wave-5 target.

4. **TypeScript-Go source-read inventory update**: The row for "Package Python bridge inventory" now names `crates/sifr_package/src/python/bridge_resolution.rs:126` and updates the classification to include "re-reads inventoried UTF-8 source for the embedded runtime table while verifying its digest," which matches the actual read at bridge_resolution.rs:126.

The full atomic-loader design remains sound: meta_path finder installed first (bridge_loader.rs:116-118), pre-existing `sys.modules` entries rejected (bridge_loader.rs:192-207), reserved-name detection short-circuits trust (object_ops.rs:504-506), `ensure_first` called before each reserved `import_module`/`resolve_target` (object_ops.rs:30-32, 54-58), probe skipping for `__sifr_bridge__.` (python_interop.rs:83-86), synthetic package sources injected for every prefix (python_bridges.rs:110-144), bridge target root filtered from `required_import_root` after splice (python_interop.rs:618-621), source content flows into cache identity via `push_python_plan_cache_key` (python_interop_plan.rs:236-286), and embedded source is properly escaped via `Debug` formatting in `render_bridge_sources` (python_runtime.rs:259-278).

No correctness, diagnostics, generated-code, cache, or scope defects surface at file/line level.

VERDICT: SATISFIED
