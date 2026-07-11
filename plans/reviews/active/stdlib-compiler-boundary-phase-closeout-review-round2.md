Skipping task tracking — this is a linear review with a single verdict deliverable.

# Ad Hoc Stdlib and Compiler Boundary Rearchitecture — Round 2 Phase-Wide Review

Scope: `d2306d3b31c8..HEAD` (M1–M6 + closure draft + Counter docs refresh at `5816168bd`).

## Round 1 follow-up: Counter documentation

**Confirmed accurate.** The refresh at `5816168bd` rewrote both flagged sections:

- **`plans/phases/06_stdlib_architecture.md:283-343`** now describes the final generic source-owned `Counter[T: Hashable]` with `counts: dict[T, int]` backing (not JSON), no compiler-intrinsic table, and a "Final Counter Ownership" matrix explicitly stating: source owns behavior, HIR/type system does normal typing, **codegen has no Counter-specific dispatch**, and **retained manifest has no Counter row or dependency feature**. Signature table for `from_list[T: Hashable]` matches source at `stdlib/sifr/collections.sifr:240`.
- **`plans/phases/13_type_system_completion.md:533`** now reads: `Standalone addition/subtraction helpers | __add__ / __sub__ source methods on Counter[T] (completed)`. Matches source at `stdlib/sifr/collections.sifr:204,225`.
- **Deleted `counter_*` intrinsic identifiers absent from `plans/phases/`**: `grep -E "counter_(from_list|get|most_common|total|values|keys|items|increment|add|sub)"` returns zero hits. Remaining `counter_` occurrences are legitimate test-fixture names (`generic_counter_int`, `stdlib_collections_counter_mutate.sifr`) and `perf_counter_ns`.

## M1–M6 reconfirmation

All seven parallel probes confirmed the Round 1 findings hold at HEAD:

