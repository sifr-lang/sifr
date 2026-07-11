# M6 Review: Stdlib/Compiler Boundary Recertification

## Verification of key mechanisms

**Native-adapter reachability guard** (`scripts/check_stdlib_native_adapter_reachability.py`, `internal_docs/stdlib_native_adapter_reachability.toml`): correctly enumerates every `pub fn` / `pub const fn` / `pub async fn` at file-scope under `crates/sifr_stdlib/src/**` (excluding `lib.rs`/`feature_contract.rs`), the module derivation via `parts[0].removesuffix(".rs")` handles the `process/` sub-directory correctly (all 51 process items resolve to `process.*`, and 47 @rust targets + 4 substrates = 51 exact). Substrate entries reject duplicates, stale rows, and missing consumers. Self-test covers valid, orphan, stale, and missing-consumer cases.

**Retained-intrinsic guard extension** (`scripts/check_stdlib_native_intrinsic_allowlist.py`): now cross-references source-declared `@compiler_intrinsic(id)` decorations, `CompilerIntrinsicId::` dispatch variants (rejecting unmapped variants), `CompilerIntrinsicId`-referencing files under `sifr_lowering/src` and `sifr_codegen/src` (excluding `*_tests.rs`), and `StdlibFeature::` → dependency-spec pairs, with orphan-feature rejection. Observed sets match manifest exactly: 17 exact intrinsics, 8 source-declared, 8 lowering files, 4 codegen files, 6 retained direct dependencies. Deleted-fallback token self-test iterates every DELETED_FALLBACK_TOKEN (rather than only the first, as on `main`).

**Schema extension** (`scripts/check_stdlib_manifest_schema.py`, `internal_docs/stdlib_retained_compiler_intrinsics.toml`): `schema_version = 2` gains `lowering_files`, `codegen_files`, `source_declared_intrinsics`, `retained_direct_dependency_features`. `has_owned_surface` now recognizes these fields, so metadata-only rows still fail. New `typed-intrinsic-dispatch-core` surface owns the 4 lowering + 3 codegen dispatch files.

**Boundary equivalence** (`verification/areas/sysroot_release/runner.py::run_boundary_equivalence`): builds a source-tree `sifr` (fresh `cargo build -p sifr`), extracts the installed archive, compiles `stdlib_boundary_recertification.sifr` (retained typed intrinsics: `assert_eq`, `assert_true`; migrated bridge: `bytes_to_hex`; primitive intrinsic: `bytes_from_hex`) under both compilers, runs both binaries, then diffs normalized `Cargo.toml` dependency shapes against the reviewed `sifr_stdlib[bytes]` snapshot. Wired into `verification/profiles/merge.json` and via `verification/areas/sysroot_release/manifest.json`.

**Negative coverage**:
- User/private `@compiler_intrinsic` rejected via lowering (`compiler_intrinsics_tests::user_and_private_sysroot_declarations_are_rejected`); package variant via `sifr_driver::tests::package_project_build_check::package_source_cannot_declare_compiler_intrinsics` (new).
- Malformed/unknown/synthesized/runtime-body variants (`malformed_unknown_synthesized_and_runtime_body_declarations_are_rejected`).
- First-class value rejection (`source_declared_intrinsic_is_not_a_first_class_value`).
- Former-name collisions preserved as ordinary calls (`imported_former_intrinsic_name_without_metadata_remains_an_ordinary_call`, `local_function_declaration_shadows_unaliased_imported_intrinsic_identity`).
- Missing private declaration (`stdlib::bootstrap_tests::missing_private_stdlib_member_is_a_structured_bootstrap_failure`, `missing_private_stdlib_module_is_a_structured_bootstrap_failure`).
- Orphan retained-dependency feature self-test present.
- Deleted fallback crate/path/schema tokens: DELETED_FALLBACK_TOKENS + DELETED_FALLBACK_PATHS + DELETED_COLLECTION_RESIDUES all scan actively.

**Documentation**: architecture doc updated with reachability rule, retained-glue ownership, boundary equivalence certification. Traceability report maps each invariant to executable evidence. Roadmap status updated.

**File-size discipline**: largest touched files are `check_stdlib_native_intrinsic_allowlist.py` at 746 lines and `runner.py` at 892 lines — under the 900-line cap.

## Non-blocking concerns

1. `PUBLIC_FN_RE = ^pub\s+(?:const\s+)?(?:async\s+)?fn` does not match `pub unsafe fn` / `pub extern fn` (`crates/sifr_stdlib/src/**` — none currently exist, so no live blind spot, but a future addition would silently bypass reachability).
2. `RUST_TARGET_RE` requires `sifr_stdlib.` immediately after `(`; `@rust.opaque(type=sifr_stdlib.io.FileHandle, …)` would not register the opaque type as a target. Only test fixtures use `@rust.opaque` today.
3. Substrate consumer check accepts either the literal `sifr_stdlib::mod::fn` reference OR presence of every `"segment".to_string()`; the latter could false-positive if the same three strings appear in unrelated code. In practice the segments are unique enough that this is safe (`scripts/check_stdlib_native_adapter_reachability.py:99`).
4. `SIFR_RUST_BRIDGE_PROBE_CACHE_DIR` is shared between the installed and source-tree builds within one boundary-equivalence run (`runner.py:812`); a per-run cache would remove any theoretical probe-cache masking.
5. Removing the per-module `feature_name()` also removed the `marker_modules_report_leaf_names` cross-check between `feature_contract::LEAF_FEATURES` and actual module names. Drift is now possible without a direct guard (relies on `Cargo.toml` feature declaration and package-planner behavior).
6. `_base_manifest` in the schema check defaults to `origin/main`; without a fetched remote, the transition check fails with a diagnostic. Not a blocker; matches CI expectations.

None of these are load-bearing failures; all are soft blind spots or minor coverage-loss items.

## Correctness spot checks

- Removed `feature_name()` / `canonicalize_locale` / `format_number` / `validate_integer_digit_limit` / `default_integer_digit_limit` / made `grapheme_indices` and `word_boundaries` private — no external `.rs` still references them (verified via repo-wide grep). Tests updated accordingly.
- Retained substrate list: all four items appear in `class_emitter.rs` (via structured-segment path construction) and `task_scope_offload_runtime.rs` (as literal `sifr_stdlib::process::process_async_*` calls); none appear in `stdlib/_sifr/process.sifr` as `@rust` targets.
- 8 `@compiler_intrinsic(...)` declarations in `stdlib/sifr/{test,task}.sifr` exactly match the two surfaces' `source_declared_intrinsics` allowlists.
- Retained direct dependency features (bigdecimal, num-bigint, num-traits, rayon, rust_decimal, tokio) all have live `StdlibFeature::Variant` insertions in non-test codegen (`lib_modules_and_codegen.rs`, `entrypoints.rs`).
- Fixture path is genuinely exercised: `bytes_from_hex` → primitive intrinsic; `bytes_to_hex` → `_sifr.bytes` @rust bridge → `sifr_stdlib::bytes::bytes_to_hex`; `assert_*` → typed `CompilerIntrinsicId::TestAssert*` dispatch. Snapshot `sifr_stdlib[bytes]` is consistent with those paths.

VERDICT: SATISFIED
