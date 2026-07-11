# M4 Review — Round 1

## Verdict

**SATISFIED**

The Counter/defaultdict residue is deleted end-to-end, the renamed manifest
rows preserve the retained-set arithmetic (17 = 2 + 7 + 3 + 4 + 1 across
`_sifr.fs`, `generated-test-glue`, `sifr.bytes::primitive_constructors`,
`_sifr.encoding::utf8_bytes_string_glue`, and `_sifr.task::language_runtime_glue`),
the `Counter[T]`, typed `defaultdict(int|list|set[, mapping])`, and checked
public `sifr.bytes` wrapper behaviors are demonstrably preserved, and the residue
guard is scoped to the exact deletion sites without shielding legitimate
in-tree usages. No blocking correctness, architecture, deletion-completeness,
manifest, or coverage defect was found.

## Deletion completeness — CONFIRMED

- `CompilerIntrinsicId::Counter{FromList,Get,MostCommon,Total,Values,Keys,Items,Increment}` and their `declaration_name`/`from_declaration_name` arms are removed from `crates/sifr_ir/src/hir_nodes.rs`; `grep -rn CounterFromList` finds no matches.
- Registry dispatch for all eight Counter IDs is deleted from `crates/sifr_codegen/src/intrinsics/registry.rs:29`.
- Lowerer files `crates/sifr_codegen/src/intrinsics/registry/collections/{counter_defaultdict_intrinsics.rs,set_and_list_intrinsics.rs}` are fully removed. `git show main:` confirms both contained only `lower_counter_*` bodies plus `arg_expr`; nothing surviving was orphaned.
- Fallback signatures for the eight `counter_*` entries plus `_defaultdict_new_impl`/`_defaultdict_get_impl`/`_defaultdict_set_impl` are removed from `crates/sifr_retained_intrinsics/src/collections_bytes_time.rs:111`.
- Registry-only Counter test `lowers_collections_counter_intrinsics_via_registry` is repurposed as `collections_bridge_helpers_are_not_intrinsics` in `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:333` with the `defaultdict_new/get/set` names added to the negative list.
- Serialized bridge is deleted at all three layers: `stdlib/_sifr/collections.sifr:38-40` (private `@rust` declarations), `stdlib/sifr/collections.sifr:1-3` (public wrappers and their `_defaultdict_*_impl` import), and `crates/sifr_stdlib/src/collections.rs:73` (Rust impls `defaultdict_new/get/set` plus the `defaultdict_helpers_preserve_default_json_behavior` test).
- `StdlibFeature::SerdeJson` retained direct spec is removed from `crates/sifr_stdlib_manifest/src/features/dependency_plan.rs:274`; `serde` and `serde_json` are removed from `retained_direct_dependency_packages` in `internal_docs/stdlib_retained_compiler_intrinsics.toml:118`. `crates/sifr_stdlib_manifest/src/features_tests.rs:16` was correctly updated to assert `sifr_stdlib` is the only dependency line emitted for `SerdeJson`.
- The `StdlibFeature::SerdeJson` enum variant and its feature-name mapping remain (unused for direct-dependency planning), which is correct: it is still needed to select the `json` sysroot feature for stdlib users of `sifr.json` and `sifr.ipc`.
- `additional_required_features` in `crates/sifr_codegen/src/intrinsics/registry/requirements.rs` has no Counter arms.
- `grep -rn "counter_from_list|counter_get|counter_most_common|counter_total|counter_values|counter_keys|counter_items|counter_increment"` across `crates/`, `stdlib/`, `internal_docs/`, `scripts/`, and `verification/` finds zero matches outside the deleted-residue check itself; the same grep for `_defaultdict_new|_defaultdict_get|_defaultdict_set` outside `target/**` finds nothing other than the residue-guard string literals.

## Manifest schema v2 renames — CONFIRMED

