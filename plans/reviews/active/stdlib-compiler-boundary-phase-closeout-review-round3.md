# Ad Hoc Stdlib and Compiler Boundary Rearchitecture — Round 3 Final Confirmation

Scope: `d2306d3b31c8..HEAD` (M1–M6 merged, closure draft `bf2ad72d0`, Counter docs refresh `5816168bd`, Counter phase-evidence correction `05fb4a449`).

## Round 2 corrections — all confirmed accurate

**Final Counter Ownership table** — `plans/phases/06_stdlib_architecture.md:317-325` now renders as valid 2-column Markdown:
- Header `| Layer | Responsibility |` paired with separator `| --- | --- |` (exactly two `---` cells).
- 4 data rows explicitly stating: source owns behavior, HIR/type system does normal typing, **codegen has no Counter-specific dispatch**, retained manifest has **no Counter row or dependency feature**.

**Pass-fixture names** — `06_stdlib_architecture.md:335-347` now points to real files, all present on disk:
- `crates/sifr/tests/e2e/pass/generic_counter_int.sifr` (417 B)
- `generic_counter_bigint.sifr` (535 B)
- `generic_counter_custom_class.sifr` (654 B)
- `counter_dict_native.sifr` (847 B)
- `counter_defaultdict_and_argparse.sifr` (2636 B)
- `crates/sifr/tests/e2e/fail/stdlib_counter_wrong_type.sifr` (181 B)

**Definition of Done** — `06_stdlib_architecture.md:371-384` now states final checked source ownership: `Counter[T: Hashable]` in `stdlib/sifr/collections.sifr`; explicit "No Counter-specific compiler intrinsic, dispatcher, retained-manifest row, or direct dependency remains"; E2E coverage across generic int, exact-int, custom-class, dict-native, and integration fixtures.

Source truth intact at `stdlib/sifr/collections.sifr:44` (`class Counter[T: Hashable]`) and `:240` (`def from_list[T: Hashable]`). Zero lingering `counter_*` compiler-intrinsic identifiers under `plans/phases/` or `plans/roadmap.md`; remaining `counter_` matches are E2E fixture names or `perf_counter_ns` — no stale reference implies a deleted intrinsic still exists.

## M1–M6 reconfirmed at HEAD

- **M1**: `emit_diagnostic` at `crates/sifr_stdlib/src/runtime_observability.rs:6` returns `Result<(), String>` across 5 tracing levels; zero `unwrap/expect/panic!/assert!/unimplemented!/todo!/unreachable!`. Bounded label vocabularies on accepted/rejected counters. Declaration at `stdlib/_sifr/runtime.sifr:8`; public wrapper `stdlib/sifr/runtime.sifr:50-56`. Boundary regression `crates/sifr/tests/runtime_observability_boundary.rs:65-67` asserts positive (`sifr_stdlib::runtime_observability::emit_diagnostic`) and negative (`metrics::`, `tracing::`). `metrics`/`tracing` absent from `retained_dependency_specs` (`dependency_plan.rs:266-282`) and TOML manifest.
- **M2**: `rg 'infer_dependencies|infer_dependency' crates/ verification/ scripts/` → 0 hits. `SysrootDependencyPlan` per fixture (`harness_model.rs:69`) and per group (`:83`); `DependencyFingerprint { dependency_inputs, resolved_plan }` at `:74-77`; fingerprint-keyed batches at `fixture_compilation.rs:375-378`; divergence refusal at `:266-271`. Authority tests at `dependency_plan_authority_tests.rs:174, 223, 239, 262` (7 total `#[test]` in 282-line file).
- **M3**: `CompilerIntrinsicId` (17 variants, `hir_nodes.rs:118-136`); `HirExpr::IntrinsicCall { intrinsic, args, ty, call_range, arg_ranges }` at `:517-523`. Exactly 4 identity holders. `FunctionType` (`definitions.rs:246-253`) signature-only. Codegen dispatch (`intrinsics/registry.rs:29-62`) totals all 17 IDs, no `_ =>`. Sysroot-only gating (`compiler_intrinsics.rs:27-34`), ellipsis-only body (`:52-80`), user/package rejection with structured diagnostics. First-class rejection (`core_and_calls.rs:213-222`). `bytes_to_hex_strict` bridge: adapter `sifr_stdlib/src/bytes.rs:13-21`, decl `stdlib/_sifr/bytes.sifr:13-15`, live caller `stdlib/sifr/hashlib.sifr:182`.
- **M4**: All 8 `counter_*` intrinsics gone from enum, HIR/lowering, codegen, and manifest; explicit forbidden list at `check_stdlib_native_intrinsic_allowlist.py:61-73`. `defaultdict_new/get/set` intrinsic identifiers gone; typed defaultdict semantics remain in lowering (`defaultdict_refinement.rs`, `mod_impl.rs:316-317`, `class_field_inference.rs:50,89`). Manifest rows correct: `sifr.collections::typed_defaultdict_language_semantics` (TOML `:63-70`) and `sifr.bytes::primitive_constructors` (TOML `:94-111`, exact `bytes_from_hex/from_integers/with_size`, no `bytes_to_hex_strict`). `serde`/`serde_json` absent from retained direct dependencies. Public bytes wrappers (`stdlib/sifr/bytes.sifr:21-30`) execute checked bodies; verified at `stateless_private_codegen_tests.rs:163-204`.
- **M5**: `crates/sifr_retained_intrinsics/` deleted. `rg 'sifr_retained_intrinsics' crates Cargo.toml Cargo.lock` → 0 hits. All 7 forbidden tokens (`re_export_intrinsic_fallbacks`, `resolve_retained_fallback`, `fallback_signature_modules`, `intrinsic_io`, `retained-fallback-signature-glue`, `get_intrinsic_module`, plus the crate name) contained exclusively in `check_stdlib_native_intrinsic_allowlist.py:82,87-91` deletion-guard lists and `check_stdlib_manifest_schema.py:47` (`intrinsic_io` has zero live occurrences anywhere). `stdlib/_sifr/{io,test,task}.sifr` absent. Bootstrap negative tests at `bootstrap_tests.rs:208, 233` (10 `#[test]` total, 248 lines). `_deleted_fallback_architecture_failures` (`:146-175`) and `_deleted_collection_residue_failures` (`:131`) run unconditionally from `main()` at `:121-122`; self-test dispatch at `:728, 744`.
- **M6**: `check_stdlib_native_adapter_reachability.py` enumerates live at each run via `_public_adapters` (`:55-65`) and `_rust_targets` (`:68-72`); `_validate` (`:75-142`) requires substrate → consumer-file evidence (structured `RustExpr::Path` vector at `:121-130` or qualified `sifr_stdlib::<path>` at `:131`) and refuses unreachable/stale entries. Set-equality guard via `_compare_sets` (`:489-500`) applied across `exact_intrinsics`, source declarations, registry/preamble/lowering/codegen files, retained direct dependency packages/features, and direct runtime roots — no substring/count-only comparisons. Orphan retained-dep scan (`:281-292`) rejects rows without live `StdlibFeature::` refs in codegen. Manifest schema v2 (`check_stdlib_manifest_schema.py:17,31-32,121-134`) backfills `lowering_files`/`codegen_files`. Installed vs source recertification (`verification/areas/sysroot_release/runner.py:174-233`) builds `stdlib_boundary_recertification.sifr` twice and byte-parity-checks Cargo shapes. 4 guardrails at `guardrails.json:53-80` invoked twice each (real + self-test) from `profile_runner.py:352-366`.

