# M5 Review Round 2 — Fallback Signature Architecture Deletion

Independent verification of the working-tree diff of
`codex/stdlib-boundary-m5-fallback-architecture-deletion` against `origin/main`,
against the M5 milestone in
`plans/issues/active/ad-hoc-stdlib-compiler-boundary-rearchitecture.md`.

## Scope inspected

Working-tree diff surface (verified against `git diff --stat origin/main`):

- Crate deletion: `crates/sifr_retained_intrinsics/**` (Cargo.toml + 10 sources,
  ~2,050 lines), workspace membership in root `Cargo.toml`, workspace
  dependency alias in root `Cargo.toml`, `Cargo.lock` package entry, downstream
  `sifr_driver/Cargo.toml` and `sifr_lowering/Cargo.toml` dependency lines.
- Driver bootstrap: `crates/sifr_driver/src/stdlib/bootstrap.rs` (fallback
  branch, `re_export_intrinsic_fallbacks`, and `intrinsic_names` bookkeeping
  removed; class-export coverage widened in `has_compiled_exports`).
- Driver bootstrap tests:
  `crates/sifr_driver/src/stdlib/bootstrap_tests.rs` (three new fixtures:
  positive private-import compile, missing-member failure, missing-module
  failure).
- Driver private codegen tests: `stateless_{collections,crypto,fs,logging,
  math,private,process,python,time}_codegen_tests.rs` (all
  `intrinsic_names`-map assertions dropped — the field no longer exists on
  `StdlibCode`).
- Lowering: `crates/sifr_lowering/src/lower/mod_impl.rs` (independent
  `_sifr.*` fallback branch removed) and
  `crates/sifr_lowering/src/lower/private_stdlib_imports.rs`
  (`resolve_retained_fallback` removed from the resolver chain).
- Lowering diagnostics tests:
  `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs`
  (`_sifr.io` renamed to `_sifr.fs` where used as a private-module handle;
  `public_sysroot_stdlib_source_falls_back_per_private_import_name` rewritten
  to assert a `NAME_MISSING_MODULE_MEMBER` diagnostic).
- Codegen: `crates/sifr_codegen/src/lib_emitter_state.rs`,
  `lib_modules_and_codegen.rs`, `module_prescan.rs`, and
  `lib_codegen_tests/classes_and_basics_codegen_tests.rs` (removes
  `StdlibCode.intrinsic_names`, `RustEmitter.stdlib_intrinsic_names`, and the
  prescan population that treated raw import names as intrinsic; adds a
  trait-impl fixture that stops relying on default `StdlibCode`).
- Sysroot sources: `stdlib/_sifr/io.sifr` and `stdlib/_sifr/test.sifr`
  deleted; `PRIVATE_STDLIB_MODULES` in `crates/sifr_stdlib_manifest/src/
  sources.rs` shortened accordingly.
- Manifest + schema: `retained-fallback-signature-glue` surface entry removed
  from `internal_docs/stdlib_retained_compiler_intrinsics.toml`;
  `fallback_signature_modules` removed from `ALLOWED_SURFACE_FIELDS`,
  planned-deletion, and per-surface allow list in
  `scripts/check_stdlib_manifest_schema.py`;
  `retained-fallback-signature-glue` added to
  `PLANNED_REARCHITECTURE_DELETIONS`.
- Guards: `scripts/check_stdlib_native_intrinsic_allowlist.py`
  (`fallback_signature_modules` observation and its self-test removed;
  `_deleted_fallback_architecture_failures` added with
  `DELETED_FALLBACK_PATHS`, `DELETED_FALLBACK_TOKENS`,
  `DELETED_FALLBACK_SCAN_ROOTS`, and a new self-test that reintroduces both
  the deleted path and a deleted token in a tempdir);
  `scripts/check_source_crate_dependency_direction.py`
  (`sifr_retained_intrinsics` removed from `ALL_SIFR_CRATES`, three
  forbidden-dependency sets, `seed_valid_repo`, and two self-test cases).
- Docs: `internal_docs/architecture.md` (crate list) and
  `internal_docs/sifr_sysroot_and_stdlib_architecture.md` (invariant bullet 6
  and the manifest/signature-authority row).