- `sifr.collections::typed_defaultdict_language_semantics` replaces `_sifr.collections::counter_defaultdict`, drops all eight `counter_*` exact intrinsics, drops the two deleted registry file rows, and cites the M6-deferred schema extension in `reason`.
- `sifr.bytes::primitive_constructors` replaces `_sifr.bytes::first_class_constructors` and removes `bytes_to_hex_strict` from `exact_intrinsics`, leaving `bytes_from_hex`, `bytes_from_integers`, `bytes_with_size` (the M3-defined final trio).
- `scripts/check_stdlib_manifest_schema.py:40` adds `_sifr.bytes::first_class_constructors` and `_sifr.collections::counter_defaultdict` to `PLANNED_REARCHITECTURE_DELETIONS`, so the closing-only deletion rule accepts the removal of the two prior retained-by-design rows. The remainder of the schema check (state, evidence, owned-surface, and self-tests) is unchanged.
- Retained-set arithmetic still totals 17 typed IDs across the manifest, matching the target set from the phase plan.

## Semantic preservation — CONFIRMED

- New E2E fixture `crates/sifr/tests/e2e/pass/collections_boundary_ownership.sifr` exercises `Counter[T]`, `defaultdict(int|list|set)`, and each of the three public `sifr.bytes` wrappers. It is registered in `verification/areas/core_language/data/create_pr_e2e_manifest.json`.
- Existing `test_defaultdict_*` lowering tests in `crates/sifr_lowering/src/lower/expressions_tests/callable_and_builtin_diagnostics.rs:257,269,280` continue to cover the language-owned typed defaultdict path; nothing in that path uses the deleted intrinsic surface.
- `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:246-272` proves the checked public `sifr.bytes` wrappers emit real function bodies with their in-source strings (`u8::from_str_radix(pair_str, 16)`, `byte out of range at index`, `bytes(size) requires a non-negative size`) and asserts none of `bytes_from_hex`, `bytes_from_integers`, `bytes_with_size` appear in the `sifr.bytes` module's `intrinsic_names` set. This directly satisfies "public wrappers execute checked source bodies and only those bodies reach primitive typed intrinsic HIR."
- `crates/sifr_driver/src/stdlib/stateless_collections_codegen_tests.rs` drops the `defaultdict_new/get/set` assertions and the related `SifrIntBridge::from(value)` assertion, matching the removed Rust surface.
- Registry-side classifier `is_defaultdict_storage_alias` / `is_collection_defaultdict_storage_alias` in `crates/sifr_codegen/src/intrinsics/registry/collections.rs` is a genuine shared helper — consumed by `intrinsic_method_emitters/builtin_core_methods.rs:81,90`, `intrinsic_method_emitters/collection_methods.rs:547`, `stmt_support_emitter/performance_lowering_gate.rs:161`, and `lower_expr/leaves_and_plain_calls.rs:388` — not a placeholder. Its `is_collection_defaultdict_storage_alias` narrower variant correctly excludes `__sifr_defaultdict_int` from the append/add-on-value paths, preserving the pre-M4 behavior.

## Residue guard scope — CONFIRMED

- `scripts/check_stdlib_native_intrinsic_allowlist.py:49-96` adds a fail-loud residue check for the eleven deleted identifiers (`counter_*` × 8 and `_defaultdict_{new,get,set}_impl`).
- `DELETED_COLLECTION_RESIDUE_ROOTS` is scoped correctly to the six loci that previously carried Counter/defaultdict adapter code: `hir_nodes.rs`, the intrinsic registry tree, `sifr_retained_intrinsics/src`, `sifr_stdlib/src/collections.rs`, and both `.sifr` collections declaration files. Because directories are scanned as `*.rs` and the two `.sifr` roots are treated as files, no `.sifr` methods outside `stdlib/{,_}sifr/collections.sifr` are inspected — no false positives from unrelated `.sifr` symbols. `Counter[T]` method names (`total`, `most_common`, `keys`, `values`, `items`, `get`, `increment`, `from_list`) do not share substrings with the `counter_` prefixed residue strings.
- No raw-name bypass remains: no `HirExpr::Call { func: "counter_*" }` or `func: "_defaultdict_*_impl"` construction remains in codegen/lowering; the only `__sifr_defaultdict_*` string constants are the typed language-storage aliases, which are the intended path.