- **M1**: `emit_diagnostic` at `crates/sifr_stdlib/src/runtime_observability.rs:6-61` returns `Result<(), String>`, five levels, bounded labels, zero panics. Private declaration and public wrapper intact. Boundary regression test at `crates/sifr/tests/runtime_observability_boundary.rs:65-67` asserts both positive and negative shape. `metrics`/`tracing` absent from `retained_direct_dependencies` (note: field is named `retained_direct_dependencies` in `dependency_plan.rs:96`, not `_packages` as Round 1's prose suggested — cosmetic).
- **M2**: `infer_dependencies*` returns zero live hits. `SysrootDependencyPlan` stored per fixture and group at `harness_model.rs:69,83`; batches fingerprinted and refused on divergence at `fixture_compilation.rs:264-271`. Four authority tests present.
- **M3**: `CompilerIntrinsicId` has exactly **17** variants at `hir_nodes.rs:118-136`. `HirExpr::IntrinsicCall` present. `FunctionType` is signature-only (zero `CompilerIntrinsicId` field). Identity flows through exactly four documented holders (`HirFunction.compiler_intrinsic`, `ExternalDefs.compiler_intrinsics`, `LowerCtx.compiler_intrinsics`, `ReExportMaps.compiler_intrinsics`). Codegen dispatch is a total match, no `_ =>`. Sysroot-only enforcement at `compiler_intrinsics.rs:27-79`. First-class rejection at `core_and_calls.rs:213-222`. `bytes_to_hex_strict` bridge intact.
- **M4**: All 8 `counter_*` intrinsics gone. `class Counter[T: Hashable]` at `collections.sifr:44`, `from_list` at `:240`. `serde`/`serde_json` absent from `retained_direct_dependency_packages` (asserted at `features_tests.rs:29-30,94-95`). Manifest rows correct with three `sifr.bytes` primitive IDs, no `bytes_to_hex_strict`.
- **M5**: `crates/sifr_retained_intrinsics/` deleted. All forbidden tokens (`sifr_retained_intrinsics`, `re_export_intrinsic_fallbacks`, `resolve_retained_fallback`, `fallback_signature_modules`, `intrinsic_io`, `retained-fallback-signature-glue`) contained only in deletion-guard lists at `scripts/check_stdlib_native_intrinsic_allowlist.py:82,86-92` and `scripts/check_stdlib_manifest_schema.py:47`. Placeholder `_sifr/{io,test,task}.sifr` absent. Bootstrap negative tests at `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:208,233`. Executable guard `_deleted_fallback_architecture_failures` runs unconditionally with self-test at `:717-737`.
- **M6**: Adapter reachability at `check_stdlib_native_adapter_reachability.py:55-142`. Set-equality guard via `_compare_sets:489-500` applied across all typed axes. Orphan retained-dep scan at `:281-292`. Manifest schema v2 backfills `lowering_files`/`codegen_files`. Installed vs source recertification at `verification/areas/sysroot_release/runner.py:174-233` (fixture at `verification/areas/sysroot_release/fixtures/stdlib_boundary_recertification.sifr`) wired into `verification/profiles/merge.json:223`. Four guardrails declared at `guardrails.json:54-79` and invoked from `verification/runner/sifr_verify/profile_runner.py:352-366`.

## Cross-cutting invariants

- **File-size guardrail PASS** (2478 files, limit 900, strict `>` at `check_file_size_guardrails.py:169`); largest touched files at exactly 900.
- **HIR maintainability guardrail PASS**.
- **Public API drift additive only** in `crates/sifr_driver/src/lib.rs`; no removed re-exports.
- **Cargo drift confined** to `sifr_retained_intrinsics` removal; `metrics/tracing/serde/serde_json` remain optional internal-only deps of `sifr_stdlib`.
- **Runtime bridge panic-safe**: zero `unwrap/expect/panic!/assert!` in `runtime_observability.rs`.
- **Dependency direction script** retains all enforcement; only `sifr_retained_intrinsics` universe entry removed.

## Non-blocking observations

1. `plans/phases/06_stdlib_architecture.md:320` — the "Final Counter Ownership" table separator has three `---` cells but the header has only two (`| Layer | Responsibility |` vs `| --- | --- | --- |`). Renders as two columns on GitHub but is technically malformed. Cosmetic.
2. `plans/phases/06_stdlib_architecture.md:337-338` — lists two pass fixtures `stdlib_collections_counter.sifr` and `stdlib_collections_counter_mutate.sifr` that do not exist under those names; actual Counter pass fixtures are `generic_counter_int.sifr`, `generic_counter_bigint.sifr`, `generic_counter_custom_class.sifr`, `counter_dict_native.sifr`, `counter_defaultdict_and_argparse.sifr` (the fail fixture `stdlib_counter_wrong_type.sifr` does exist as claimed). This is a pre-existing doc reference that the Counter refresh commit did not update; the phase evidence is present in the repo, only the names disagree.
3. `scripts/check_stdlib_native_intrinsic_allowlist.py` `DELETED_COLLECTION_RESIDUES` omits `counter_add`/`counter_sub` from its explicit token list; those variants were never landed as intrinsics in this branch (they became source `__add__`/`__sub__` methods) and the manifest set-equality + dispatch guards would still reject any restoration, so functional risk is nil.
4. Round 1's own prose used `retained_direct_dependency_packages` in one place; the actual code field is `retained_direct_dependencies` (`dependency_plan.rs:96`). Semantics equivalent.

None of the above blocks phase closure or invalidates any acceptance criterion. Every M1–M6 invariant is backed by executable enforcement (set-equality guards, negative tests, self-tests, installed/source recertification) that would fail on regression. Public API is preserved. No user-triggerable panics. No first-party file exceeds the 900-line cap. The full local merge gate remains the only outstanding action, which the phase blueprint explicitly defers past reviewer satisfaction.

VERDICT: SATISFIED