- Verification wiring: `create-pr.json`, `merge.json`, `nightly.json`,
  `release.json` (blocking crate-test entries dropped);
  `verification/areas/coverage_matrix/data/cargo_metadata_classification.json`
  (crate row dropped);
  `verification/areas/generated_code_quality/generated_code_quality.py`
  (producer-fingerprint tuple pruned).
- E2E fixtures / diagnostic baselines: `crates/sifr/tests/e2e/fail/
  import_intrinsic.sifr`, `crates/sifr/tests/e2e/pass/intrinsics_block_test.
  sifr`, `verification/areas/diagnostics/{data,fixtures}/**` updated to use
  `_sifr.fs` (still a private module that user code cannot import).

## Findings

No findings.

## Acceptance-criteria verification

- **Deleted crate / Cargo / config.**
  `git status` confirms `crates/sifr_retained_intrinsics/**` deleted (11 files,
  ~2,050 LOC). Root `Cargo.toml` `members` and `workspace.dependencies` no
  longer contain `sifr_retained_intrinsics`; `Cargo.lock` no longer has the
  package entry (nor any dependency edge into it). `crates/sifr_driver/
  Cargo.toml` and `crates/sifr_lowering/Cargo.toml` no longer list
  `sifr_retained_intrinsics`. Verification profiles
  (`create-pr.json`, `merge.json`, `nightly.json`, `release.json`) drop the
  now-nonexistent test job. `cargo_metadata_classification.json` drops its row.
  `generated_code_quality.py` producer fingerprint no longer references the
  crate. `rg -n 'sifr_retained_intrinsics' crates Cargo.toml Cargo.lock`
  returns no match, satisfying the plan's exact acceptance command.
- **Deleted driver / lowering / codegen fallback paths and raw
  intrinsic-name metadata.**
  `bootstrap.rs` no longer contains `re_export_intrinsic_fallbacks`, the
  missing-module fallback branch, or `intrinsic_names_for_module`
  bookkeeping. `mod_impl.rs` no longer contains the independent
  `sifr_retained_intrinsics::get_intrinsic_module` fallback branch that
  handled `_sifr.*` imports outside of `resolve_compiled_private_imports`.
  `private_stdlib_imports.rs` no longer contains `resolve_retained_fallback`
  and no longer chains it into the resolver.
  `StdlibCode.intrinsic_names` is deleted; the codegen emitter no longer
  clones an intrinsic-name map into `RustEmitter.stdlib_intrinsic_names`; the
  entire `module_prescan` code path that treated raw import names as
  intrinsics is removed. `RustEmitter.intrinsic_functions` remains, but it is
  now populated only by `apply_intrinsic_registry_side_effects`
  (`intrinsic_method_emitters/builtin_core_methods.rs`), keyed off typed
  `CompilerIntrinsicId::declaration_name()` — i.e. by typed intrinsic
  dispatch, never by import name.
- **Deleted placeholders.**
  `stdlib/_sifr/io.sifr` and `stdlib/_sifr/test.sifr` are removed. The
  `PRIVATE_STDLIB_MODULES` list in `crates/sifr_stdlib_manifest/src/
  sources.rs` drops both entries so the manifest inventory no longer expects
  them. `rg '_sifr\.(io|test)' stdlib crates verification` finds no live
  importer of either module in stdlib or verification sources; the remaining
  `_sifr.io` mentions in `crates/sifr_driver/src/build/sysroot_interop.rs`
  are self-contained synthetic fixtures written into a `TempSysroot` for
  interop-resolution tests, not references to the deleted placeholder.
- **Manifest / schema deletion.**
  `stdlib_retained_compiler_intrinsics.toml` no longer contains a
  `retained-fallback-signature-glue` surface. `check_stdlib_manifest_schema.
  py` removes `fallback_signature_modules` from `ALLOWED_SURFACE_FIELDS` and
  from every per-surface field enumeration, and adds
  `retained-fallback-signature-glue` to `PLANNED_REARCHITECTURE_DELETIONS`.
  Guard self-test (`--self-test`) still passes.