## Non-blocking notes

1. `crates/sifr_codegen/src/lib_codegen_tests/structured_intrinsic_codegen_tests.rs` (187 lines, 3 tests) was deleted rather than converted. The three deleted tests covered `HirExpr::IntrinsicCall` structured lowering in three shapes: bare expr, nested intrinsic argument, and intrinsic argument that carries a typed method call. `structured_lowering_codegen_tests.rs` was retained but rewritten to use `HirExpr::IntLiteral(7)` for its copy-typed let/assign/return cases, so it no longer exercises intrinsic call expressions. The `test_generate_rust_open_uses_canonical_filehandle_constructor` fixture in `iterators_and_generators_codegen_tests.rs:232` covers the bare-intrinsic path for `OpenText`, but there is a soft coverage gap for the nested and typed-method-call-arg variants. Rewriting the two lost tests around `CompilerIntrinsicId::BytesFromHex` or `OpenText` would restore parity in a follow-up; not a blocker for this milestone.

2. `sifr.collections::typed_defaultdict_language_semantics` in the manifest uses `stdlib/sifr/collections.sifr` as `declaration_files`, but that file does not actually declare `defaultdict` — the identifier is intercepted at import time in `sifr_lowering/src/lower/mod_impl.rs:342` (`explicit_defaultdict_bindings`). The row's `reason` explicitly acknowledges the schema limitation and defers precise cross-crate file enumeration to M6. Acceptable given the M6 sequencing, but worth flagging so M6 does not silently inherit an imprecise pointer.

3. `plans/phases/06_stdlib_architecture.md:285,301,304,323,336` still describes Counter behavior in terms of the deleted `counter_from_list`, `counter_get`, and `counter_most_common` intrinsics. This is a historical design document. The parent phase plan's M6 task list explicitly owns updating phase docs, so no action is needed in M4. The residue guard does not scan `plans/`, so this is not a false-negative in that guard.

4. The `is_dir()` branch in `_deleted_collection_residue_failures` assumes the six roots exist. Removing any of those files or the retained-intrinsics crate later (M5 will do so for `sifr_retained_intrinsics`) will need this guard adjusted to avoid a `FileNotFoundError`. Non-blocking for M4; M5's plan already contemplates deleting `sifr_retained_intrinsics`.

## Validation gaps to run in the create-PR gate

Static review confirmed structure and content but cannot substitute for executing the validation matrix. The following must land green before merge:

- `cargo check -p sifr_ir -p sifr_codegen -p sifr_retained_intrinsics -p sifr_stdlib -p sifr_stdlib_manifest -p sifr_driver -p sifr` (affected crates).
- `cargo test -p sifr_codegen` (codegen 743/743, including the renamed `collections_bridge_helpers_are_not_intrinsics` test and the M4-modified structured lowering tests).
- Typed-defaultdict lowering filter suite (8 passing) and the two Counter/defaultdict filters (`test_defaultdict_accepts_counter_initial_mapping`, `test_imported_counter_iterable_constructor_remains_unsupported`).
- `cargo test -p sifr_driver stateless_collections_codegen_tests stateless_private_codegen_tests` (driver bootstrap/codegen proofs).
- `cargo test -p sifr_stdlib_manifest` (28/28).
- `cargo test -p sifr_retained_intrinsics` (4/4).
- Native runs for `collections_boundary_ownership`, `generic_counter_int`, `generic_counter_bigint`, `generic_counter_custom_class`, and `defaultdict_len_and_deque`.
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py` (main and `--self-test`).
- `python3 scripts/check_stdlib_manifest_schema.py` (main and `--self-test`).
- `python3 scripts/check_stdlib_bootstrap_ordering.py` (main and `--self-test`).
- `scripts/run_all_tests.sh --profile create-pr` as the authoritative gate.

## Summary

M4's Collections Residue Removal is complete, semantically preserved, and
appropriately guarded. No blocking findings. Proceed to the create-PR gate.