## Cross-cutting invariants

- **File-size guardrail PASS**: `check_file_size_guardrails.py` scans 2478 files at strict `>900`; largest touched are exactly 900 lines (5 files); no touched file exceeds the cap. `unicode_data/generated.rs` (14,779 lines) is `@generated` and excluded.
- **HIR maintainability guardrail PASS**: `check_hir_maintainability_guardrails.py` → exit 0.
- **Panic safety**: zero `unwrap/expect/panic!/assert!/unimplemented!/todo!/unreachable!` in `runtime_observability.rs` (62 lines).
- **Public API additive only**: `crates/sifr_driver/src/lib.rs` adds `generate_dependency_cargo_toml`, `sysroot_cargo_config_args`, `try_generate_standalone_dependency_plan`, `InteropBuildPlan`; every prior re-export preserved. No changes in `sifr_ir/lib.rs`, `sifr_hir/lib.rs`, `sifr_codegen/lib.rs`, `sifr_stdlib/lib.rs`, `sifr_type_system/lib.rs`, `sifr_python_ast/lib.rs`, `sifr_python_parser/lib.rs`.
- **Cargo.lock**: 3 hunks, 9 deletions total — solely the `sifr_retained_intrinsics` package block plus its two consumers.
- **Dependency direction script preserved**: `check_source_crate_dependency_direction.py` retains all rules/self-tests; only the deleted crate removed from its universe.
- **Field-name reality**: `retained_direct_dependencies` in `dependency_plan.rs:96,107,163,176,250` (round 2's earlier `_packages` prose was cosmetic; the code uses `_dependencies`).

## Non-blocking observations from Round 2

None require action. `intrinsic_io` is a defense-in-depth token with no residual live references. `counter_add/counter_sub` intrinsics were never landed on this branch (they exist only as source `__add__`/`__sub__` methods on `Counter[T]`), and the manifest set-equality + dispatch guards would still reject any restoration.

## Summary

Every M1–M6 acceptance criterion holds at HEAD with executable enforcement — set-equality guards, negative tests, self-tests, installed/source recertification, and total-match codegen dispatch. Round 2's three doc corrections are accurate and complete. No new issue exists. Public API preserved, no user-triggerable panics, no first-party file over 900 lines, no fallback recovery, no source scanning, adapter reachability enforced. The full local merge gate is the only outstanding step.

VERDICT: SATISFIED