- **Compiled-source-only signature authority.**
  `resolve_compiled_private_imports` now attempts only `resolve_function`,
  `resolve_class`, and `resolve_constant` against `externals`; there is no
  recovery step, and a missed name emits `name_diagnostics::missing_member`
  (mapped to `NAME_MISSING_MODULE_MEMBER`). The `has_compiled_exports` guard
  in `bootstrap.rs` was widened to include the `classes` map, which is
  necessary now that only compiled maps count — a private declaration that
  exports only classes still triggers re-export processing rather than being
  silently dropped. `retained_public_declarations_export_typed_compiler_identity`
  continues to assert `sifr.test.assert_eq` and `sifr.task.current_context`
  carry `CompilerIntrinsicId::{TestAssertEqual, TaskCurrentContext}` from
  compiled source only.
- **Deterministic structured bootstrap failure.**
  `bootstrap_tests.rs` adds three tests. `private_stdlib_imports_resolve_only_
  from_compiled_source_exports` proves a real private-declaration import
  compiles and registers its `_sifr.*` transitive dep. `missing_private_
  stdlib_member_is_a_structured_bootstrap_failure` and `missing_private_
  stdlib_module_is_a_structured_bootstrap_failure` assert that every emitted
  diagnostic carries `DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE` (SIFR-STDLIB-
  0003) and mentions the offending module/name. The lowering-level assertion
  is complemented in `name_import_diagnostics_tests.rs`, where the former
  `..._falls_back_per_private_import_name` test was rewritten to
  `..._rejects_uncompiled_private_import_name` and now asserts a
  `NAME_MISSING_MODULE_MEMBER` diagnostic — no recovery path remains.
- **Permanent guard coverage and negative self-tests.**
  `check_stdlib_native_intrinsic_allowlist.py` adds
  `_deleted_fallback_architecture_failures` and calls it unconditionally from
  `main()`. `DELETED_FALLBACK_PATHS` covers `crates/sifr_retained_intrinsics`,
  `stdlib/_sifr/io.sifr`, and `stdlib/_sifr/test.sifr`.
  `DELETED_FALLBACK_TOKENS` covers `sifr_retained_intrinsics`,
  `fallback_signature_modules`, `resolve_retained_fallback`,
  `re_export_intrinsic_fallbacks`, and `get_intrinsic_module`, all searched
  under `DELETED_FALLBACK_SCAN_ROOTS`
  (`Cargo.toml`, `Cargo.lock`, `crates/**`,
  `internal_docs/stdlib_retained_compiler_intrinsics.toml`,
  `internal_docs/architecture.md`,
  `internal_docs/sifr_sysroot_and_stdlib_architecture.md`,
  `scripts/check_source_crate_dependency_direction.py`,
  `scripts/check_stdlib_manifest_schema.py`,
  `verification/profiles`,
  `verification/areas/coverage_matrix/data/cargo_metadata_classification.
  json`,
  `verification/areas/generated_code_quality/generated_code_quality.py`).
  The self-test builds a fixture root containing a restored deleted-crate
  directory and a Cargo.toml with the deleted `resolve_retained_fallback`
  token, calls the guard against that fixture, and rejects the run.
  `check_source_crate_dependency_direction.py` also drops the crate from
  `ALL_SIFR_CRATES` and its three forbidden-dep sets, removes the crate rule,
  removes it from `seed_valid_repo`, and drops the two dedicated self-test
  cases. All guards pass locally (bootstrap ordering
  `private=28, public=61`; manifest schema `surfaces=9, schema_version=2`;
  native intrinsic allowlist `exact_intrinsics=17, registry_files=8,
  preamble_files=9, retained_direct_dependency_packages=6, direct_runtime_
  roots=2`; dependency direction PASS; file size PASS; HIR guardrails PASS).
- **Test-strength preservation.**
  All removed assertions in `stateless_{collections,crypto,fs,logging,math,
  private,process,python,time}_codegen_tests.rs` and
  `classes_and_basics_codegen_tests.rs` are the ones that read the deleted
  `StdlibCode.intrinsic_names` map. The surviving assertions still verify
  that private declarations route through `sifr_stdlib::<module>::<fn>`,
  that transitive deps carry `_sifr.<mod>` for the public module, and that
  private-code SHA and source-path continue to be checked. Two new positive
  cases (`stdlib_class_exports_preserve_parent_markers`,
  `python_core_re_exports_preserve_callable_metadata`,
  `retained_public_declarations_export_typed_compiler_identity` — unchanged
  in scope, present in the base) and three new bootstrap cases add strictly
  more coverage than was removed. `trait_impl_fixture_stdlib_code` in
  `lib_codegen_tests.rs` restores a runnable fixture for the multi-module
  trait-impl visibility test that previously implicitly relied on the
  deleted default.
- **Docs.**
  `internal_docs/architecture.md` no longer lists the deleted crate.
  `internal_docs/sifr_sysroot_and_stdlib_architecture.md` invariant 6 now
  reads "`sifr_retained_intrinsics` and all fallback-resolution paths are
  deleted"; the manifest row states "fallback signature tables have been
  deleted" and pins signature authority to private declaration source plus
  typed HIR. Historical mentions in `plans/**` describe pre-migration state
  and are correctly not part of the guard scan roots.
- **Dependency direction.**
  `check_source_crate_dependency_direction.py` drops
  `sifr_retained_intrinsics` from `ALL_SIFR_CRATES` and from
  `IR_FORBIDDEN_DEPENDENCIES`, `STDLIB_FORBIDDEN_DEPENDENCIES`, and
  `GENERATED_STDLIB_FORBIDDEN_DEPENDENCIES`. The crate no longer exists as a
  dependency edge in `Cargo.lock` for `sifr_driver` or `sifr_lowering`.
  Self-test PASS.
- **Panic safety.**
  `rg '\.unwrap\(\)|\.expect\(|panic!' crates/sifr_driver/src/stdlib/
  bootstrap.rs crates/sifr_lowering/src/lower/private_stdlib_imports.rs
  crates/sifr_lowering/src/lower/mod_impl.rs` returns no hits in production
  paths. Missing-declaration failures propagate through `Result`/diagnostic
  channels with `DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE` and
  `DiagnosticCode::NAME_MISSING_MODULE_MEMBER`. Fixture-only `expect()` calls
  in `bootstrap_tests.rs` are constrained to `sifr_sysroot::resolve_sysroot`
  and `std::fs::canonicalize` of the development stdlib root — both
  programmer invariants at test-fixture level, not user paths.
- **900-line cap.**
  `bootstrap.rs` 603 lines; `bootstrap_tests.rs` 248 lines;
  `mod_impl.rs` 800 lines; `private_stdlib_imports.rs` 218 lines;
  `check_stdlib_native_intrinsic_allowlist.py` 567 lines;
  `check_source_crate_dependency_direction.py` verified under cap.
  `python3 scripts/check_file_size_guardrails.py` PASS (2477 files scanned,
  limit 900 lines).

## Validation assessment

The create-PR gate reported in the user prompt is green with blocking budgets
respected (crate tests 163,565 ms, runtime/platform 56,958 ms, E2E 130/130 at
383,025 ms). Locally re-run guards during this review confirm the strengthened
schemas: allowlist guard PASS with the new exact-intrinsics count of 17 and
no `fallback_signature_modules` line in the summary; manifest schema guard
PASS with `retained-fallback-signature-glue` now in the planned-deletion set;
dependency-direction guard PASS with the deleted crate absent from every
enumerated crate set; bootstrap ordering guard PASS with `private=28,
public=61`; file size and HIR guardrails PASS; both new guard self-tests PASS
(schema and allowlist).

The new bootstrap tests are the executable proof that missing declarations
now fail as `STDLIB_BOOTSTRAP_FAILURE` rather than silently recovering. The
rewritten lowering test proves the same at the lowering boundary via
`NAME_MISSING_MODULE_MEMBER`. Together with the guard's path/token scan and
its self-test, restoration of any deleted fallback artifact is now
mechanically rejected.

## Final verdict

SATISFIED
