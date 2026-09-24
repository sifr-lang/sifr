# DX.5 stdlib metadata transport and consumer migration inventory

Source base: `3f070fcdbc8a205104291d32b91edd8e3c59b12f` (DX.4 record).
This is the DX.5 schema/inventory boundary. It does not enable a producer command,
change checking, replace `ExternalDefs`, or select metadata in normal CLI/LSP use.
DX.6 owns checked-source conversion/production. DX.7 first migrates the source
provider to layered immutable baseline plus mutable project overlay, then enables
metadata only after source parity and concurrent-project isolation tests pass.

## Transport, identity and lifetime

`sifr_sysroot::metadata` owns the private v1 container. Its 120-byte header holds
magic, schema version, record count, compiler/semantic-target/stdlib-input IDs and
exact file length. Each 92-byte sorted directory entry holds a 32-byte record ID,
kind, reserved flags, fixed-width offset/size, decoded allocation bound and SHA-256.
All integers are little endian. Offsets must be contiguous, non-overlapping and
cover the file exactly. The decoder checks compatibility and bounds before any
payload. A record uses canonical JSON over an explicit wire DTO, never serialization
of a live compiler value. Duplicate/noncanonical map keys, extra fields, unknown
variants and noncanonical ID strings reject on consumption.

All non-scalar data is a typed record reference, including strings, functions,
expressions, classes, declaration metadata and interop contracts. The explicit
record families mirror current checked payload fields; `SemanticExports` contains
every `ExternalDefs` field once per module. HIR child nodes use references, so a
record never recursively owns another HIR or type. Integer literals retain exact
integer text; floats retain IEEE-754 bits; Unicode chars retain validated scalar values.
Ordered vectors retain declaration, operand, parameter and diagnostic source order.
Unordered maps/sets use sorted maps/sets keyed by deterministic IDs. Normalization
and type equality remain owned by the existing type system: DX.6 must convert its
normalized values, not invent new equality from names or wire hashes.

`Ref::anchor` hashes kind plus length-delimited stable anchor components. Package
anchors include package name/version/source digest; declaration anchors add module
and symbol; binders add declaration and nested binder path. They are assigned
before graph edges. `intern` content-addresses canonical immutable records and
rejects conflicting definitions for the same anchor. Neither pointer addresses,
allocation ordinals, live source-map IDs nor producer absolute paths are anchors.
Text may contain arbitrary program literals; only source/navigation identity fields
are interpreted as paths and must be normalized relative packaged paths.

Nominal instances store a declaration ID, a separate occurrence/view ID and type
argument IDs. Full and partial structural views retain distinct inventories, display
names and optional existing nominal-identity spellings. The declaration contains
its original package/module/symbol; a view does not replace nominal authority.
Type parameters use binder+slot. `SourceOriginId` becomes source location plus local
ordinal; `BindingId` becomes declaration binder plus local binding slot. Re-interning
in DX.7 resolves anchors through normal package/nominal/binder/source identity rules,
scoped to the selected store and demanded compilation context. There is deliberately
no unbounded conversion back to deeply owned live `Type::Class` values here.

The store owns an indexed seekable input and retains one `Arc<T>` per decoded ID.
Under cache pressure it evicts only cache-exclusive owners; live caller handles pin
their records. Exhaustion by active handles remains an explicit resource error.
An index lookup of a nominal type does not read its large field/method view.
Separate stores have separate owners even when wire IDs match. The immutable input
must be pinned by the caller: DX.6/DX.8 own publication/read leases and installed
lifetime. This decoder does not authorize replacing an open file in place.

Structural graphs are checked with an iterative color/depth/work traversal;
nominal/module/binder anchors terminate expansion. Nominal recursion is permitted;
body and unanchored structural cycles reject. Defaults bound file bytes, directory
count, per-record bytes, total retained/decode work and depth. A conservative
4096-byte fixed plus 32-times encoded-byte allocation charge bounds JSON decoding and retained records;
it is a safety allowance, not a measured allocation report. Budget exhaustion is
an actionable error and is not a source-bootstrap fallback.

`RustPayload` stores exact per-module bytes, name mappings, source attribution,
generator inventory and `FragmentValidation`. Validation binds emitted bytes,
compiler/target, grammar identity and `StdlibModule` boundary. DX.6 must issue it
only after real fragment validation. Digest binding is integrity, not authentication
or proof that a producer ran a validator. Consumers still compare the required
grammar identity, revalidate transformed boundaries and run final rustc; no boolean
or stored fragment record authorizes skipping final-program checks. Generic functions,
generic classes and late project-policy classes are separate `TemplatePayload` roles.

## Payload classification and owning migrations

| Current owner/field family | Persisted record / derivation | DX.6/DX.7 responsibility |
| --- | --- | --- |
| `ExternalDefs.functions`, classes, constants, generic aliases | `SemanticExports`, `FunctionType`, indexed `Type`, declaration/view/binder tables | Lowering/provider reads; project collectors write overlay only |
| Intrinsic IDs, error types, callable conventions/varargs/defaults/workloads/Python shapes/callback targets | Same named `SemanticExports` fields and typed IR records | Preserve call lowering and reserved/private authority |
| Class type parameters/bounds/instance methods/structural methods/opaque and structural Rust classes/consuming methods | Same named fields; `StructuralMethodExport`, `HirParam`, convention records | Preserve partial class views, receiver/method contracts and imported-default precedence |
| Class field defaults, function defaults and constant integer values | Ordered index/expression pairs and interned arbitrary-precision integer text | Checking demands only required body nodes; do not infer defaults from constructors |
| Declaration metadata/descriptors, adapter providers/markers/selections, attached API sets/APIs, descriptor functions, applied metadata | Same named fields and explicit specialization DTO family | Preserve package identities, typed values, origins/ranges and declaration order |
| Const functions, specialization requests/outputs, JSON integer boundary requests | `HirFunction` body graphs and explicit specialization DTOs | Preserve checked const bodies and complete method slots; no live component result rerun |
| `StdlibCode.hir_modules` | `HirModule` inventory graphs; declaration and interop summary directories | Check/import demand uses summaries; templates load only demanded HIR; never scan every module body |
| `StdlibEmissionCode.module_rust_code` | `RustPayload`; `Module.source` gives packaged path/content identity; nominal names derive from declared `HirClass` inventory | Codegen reuses exact bytes and existing public emit markers/newline; no layout redesign |
| module constants / function signatures / class fields | `SemanticExports` types plus `RustPayload.names`; `FunctionType` projects parameter type/convention and return; `NominalView.fields` | Explicit bounded demanded projection replaces whole-map clone |
| transitive deps / generator functions | `InteropSummary.codegen_dependencies` and `.intrinsics`; `RustPayload.generators` | Derive traversal from direct summary edges; do not decode HIR for demand discovery |
| generic classes / parameters / templates / module class templates | `TemplatePayload`, `HirClass.type_params`, class view and declaration binder | Preserve generic inference and late project-policy specialization |
| `StdlibCompiled.interop` / module sources | `InteropSummary` Rust/Python contracts and `Module.source`; support/features are explicit | Runtime/environment/trust probes are resolved for current project; do not persist ambient success or secret environment |
| `ResolvedSysroot`, ready cache slots, syntax sessions, process-local projections | Derivable toolchain/store owners and demanded syntax transforms, not serialized | Driver owns leases/cache resolution; codegen owns parse/transform validation; retry corrected initialization in DX.7/DX.8 |
| Analysis navigation/source map | `SourceFile`, `SourceLocation`, source-relative `TextRange` | Demand-read packaged sources; re-intern source IDs in snapshot; preserve private access and UTF conversion |

## Mutation and lifetime boundaries

- `sifr_lowering::lower_module_with_externals_name_and_options`, sibling public
  lowerers and `LowerCtx::with_external_defs` borrow the layered view. `LowerCtx`
  currently holds `Cow<ExternalDefs>`; its mutation must target the project overlay.
- Frontend graph/query helpers and `WorkspaceSession` currently retain both base
  and combined owned definitions. They will share an immutable source/metadata
  baseline and retain revision-specific overlays. `collect_module_exports`,
  callable/descriptor/Rust-class export writers and `take_module_specialization_metadata`
  must publish/take only overlay state atomically. Failure/deletion invalidates exports.
- Driver `project/frontend.rs::{compile_frontend_modules,
  compile_single_frontend_module_with_source_and_options, collect_project_hir_modules,
  collect_project_hir_source_modules, collect_project_hir_source_modules_with_options}`
  currently take/extend owned definitions. They must receive the same baseline view
  plus an owned project overlay; taking an `Arc` of the old aggregate is insufficient.
- `build/entrypoint.rs`, test-runner preparation, public stdlib accessors and
  `stdlib::{bootstrap,cache,tooling,interop,re_exports}` are separate preparation,
  clone-projection, borrowed-code and source-lifetime sites. Bootstrap writes the
  source producer's local definitions; application collectors never mutate it.
- Analysis `host/overlay_updates.rs` constructs sessions from driver definitions
  and auxiliary sources; `host/stdlib_navigation.rs` and `symbols.rs` currently
  enumerate source-derived symbols. Replace them with the small module/name index
  and demanded source locations. LSP handlers consume analysis snapshots indirectly;
  they must not gain their own metadata decoder or independent stdlib lifetime.
- Codegen import-signature, emission, support and interop-demand readers borrow the
  selected module record closure. Generic/template specialization outputs remain
  project-owned; existing mutation of `StdlibEmissionCode` is producer-local.

## Exact pinned sites

The table below is mechanically checked against source paths/symbols. Source lines
list every matching stdlib/external access within that symbol; operation tags are
an explicit conservative union within the complete function (an enumeration or clone
may also touch project data). Tests are classified as setup/parity rather than live
migration sites. LSP entries without direct stdlib references are indirect snapshot
consumers. Struct fields and owners are covered by the payload and lifetime tables
above. `python3 scripts/check_dx_metadata_inventory.py` verifies site coverage and
all `ExternalDefs`/IR struct fields; regeneration requires reviewing classifications.

<!-- dx5-sites:start -->

| Exact source symbol | Current access (conservative union within symbol) | DX.7 migration | Matching source lines |
| --- | --- | --- | --- |
| `crates/sifr_lowering/src/compiled_identity.rs:2` `compiled_input_tokens` | read, borrow/retain | layered view + demanded record handles | 17 |
| `crates/sifr_lowering/src/hir_snapshot_tests.rs:53` `lower_source` | read, borrow/retain | test setup/parity | 58 |
| `crates/sifr_lowering/src/lower/attached_api_declarations.rs:153` `finalize` | read, enumerate | layered view + demanded record handles | 162 |
| `crates/sifr_lowering/src/lower/attached_api_surfaces.rs:130` `provisional_declarations` | read, enumerate | layered view + demanded record handles | 137 |
| `crates/sifr_lowering/src/lower/attached_api_surfaces.rs:157` `declarations_for_set` | read, enumerate | layered view + demanded record handles | 169 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:10` `public_module` | read | test setup/parity | 12 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:53` `user_and_private_sysroot_declarations_are_rejected` | read, borrow/retain | test setup/parity | 59, 65 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:71` `malformed_unknown_synthesized_and_runtime_body_declarations_are_rejected` | read | test setup/parity | 91 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:97` `imported_alias_call_preserves_identity_and_source_ranges` | read, mutate overlay/producer | test setup/parity | 101, 102, 125, 134 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:158` `source_declared_intrinsic_is_not_a_first_class_value` | read, mutate overlay/producer | test setup/parity | 161, 162, 178, 186 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:191` `imported_former_intrinsic_name_without_metadata_remains_an_ordinary_call` | read, mutate overlay/producer | test setup/parity | 195, 196, 219 |
| `crates/sifr_lowering/src/lower/compiler_intrinsics_tests.rs:231` `local_function_declaration_shadows_unaliased_imported_intrinsic_identity` | read, mutate overlay/producer | test setup/parity | 234, 251, 256, 265 |
| `crates/sifr_lowering/src/lower/descriptor_declarations.rs:217` `import_bindings` | read, clone projection | layered view + demanded record handles | 226, 236, 245, 254, 263 |
| `crates/sifr_lowering/src/lower/descriptor_declarations.rs:616` `resolve_declared_type` | read | layered view + demanded record handles | 624 |
| `crates/sifr_lowering/src/lower/descriptor_declarations.rs:639` `external_provider` | read | layered view + demanded record handles | 643 |
| `crates/sifr_lowering/src/lower/descriptor_declarations.rs:671` `finalize_markers_and_selections` | read, enumerate | layered view + demanded record handles | 680, 709 |
| `crates/sifr_lowering/src/lower/expressions_tests/callable_and_builtin_diagnostics.rs:333` `test_defaultdict_accepts_counter_initial_mapping` | read | test setup/parity | 334 |
| `crates/sifr_lowering/src/lower/expressions_tests/callable_and_builtin_diagnostics.rs:345` `test_defaultdict_subscript_read_is_non_optional_value_type` | read | test setup/parity | 346 |
| `crates/sifr_lowering/src/lower/expressions_tests/callable_and_builtin_diagnostics.rs:356` `test_defaultdict_membership_checks_lower` | read | test setup/parity | 357 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs:6` `defaultdict_binding_types` | read, enumerate | layered view + demanded record handles | 7 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs:60` `defaultdict_int_conflicting_augassign_key_is_rejected` | read, enumerate | test setup/parity | 62 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs:72` `initialized_defaultdict_int_keeps_declared_key_type` | read, enumerate | test setup/parity | 74 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs:84` `nested_function_shadowing_keeps_independent_defaultdict_key_types` | read, enumerate | test setup/parity | 86 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_augassign_refinement.rs:141` `unhashable_defaultdict_int_augassign_key_reports_one_capability_error` | read, enumerate | test setup/parity | 144 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:5` `binding_and_constructor_types` | read, enumerate | layered view + demanded record handles | 6 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:29` `defaultdict_iterable_mutators_carry_checked_backing_storage_places` | read, enumerate | test setup/parity | 31 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:65` `defaultdict_set_pop_retains_checked_mutable_backing_storage` | read, enumerate | test setup/parity | 67 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:97` `set_pop_rejects_immutable_parameter_mutation` | read, enumerate | test setup/parity | 100 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:148` `conflicting_defaultdict_set_elements_report_container_conflict` | read | test setup/parity | 151 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:164` `conflicting_defaultdict_keys_report_container_conflict` | read | test setup/parity | 167 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:180` `sibling_defaultdict_bindings_keep_independent_declaration_types` | read, enumerate | test setup/parity | 182 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:228` `lowering_inexact_index_elements_do_not_force_declaration_hints` | read | test setup/parity | 230, 233, 236 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:275` `tuple_key_with_unresolved_member_is_not_adopted` | read | test setup/parity | 277 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:281` `incomplete_defaultdict_nested_return_reports_missing_annotation` | read | test setup/parity | 283 |
| `crates/sifr_lowering/src/lower/expressions_tests/defaultdict_order_independent_inference.rs:297` `nested_defaultdict_shadow_does_not_merge_with_outer_hint` | read | test setup/parity | 299 |
| `crates/sifr_lowering/src/lower/expressions_tests/dict_augassign_checked_error.rs:78` `annotated_defaultdict_keeps_factory_semantics_and_declared_shape` | read, enumerate | test setup/parity | 80 |
| `crates/sifr_lowering/src/lower/expressions_tests/minmax_sorted_sum.rs:452` `test_list_missing_method_has_stdlib_code` | read, enumerate | test setup/parity | 452 |
| `crates/sifr_lowering/src/lower/expressions_tests/minmax_sorted_sum.rs:547` `test_dict_missing_method_has_stdlib_code` | read, enumerate | test setup/parity | 547 |
| `crates/sifr_lowering/src/lower/expressions_tests/minmax_sorted_sum.rs:602` `test_set_missing_method_has_stdlib_code` | read, enumerate | test setup/parity | 602 |
| `crates/sifr_lowering/src/lower/expressions_tests/minmax_sorted_sum.rs:662` `test_str_missing_method_has_stdlib_code` | read, enumerate | test setup/parity | 662 |
| `crates/sifr_lowering/src/lower/expressions_tests/minmax_sorted_sum.rs:702` `test_tuple_missing_method_has_stdlib_code` | read, enumerate | test setup/parity | 702 |
| `crates/sifr_lowering/src/lower/expressions_tests/minmax_sorted_sum.rs:854` `test_generic_type_missing_method_has_stdlib_code` | read, enumerate | test setup/parity | 854 |
| `crates/sifr_lowering/src/lower/expressions_tests/ownership_and_async.rs:702` `test_defaultdict_list_call_resolves_with_explicit_import` | read | test setup/parity | 703 |
| `crates/sifr_lowering/src/lower/expressions_tests/ownership_and_async.rs:714` `test_defaultdict_alias_call_resolves_with_explicit_import` | read | test setup/parity | 715 |
| `crates/sifr_lowering/src/lower/expressions_tests/ownership_and_async.rs:726` `test_defaultdict_keyword_constructor_unsupported_has_stdlib_code` | read, enumerate | test setup/parity | 726, 728 |
| `crates/sifr_lowering/src/lower/expressions_tests/ownership_and_async.rs:739` `test_defaultdict_unpacked_keyword_constructor_unsupported_has_stdlib_code` | read, enumerate | test setup/parity | 739, 741 |
| `crates/sifr_lowering/src/lower/expressions_tests/support.rs:13` `lower_source_with_externals` | read, borrow/retain | layered view + demanded record handles | 15, 18 |
| `crates/sifr_lowering/src/lower/expressions_tests/support.rs:21` `lower_source_with_stdlib_collections` | read, mutate overlay/producer | layered view + demanded record handles | 21, 25, 26, 30 |
| `crates/sifr_lowering/src/lower/external_defs.rs:23` `attached_api_set_membership_checks_the_stored_canonical_identity` | read | test setup/parity | 24 |
| `crates/sifr_lowering/src/lower/external_defs_context_tests.rs:4` `external_input` | read | test setup/parity | 4, 5 |
| `crates/sifr_lowering/src/lower/external_defs_context_tests.rs:31` `lowering_external_defs_context_borrows_complete_input` | read, borrow/retain | test setup/parity | 34, 35, 36, 38, 42, 49, 51, 52 |
| `crates/sifr_lowering/src/lower/external_defs_context_tests.rs:57` `lowering_external_defs_preserves_success_and_failure_inputs` | read | test setup/parity | 76, 89 |
| `crates/sifr_lowering/src/lower/import_diagnostics.rs:59` `bare_stdlib` | read | layered view + demanded record handles | 59, 61, 68, 72, 82, 85 |
| `crates/sifr_lowering/src/lower/import_diagnostics.rs:90` `bare_stdlib_help` | read | layered view + demanded record handles | 90, 91, 95, 99, 102, 107 |
| `crates/sifr_lowering/src/lower/import_resolution.rs:9` `effective_import_module_name` | read, borrow/retain | layered view + demanded record handles | 13, 22 |
| `crates/sifr_lowering/src/lower/import_resolution.rs:30` `external_module_exists` | read, borrow/retain | layered view + demanded record handles | 31, 34, 35, 36, 37 |
| `crates/sifr_lowering/src/lower/imported_defaults.rs:10` `import_user_callable` | read, borrow/retain | layered view + demanded record handles | 12, 18, 33, 34, 35, 38, 41, 44, 47 |
| `crates/sifr_lowering/src/lower/imported_defaults.rs:53` `import_rust_threadsafe_callback_target` | read, borrow/retain | layered view + demanded record handles | 55, 60 |
| `crates/sifr_lowering/src/lower/imported_defaults.rs:71` `import_rust_threadsafe_callback_class` | read, borrow/retain | layered view + demanded record handles | 73, 78 |
| `crates/sifr_lowering/src/lower/imported_defaults.rs:90` `import_callable_generic_metadata` | read, borrow/retain | layered view + demanded record handles | 92, 97, 105 |
| `crates/sifr_lowering/src/lower/imports.rs:11` `runtime_hir_import` | read, borrow/retain | layered view + demanded record handles | 15, 17 |
| `crates/sifr_lowering/src/lower/imports.rs:28` `import_constant` | read, borrow/retain | layered view + demanded record handles | 30, 36, 47 |
| `crates/sifr_lowering/src/lower/imports.rs:62` `import_generic_type_alias` | read, borrow/retain | layered view + demanded record handles | 64, 70 |
| `crates/sifr_lowering/src/lower/imports.rs:85` `register_imported_class_instance_methods` | read, enumerate, borrow/retain | layered view + demanded record handles | 87, 94, 99 |
| `crates/sifr_lowering/src/lower/imports.rs:113` `register_imported_rust_consuming_methods` | read, enumerate, borrow/retain | layered view + demanded record handles | 115, 120 |
| `crates/sifr_lowering/src/lower/imports.rs:134` `register_imported_rust_opaque_class` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 136, 141, 148 |
| `crates/sifr_lowering/src/lower/imports.rs:157` `class_aliases_by_module` | read, enumerate, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 159, 174, 192 |
| `crates/sifr_lowering/src/lower/imports.rs:204` `report_missing_stdlib_member` | read | layered view + demanded record handles | 204 |
| `crates/sifr_lowering/src/lower/imports.rs:213` `report_unknown_stdlib_module` | read | layered view + demanded record handles | 213 |
| `crates/sifr_lowering/src/lower/imports.rs:232` `resolve_imports_early` | read, enumerate, mutate overlay/producer, clone projection, borrow/retain | layered view + demanded record handles | 234, 252, 262, 335, 353, 363, 370, 377, 384, 390, 401, 411, 417, 437, 449, 454, 461, 470, 480, 492, 508, 520 |
| `crates/sifr_lowering/src/lower/imports.rs:537` `register_imported_structural_identity_inputs` | read, borrow/retain | layered view + demanded record handles | 539, 544, 552, 561 |
| `crates/sifr_lowering/src/lower/mod_context.rs:296` `with_external_defs` | read, borrow/retain | layered view + demanded record handles | 296, 297 |
| `crates/sifr_lowering/src/lower/mod_context.rs:301` `new` | read, borrow/retain | layered view + demanded record handles | 403 |
| `crates/sifr_lowering/src/lower/mod_context.rs:446` `is_stdlib_lowering` | read | layered view + demanded record handles | 446 |
| `crates/sifr_lowering/src/lower/mod_context.rs:502` `can_import_private_stdlib_declarations` | read | layered view + demanded record handles | 502, 503 |
| `crates/sifr_lowering/src/lower/mod_context.rs:754` `can_import_private_stdlib_declarations` | read | layered view + demanded record handles | 754 |
| `crates/sifr_lowering/src/lower/mod_context.rs:759` `lower_module` | read, borrow/retain | layered view + demanded record handles | 760 |
| `crates/sifr_lowering/src/lower/mod_context.rs:763` `lower_module_sysroot_public_stdlib` | read, borrow/retain | layered view + demanded record handles | 763, 768 |
| `crates/sifr_lowering/src/lower/mod_context.rs:772` `lower_module_sysroot_public_stdlib_with_externals` | read, borrow/retain | layered view + demanded record handles | 772, 774, 778 |
| `crates/sifr_lowering/src/lower/mod_context.rs:782` `lower_module_sysroot_private_declaration_with_externals` | read, borrow/retain | layered view + demanded record handles | 784, 788 |
| `crates/sifr_lowering/src/lower/mod_context.rs:791` `lower_module_with_externals` | read, borrow/retain | layered view + demanded record handles | 793, 796 |
| `crates/sifr_lowering/src/lower/mod_context.rs:799` `lower_module_with_externals_and_name` | read, borrow/retain | layered view + demanded record handles | 802, 805 |
| `crates/sifr_lowering/src/lower/mod_context.rs:808` `lower_module_with_externals_name_and_options` | read, borrow/retain | layered view + demanded record handles | 811, 817 |
| `crates/sifr_lowering/src/lower/mod_impl.rs:15` `lower_module_impl` | read, enumerate, mutate overlay/producer, clone projection, borrow/retain | layered view + demanded record handles | 17, 20, 72, 74, 298, 353, 357, 359, 378, 379, 380, 381, 387, 391, 402, 407, 412, 413, 418, 428, 438, 447, 449, 459, 471, 479, 491, 497, 498, 503, 520, 530, 536, 546, 556, 568, 581, 585, 587, 605, 620, 626, 629, 630, 632, 634, 664, 671, 681, 693, 702, 707, 719, 729, 739, 756, 766, 784, 791, 792, 794 |
| `crates/sifr_lowering/src/lower/module_constants_lowering.rs:314` `lower_private_declaration_constants` | read, borrow/retain | layered view + demanded record handles | 318 |
| `crates/sifr_lowering/src/lower/module_constants_lowering.rs:366` `annotated_scalar_module_constant_type_mismatch_is_diagnostic` | read, enumerate, borrow/retain | test setup/parity | 370 |
| `crates/sifr_lowering/src/lower/module_constants_lowering.rs:383` `private_declaration_scalar_module_constant_alias_is_diagnostic` | read, enumerate, borrow/retain | test setup/parity | 388 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:62` `missing_stdlib_member_has_name_code` | read, enumerate, mutate overlay/producer | test setup/parity | 62, 65, 66, 69 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:82` `deferred_stdlib_module_has_import_code` | read, enumerate | test setup/parity | 82 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:135` `public_sysroot_stdlib_source_can_import_private_declarations` | read, enumerate, mutate overlay/producer | test setup/parity | 135, 138, 139, 149 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:162` `public_sysroot_stdlib_source_resolves_compiled_private_constants` | read, enumerate, mutate overlay/producer | test setup/parity | 162, 165, 166, 171 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:184` `public_sysroot_stdlib_source_rejects_uncompiled_private_import_name` | read, enumerate, mutate overlay/producer | test setup/parity | 184, 187, 188, 193 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:207` `public_sysroot_stdlib_source_resolves_compiled_private_classes` | read, enumerate, mutate overlay/producer | test setup/parity | 207, 210, 211, 226 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:235` `attached_api_set_import_requires_the_stored_canonical_identity` | read, mutate overlay/producer | test setup/parity | 238, 239, 253, 267 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:272` `builtin_open_preserves_an_aliased_imported_text_handle_identity` | read, mutate overlay/producer | test setup/parity | 275, 276, 296 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:301` `builtin_open_preserves_an_aliased_imported_binary_handle_identity` | read, mutate overlay/producer | test setup/parity | 304, 305, 325 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:330` `builtin_binary_open_supplies_the_read_bytes_size_default` | read, mutate overlay/producer | test setup/parity | 333, 334, 360 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:416` `builtin_open_inferred_bindings_keep_canonical_handle_identities` | read, enumerate, mutate overlay/producer | test setup/parity | 435, 436, 463 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:501` `private_sysroot_declaration_source_cannot_import_private_declarations` | read, enumerate, borrow/retain | test setup/parity | 506 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:534` `bare_stdlib_import_from_has_targeted_import_code_and_args` | read, enumerate | test setup/parity | 534 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:550` `bare_stdlib_import_from_alias_preserves_imported_names_arg` | read, enumerate | test setup/parity | 550 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:564` `bare_stdlib_import_statement_has_targeted_import_code` | read, enumerate | test setup/parity | 564 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:580` `bare_stdlib_submodule_root_fallback_reports_unavailable_embedded_module` | read, enumerate | test setup/parity | 580 |
| `crates/sifr_lowering/src/lower/name_import_diagnostics_tests.rs:610` `private_import_member_has_import_code` | read, enumerate, mutate overlay/producer | test setup/parity | 613, 614, 617 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:5` `resolve_compiled_private_imports` | read, borrow/retain | layered view + demanded record handles | 7, 13, 18, 19, 20 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:37` `resolve_function` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 39, 44, 54 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:58` `import_function_metadata` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 60, 67, 72, 75, 78, 81, 84, 87, 95 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:105` `resolve_class` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 107, 112, 122, 127, 128, 136, 139, 140 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:144` `import_class_type_params` | read, borrow/retain | layered view + demanded record handles | 146, 151 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:169` `register_constructor` | read, borrow/retain | layered view + demanded record handles | 171, 182, 185, 188 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:193` `import_class_bounds` | read, borrow/retain | layered view + demanded record handles | 195, 200 |
| `crates/sifr_lowering/src/lower/private_stdlib_imports.rs:210` `resolve_constant` | read, borrow/retain | layered view + demanded record handles | 212, 217, 225 |
| `crates/sifr_lowering/src/lower/python_arrow_contract_tests.rs:80` `bridge_arrow_producer_rewrites_to_package_runtime_identity` | read, borrow/retain | test setup/parity | 88 |
| `crates/sifr_lowering/src/lower/python_async_tests.rs:29` `python_externals` | read, mutate overlay/producer | test setup/parity | 29, 71, 72, 75, 76, 79, 80 |
| `crates/sifr_lowering/src/lower/python_bridge_tests.rs:19` `bridge_target_is_a_hard_error_without_package_authority` | read, enumerate, borrow/retain | test setup/parity | 24 |
| `crates/sifr_lowering/src/lower/python_bridge_tests.rs:39` `bridge_target_rewrites_to_resolved_package_when_activated` | read, borrow/retain | test setup/parity | 44 |
| `crates/sifr_lowering/src/lower/python_bridge_tests.rs:67` `bridge_target_requires_an_inventoried_module` | read, enumerate, borrow/retain | test setup/parity | 72 |
| `crates/sifr_lowering/src/lower/python_bridge_tests.rs:96` `reserved_bridge_target_cannot_be_reinterpreted_as_an_external_distribution` | read, enumerate, borrow/retain | test setup/parity | 101, 124 |
| `crates/sifr_lowering/src/lower/python_bridge_tests.rs:142` `bridge_authority_is_scoped_to_the_declaring_module` | read, enumerate, borrow/retain | test setup/parity | 147 |
| `crates/sifr_lowering/src/lower/python_bridge_tests.rs:170` `nested_inventoried_bridge_module_rewrites_to_the_resolved_package` | read, borrow/retain | test setup/parity | 176 |
| `crates/sifr_lowering/src/lower/python_buffer_contract_tests.rs:57` `bridge_buffer_producer_rewrites_to_package_runtime_identity` | read, borrow/retain | test setup/parity | 65 |
| `crates/sifr_lowering/src/lower/python_interop_callback_tests.rs:100` `callback_declaration_keeps_same_basename_error_payloads_distinct` | read, mutate overlay/producer | test setup/parity | 126, 127, 132, 134 |
| `crates/sifr_lowering/src/lower/python_trust_tests.rs:7` `python_externals` | read, mutate overlay/producer | test setup/parity | 7, 36, 37, 40, 41, 42 |
| `crates/sifr_lowering/src/lower/return_lowering.rs:13` `lower_return` | read | layered view + demanded record handles | 83 |
| `crates/sifr_lowering/src/lower/rust_interop_structural_tests.rs:10` `structural_externals` | read, mutate overlay/producer | test setup/parity | 10, 11, 12, 72 |
| `crates/sifr_lowering/src/lower/rust_interop_structural_tests.rs:639` `structural_bound_rejects_imported_enum_with_unrepresentable_metadata` | read, enumerate, mutate overlay/producer | test setup/parity | 640, 641, 652, 680 |
| `crates/sifr_lowering/src/lower/template_string_tests.rs:6` `lower` | read, borrow/retain | test setup/parity | 11 |
| `crates/sifr_lowering/src/lower/type_bounds.rs:318` `supports_static_program_type` | read, enumerate | layered view + demanded record handles | 333 |
| `crates/sifr_lowering/src/lower/workload_annotations.rs:234` `offload_workload` | read | layered view + demanded record handles | 243 |
| `crates/sifr_lowering/src/lower/workload_annotations.rs:253` `known_stdlib_offload_target` | read | layered view + demanded record handles | 253 |
| `crates/sifr_frontend/src/callable_exports.rs:49` `copy_imported` | read, borrow/retain | layered view + demanded record handles | 51, 56, 63 |
| `crates/sifr_frontend/src/callable_exports.rs:76` `store` | read, mutate overlay/producer | layered view + demanded record handles | 76, 77 |
| `crates/sifr_frontend/src/callable_identities.rs:11` `resolve` | read, enumerate, borrow/retain | layered view + demanded record handles | 14, 20 |
| `crates/sifr_frontend/src/callable_identities.rs:48` `contract_for_identity` | read, borrow/retain | layered view + demanded record handles | 51, 57 |
| `crates/sifr_frontend/src/callable_identities.rs:106` `free_or_constructor` | read, enumerate | layered view + demanded record handles | 141, 155 |
| `crates/sifr_frontend/src/callable_identities.rs:173` `contract` | read, enumerate, clone projection | layered view + demanded record handles | 230, 238, 263, 289 |
| `crates/sifr_frontend/src/callable_identities.rs:297` `method` | read, enumerate | layered view + demanded record handles | 323 |
| `crates/sifr_frontend/src/callable_identities.rs:388` `imported_constructor` | read, enumerate | layered view + demanded record handles | 400 |
| `crates/sifr_frontend/src/callable_identities.rs:449` `type_identity` | read, enumerate | layered view + demanded record handles | 470 |
| `crates/sifr_frontend/src/class_method_exports.rs:37` `imported_instance_methods` | read, borrow/retain | layered view + demanded record handles | 38, 42 |
| `crates/sifr_frontend/src/class_method_exports.rs:90` `adapted_handler_method` | read, enumerate, borrow/retain | layered view + demanded record handles | 96, 111, 126, 170 |
| `crates/sifr_frontend/src/class_method_exports.rs:193` `adapted_imported_boundary_method` | read, enumerate, borrow/retain | layered view + demanded record handles | 199, 201 |
| `crates/sifr_frontend/src/class_method_exports.rs:220` `structural_method_map` | read, enumerate, mutate overlay/producer, clone projection, borrow/retain | layered view + demanded record handles | 225, 293 |
| `crates/sifr_frontend/src/class_method_exports.rs:331` `record_imported` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 333, 338, 340 |
| `crates/sifr_frontend/src/class_method_exports.rs:350` `store` | read, mutate overlay/producer | layered view + demanded record handles | 350, 351, 355 |
| `crates/sifr_frontend/src/descriptor_exports.rs:8` `erase_marker_imports` | read, borrow/retain | layered view + demanded record handles | 8, 10, 11 |
| `crates/sifr_frontend/src/descriptor_exports.rs:81` `store` | read, enumerate, mutate overlay/producer, clone projection | layered view + demanded record handles | 85, 140, 147, 154, 161, 172, 178, 184, 190, 196, 202, 208, 214 |
| `crates/sifr_frontend/src/early_adapters/adapter_input.rs:12` `adapter_input` | read, enumerate, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 15, 20, 37 |
| `crates/sifr_frontend/src/early_adapters/adapter_input.rs:56` `inherited_values` | read, borrow/retain | layered view + demanded record handles | 59, 62 |
| `crates/sifr_frontend/src/early_adapters/handler_plans.rs:13` `validate_handlers` | read, enumerate, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 17, 21 |
| `crates/sifr_frontend/src/early_adapters/handler_plans.rs:113` `inherited_handler_plans` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 116, 140 |
| `crates/sifr_frontend/src/early_adapters/handler_plans.rs:153` `inherited_handlers_require_the_declared_data_parent_shape_locally` | read, borrow/retain | test setup/parity | 172 |
| `crates/sifr_frontend/src/early_adapters/inheritance.rs:9` `declaration_value` | read, enumerate, borrow/retain | layered view + demanded record handles | 12, 56, 65 |
| `crates/sifr_frontend/src/early_adapters/inheritance.rs:160` `parent_bindings` | read, enumerate, borrow/retain | layered view + demanded record handles | 163, 183 |
| `crates/sifr_frontend/src/early_adapters/inheritance.rs:285` `parent_selection` | read, enumerate, borrow/retain | layered view + demanded record handles | 288, 302 |
| `crates/sifr_frontend/src/early_adapters/inheritance.rs:393` `canonical_imported_parent_wins_over_a_colliding_local_selection` | read, mutate overlay/producer | test setup/parity | 395, 396, 402 |
| `crates/sifr_frontend/src/early_adapters/inheritance.rs:408` `canonical_local_parent_keeps_the_local_selection` | read | test setup/parity | 410, 411 |
| `crates/sifr_frontend/src/early_adapters/inheritance.rs:417` `imported_parent_bindings_ignore_a_colliding_local_class` | read, mutate overlay/producer | test setup/parity | 421, 422, 431 |
| `crates/sifr_frontend/src/early_adapters.rs:31` `run` | read, borrow/retain | layered view + demanded record handles | 34, 42 |
| `crates/sifr_frontend/src/early_adapters.rs:55` `run_one` | read, enumerate, borrow/retain | layered view + demanded record handles | 58, 92, 108, 152, 176 |
| `crates/sifr_frontend/src/early_adapters.rs:197` `const_functions` | read, enumerate, borrow/retain | layered view + demanded record handles | 199, 217 |
| `crates/sifr_frontend/src/early_adapters.rs:235` `validate_issues` | read, enumerate, borrow/retain | layered view + demanded record handles | 238, 306 |
| `crates/sifr_frontend/src/early_adapters.rs:335` `validate_plan` | read, enumerate, borrow/retain | layered view + demanded record handles | 338, 355, 371, 381, 414 |
| `crates/sifr_frontend/src/early_adapters.rs:436` `attached_api_set_exists` | read, enumerate, borrow/retain | layered view + demanded record handles | 439, 448 |
| `crates/sifr_frontend/src/early_adapters.rs:452` `validate_fields` | read, enumerate, borrow/retain | layered view + demanded record handles | 457, 492, 513, 541 |
| `crates/sifr_frontend/src/early_adapters.rs:586` `parse_metadata` | read, enumerate, borrow/retain | layered view + demanded record handles | 590, 626 |
| `crates/sifr_frontend/src/early_adapters.rs:655` `normalized_field_types` | read, enumerate, borrow/retain | layered view + demanded record handles | 658, 673, 682 |
| `crates/sifr_frontend/src/early_adapters.rs:730` `expected_field_contracts` | read, enumerate, borrow/retain | layered view + demanded record handles | 733, 738 |
| `crates/sifr_frontend/src/editor_semantics.rs:50` `editor_semantics_from_module` | read, enumerate, borrow/retain | layered view + demanded record handles | 56, 58 |
| `crates/sifr_frontend/src/editor_semantics.rs:582` `callable_signatures` | read, enumerate, mutate overlay/producer, clone projection, borrow/retain | layered view + demanded record handles | 585, 588 |
| `crates/sifr_frontend/src/export_type_localization.rs:5` `should_export_callable` | read | layered view + demanded record handles | 6 |
| `crates/sifr_frontend/src/export_type_localization.rs:28` `imported_class_ancestry` | read, enumerate, borrow/retain | layered view + demanded record handles | 30, 34 |
| `crates/sifr_frontend/src/export_type_localization.rs:126` `reexport_class_aliases` | read, enumerate, borrow/retain | layered view + demanded record handles | 128, 132 |
| `crates/sifr_frontend/src/export_type_localization.rs:153` `copy_function_generic_metadata` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 154, 161, 168 |
| `crates/sifr_frontend/src/export_type_localization.rs:177` `copy_class_generic_metadata` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 178, 186, 194 |
| `crates/sifr_frontend/src/frontend_product.rs:33` `compile_frontend_product` | read, enumerate, mutate overlay/producer, clone projection | layered view + demanded record handles | 35, 53, 71, 75, 76, 81, 83 |
| `crates/sifr_frontend/src/frontend_product.rs:89` `compile_frontend_product_module` | read, mutate overlay/producer | layered view + demanded record handles | 93, 97, 102 |
| `crates/sifr_frontend/src/frontend_product.rs:108` `compile_prepared_frontend_product_module` | read, mutate overlay/producer | layered view + demanded record handles | 112, 119, 134 |
| `crates/sifr_frontend/src/graph_cache_and_queries/external_overlay.rs:7` `clear_module_caches` | read, enumerate | layered view + demanded record handles | 22 |
| `crates/sifr_frontend/src/graph_cache_and_queries/external_overlay.rs:31` `rebuild_external_defs_from_lowered` | read | layered view + demanded record handles | 34 |
| `crates/sifr_frontend/src/graph_cache_and_queries/external_overlay.rs:61` `prepare_external_defs` | read, mutate overlay/producer | layered view + demanded record handles | 63 |
| `crates/sifr_frontend/src/graph_cache_and_queries/external_overlay.rs:81` `context` | read | layered view + demanded record handles | 81 |
| `crates/sifr_frontend/src/graph_cache_and_queries/external_overlay.rs:93` `source_overlay_failed_and_deleted_exports_are_invalidated` | read | test setup/parity | 96, 105, 123, 137, 152 |
| `crates/sifr_frontend/src/graph_cache_and_queries/external_overlay.rs:159` `source_overlay_projects_share_only_immutable_baseline` | read | test setup/parity | 160, 190, 191, 195, 203 |
| `crates/sifr_frontend/src/graph_cache_and_queries/loaders.rs:46` `load_single_file` | read | layered view + demanded record handles | 47 |
| `crates/sifr_frontend/src/graph_cache_and_queries/loaders.rs:50` `load_single_file_with_external_defs` | read | layered view + demanded record handles | 52, 56 |
| `crates/sifr_frontend/src/graph_cache_and_queries/loaders.rs:61` `load_single_file_with_external_defs_and_auxiliary_sources` | read, clone projection, borrow/retain | layered view + demanded record handles | 63, 98, 99 |
| `crates/sifr_frontend/src/graph_cache_and_queries/loaders.rs:107` `load_project` | read | layered view + demanded record handles | 111 |
| `crates/sifr_frontend/src/graph_cache_and_queries/loaders.rs:114` `load_project_with_external_defs` | read | layered view + demanded record handles | 117, 122 |
| `crates/sifr_frontend/src/graph_cache_and_queries/loaders.rs:127` `load_project_with_external_defs_and_auxiliary_sources` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 130, 234, 235 |
| `crates/sifr_frontend/src/graph_cache_and_queries/persistent_checks.rs:135` `from_resolved_sources` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 138 |
| `crates/sifr_frontend/src/graph_cache_and_queries/persistent_checks.rs:194` `from_resolved_check_modules` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 197 |
| `crates/sifr_frontend/src/graph_cache_and_queries/source_updates.rs:9` `update_module_source` | read, enumerate, clone projection | layered view + demanded record handles | 79 |
| `crates/sifr_frontend/src/graph_cache_and_queries.rs:273` `compile_module_hir` | read, borrow/retain | layered view + demanded record handles | 276, 279 |
| `crates/sifr_frontend/src/graph_cache_and_queries.rs:282` `compile_module_hir_with_source` | read, borrow/retain | layered view + demanded record handles | 285, 292 |
| `crates/sifr_frontend/src/graph_cache_and_queries.rs:299` `compile_module_hir_with_source_and_options` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 302, 307, 315, 326, 331, 349, 407, 436 |
| `crates/sifr_frontend/src/graph_cache_and_queries.rs:607` `ensure_lowered` | read, enumerate, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 645, 653, 670 |
| `crates/sifr_frontend/src/graph_cache_and_queries.rs:733` `ensure_analysis` | read | layered view + demanded record handles | 761 |
| `crates/sifr_frontend/src/handler_ancestry.rs:16` `resolve` | read, enumerate, borrow/retain | layered view + demanded record handles | 22, 31 |
| `crates/sifr_frontend/src/handler_ancestry.rs:37` `walk_local` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 43, 85, 90 |
| `crates/sifr_frontend/src/handler_ancestry.rs:132` `compile` | read, borrow/retain | layered view + demanded record handles | 132, 137 |
| `crates/sifr_frontend/src/handler_ancestry.rs:144` `unspecialized_generic_ancestor_keeps_its_symbolic_argument` | read, borrow/retain | test setup/parity | 153 |
| `crates/sifr_frontend/src/handler_ancestry.rs:163` `unspecialized_generic_ancestor_rejects_missing_root_arguments` | read, borrow/retain | test setup/parity | 174 |
| `crates/sifr_frontend/src/handler_ancestry.rs:181` `imported_generic_ancestor_rejects_parameter_arity_mismatch` | read, mutate overlay/producer | test setup/parity | 182, 183, 184, 188, 190, 205 |
| `crates/sifr_frontend/src/package_issues.rs:108` `issue_templates` | read, borrow/retain | layered view + demanded record handles | 109, 113 |
| `crates/sifr_frontend/src/persistence/tests.rs:159` `analyzes_the_captured_bytes_despite_writer` | read | test setup/parity | 172 |
| `crates/sifr_frontend/src/persistence/tests.rs:350` `dx12_completed_deterministic_source_error_diagnostics_roundtrip` | read, enumerate | test setup/parity | 359 |
| `crates/sifr_frontend/src/query_diagnostics/const_reexports.rs:16` `copy_const_function_and_defaults` | read, mutate overlay/producer, clone projection, borrow/retain | layered view + demanded record handles | 17, 24, 33 |
| `crates/sifr_frontend/src/query_diagnostics/rust_class_exports.rs:24` `record_imported` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 25, 32, 39 |
| `crates/sifr_frontend/src/query_diagnostics/rust_class_exports.rs:48` `replace_module` | read, mutate overlay/producer | layered view + demanded record handles | 49, 55, 57, 62, 64 |
| `crates/sifr_frontend/src/query_diagnostics.rs:179` `collect_module_exports` | read, enumerate, mutate overlay/producer, clone projection | layered view + demanded record handles | 182, 209, 211, 361, 377, 378, 389, 396, 403, 410, 418, 428, 430, 434, 446, 452, 463, 475, 481, 493, 494, 498, 502, 505, 508, 510, 512, 517, 523, 529, 534, 536, 542, 548, 553, 555, 561, 566, 571, 576, 581, 586, 590, 591, 595 |
| `crates/sifr_frontend/src/slot_table.rs:51` `resolve_method_slots` | read, enumerate, borrow/retain | layered view + demanded record handles | 57, 128, 146, 180 |
| `crates/sifr_frontend/src/slot_table.rs:371` `owner_type_for_identity` | read, enumerate, borrow/retain | layered view + demanded record handles | 376, 416 |
| `crates/sifr_frontend/src/slot_table.rs:423` `resolve_method_slot` | read, enumerate, borrow/retain | layered view + demanded record handles | 429, 497, 499, 535, 543 |
| `crates/sifr_frontend/src/slot_table.rs:598` `finish_method_slot` | read, borrow/retain | layered view + demanded record handles | 602, 713, 714 |
| `crates/sifr_frontend/src/slot_table.rs:753` `structural_slot_type_supported` | read, enumerate, borrow/retain | layered view + demanded record handles | 757, 782 |
| `crates/sifr_frontend/src/slot_table_tests.rs:7` `compile` | read, borrow/retain | test setup/parity | 10, 16 |
| `crates/sifr_frontend/src/slot_table_tests.rs:21` `install_specializer` | read, enumerate, mutate overlay/producer | test setup/parity | 21, 43, 44 |
| `crates/sifr_frontend/src/slot_table_tests.rs:48` `typed_empty_method_slot_field_emits_no_table` | read, mutate overlay/producer | test setup/parity | 49, 50, 60 |
| `crates/sifr_frontend/src/slot_table_tests.rs:91` `reexported_method_slot_resolves_by_qualified_owner_identity` | read, mutate overlay/producer | test setup/parity | 92, 93, 105, 108, 112, 115, 127 |
| `crates/sifr_frontend/src/slot_table_tests.rs:139` `unavailable_slot_target_uses_method_diagnostic` | read, enumerate, mutate overlay/producer | test setup/parity | 140, 141, 151 |
| `crates/sifr_frontend/src/slot_table_tests.rs:157` `infallible_slot_uses_checked_output_signature` | read, mutate overlay/producer | test setup/parity | 158, 159, 174 |
| `crates/sifr_frontend/src/slot_table_tests.rs:184` `receiver_only_slot_records_receiver_input_role` | read, mutate overlay/producer | test setup/parity | 185, 186, 200 |
| `crates/sifr_frontend/src/slot_table_tests.rs:210` `receiver_slot_with_owned_value_uses_composite_structural_input` | read, mutate overlay/producer | test setup/parity | 211, 212, 226 |
| `crates/sifr_frontend/src/slot_table_tests.rs:242` `receiver_slot_requires_an_owned_composite_value_input` | read, enumerate, mutate overlay/producer | test setup/parity | 243, 244, 262 |
| `crates/sifr_frontend/src/slot_table_tests.rs:276` `conflicting_context_borrow_modes_use_context_diagnostic` | read, enumerate, mutate overlay/producer | test setup/parity | 277, 279, 305 |
| `crates/sifr_frontend/src/slot_table_tests.rs:311` `static_method_slot_requires_one_value_input` | read, enumerate, mutate overlay/producer | test setup/parity | 312, 313, 329 |
| `crates/sifr_frontend/src/slot_table_tests.rs:343` `method_slot_rejects_more_than_value_and_context_inputs` | read, enumerate, mutate overlay/producer | test setup/parity | 344, 345, 361 |
| `crates/sifr_frontend/src/specialization_runner/tests/integer_boundary_tests.rs:4` `non_package_integer_boundary_fixture_fails_closed_for_missing_or_unsafe_policy` | read, borrow/retain | test setup/parity | 12, 27, 38 |
| `crates/sifr_frontend/src/specialization_runner.rs:19` `run_specializations` | read, enumerate, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 22, 65, 78, 136, 148 |
| `crates/sifr_frontend/src/specialization_runner.rs:339` `compile` | read, borrow/retain | layered view + demanded record handles | 342, 348 |
| `crates/sifr_frontend/src/specialization_runner.rs:433` `package_warning_flows_through_frontend_warning_channel` | read, enumerate, mutate overlay/producer | test setup/parity | 434, 438, 441, 443, 471, 478, 494, 506, 514 |
| `crates/sifr_frontend/src/specialization_runner.rs:548` `imported_annotated_methods_preserve_nested_program_identity` | read, mutate overlay/producer | test setup/parity | 550, 554, 557, 577, 578, 590 |
| `crates/sifr_frontend/src/specialization_runner.rs:549` `compile_consumer` | read, mutate overlay/producer | test setup/parity | 550, 554, 557, 577, 578, 590 |
| `crates/sifr_frontend/src/specialization_runner.rs:610` `qualified_reserved_slot_list_resolves_checked_method_contract` | read, mutate overlay/producer | test setup/parity | 611, 624, 627, 643 |
| `crates/sifr_frontend/src/specialization_runner.rs:659` `invalid_reserved_slot_list_uses_slot_diagnostic` | read, enumerate, mutate overlay/producer | test setup/parity | 660, 673, 676, 686 |
| `crates/sifr_frontend/src/specialization_runner.rs:696` `reexported_nominal_shapes_match_local_shapes_without_consumer_fallbacks` | read, mutate overlay/producer | test setup/parity | 697, 740, 748, 754, 759, 762, 773, 781, 787, 793 |
| `crates/sifr_frontend/src/specialization_runner.rs:824` `package_fatal_flows_through_registry_owned_frontend_error` | read, mutate overlay/producer | test setup/parity | 825, 829, 832, 834 |
| `crates/sifr_frontend/src/specialization_runner.rs:843` `undeclared_package_issue_fails_closed_as_malformed` | read, mutate overlay/producer | test setup/parity | 844, 848, 851, 853 |
| `crates/sifr_frontend/src/specialization_runner.rs:858` `package_cannot_forge_a_source_origin` | read, mutate overlay/producer | test setup/parity | 859, 862, 864, 866 |
| `crates/sifr_frontend/src/sql_editor_tests.rs:8` `documents_cover_sql_tokens_holes_semantics_and_fragment_scope` | read, enumerate, borrow/retain | test setup/parity | 14 |
| `crates/sifr_frontend/src/structural_shape/ancestry_tests.rs:39` `unresolved_handler_ancestry_uses_an_opaque_checked_contract` | read, borrow/retain | test setup/parity | 94 |
| `crates/sifr_frontend/src/structural_shape/generic_fields.rs:11` `effective_fields` | read, enumerate, borrow/retain | layered view + demanded record handles | 19, 28, 55 |
| `crates/sifr_frontend/src/structural_shape/generic_fields.rs:64` `class_type_params` | read, enumerate, borrow/retain | layered view + demanded record handles | 70, 84 |
| `crates/sifr_frontend/src/structural_shape/methods.rs:125` `described_methods` | read, enumerate | layered view + demanded record handles | 131, 202, 217 |
| `crates/sifr_frontend/src/structural_shape/methods.rs:225` `described_handler` | read, enumerate | layered view + demanded record handles | 232, 285, 314, 323, 331, 355 |
| `crates/sifr_frontend/src/structural_shape/methods.rs:383` `imported_handler_method` | read, enumerate | layered view + demanded record handles | 385, 392 |
| `crates/sifr_frontend/src/structural_shape/methods.rs:402` `imported_handler_detail` | read, enumerate | layered view + demanded record handles | 408, 417, 420, 427 |
| `crates/sifr_frontend/src/structural_shape/methods.rs:438` `described_exported_methods` | read, enumerate, clone projection | layered view + demanded record handles | 448, 486, 507, 530 |
| `crates/sifr_frontend/src/structural_shape/methods.rs:543` `described_method` | read, enumerate | layered view + demanded record handles | 556, 581, 598, 605 |
| `crates/sifr_frontend/src/structural_shape/nominal_types.rs:8` `describe_enum` | read, enumerate, borrow/retain | layered view + demanded record handles | 14, 25 |
| `crates/sifr_frontend/src/structural_shape/nominal_types.rs:47` `describe_newtype` | read, borrow/retain | layered view + demanded record handles | 53, 65, 73 |
| `crates/sifr_frontend/src/structural_shape/nominal_types.rs:86` `declaration_metadata_for` | read, enumerate, borrow/retain | layered view + demanded record handles | 92, 104 |
| `crates/sifr_frontend/src/structural_shape/tests.rs:6` `describe_type` | read, borrow/retain | test setup/parity | 7 |
| `crates/sifr_frontend/src/structural_shape.rs:110` `describe_type_with_externals` | read, borrow/retain | layered view + demanded record handles | 114, 116 |
| `crates/sifr_frontend/src/structural_shape.rs:119` `describe_type_inner` | read, borrow/retain | layered view + demanded record handles | 123, 130 |
| `crates/sifr_frontend/src/structural_shape.rs:140` `describe_node` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 144, 161, 169, 176, 184, 191, 196, 209, 221, 237 |
| `crates/sifr_frontend/src/structural_shape.rs:248` `describe_union` | read, enumerate, borrow/retain | layered view + demanded record handles | 252, 264, 271 |
| `crates/sifr_frontend/src/structural_shape.rs:277` `describe_class` | read, enumerate, mutate overlay/producer, clone projection, borrow/retain | layered view + demanded record handles | 284, 313, 325, 348, 352, 357, 361, 376, 383, 401, 405, 410, 416, 432, 438 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:12` `compile` | read, borrow/retain | test setup/parity | 12, 17 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:35` `imported_generic_parent_handler_uses_the_concrete_child_argument` | read, mutate overlay/producer | test setup/parity | 36, 49, 51, 67, 110 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:127` `imported_adapted_generic_field_plan_uses_the_concrete_argument` | read, mutate overlay/producer | test setup/parity | 128, 135, 157, 162, 168 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:181` `imported_adapted_field_substitution_respects_nested_generic_scope` | read, mutate overlay/producer | test setup/parity | 182, 193, 232, 237, 243, 264, 281, 287 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:306` `fully_imported_adapted_handler_preserves_metadata_and_concrete_types` | read, mutate overlay/producer | test setup/parity | 307, 320, 356, 361, 367 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:391` `imported_transitive_generic_handler_uses_the_concrete_child_argument` | read, enumerate, mutate overlay/producer | test setup/parity | 392, 408, 453, 455, 464, 480, 515 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:538` `recollection_removes_stale_structural_shape_exports` | read, mutate overlay/producer | test setup/parity | 539, 551, 553, 554, 555, 556, 557, 559, 560, 561, 562, 563, 564 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:568` `structural_method_storage_is_allocated_only_while_demanded` | read, mutate overlay/producer | test setup/parity | 569, 570, 571, 572, 580, 581, 582, 584, 585, 586, 588, 589, 590, 591, 592, 593, 594, 595, 596 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:600` `imported_nominal_ignores_colliding_local_class_shape` | read, mutate overlay/producer | test setup/parity | 601, 618, 624, 626, 643, 649 |
| `crates/sifr_frontend/src/structural_shape_import_tests.rs:659` `public_import_preserves_private_generic_nested_shape` | read, mutate overlay/producer | test setup/parity | 660, 677, 683, 685, 687, 694, 702, 708 |
| `crates/sifr_frontend/src/template_documents.rs:209` `lowered_template` | read, borrow/retain | layered view + demanded record handles | 214 |
| `crates/sifr_frontend/src/typed_descriptors/nested_const_calls.rs:7` `nested_const_call` | read, enumerate | layered view + demanded record handles | 67 |
| `crates/sifr_frontend/src/typed_descriptors.rs:21` `collect` | read, borrow/retain | layered view + demanded record handles | 25, 27, 34 |
| `crates/sifr_frontend/src/typed_descriptors.rs:68` `new` | read, enumerate, borrow/retain | layered view + demanded record handles | 72, 80, 97 |
| `crates/sifr_frontend/src/typed_descriptors.rs:400` `function_type` | read, enumerate | layered view + demanded record handles | 410 |
| `crates/sifr_frontend/src/typed_descriptors.rs:486` `argument_value` | read, enumerate | layered view + demanded record handles | 496 |
| `crates/sifr_frontend/src/typed_descriptors.rs:540` `const_call_target` | read, enumerate, clone projection | layered view + demanded record handles | 577 |
| `crates/sifr_frontend/src/typed_descriptors.rs:602` `function_defaults` | read | layered view + demanded record handles | 614 |
| `crates/sifr_frontend/src/typed_descriptors.rs:622` `const_functions` | read, enumerate | layered view + demanded record handles | 633 |
| `crates/sifr_frontend/src/workspace_residency.rs:123` `refresh_after_reload` | read, enumerate, borrow/retain | layered view + demanded record handles | 163 |
| `crates/sifr_frontend/src/workspace_residency.rs:198` `retain_stdlib_root` | read | layered view + demanded record handles | 198, 199, 202 |
| `crates/sifr_frontend/src/workspace_session.rs:247` `open_project_with_external_defs` | read | layered view + demanded record handles | 249, 252 |
| `crates/sifr_frontend/src/workspace_session.rs:257` `open_project_with_external_defs_and_auxiliary_sources` | read | layered view + demanded record handles | 259, 264 |
| `crates/sifr_frontend/src/workspace_session.rs:272` `project` | read | layered view + demanded record handles | 273 |
| `crates/sifr_frontend/src/workspace_session.rs:277` `project_with_external_defs` | read | layered view + demanded record handles | 277, 278 |
| `crates/sifr_frontend/src/workspace_session.rs:282` `project_with_external_defs_and_auxiliary_sources` | read, borrow/retain | layered view + demanded record handles | 284, 299 |
| `crates/sifr_frontend/src/workspace_session.rs:304` `open_single_file` | read | layered view + demanded record handles | 305 |
| `crates/sifr_frontend/src/workspace_session.rs:308` `open_single_file_with_external_defs` | read | layered view + demanded record handles | 310, 314 |
| `crates/sifr_frontend/src/workspace_session.rs:319` `open_single_file_with_external_defs_and_auxiliary_sources` | read, clone projection | layered view + demanded record handles | 321, 327 |
| `crates/sifr_frontend/src/workspace_session.rs:344` `single_file` | read | layered view + demanded record handles | 345 |
| `crates/sifr_frontend/src/workspace_session.rs:349` `single_file_with_external_defs` | read | layered view + demanded record handles | 352, 357 |
| `crates/sifr_frontend/src/workspace_session.rs:363` `single_file_with_external_defs_and_auxiliary_sources` | read, borrow/retain | layered view + demanded record handles | 366, 376 |
| `crates/sifr_frontend/src/workspace_session.rs:381` `new` | read, borrow/retain | layered view + demanded record handles | 385 |
| `crates/sifr_frontend/src/workspace_session.rs:579` `retain_stdlib_root` | read | layered view + demanded record handles | 579, 580 |
| `crates/sifr_frontend/src/workspace_session_tests.rs:273` `config_registry_pending_reload_and_extra_watch_roots_are_snapshot_visible` | read, enumerate, borrow/retain | test setup/parity | 281 |
| `crates/sifr_compiler_services/src/compiler_context.rs:28` `with_metadata_override` | read, borrow/retain | layered view + demanded record handles | 30 |
| `crates/sifr_compiler_services/src/compiler_context.rs:33` `from_resolved` | read, borrow/retain | layered view + demanded record handles | 44 |
| `crates/sifr_compiler_services/src/compiler_context.rs:50` `shares_metadata_generation` | read, borrow/retain | layered view + demanded record handles | 51 |
| `crates/sifr_compiler_services/src/compiler_context.rs:74` `without_cached_metadata` | read, borrow/retain | layered view + demanded record handles | 76 |
| `crates/sifr_compiler_services/src/compiler_context.rs:96` `with_cache_root` | read, borrow/retain | layered view + demanded record handles | 98 |
| `crates/sifr_compiler_services/src/compiler_context.rs:130` `metadata_provider` | read, borrow/retain | layered view + demanded record handles | 133 |
| `crates/sifr_compiler_services/src/compiler_context.rs:157` `stdlib_navigation` | read, borrow/retain | layered view + demanded record handles | 157, 161 |
| `crates/sifr_compiler_services/src/compiler_context.rs:169` `metadata_stats` | read | layered view + demanded record handles | 171 |
| `crates/sifr_compiler_services/src/compiler_context.rs:223` `dx_identity_contexts_do_not_share_incompatible_stdlib_owners` | read, borrow/retain | test setup/parity | 223, 226, 228 |
| `crates/sifr_compiler_services/src/compiler_context.rs:272` `compiled_input_tokens` | read | layered view + demanded record handles | 283, 284 |
| `crates/sifr_compiler_services/src/export_policy.rs:1` `should_export_callable` | read | layered view + demanded record handles | 2 |
| `crates/sifr_compiler_services/src/metadata/ensure.rs:84` `ensure_with_hook` | read, borrow/retain | layered view + demanded record handles | 98 |
| `crates/sifr_compiler_services/src/metadata/ensure.rs:364` `development_metadata_path` | read | layered view + demanded record handles | 375 |
| `crates/sifr_compiler_services/src/metadata/exports.rs:4` `origin` | read, enumerate | layered view + demanded record handles | 7, 21 |
| `crates/sifr_compiler_services/src/metadata/exports.rs:48` `project` | read, enumerate | layered view + demanded record handles | 49 |
| `crates/sifr_compiler_services/src/metadata/exports.rs:93` `bind_reference` | read | layered view + demanded record handles | 94 |
| `crates/sifr_compiler_services/src/metadata/mod.rs:209` `reencode_qualified` | read | layered view + demanded record handles | 210, 214, 216 |
| `crates/sifr_compiler_services/src/metadata/production.rs:25` `capture` | read | layered view + demanded record handles | 45, 58, 62, 73, 74, 83, 96 |
| `crates/sifr_compiler_services/src/metadata/production.rs:104` `produce` | read, enumerate | layered view + demanded record handles | 106, 120 |
| `crates/sifr_compiler_services/src/metadata/project.rs:24` `project` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 25, 40, 63, 143 |
| `crates/sifr_compiler_services/src/metadata/project_results.rs:39` `encode_project_results` | read, enumerate, borrow/retain | layered view + demanded record handles | 42 |
| `crates/sifr_compiler_services/src/metadata/reader/mod.rs:70` `decode_project_results` | read, enumerate | layered view + demanded record handles | 73 |
| `crates/sifr_compiler_services/src/metadata/reader/provider.rs:61` `semantic` | read, borrow/retain | layered view + demanded record handles | 61 |
| `crates/sifr_compiler_services/src/metadata/reader/provider.rs:81` `prepare` | read | layered view + demanded record handles | 81, 82 |
| `crates/sifr_compiler_services/src/metadata/reader/qualification.rs:6` `qualify` | read, enumerate | layered view + demanded record handles | 8, 39, 46 |
| `crates/sifr_compiler_services/src/metadata/reader/qualification.rs:82` `inspect_metadata` | read | layered view + demanded record handles | 130 |
| `crates/sifr_compiler_services/src/metadata/reader/selection.rs:46` `select` | read, borrow/retain | layered view + demanded record handles | 110 |
| `crates/sifr_compiler_services/src/metadata/reader/semantic.rs:3` `project` | read | layered view + demanded record handles | 7, 9 |
| `crates/sifr_compiler_services/src/metadata/reader/support.rs:12` `materialize` | read, enumerate, borrow/retain | layered view + demanded record handles | 17, 18, 20, 35, 48, 69, 149, 153, 164, 184, 190, 193, 198 |
| `crates/sifr_compiler_services/src/metadata/reader/support_projection.rs:9` `project_signatures` | read, enumerate, clone projection, borrow/retain | layered view + demanded record handles | 12, 65, 66, 72, 73, 74, 76, 77, 86, 87, 95, 99, 100 |
| `crates/sifr_compiler_services/src/metadata/rust_payload.rs:11` `project` | read, enumerate | layered view + demanded record handles | 12, 18, 27, 112 |
| `crates/sifr_compiler_services/src/metadata/semantic.rs:3` `project` | read, enumerate | layered view + demanded record handles | 4 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:22` `compile_stdlib_sources_with_sysroot` | read, enumerate, mutate overlay/producer, clone projection, borrow/retain | canonical source-only producer (retain) | 22, 26, 27, 29, 33, 34, 35, 36, 69, 92, 99, 101, 109, 158, 162, 166, 182, 184, 205, 207, 214, 273, 307, 318, 320, 335, 336, 339, 340, 393, 394, 407, 408, 414, 415, 416, 418, 419, 428, 429, 437, 441, 442, 447, 448, 452, 456, 460, 464, 466, 470, 474, 479, 484, 489, 494, 499, 504, 509, 513, 516, 517, 519, 521, 522 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:526` `stdlib_rust_source` | read | layered view + demanded record handles | 526, 535 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:542` `canonical_stdlib_source_path` | read | layered view + demanded record handles | 542, 548, 554 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:573` `lower_stdlib_source` | read, borrow/retain | layered view + demanded record handles | 573, 576, 580, 583 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:588` `canonical_stdlib_type` | read | layered view + demanded record handles | 588 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:592` `canonicalize_stdlib_hir_signatures` | read | layered view + demanded record handles | 592, 601, 609, 612 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:626` `canonicalize_stdlib_hir_function` | read | layered view + demanded record handles | 626 |
| `crates/sifr_compiler_services/src/stdlib/bootstrap.rs:707` `stdlib_class_template` | read, enumerate | layered view + demanded record handles | 707 |
| `crates/sifr_compiler_services/src/stdlib/interop.rs:26` `build_stdlib_rust_interop` | read, enumerate | layered view + demanded record handles | 26 |
| `crates/sifr_compiler_services/src/stdlib/interop.rs:86` `private_interop_pending_inventory_preserves_lifetime_and_source_order` | read, enumerate, borrow/retain | test setup/parity | 98, 107 |
| `crates/sifr_compiler_services/src/stdlib/mod.rs:34` `external_defs` | read | layered view + demanded record handles | 34, 36, 38 |
| `crates/sifr_compiler_services/src/stdlib/re_exports.rs:17` `re_export_stdlib_imports` | read, enumerate, borrow/retain | layered view + demanded record handles | 17, 19, 51, 58 |
| `crates/sifr_compiler_services/src/stdlib/re_exports.rs:65` `copy_named_exports` | read, mutate overlay/producer, borrow/retain | layered view + demanded record handles | 67, 74, 85, 90, 97, 106, 118, 121 |
| `crates/sifr_compiler_services/src/stdlib/re_exports.rs:131` `copy_callable_metadata` | read, borrow/retain | layered view + demanded record handles | 133, 138, 150, 162, 171 |
| `crates/sifr_compiler_services/src/stdlib/re_exports.rs:212` `synthetic_sysroot_re_export_preserves_compiler_identity` | read | test setup/parity | 213, 254 |
| `crates/sifr_compiler_services/src/stdlib/re_exports.rs:281` `public_aliased_imports_use_the_local_export_name` | read | test setup/parity | 282, 304 |
| `crates/sifr_compiler_services/src/stdlib/re_exports.rs:328` `unapproved_private_imports_do_not_enter_public_exports` | read | test setup/parity | 329, 369 |
| `crates/sifr_compiler_services/src/stdlib/tooling.rs:62` `tooling_sources` | read | layered view + demanded record handles | 66 |
| `crates/sifr_driver/src/build/application_profile_tests.rs:7` `dx10_b05_actual_profiles_keep_asserts_overflow_and_unwind_boundaries` | read | test setup/parity | 11 |
| `crates/sifr_driver/src/build/cargo_manifest.rs:38` `try_generate_standalone_dependency_plan` | read | layered view + demanded record handles | 39, 44 |
| `crates/sifr_driver/src/build/cargo_manifest.rs:51` `try_generate_sysroot_dependency_plan` | read | layered view + demanded record handles | 52, 64 |
| `crates/sifr_driver/src/build/cargo_manifest.rs:84` `render_dependency_cargo_toml` | read, enumerate | layered view + demanded record handles | 100, 101, 103 |
| `crates/sifr_driver/src/build/cargo_manifest.rs:665` `generated_cargo_toml_renders_sysroot_crates_before_retained_glue_deps` | read | test setup/parity | 695 |
| `crates/sifr_driver/src/build/cargo_manifest.rs:727` `test_dependency_plan` | read | layered view + demanded record handles | 732 |
| `crates/sifr_driver/src/build/entrypoint.rs:130` `compile_single_file_entrypoint_with_metadata_and_options` | read, enumerate, borrow/retain | layered view + demanded record handles | 143, 144, 145, 148, 150 |
| `crates/sifr_driver/src/build/entrypoint.rs:451` `from_entrypoint_with_stages` | read, enumerate, mutate overlay/producer, clone projection | layered view + demanded record handles | 464, 465, 473, 493, 547, 642, 661, 662, 665, 697 |
| `crates/sifr_driver/src/build/entrypoint.rs:748` `into_single_file_frontend` | read | layered view + demanded record handles | 756 |
| `crates/sifr_driver/src/build/entrypoint.rs:772` `into_generated_binary_project_with_probe_policy` | read, enumerate, borrow/retain | layered view + demanded record handles | 777, 778, 779, 785, 792, 803 |
| `crates/sifr_driver/src/build/entrypoint_resolution.rs:8` `package_cargo_resolution_policy` | read | layered view + demanded record handles | 10, 22, 32 |
| `crates/sifr_driver/src/build/entrypoint_single_file.rs:8` `into_frontend` | read, borrow/retain | layered view + demanded record handles | 9, 12, 29, 58 |
| `crates/sifr_driver/src/build/entrypoint_tests.rs:21` `test_single_file_entrypoint_plan_generates_main_only_project` | read | test setup/parity | 38 |
| `crates/sifr_driver/src/build/entrypoint_tests.rs:163` `test_project_entrypoint_plan_retains_final_reachable_dependency_metadata` | read | test setup/parity | 201, 205, 207, 209, 210, 213, 214 |
| `crates/sifr_driver/src/build/entrypoint_tests.rs:231` `test_project_entrypoint_plan_ignores_unreachable_dependency_metadata` | read | test setup/parity | 262 |
| `crates/sifr_driver/src/build/materialize.rs:39` `materialize_binary_project_with_report` | read | layered view + demanded record handles | 48 |
| `crates/sifr_driver/src/build/materialize.rs:67` `materialize_binary_project_sources` | read | layered view + demanded record handles | 76 |
| `crates/sifr_driver/src/build/materialize.rs:153` `materialize_cached_binary_project_with_report` | read | layered view + demanded record handles | 169 |
| `crates/sifr_driver/src/build/materialize_tests.rs:112` `cache_owned_nested_generated_roots_survive_stale_cleanup_and_test_runner` | read | test setup/parity | 206 |
| `crates/sifr_driver/src/build/materialize_tests.rs:305` `binary_project_cache_key_uses_sysroot_dependency_plan_inputs` | read | test setup/parity | 308, 314 |
| `crates/sifr_driver/src/build/materialize_tests.rs:404` `sysroot_tls_native_link_evidence_is_explicitly_trusted` | read | test setup/parity | 408, 409 |
| `crates/sifr_driver/src/build/materialize_tests.rs:431` `sysroot_http_native_link_evidence_inherits_tls_provider_trust` | read | test setup/parity | 435, 436 |
| `crates/sifr_driver/src/build/materialize_tests.rs:448` `base_project` | read | test setup/parity | 452 |
| `crates/sifr_driver/src/build/materialize_tests.rs:461` `test_dependency_plan` | read | test setup/parity | 463 |
| `crates/sifr_driver/src/build/portable_project.rs:215` `rewrite_lock_sources` | read, enumerate | layered view + demanded record handles | 255 |
| `crates/sifr_driver/src/build/portable_project.rs:324` `local_stdlib_uses_runtime` | read, enumerate | layered view + demanded record handles | 324 |
| `crates/sifr_driver/src/build/portable_project.rs:465` `rewrite_test_lock` | read | layered view + demanded record handles | 478 |
| `crates/sifr_driver/src/build/portable_project.rs:577` `portable_lock_preserves_stdlib_without_optional_runtime` | read | test setup/parity | 577 |
| `crates/sifr_driver/src/build/portable_project.rs:605` `portable_lock_rejects_missing_required_sysroot_packages` | read | test setup/parity | 606, 613 |
| `crates/sifr_driver/src/build/project_codegen.rs:61` `codegen_single_file_frontend` | read | layered view + demanded record handles | 64, 65, 71, 73 |
| `crates/sifr_driver/src/build/project_codegen.rs:188` `generated_single_file_binary_project` | read | layered view + demanded record handles | 210 |
| `crates/sifr_driver/src/build/project_codegen.rs:219` `generated_project_binary_project` | read, enumerate | layered view + demanded record handles | 220, 224, 225, 229, 233, 240, 251, 304 |
| `crates/sifr_driver/src/build/project_codegen.rs:387` `base_project` | read | layered view + demanded record handles | 391 |
| `crates/sifr_driver/src/build/python_bridges.rs:171` `base_project` | read | layered view + demanded record handles | 175 |
| `crates/sifr_driver/src/build/python_interop_tests.rs:319` `project` | read | test setup/parity | 374 |
| `crates/sifr_driver/src/build/rust_interop_advanced_data_contract_tests.rs:415` `generated_from_source` | read | test setup/parity | 423, 429 |
| `crates/sifr_driver/src/build/rust_interop_async_contract_tests.rs:329` `generated_from_source` | read | test setup/parity | 337, 343 |
| `crates/sifr_driver/src/build/rust_interop_callback_contract_tests.rs:442` `generated_from_source` | read | test setup/parity | 447, 452 |
| `crates/sifr_driver/src/build/rust_interop_cargo_inputs.rs:130` `sysroot_cargo_inputs` | read | layered view + demanded record handles | 144 |
| `crates/sifr_driver/src/build/rust_interop_cargo_inputs.rs:306` `skip_quoted` | read | layered view + demanded record handles | 447, 664, 667, 675, 676, 695, 696, 697, 699, 711, 722 |
| `crates/sifr_driver/src/build/rust_interop_cargo_inputs.rs:659` `sysroot_metadata_identity_binds_authority_and_tree_payloads` | read | test setup/parity | 664, 667, 675, 676, 695, 696, 697, 699, 711, 722 |
| `crates/sifr_driver/src/build/rust_interop_contract_tests.rs:439` `base_project_with_contracts` | read | test setup/parity | 446 |
| `crates/sifr_driver/src/build/rust_interop_panic_contract_tests.rs:310` `package_rust_interop_sifr_stdlib_named_root_still_requires_explicit_policy` | read | test setup/parity | 310 |
| `crates/sifr_driver/src/build/rust_interop_probe_tests.rs:90` `sysroot_probe_manifest_enables_declared_stdlib_features` | read | test setup/parity | 90 |
| `crates/sifr_driver/src/build/rust_interop_probe_tests.rs:161` `python_raw_callback_probe_uses_concrete_stdlib_error_type` | read | test setup/parity | 161 |
| `crates/sifr_driver/src/build/rust_interop_probe_tests.rs:268` `sysroot_stdlib_probe_features_follow_target_module_segment` | read | test setup/parity | 268, 269, 282 |
| `crates/sifr_driver/src/build/rust_interop_probe_tests.rs:288` `sysroot_stdlib_probe_features_normalize_rust_module_separators` | read | test setup/parity | 288, 289, 302 |
| `crates/sifr_driver/src/build/rust_interop_probe_tests.rs:308` `sysroot_stdlib_probe_features_ignore_undeclared_target_segment` | read | test setup/parity | 308, 309, 321 |
| `crates/sifr_driver/src/build/rust_interop_tests.rs:39` `package_rust_interop_rejects_private_stdlib_impersonation` | read | test setup/parity | 39 |
| `crates/sifr_driver/src/build/rust_interop_tests.rs:585` `base_project` | read | test setup/parity | 589 |
| `crates/sifr_driver/src/build/rust_interop_trust.rs:41` `effective_panic_policy` | read | layered view + demanded record handles | 50 |
| `crates/sifr_driver/src/build/rust_interop_trust.rs:87` `is_implicit_sysroot_stdlib_no_panic` | read | layered view + demanded record handles | 87 |
| `crates/sifr_driver/src/build/rust_interop_zero_copy_contract_tests.rs:342` `generated_from_fixture_source` | read | test setup/parity | 347, 352 |
| `crates/sifr_driver/src/build/single_file_interop_cache.rs:26` `resolve_single_file_metadata` | read | layered view + demanded record handles | 29, 33, 40, 42, 47 |
| `crates/sifr_driver/src/build/single_file_interop_cache.rs:54` `resolve_cached_stdlib_interop` | read, borrow/retain | layered view + demanded record handles | 54, 57, 59, 71 |
| `crates/sifr_driver/src/build/single_file_interop_cache.rs:75` `resolve_interop` | read | layered view + demanded record handles | 78, 82 |
| `crates/sifr_driver/src/build/single_file_interop_cache.rs:87` `stdlib_interop_cache_key` | read | layered view + demanded record handles | 87, 88, 91 |
| `crates/sifr_driver/src/build/sql_application_queries.rs:28` `compile_application_queries` | read, enumerate | layered view + demanded record handles | 42, 72, 120, 140, 253 |
| `crates/sifr_driver/src/build/sql_profiles.rs:154` `install_compiler_externals` | read, mutate overlay/producer | layered view + demanded record handles | 154 |
| `crates/sifr_driver/src/build/sql_profiles_tests.rs:28` `package_compilation_prepares_profiles_offline_and_binds_source_bytes` | read | test setup/parity | 84, 101 |
| `crates/sifr_driver/src/build/sql_profiles_tests.rs:145` `package_query_declarations_emit_non_empty_compatibility_artifact` | read, enumerate, mutate overlay/producer | test setup/parity | 154, 156, 164, 184 |
| `crates/sifr_driver/src/build/sql_profiles_tests.rs:199` `sql_discovery_requires_imported_profile_namespaces` | read, mutate overlay/producer | test setup/parity | 219, 221, 229 |
| `crates/sifr_driver/src/build/sql_profiles_tests.rs:246` `portable_schema_source_lowers_and_runtime_witness_uses_fail` | read, enumerate, mutate overlay/producer, clone projection | test setup/parity | 253, 255, 259, 264, 327, 340 |
| `crates/sifr_driver/src/build/sql_profiles_tests.rs:351` `migration_sifr_source_discovers_checked_ordered_steps` | read | test setup/parity | 355, 357, 360 |
| `crates/sifr_driver/src/build/sql_profiles_tests.rs:379` `lower_sql_fixture` | read | test setup/parity | 381, 391 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:37` `generated` | read, borrow/retain | test setup/parity | 42, 43, 47 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:50` `resolve` | read | test setup/parity | 52, 56 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:67` `stdlib_interop_demand_additional_modules_excludes_unrelated_backends` | read, enumerate | test setup/parity | 67, 69, 70, 90, 95 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:106` `stdlib_interop_demand_transitive_reexports_and_support` | read, enumerate, borrow/retain | test setup/parity | 106, 150, 161, 164, 166, 170, 171, 181, 182, 246 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:256` `stdlib_interop_demand_real_python_retains_contracts` | read, enumerate | test setup/parity | 256, 258, 259, 268, 274, 287, 295 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:306` `stdlib_interop_demand_project_single_file_parity` | read | test setup/parity | 306, 308, 309, 313, 318, 319, 320, 329, 334 |
| `crates/sifr_driver/src/build/stdlib_interop_demand_tests.rs:340` `stdlib_interop_demand_cache_disjoint_programs` | read | test setup/parity | 340, 345, 346, 347, 349, 360, 361, 362, 371, 373, 374, 375, 383, 385 |
| `crates/sifr_driver/src/build/stdlib_interop_startup_tests.rs:9` `lowered` | read | test setup/parity | 9 |
| `crates/sifr_driver/src/build/stdlib_interop_startup_tests.rs:18` `stdlib_interop_startup_project_selection_is_once_per_complete_application` | read, enumerate | test setup/parity | 18, 19, 20, 21, 24, 26, 27, 30, 40, 41, 44, 69, 79 |
| `crates/sifr_driver/src/build/stdlib_interop_startup_tests.rs:88` `stdlib_interop_startup_readonly_walk_preserves_hidden_edges` | read, enumerate, borrow/retain | test setup/parity | 88, 89, 95, 98, 99, 101, 157, 158, 164, 165, 177, 182, 207, 210, 214, 222, 230 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:28` `attach_stdlib_rust_interop` | read | layered view + demanded record handles | 28, 31, 34, 41, 47 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:86` `merge_interop_plan` | read | layered view + demanded record handles | 86, 90, 95, 100, 101, 114 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:119` `merge_contexts` | read | layered view + demanded record handles | 121, 123, 127, 133, 137, 141, 144, 147, 149, 151 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:155` `stdlib_context` | read, enumerate | layered view + demanded record handles | 155, 156, 164, 169, 185, 186 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:213` `sysroot_package` | read | layered view + demanded record handles | 232 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:250` `sysroot_backends` | read | layered view + demanded record handles | 258 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:305` `attach_stdlib_rust_interop` | read | layered view + demanded record handles | 305, 308, 310, 311 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:315` `sysroot_private_interop_resolves_canonical_stdlib_crate` | read | test setup/parity | 315, 321, 322, 327 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:348` `sysroot_private_interop_rejects_non_sysroot_target_root` | read | test setup/parity | 360, 362 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:377` `sysroot_private_opaque_interop_resolves_self_close_method` | read | test setup/parity | 384, 406, 423, 430, 532, 539, 551, 565, 743, 744, 745 |
| `crates/sifr_driver/src/build/sysroot_interop.rs:797` `write_stdlib_crate` | read | layered view + demanded record handles | 797 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:25` `attach_stdlib_rust_interop` | read | test setup/parity | 25, 28, 30, 31 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:35` `private_stdlib_interop_resolves_sysroot_crate_target` | read | test setup/parity | 35, 36, 40 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:67` `private_stdlib_interop_rejects_omitted_policy_for_runtime_target` | read | test setup/parity | 67, 68, 72 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:87` `private_stdlib_interop_rejects_non_sysroot_target_root` | read | test setup/parity | 87, 88, 92 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:107` `sysroot_interop_dependency_plan_keeps_sysroot_vendor_mode` | read, enumerate | test setup/parity | 108, 111 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:150` `merged_user_and_private_stdlib_interop_both_resolve` | read, enumerate | test setup/parity | 150, 151, 161 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:190` `merged_user_and_private_stdlib_interop_keeps_user_trust_separate` | read | test setup/parity | 190, 191, 214 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:224` `private_stdlib_interop` | read | test setup/parity | 224, 231 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:252` `user_interop` | read | test setup/parity | 258 |
| `crates/sifr_driver/src/build/sysroot_interop_tests.rs:335` `base_project` | read | test setup/parity | 339 |
| `crates/sifr_driver/src/build/test_runner_interop.rs:14` `finalize_test_runner_project` | read, enumerate | test setup/parity | 16, 22, 28, 58 |
| `crates/sifr_driver/src/compiled_identity.rs:2` `compiled_input_tokens` | read, borrow/retain | layered view + demanded record handles | 17, 18 |
| `crates/sifr_driver/src/frontend/api.rs:70` `compile_with_metadata` | read | layered view + demanded record handles | 79 |
| `crates/sifr_driver/src/metadata_producer/project_results/tests.rs:7` `compatibility` | read | test setup/parity | 11 |
| `crates/sifr_driver/src/metadata_producer/project_results/tests.rs:15` `dx12_typed_interface_checked_and_codegen_roundtrip` | read | test setup/parity | 36, 40 |
| `crates/sifr_driver/src/metadata_producer/project_results/tests.rs:209` `real_writer_cannot_relabel_captured_analysis` | read | test setup/parity | 231 |
| `crates/sifr_driver/src/metadata_producer/tests.rs:156` `dx6_canonical_inventory_producer_without_metadata` | read, enumerate | test setup/parity | 174, 196, 200, 206 |
| `crates/sifr_driver/src/metadata_producer/tests.rs:414` `dx6_r10_linked_real_stdlib_test_adapter` | read | test setup/parity | 414 |
| `crates/sifr_driver/src/metadata_producer/tests.rs:450` `portable_records_ignore_only_the_producer_envelope` | read | test setup/parity | 453, 486, 487 |
| `crates/sifr_driver/src/metadata_reader/corpus_tests.rs:4` `full_corpus_exact_emission` | read, enumerate | test setup/parity | 6, 18, 62, 81, 85, 87, 90, 111, 147 |
| `crates/sifr_driver/src/metadata_reader/generation_tests.rs:14` `install` | read, enumerate | test setup/parity | 17, 47 |
| `crates/sifr_driver/src/metadata_reader/generation_tests.rs:53` `installed_relocation_and_generation_switch_pin_all_sources` | read, enumerate, borrow/retain | test setup/parity | 69, 98 |
| `crates/sifr_driver/src/metadata_reader/tests.rs:41` `source_metadata_diagnostics_and_emission_agree` | read, enumerate | test setup/parity | 43, 58, 96, 98, 101, 117, 143 |
| `crates/sifr_driver/src/metadata_reader/tests.rs:184` `dx7_navigation_demand_shares_index_and_reads_only_selected_source` | read, enumerate, borrow/retain | test setup/parity | 186, 187 |
| `crates/sifr_driver/src/metadata_reader/tests.rs:217` `metadata_projects_keep_failed_deleted_and_repaired_exports_isolated` | read, borrow/retain | test setup/parity | 222 |
| `crates/sifr_driver/src/project/discovery.rs:458` `bare_stdlib_source_diagnostic` | read | layered view + demanded record handles | 458, 459, 471, 475, 497 |
| `crates/sifr_driver/src/project/discovery.rs:502` `bare_stdlib_help` | read | layered view + demanded record handles | 502, 503, 507, 511, 514, 519 |
| `crates/sifr_driver/src/project/discovery.rs:618` `parse_import_closure_source_modules` | read, enumerate | layered view + demanded record handles | 665, 666, 668, 669 |
| `crates/sifr_driver/src/project/frontend.rs:14` `compile_frontend_modules` | read, enumerate | layered view + demanded record handles | 16, 38 |
| `crates/sifr_driver/src/project/frontend.rs:44` `compile_single_frontend_module_with_source_and_options` | read | layered view + demanded record handles | 48, 61 |
| `crates/sifr_driver/src/project/frontend.rs:65` `collect_project_hir_modules` | read | layered view + demanded record handles | 67, 71 |
| `crates/sifr_driver/src/project/frontend.rs:76` `collect_project_hir_source_modules` | read | layered view + demanded record handles | 78, 82 |
| `crates/sifr_driver/src/project/frontend.rs:87` `collect_project_hir_source_modules_with_options` | read, enumerate | layered view + demanded record handles | 89, 108 |
| `crates/sifr_driver/src/project/package_discovery.rs:172` `package_import_source_diagnostic` | read | layered view + demanded record handles | 191, 192, 194, 195 |
| `crates/sifr_driver/src/project/package_discovery.rs:242` `package_bare_stdlib_source_diagnostic` | read | layered view + demanded record handles | 242, 243, 253, 257, 294 |
| `crates/sifr_driver/src/project_cache/interface_reuse.rs:32` `restore` | read, enumerate | layered view + demanded record handles | 38 |
| `crates/sifr_driver/src/project_cache/mod.rs:63` `check_saved_sources` | read, enumerate | layered view + demanded record handles | 153 |
| `crates/sifr_driver/src/project_cache/mod.rs:175` `check` | read, enumerate | layered view + demanded record handles | 183 |
| `crates/sifr_driver/src/stdlib/bootstrap.rs:29` `compile_stdlib` | read, borrow/retain | layered view + demanded record handles | 29, 33, 37, 38 |
| `crates/sifr_driver/src/stdlib/bootstrap.rs:49` `compile_stdlib_uncached` | read | layered view + demanded record handles | 49, 50 |
| `crates/sifr_driver/src/stdlib/bootstrap.rs:54` `compile_stdlib_for_context` | read | layered view + demanded record handles | 54, 58, 64 |
| `crates/sifr_driver/src/stdlib/bootstrap.rs:68` `compile_stdlib_sources_with_sysroot` | read | canonical source-only producer (retain) | 68, 73 |
| `crates/sifr_driver/src/stdlib/bootstrap_fixture_tests.rs:5` `stdlib_interop_startup_bootstrap_preserves_inventory_without_application_plan` | read, enumerate | test setup/parity | 5, 6, 7, 17, 20, 21, 22, 28, 34, 39 |
| `crates/sifr_driver/src/stdlib/bootstrap_fixture_tests.rs:50` `fixture_source` | read | test setup/parity | 51, 57 |
| `crates/sifr_driver/src/stdlib/bootstrap_fixture_tests.rs:62` `compile_fixture_sources` | read | test setup/parity | 66 |
| `crates/sifr_driver/src/stdlib/bootstrap_fixture_tests.rs:77` `private_stdlib_imports_resolve_only_from_compiled_source_exports` | read | test setup/parity | 77, 102 |
| `crates/sifr_driver/src/stdlib/bootstrap_fixture_tests.rs:109` `missing_private_stdlib_member_is_a_structured_bootstrap_failure` | read, enumerate | test setup/parity | 109 |
| `crates/sifr_driver/src/stdlib/bootstrap_fixture_tests.rs:136` `missing_private_stdlib_module_is_a_structured_bootstrap_failure` | read, enumerate | test setup/parity | 136 |
| `crates/sifr_driver/src/stdlib/bootstrap_template_tests.rs:4` `stdlib_bootstrap_syntax_session_preserves_full_inventory_and_source_order` | read, enumerate | test setup/parity | 4, 5, 7, 12, 14, 22 |
| `crates/sifr_driver/src/stdlib/bootstrap_template_tests.rs:30` `stdlib_bootstrap_borrowed_emission_preserves_imports_and_generic_templates` | read, enumerate, borrow/retain | test setup/parity | 30, 31, 34, 39, 44, 51, 56, 59, 64 |
| `crates/sifr_driver/src/stdlib/bootstrap_template_tests.rs:73` `stdlib_structural_templates_retain_signatures_without_bodies` | read, enumerate | test setup/parity | 73, 74, 77, 82, 92, 98, 113 |
| `crates/sifr_driver/src/stdlib/bootstrap_template_tests.rs:139` `recursive_json_structural_contracts_follow_the_shared_project_owner` | read, enumerate | test setup/parity | 140 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:93` `stdlib_class_exports_preserve_parent_markers_and_generic_templates` | read, enumerate | test setup/parity | 93, 94, 192, 202, 206 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:216` `python_core_re_exports_preserve_callable_metadata` | read | test setup/parity | 217 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:243` `python_call_helpers_borrow_argument_collections` | read, enumerate | test setup/parity | 244 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:272` `retained_public_declarations_export_typed_compiler_identity` | read | test setup/parity | 273 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:293` `numeric_and_random_modules_export_only_canonical_operation_names` | read | test setup/parity | 294 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:365` `runtime_information_modules_export_only_canonical_operation_names` | read | test setup/parity | 366 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:452` `text_and_data_modules_export_only_canonical_operation_names` | read | test setup/parity | 453 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:624` `binary_and_hashing_exports_use_only_first_class_bytes_contracts` | read, enumerate | test setup/parity | 625 |
| `crates/sifr_driver/src/stdlib/bootstrap_tests.rs:724` `collections_and_sorted_insert_modules_export_only_canonical_operations` | read | test setup/parity | 725 |
| `crates/sifr_driver/src/stdlib/cache.rs:5` `get_or_init_stdlib_cache` | read, borrow/retain | layered view + demanded record handles | 5 |
| `crates/sifr_driver/src/stdlib/cache.rs:16` `project_stdlib_cache` | read, borrow/retain | layered view + demanded record handles | 16 |
| `crates/sifr_driver/src/stdlib/cache_tests.rs:7` `stdlib_cache_shared_bundle_outlives_cache_and_keeps_definitions_isolated` | read, enumerate, clone projection, borrow/retain | test setup/parity | 7, 11, 13, 15, 31, 35 |
| `crates/sifr_driver/src/stdlib/cache_tests.rs:50` `stdlib_interop_startup_defs_projection_preserves_cached_errors_and_isolation` | read, enumerate, clone projection, borrow/retain | test setup/parity | 50, 53, 57, 64, 71, 76, 80, 83, 85, 111, 123, 126 |
| `crates/sifr_driver/src/stdlib/cache_tests.rs:138` `test_get_or_init_stdlib_cache_reuses_successful_compilation` | read, borrow/retain | test setup/parity | 138, 142, 144, 147, 156, 157 |
| `crates/sifr_driver/src/stdlib/cache_tests.rs:162` `test_get_or_init_stdlib_cache_reuses_error_without_fallback_rebuild` | read, borrow/retain | test setup/parity | 162, 166, 176, 178 |
| `crates/sifr_driver/src/stdlib/stateless_crypto_codegen_tests.rs:4` `crypto_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 4, 5, 8, 52, 106, 113, 120 |
| `crates/sifr_driver/src/stdlib/stateless_fs_codegen_tests.rs:5` `fs_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 5, 6, 9, 82 |
| `crates/sifr_driver/src/stdlib/stateless_logging_codegen_tests.rs:5` `logging_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 5, 6, 9, 39 |
| `crates/sifr_driver/src/stdlib/stateless_math_codegen_tests.rs:4` `math_private_declarations_codegen_through_sifr_stdlib` | read, enumerate | test setup/parity | 4, 5, 8, 45, 51, 56 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:5` `platform_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 5, 6, 9, 27 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:34` `sys_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 34, 35, 38, 76, 83, 90, 97 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:104` `html_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 104, 105, 108, 120 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:127` `calendar_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 127, 128, 131, 144 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:151` `uuid_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 151, 152, 155, 173 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:180` `regex_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 180, 181, 184, 234 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:249` `url_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 249, 250, 253, 286 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:303` `toml_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 303, 304, 307, 322 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:337` `json_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 337, 338, 341, 366 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:384` `encoding_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 384, 385, 388, 416 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:434` `unicode_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 434, 435, 438, 476 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:493` `compression_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 493, 494, 497, 537, 544 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:577` `datetime_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 577, 578, 581, 604 |
| `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:625` `i18n_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 625, 626, 629, 668 |
| `crates/sifr_driver/src/stdlib/stateless_process_codegen_tests.rs:4` `process_sync_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 4, 5, 8, 65, 73 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:5` `python_primitive_constructors_codegen_through_sifr_stdlib` | read | test setup/parity | 5, 6, 9, 37, 44 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:51` `python_primitive_extractors_codegen_through_sifr_stdlib` | read | test setup/parity | 51, 52, 55 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:98` `python_object_core_codegen_through_sifr_stdlib` | read | test setup/parity | 98, 99, 102 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:129` `python_collection_constructors_codegen_through_sifr_stdlib` | read | test setup/parity | 129, 130, 133 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:152` `python_call_helpers_codegen_through_sifr_stdlib` | read | test setup/parity | 152, 153, 156 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:172` `python_copy_helpers_codegen_through_sifr_stdlib` | read | test setup/parity | 172, 173, 176 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:213` `python_zero_copy_helpers_codegen_through_sifr_stdlib` | read | test setup/parity | 213, 244, 247, 276 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:299` `python_context_coroutine_helpers_codegen_through_sifr_stdlib` | read | test setup/parity | 299, 300, 303 |
| `crates/sifr_driver/src/stdlib/stateless_python_codegen_tests.rs:322` `python_callback_helpers_codegen_through_sifr_stdlib` | read | test setup/parity | 322, 323, 326, 354, 361 |
| `crates/sifr_driver/src/stdlib/stateless_time_codegen_tests.rs:4` `time_private_declarations_codegen_through_sifr_stdlib` | read | test setup/parity | 4, 5, 8, 42 |
| `crates/sifr_driver/src/stdlib/types.rs:18` `for_codegen` | read, borrow/retain | layered view + demanded record handles | 27 |
| `crates/sifr_driver/src/stdlib/types.rs:48` `from` | read | layered view + demanded record handles | 48 |
| `crates/sifr_driver/src/test_runner/artifacts.rs:50` `generate_test_runner_cargo_toml` | read | layered view + demanded record handles | 51, 55 |
| `crates/sifr_driver/src/test_runner/artifacts.rs:67` `try_generate_test_runner_cargo_plan` | read | layered view + demanded record handles | 68, 74 |
| `crates/sifr_driver/src/test_runner/execution.rs:28` `execute_test_runner_project` | read, enumerate | layered view + demanded record handles | 45 |
| `crates/sifr_driver/src/test_runner/execution.rs:430` `test_runner_cargo_modes_use_build_resolution_and_trace_policy` | read, enumerate | test setup/parity | 432, 452 |
| `crates/sifr_driver/src/test_runner/execution.rs:499` `test_runner_mutable_root_and_final_snapshot_stay_distinct` | read, enumerate | test setup/parity | 521 |
| `crates/sifr_driver/src/test_runner/execution.rs:577` `test_runner_same_key_concurrency_publishes_one_snapshot` | read, enumerate | test setup/parity | 596 |
| `crates/sifr_driver/src/test_runner/execution.rs:632` `test_runner_cache_key_uses_sysroot_dependency_plan_inputs` | read | test setup/parity | 642, 646, 673 |
| `crates/sifr_driver/src/test_runner/interop_tests.rs:33` `stdlib_interop_test_project_materializes_selected_contracts` | read, enumerate | test setup/parity | 33, 52, 65 |
| `crates/sifr_driver/src/test_runner/interop_tests.rs:94` `stdlib_interop_test_project_empty_demand_stays_empty` | read | test setup/parity | 94, 108, 111 |
| `crates/sifr_driver/src/test_runner/orchestrator.rs:79` `build_test_runner_project_with_package` | read, enumerate, clone projection | layered view + demanded record handles | 104, 106, 107, 109, 115, 162, 174, 203, 216, 219 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:76` `compile_with_contract` | read, clone projection | test setup/parity | 81, 85 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:90` `required_const_and_factory_defaults_finalize_constructor_parameters` | read | test setup/parity | 103, 114 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:140` `provisional_descriptor_defaults_do_not_reject_later_required_fields` | read, enumerate | test setup/parity | 151 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:165` `descriptor_arguments_evaluate_nested_imported_const_calls` | read | test setup/parity | 175 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:187` `descriptor_arguments_evaluate_reexported_const_calls` | read, clone projection | test setup/parity | 210, 214, 217 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:229` `adapter_identities_ignore_source_movement_and_distinguish_default_states` | read | test setup/parity | 233 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:278` `unrelated_declaration_order_does_not_change_adapter_identity` | read | test setup/parity | 287 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:311` `relevant_adapter_edits_invalidate_invocation_and_post_adapter_identity` | read | test setup/parity | 325 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:339` `inherited_fields_keep_parent_identity_and_concrete_generic_types` | read | test setup/parity | 355 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:369` `imported_concrete_generic_parent_keeps_provider_and_field_identity` | read, clone projection | test setup/parity | 393, 397, 400 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:411` `incompatible_inherited_field_reannotation_is_rejected` | read, enumerate, clone projection | test setup/parity | 429, 433 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:444` `compatible_inherited_reannotation_preserves_local_default_ordering` | read | test setup/parity | 458 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:468` `reexported_default_descriptors_finalize_constructor_parameters` | read, enumerate, clone projection | test setup/parity | 508, 512, 515 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:527` `checked_default_factories_support_functions_static_methods_and_constructors` | read, enumerate | test setup/parity | 550, 562 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:576` `adapted_factory_defaults_retain_static_specialization_output` | read, enumerate, clone projection | test setup/parity | 621, 625, 628, 635 |
| `crates/sifr_driver/src/tests/adapter_defaults.rs:643` `invalid_constant_and_factory_defaults_are_rejected_by_the_adapter_boundary` | read, enumerate, clone projection | test setup/parity | 673, 677 |
| `crates/sifr_driver/src/tests/async_python_error_channel.rs:16` `check_example` | read | test setup/parity | 17, 21 |
| `crates/sifr_driver/src/tests/async_python_error_channel.rs:65` `async_python_error_channel_retains_stdlib_ancestry_without_data_parent` | read, enumerate | test setup/parity | 65, 66, 67 |
| `crates/sifr_driver/src/tests/async_python_error_channel.rs:88` `async_python_error_channel_preserves_local_and_imported_error_ancestry` | read | test setup/parity | 103, 104, 106 |
| `crates/sifr_driver/src/tests/async_python_error_channel.rs:126` `async_python_error_channel_preserves_same_named_stdlib_root_ancestry` | read | test setup/parity | 126 |
| `crates/sifr_driver/src/tests/async_python_error_channel_native.rs:69` `async_python_error_channel_native_stdlib_nominal_collisions` | read | test setup/parity | 69 |
| `crates/sifr_driver/src/tests/attached_api_aliases.rs:6` `local_generic_type_alias_forwards_attached_type_calls` | read, clone projection | test setup/parity | 24, 28, 31 |
| `crates/sifr_driver/src/tests/attached_api_codegen.rs:53` `attached_type_api_codegen_keeps_the_concrete_owner_and_exact_bounds` | read, enumerate | test setup/parity | 68, 69, 71, 75, 77, 78 |
| `crates/sifr_driver/src/tests/discovery_and_workspace.rs:368` `test_workspace_resolver_reclassifies_unresolved_bare_stdlib_import` | read | test setup/parity | 368 |
| `crates/sifr_driver/src/tests/discovery_and_workspace.rs:410` `test_workspace_resolver_prefers_real_user_module_over_bare_stdlib_match` | read | test setup/parity | 410 |
| `crates/sifr_driver/src/tests/discovery_and_workspace.rs:445` `test_workspace_resolver_keeps_stdlib_imports_out_of_filesystem_resolution` | read | test setup/parity | 445 |
| `crates/sifr_driver/src/tests/early_adapters/identity_tests.rs:4` `adapter_edits_but_not_consumer_source_movement_invalidate_program_identity` | read, clone projection | test setup/parity | 7, 11, 14, 21 |
| `crates/sifr_driver/src/tests/early_adapters.rs:76` `compile_errors` | read, clone projection | test setup/parity | 80, 84 |
| `crates/sifr_driver/src/tests/early_adapters.rs:135` `attached_api_owner_accepts_method_slots_bound` | read, clone projection | test setup/parity | 163, 167 |
| `crates/sifr_driver/src/tests/early_adapters.rs:172` `erased_marker_runs_adapter_and_specializes_without_layout_or_constructor_cost` | read, enumerate, clone projection | test setup/parity | 187, 191, 195, 201, 219, 231, 241, 250 |
| `crates/sifr_driver/src/tests/early_adapters.rs:264` `adapter_plan_retains_selected_attached_api_set` | read, enumerate, clone projection | test setup/parity | 281, 285, 288, 301 |
| `crates/sifr_driver/src/tests/early_adapters.rs:468` `unbound_generic_adapter_declaration_does_not_request_a_static_program` | read, clone projection | test setup/parity | 483, 487 |
| `crates/sifr_driver/src/tests/early_adapters.rs:492` `concrete_generic_adapted_child_receives_concrete_attached_signature` | read, enumerate, clone projection | test setup/parity | 520, 524, 527, 534, 550 |
| `crates/sifr_driver/src/tests/early_adapters.rs:683` `marker_reexports_keep_canonical_selection_but_not_runtime_imports` | read, enumerate, clone projection | test setup/parity | 698, 702, 705, 712 |
| `crates/sifr_driver/src/tests/early_adapters.rs:770` `marker_does_not_consume_the_optional_data_parent_and_adapter_sees_its_identity` | read, enumerate, clone projection | test setup/parity | 787, 791, 794, 800 |
| `crates/sifr_driver/src/tests/handler_descriptors.rs:62` `compile_errors` | read, clone projection | test setup/parity | 63, 67 |
| `crates/sifr_driver/src/tests/handler_descriptors.rs:74` `handler_descriptors_select_checked_receivers_in_declaration_order` | read, enumerate, clone projection | test setup/parity | 105, 109, 112, 141 |
| `crates/sifr_driver/src/tests/handler_descriptors.rs:276` `inherited_handlers_precede_local_handlers_deterministically` | read, enumerate, clone projection | test setup/parity | 298, 302, 305, 327 |
| `crates/sifr_driver/src/tests/handler_descriptors.rs:343` `imported_inherited_handler_keeps_its_checked_owner` | read, enumerate, clone projection | test setup/parity | 374, 378, 381 |
| `crates/sifr_driver/src/tests/handler_descriptors.rs:426` `handler_target_changes_static_program_identity` | read, clone projection | test setup/parity | 441, 445, 448 |
| `crates/sifr_driver/src/tests/handler_descriptors.rs:484` `self_annotation_keeps_the_current_generic_specialization` | read, enumerate, clone projection | test setup/parity | 494, 498, 501 |
| `crates/sifr_driver/src/tests/imported_attached_apis.rs:30` `imported_selected_owner_uses_finalized_set_through_type_and_instance_aliases` | read, clone projection | test setup/parity | 47, 51, 54 |
| `crates/sifr_driver/src/tests/imported_attached_apis.rs:78` `imported_generic_type_alias_forwards_attached_type_calls` | read, clone projection | test setup/parity | 96, 100, 103 |
| `crates/sifr_driver/src/tests/inheritance_metadata.rs:6` `imported_parent_defaults_are_flattened_into_child_hir` | read, enumerate | test setup/parity | 19, 20, 22 |
| `crates/sifr_driver/src/tests/package_visibility_tests.rs:4` `test_check_package_project_rejects_private_dependency_module` | read | test setup/parity | 49, 74, 77 |
| `crates/sifr_driver/src/tests/project_build_check.rs:734` `test_build_project_includes_reachable_support_module_stdlib_crates_in_manifest` | read | test setup/parity | 734 |
| `crates/sifr_driver/src/tests/project_build_check.rs:775` `test_build_project_manifest_ignores_unreachable_support_module_stdlib_crates` | read | test setup/parity | 775 |
| `crates/sifr_driver/src/tests/project_generic_identity.rs:93` `test_build_project_centralizes_stdlib_nominal_union_payload` | read | test setup/parity | 93 |
| `crates/sifr_driver/src/tests/project_graph.rs:13` `test_compile_frontend_modules_uses_explicit_diagnostic_style` | read, enumerate, clone projection | test setup/parity | 25, 29, 36 |
| `crates/sifr_driver/src/tests/project_graph.rs:55` `test_check_and_project_lowering_share_typecheck_rules` | read | test setup/parity | 65, 66, 67 |
| `crates/sifr_driver/src/tests/project_graph.rs:85` `test_project_lowering_propagates_imported_rust_opaque_close_ownership` | read, enumerate | test setup/parity | 113, 114, 116 |
| `crates/sifr_driver/src/tests/project_graph.rs:128` `test_project_lowering_propagates_reexported_rust_opaque_close_ownership` | read, enumerate | test setup/parity | 160, 161, 163 |
| `crates/sifr_driver/src/tests/project_graph.rs:175` `test_collect_project_modules_supports_single_level_relative_import` | read | test setup/parity | 198, 199, 200, 202, 203 |
| `crates/sifr_driver/src/tests/project_graph.rs:207` `test_collect_project_modules_allows_non_main_stdlib_imports` | read | test setup/parity | 207, 232, 233, 234, 236, 237 |
| `crates/sifr_driver/src/tests/project_graph.rs:241` `test_collect_project_modules_resolves_non_main_local_dependencies` | read | test setup/parity | 275, 276, 277, 279, 280, 281 |
| `crates/sifr_driver/src/tests/project_graph.rs:458` `test_collect_project_modules_reports_unknown_module_in_non_main` | read, enumerate | test setup/parity | 483, 484, 485 |
| `crates/sifr_driver/src/tests/project_graph.rs:495` `test_project_lowering_preserves_retained_callback_contract_through_reexports` | read, enumerate | test setup/parity | 542, 543, 544 |
| `crates/sifr_driver/src/tests/project_graph.rs:556` `test_project_lowering_preserves_imported_method_callback_contract` | read, enumerate | test setup/parity | 594, 595, 596 |
| `crates/sifr_driver/src/tests/project_graph.rs:608` `test_collect_project_modules_cycle_reports_error` | read, enumerate | test setup/parity | 644, 645, 646 |
| `crates/sifr_driver/src/tests/project_graph_constants.rs:4` `test_collect_project_modules_exports_local_constants` | read | test setup/parity | 37, 38, 39, 42, 48 |
| `crates/sifr_driver/src/tests/project_graph_constants.rs:61` `test_project_lowering_fits_imported_integer_constants` | read, enumerate | test setup/parity | 84, 85, 86, 89 |
| `crates/sifr_driver/src/tests/project_graph_constants.rs:105` `test_project_lowering_does_not_fold_shadowed_imported_integer_constant` | read, enumerate | test setup/parity | 128, 129, 130 |
| `crates/sifr_driver/src/tests/single_file_frontend.rs:328` `test_callback_error_union_same_basename_identities_compile_distinctly` | read | test setup/parity | 349 |
| `crates/sifr_driver/src/tests/single_file_frontend.rs:617` `test_source_python_error_contract_without_interop_has_no_runtime_bridge` | read | test setup/parity | 630 |
| `crates/sifr_driver/src/tests/single_file_frontend.rs:634` `test_source_python_error_contract_with_interop_gets_runtime_bridge` | read | test setup/parity | 649 |
| `crates/sifr_driver/src/tests/single_file_frontend.rs:655` `test_raw_python_generic_conversion_and_object_methods_share_runtime_bridge` | read | test setup/parity | 685 |
| `crates/sifr_driver/src/tests/stdlib_exports.rs:8` `compiled_stdlib_exports_match_public_reference` | read, enumerate, clone projection | test setup/parity | 8, 9 |
| `crates/sifr_driver/src/tests/stdlib_exports.rs:96` `compiled_stdlib_same_operation_groups_have_one_public_name` | read, enumerate | test setup/parity | 96, 97 |
| `crates/sifr_driver/src/tests/stdlib_exports.rs:134` `stdlib_heapq_exports_allowlisted_private_max_heap_helpers` | read | test setup/parity | 134, 135 |
| `crates/sifr_driver/src/tests/stdlib_exports.rs:152` `stdlib_integer_constants_fold_in_project_fixed_width_initializers` | read, enumerate | test setup/parity | 152, 167, 168, 172, 175 |
| `crates/sifr_driver/src/tests/test_runner.rs:134` `test_run_tests_support_module_named_main_imports_root_owned_unions` | read | test setup/parity | 175, 177, 182 |
| `crates/sifr_driver/src/tests/test_runner.rs:550` `test_generate_test_runner_cargo_toml_includes_required_features` | read | test setup/parity | 551, 553, 554, 555, 556, 559 |
| `crates/sifr_driver/src/tests/test_runner.rs:570` `test_generate_test_runner_cargo_toml_preserves_stdlib_deps` | read | test setup/parity | 570, 571, 574 |
| `crates/sifr_driver/src/tests/test_runner.rs:582` `test_generate_test_runner_cargo_toml_uses_stdlib_toml_feature` | read | test setup/parity | 582, 583, 586 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:72` `descriptor_calls_in_runtime_locations_are_rejected` | read, enumerate, clone projection | test setup/parity | 81, 85 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:98` `descriptor_const_failure_uses_stable_meta_diagnostic` | read, enumerate, clone projection | test setup/parity | 107, 111 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:122` `descriptor_const_budget_failure_is_stable` | read, enumerate, clone projection | test setup/parity | 131, 135 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:148` `mixed_canonical_providers_fail_before_descriptor_evaluation` | read, enumerate, clone projection | test setup/parity | 186, 190 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:206` `typed_descriptor_project_preserves_all_kinds_and_callable_identity` | read, enumerate, clone projection | test setup/parity | 225, 229, 233, 239, 246, 277 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:288` `nested_annotated_descriptors_flatten_inner_to_outer_before_rhs` | read, enumerate, clone projection | test setup/parity | 298, 302, 305 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:325` `unrelated_same_basename_is_not_a_descriptor` | read, clone projection | test setup/parity | 344, 348, 352 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:359` `callable_identities_support_static_methods_and_type_constructors` | read, enumerate, clone projection | test setup/parity | 377, 381, 384 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:414` `ordinary_class_assignment_keeps_its_existing_diagnostic` | read, enumerate, clone projection | test setup/parity | 424, 428 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:437` `provider_descriptor_union_accepts_assignable_record_members` | read, enumerate, clone projection | test setup/parity | 487, 491, 494 |
| `crates/sifr_driver/src/tests/typed_descriptors.rs:506` `descriptor_return_type_is_checked_against_provider_before_use` | read, enumerate, clone projection | test setup/parity | 526, 530 |
| `crates/sifr_codegen/src/body_analysis/call_conventions.rs:8` `collect_call_param_conventions` | read, enumerate | layered view + demanded record handles | 10, 12, 21 |
| `crates/sifr_codegen/src/body_analysis/call_conventions.rs:25` `collect_local_call_param_conventions` | read, enumerate | layered view + demanded record handles | 27, 38, 60 |
| `crates/sifr_codegen/src/body_analysis.rs:40` `build` | read, enumerate | layered view + demanded record handles | 42, 45 |
| `crates/sifr_codegen/src/checked_place.rs:743` `checked_sequence_read_guard_for_ir` | read | layered view + demanded record handles | 772 |
| `crates/sifr_codegen/src/class_emitter.rs:409` `lower_display_body_for_custom_str` | read, enumerate | layered view + demanded record handles | 427 |
| `crates/sifr_codegen/src/class_method_emitter.rs:44` `lower_class_expr_strict` | read | layered view + demanded record handles | 46, 57, 60 |
| `crates/sifr_codegen/src/class_method_emitter.rs:589` `lower_class_method_item` | read, enumerate | layered view + demanded record handles | 617 |
| `crates/sifr_codegen/src/compiled_identity.rs:2` `compiled_input_tokens` | read, borrow/retain | layered view + demanded record handles | 16 |
| `crates/sifr_codegen/src/entrypoints.rs:18` `generate_rust_test` | read | test setup/parity | 22, 31, 32 |
| `crates/sifr_codegen/src/entrypoints.rs:38` `generate_rust_test_with_project_policy` | read, enumerate | layered view + demanded record handles | 41, 56, 95, 110, 318, 319, 323, 324, 327, 330, 333, 336, 339, 344 |
| `crates/sifr_codegen/src/entrypoints.rs:350` `deferred_test_codegen_result` | read, enumerate | layered view + demanded record handles | 394, 395, 398 |
| `crates/sifr_codegen/src/entrypoints.rs:405` `generate_rust_with_metadata` | read | layered view + demanded record handles | 406 |
| `crates/sifr_codegen/src/error_refs.rs:25` `collect_referenced_builtin_error_classes` | read | layered view + demanded record handles | 27, 34 |
| `crates/sifr_codegen/src/error_refs.rs:42` `collect_error_references` | read | layered view + demanded record handles | 44, 86 |
| `crates/sifr_codegen/src/expr_render_helpers/field_and_stdlib_rewrites.rs:218` `try_lower_registry_expr_result` | read | layered view + demanded record handles | 223 |
| `crates/sifr_codegen/src/expr_render_helpers/field_and_stdlib_rewrites.rs:226` `rewrite_stdlib_constant_idents_in_expr` | read | layered view + demanded record handles | 226, 237, 243, 264, 270, 287, 292, 305, 317, 329, 333, 334, 368, 380, 387, 388, 391, 393, 394, 398, 401, 404, 414, 420, 423, 430, 431, 433, 436, 444, 448, 460, 479, 493, 502, 509, 515, 521, 529, 530, 531, 534, 537, 540, 543, 544 |
| `crates/sifr_codegen/src/expr_render_helpers/field_and_stdlib_rewrites.rs:554` `rewrite_stdlib_constant_idents_in_stmt` | read | layered view + demanded record handles | 554, 566, 620, 628, 631, 635, 636, 667, 668, 712, 715, 718, 719, 723, 739, 742, 746, 757, 760, 764, 769, 777, 781, 788, 791, 799, 807, 811, 814, 820, 836, 842 |
| `crates/sifr_codegen/src/expr_render_helpers/sifr_int_parse_helpers.rs:3` `rewrite_special_ident` | read | layered view + demanded record handles | 8 |
| `crates/sifr_codegen/src/expr_render_helpers/sifr_int_parse_helpers.rs:288` `is_sifr_int_module_constant_func` | read, enumerate | layered view + demanded record handles | 292 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:179` `emitter_with_large_int_const` | read | test setup/parity | 181 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:189` `rewrites_large_int_module_const_arithmetic_to_sifr_int_operands` | read | test setup/parity | 191 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:219` `rewrites_large_int_floor_division_by_nonzero_literal_to_checked_runtime_call` | read | test setup/parity | 221 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:270` `rewrites_large_int_modulo_by_nonzero_literal_to_checked_runtime_call` | read | test setup/parity | 272 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:296` `rewrites_large_int_module_const_let_type_to_sifr_int` | read | test setup/parity | 298 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:316` `local_binding_shadows_large_int_module_const_rewrite` | read | test setup/parity | 323 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:329` `rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands` | read | test setup/parity | 331, 338 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:365` `rewrites_large_int_module_const_comparison_to_sifr_int_operands` | read | test setup/parity | 367 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:403` `rewrites_registered_sifr_int_local_comparison_to_borrowed_operands` | read | test setup/parity | 405, 412 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:440` `rewrites_forced_sifr_int_assignment_target_storage` | read | test setup/parity | 447, 466 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:486` `rewrites_sifr_int_value_position_aliases_to_clone` | read | test setup/parity | 497, 513 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:528` `rewrites_forced_sifr_int_augassign_to_assignment` | read | test setup/parity | 535 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:567` `rewrites_sifr_int_augassign_registered_source_to_borrowed_operand` | read | test setup/parity | 578 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:599` `rewrites_sifr_int_augassign_for_supported_ops` | read | test setup/parity | 607 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:627` `rewrites_sifr_int_floor_mod_augassign_by_nonzero_literal_to_assignment` | read | test setup/parity | 638 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:677` `rewrites_sifr_int_returning_function_call_let_type` | read | test setup/parity | 684 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:704` `rewrites_sifr_int_returning_function_call_named_i64_let_type` | read | test setup/parity | 711 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:731` `rewrites_sifr_int_returning_function_call_with_args_let_type` | read | test setup/parity | 738 |
| `crates/sifr_codegen/src/expr_render_helpers/tests.rs:761` `closure_block_returns_do_not_inherit_sifr_int_return_state` | read | test setup/parity | 765 |
| `crates/sifr_codegen/src/field_analysis_helpers.rs:67` `detect_recursive_fields` | read, enumerate, borrow/retain | layered view + demanded record handles | 95, 96, 98 |
| `crates/sifr_codegen/src/function_emitter/generator_bodies.rs:15` `emit_function` | read, enumerate | layered view + demanded record handles | 56, 82, 93 |
| `crates/sifr_codegen/src/function_emitter/scope_and_function_types.rs:332` `module_sifr_int_bindings` | read, enumerate | layered view + demanded record handles | 333 |
| `crates/sifr_codegen/src/function_emitter/scope_and_function_types.rs:382` `try_lower_structured_nested_function_stmt` | read, enumerate | layered view + demanded record handles | 402 |
| `crates/sifr_codegen/src/function_like_lowering.rs:10` `lower_function_like_body` | read | layered view + demanded record handles | 41, 69, 112 |
| `crates/sifr_codegen/src/generated_dependency_metadata.rs:13` `retain_generated_dependency_metadata` | read, enumerate | layered view + demanded record handles | 15, 25, 28, 29 |
| `crates/sifr_codegen/src/generated_dependency_metadata.rs:43` `retains_stdlib_module` | read, enumerate | layered view + demanded record handles | 43, 45, 51, 53 |
| `crates/sifr_codegen/src/generated_dependency_metadata.rs:56` `direct_features` | read, enumerate | layered view + demanded record handles | 66, 81 |
| `crates/sifr_codegen/src/generated_dependency_metadata.rs:95` `visit_path` | read, enumerate | layered view + demanded record handles | 101 |
| `crates/sifr_codegen/src/generated_dependency_metadata.rs:142` `removes_pruned_stdlib_and_runtime_metadata` | read | test setup/parity | 142 |
| `crates/sifr_codegen/src/generated_dependency_metadata.rs:210` `retains_hyphenated_stdlib_features_for_rust_module_paths` | read | test setup/parity | 210, 230 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer/project_support_pruning.rs:35` `prune_generated_support_for_consumers` | read | layered view + demanded record handles | 41, 45, 49, 62, 63, 81, 82 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer/project_support_pruning.rs:89` `prune_generated_project_prelude_for_consumers` | read, enumerate | layered view + demanded record handles | 125, 135, 140 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer/project_support_pruning.rs:255` `import_project_prelude_bindings` | read | layered view + demanded record handles | 277, 280 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer/support_import_cleanup.rs:27` `refresh_once` | read, enumerate | layered view + demanded record handles | 30, 33 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer/support_import_cleanup.rs:61` `refresh_scope` | read, enumerate | layered view + demanded record handles | 65 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer/support_import_cleanup.rs:104` `refresh_support_root_imports` | read, enumerate | layered view + demanded record handles | 142 |
| `crates/sifr_codegen/src/generated_rust_canonicalizer_capture_tests.rs:153` `implicit_format_capture_support_imports_preserve_block_binding_lifetimes` | read | test setup/parity | 180 |
| `crates/sifr_codegen/src/generated_support_regression_tests.rs:4` `support_impls_do_not_define_builtin_import_bindings` | read | test setup/parity | 13 |
| `crates/sifr_codegen/src/generated_support_regression_tests.rs:134` `proven_inherent_calls_do_not_require_same_named_extension_traits` | read | test setup/parity | 136, 137, 141, 148 |
| `crates/sifr_codegen/src/generated_support_regression_tests.rs:158` `shadowed_and_macro_receiver_calls_keep_required_extension_imports` | read | test setup/parity | 160, 172 |
| `crates/sifr_codegen/src/generated_support_regression_tests.rs:181` `unrelated_user_type_basenames_do_not_prove_inherent_dispatch` | read | test setup/parity | 183, 184, 186 |
| `crates/sifr_codegen/src/generated_support_regression_tests.rs:246` `external_type_paths_and_sibling_trait_parameters_do_not_prove_inherent_dispatch` | read | test setup/parity | 247, 256 |
| `crates/sifr_codegen/src/generated_visibility.rs:63` `crate_visible_generated_support_source` | read, enumerate | layered view + demanded record handles | 64, 68, 73 |
| `crates/sifr_codegen/src/generated_visibility.rs:212` `generated_support_import` | read | layered view + demanded record handles | 216 |
| `crates/sifr_codegen/src/generated_visibility.rs:220` `generated_support_import_with_inherent` | read | layered view + demanded record handles | 223, 225, 226, 234 |
| `crates/sifr_codegen/src/generic_bounds_helpers.rs:254` `rust_ir_class_type_with_generics` | read, enumerate | layered view + demanded record handles | 261, 270 |
| `crates/sifr_codegen/src/generic_bounds_helpers.rs:403` `type_contains_generic_class` | read, enumerate | layered view + demanded record handles | 405 |
| `crates/sifr_codegen/src/generic_bounds_helpers.rs:431` `render_generic_class_type` | read, enumerate | layered view + demanded record handles | 432, 447 |
| `crates/sifr_codegen/src/generic_bounds_helpers.rs:454` `infer_generic_class_type_args` | read, enumerate | layered view + demanded record handles | 455 |
| `crates/sifr_codegen/src/generic_bounds_helpers.rs:550` `collect_typevar_bindings` | read, enumerate | layered view + demanded record handles | 663 |
| `crates/sifr_codegen/src/generic_bounds_helpers_tests.rs:26` `canonical_generic_class_rendering_preserves_concrete_arguments` | read | test setup/parity | 28, 37 |
| `crates/sifr_codegen/src/generic_bounds_helpers_tests.rs:46` `nested_generic_classes_keep_emitter_owned_arguments` | read | test setup/parity | 48, 50 |
| `crates/sifr_codegen/src/helpers/helpers_impl.rs:749` `collect_mutated_vars_with_sigs` | read | layered view + demanded record handles | 751, 753 |
| `crates/sifr_codegen/src/hir_analysis/queries/queries_impl.rs:172` `collect_mutated_vars` | read, enumerate | layered view + demanded record handles | 174, 216, 223, 245, 259, 348, 364, 370 |
| `crates/sifr_codegen/src/hir_analysis/queries/queries_impl.rs:214` `collect_local_function_param_conventions` | read, enumerate | layered view + demanded record handles | 216, 223 |
| `crates/sifr_codegen/src/inline_syntax_tests.rs:163` `support_reuse_preserves_canonical_rendering_bytes_and_order` | read | test setup/parity | 164, 170, 172 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/builtin_core_methods.rs:11` `apply_intrinsic_registry_side_effects` | read | layered view + demanded record handles | 19, 20, 26, 27, 33, 34 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/builtin_core_methods.rs:577` `builtin_open_text_roots_text_handle_support` | read | test setup/parity | 588, 590 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/builtin_numeric.rs:10` `try_lower_registry_numeric_builtin_call_expr` | read, enumerate | layered view + demanded record handles | 795 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/literal_and_intrinsic_exprs.rs:108` `try_lower_registry_intrinsic_call_expr` | read, enumerate | layered view + demanded record handles | 123 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/plain_call_args.rs:372` `resolve_plain_call_param_info` | read | layered view + demanded record handles | 378, 386 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/plain_call_args.rs:398` `try_build_registry_callable_convention_alignment_expr` | read, enumerate | layered view + demanded record handles | 413 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/plain_call_args.rs:514` `resolve_registry_method_params` | read, enumerate | layered view + demanded record handles | 521 |
| `crates/sifr_codegen/src/intrinsic_method_emitters/recursive_exprs.rs:9` `try_lower_registry_expr_recursive` | read, enumerate | layered view + demanded record handles | 270 |
| `crates/sifr_codegen/src/intrinsics/registry/open_text_handles.rs:5` `encoding_constructor_path` | read | layered view + demanded record handles | 7 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:54` `math_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 54 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:74` `json_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 74 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:96` `encoding_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 96 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:126` `runtime_module_dependency_metadata_includes_observability_facades` | read, enumerate | test setup/parity | 127 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:142` `lowers_task_current_context_as_language_runtime_glue` | read | test setup/parity | 148 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:159` `unicode_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 159 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:189` `i18n_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 189 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:214` `env_and_sys_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 214 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:246` `os_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 246 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:256` `signal_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 256 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:266` `fs_text_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 266 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:282` `fs_path_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 282 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:352` `lowers_encoding_intrinsics_via_registry` | read | test setup/parity | 358, 363 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:423` `time_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 423 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:452` `random_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 452 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:478` `re_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 478 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:505` `url_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 505 |
| `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:532` `core_hash_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 532 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:36` `python_primitive_constructors_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 36 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:53` `python_primitive_extractors_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 53 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:84` `python_object_core_is_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 84 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:117` `python_collection_constructors_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 117 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:132` `python_call_helpers_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 132 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:164` `python_copy_helpers_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 164 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:213` `python_zero_copy_helpers_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 213 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:255` `python_context_coroutine_helpers_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 255 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:300` `toml_intrinsic_is_owned_by_compiled_stdlib_declaration` | read | test setup/parity | 300 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:312` `datetime_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 312 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:327` `process_sync_timeout_intrinsics_lower_through_private_stdlib` | read, enumerate | test setup/parity | 327 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:483` `compression_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 483 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:509` `base_encoding_intrinsics_are_owned_by_compiled_stdlib_declarations` | read, enumerate | test setup/parity | 509 |
| `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:541` `extended_hash_intrinsics_are_owned_by_compiled_stdlib_declarations` | read | test setup/parity | 541 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_control_codegen_tests.rs:13` `generate_rust_from_source_with_stdlib_collections` | read, mutate overlay/producer | test setup/parity | 13, 15, 16, 21 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:4` `test_task_timeout_context_manager_wraps_awaits` | read | test setup/parity | 23 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:177` `test_sync_main_does_not_require_tokio` | read | test setup/parity | 196 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:201` `test_generate_project_emits_tokio_dependency_when_required` | read | test setup/parity | 203 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_task_runtime_codegen_tests.rs:3` `test_task_group_basic_lowers_to_scope_runtime_substrate` | read | test setup/parity | 59 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_task_runtime_codegen_tests.rs:64` `test_task_gather_lowers_to_private_gather_helper` | read | test setup/parity | 95 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_task_runtime_codegen_tests.rs:100` `test_scope_spawn_fallible_coroutine_lowers_to_result_spawn_helper` | read | test setup/parity | 144 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_task_runtime_codegen_tests.rs:189` `test_task_race_lowers_to_private_race_helper` | read | test setup/parity | 223 |
| `crates/sifr_codegen/src/lib_codegen_tests/async_task_runtime_codegen_tests.rs:262` `test_task_select_lowers_to_private_select_helper` | read | test setup/parity | 318 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:102` `process_child_resource_derives_are_module_scoped` | read | test setup/parity | 138 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:313` `test_render_expr_lowering_rewrites_module_constant_ident` | read | test setup/parity | 316 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:334` `test_render_expr_lowering_uses_module_constant_for_stdlib_named_constant` | read | test setup/parity | 334, 338 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:357` `test_render_expr_lowering_rewrites_module_constant_helper_call` | read | test setup/parity | 359 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:423` `test_structured_stmt_path_rewrites_stdlib_constant_name` | read | test setup/parity | 423, 459, 460, 465 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:709` `test_generate_rust_multi_skips_stdlib_use_paths_in_non_main_modules` | read | test setup/parity | 709 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_codegen_tests.rs:787` `test_generate_rust_multi_with_metadata_omits_unused_dependency_closure` | read | test setup/parity | 849, 850, 857, 862 |
| `crates/sifr_codegen/src/lib_codegen_tests/classes_and_basics_multi_module_codegen_tests.rs:4` `test_generate_rust_multi_with_metadata_preserves_trait_impl_visibility` | read | test setup/parity | 73, 77 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:627` `test_generate_project_emits_sifr_runtime_path_dependency_when_required` | read | test setup/parity | 629 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:641` `test_async_main_entrypoint_gets_tokio_bootstrap_dependency` | read | test setup/parity | 661 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:666` `test_async_result_main_entrypoint_keeps_result_return` | read | test setup/parity | 693 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:698` `test_task_sleep_lowers_to_tokio_sleep_and_requires_tokio` | read | test setup/parity | 718 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:723` `test_task_sleep_requires_tokio_without_async_main` | read | test setup/parity | 744 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:749` `test_task_scope_context_materializes_runtime_container` | read | test setup/parity | 777 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:782` `test_scope_spawn_lowers_to_owned_task_handle_substrate` | read | test setup/parity | 812 |
| `crates/sifr_codegen/src/lib_codegen_tests/collections_and_stdlib_codegen_tests.rs:817` `test_spawn_blocking_lowers_to_distinct_blocking_task_substrate` | read | test setup/parity | 853 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_augassign_codegen_tests.rs:4` `variable_string_key_defaultdict_counter_keeps_entry_default_codegen` | read | test setup/parity | 5 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_augassign_codegen_tests.rs:18` `variable_integer_key_defaultdict_counter_keeps_entry_default_codegen` | read | test setup/parity | 19 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_augassign_codegen_tests.rs:32` `nested_shadowed_defaultdict_counters_keep_independent_key_types` | read | test setup/parity | 33 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_augassign_codegen_tests.rs:42` `nested_scalar_shadow_is_not_retyped_as_defaultdict` | read | test setup/parity | 43 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_augassign_codegen_tests.rs:53` `late_nested_shadow_does_not_clear_enclosing_defaultdict_annotation` | read | test setup/parity | 54 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:4` `defaultdict_set_pop_uses_the_checked_bucket_without_compile_error` | read | test setup/parity | 5 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:25` `read_before_write_defaultdict_set_has_concrete_declaration_codegen` | read | test setup/parity | 26 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:36` `tuple_key_defaultdict_set_has_concrete_declaration_codegen` | read | test setup/parity | 37 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:47` `list_slice_append_uses_defaultdict_entry_insertion` | read | test setup/parity | 48 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:58` `string_slice_append_uses_defaultdict_entry_insertion` | read | test setup/parity | 59 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:69` `borrowed_string_defaultdict_set_operations_use_owned_storage_and_direct_lookup` | read | test setup/parity | 70 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:85` `borrowed_string_iterable_literals_store_owned_values` | read | test setup/parity | 86 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:102` `list_extend_mutates_the_defaultdict_entry_without_cloning_the_bucket` | read | test setup/parity | 103 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:115` `set_update_mutates_the_defaultdict_entry_without_cloning_the_bucket` | read | test setup/parity | 116 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:128` `generally_lowered_iterables_cannot_fall_back_to_cloned_defaultdict_buckets` | read | test setup/parity | 129 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:139` `iterable_arguments_are_materialized_before_borrowing_the_destination_bucket` | read | test setup/parity | 140 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:164` `variadic_set_bucket_updates_never_fall_back_to_cloned_receivers` | read | test setup/parity | 165 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:196` `iterable_mutation_evaluates_key_before_arguments_and_bucket_borrow` | read | test setup/parity | 197 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:221` `scalar_mutation_stages_key_and_arguments_before_one_bucket_borrow` | read | test setup/parity | 222 |
| `crates/sifr_codegen/src/lib_codegen_tests/defaultdict_order_independent_codegen_tests.rs:248` `scalar_mutation_remove_evaluates_value_outside_the_search_closure` | read | test setup/parity | 249 |
| `crates/sifr_codegen/src/lib_codegen_tests/inline_assembly_contract_tests.rs:2` `test_generate_rust_with_stdlib_assembles_single_rust_file` | read | test setup/parity | 2 |
| `crates/sifr_codegen/src/lib_codegen_tests/iterators_and_generators_codegen_tests.rs:183` `test_generate_rust_open_uses_canonical_filehandle_constructor` | read | test setup/parity | 228 |
| `crates/sifr_codegen/src/lib_codegen_tests/iterators_and_generators_codegen_tests.rs:263` `test_generate_rust_defaultdict_int_augassign_uses_entry_default` | read | test setup/parity | 264 |
| `crates/sifr_codegen/src/lib_codegen_tests/iterators_and_generators_codegen_tests.rs:276` `test_generate_rust_defaultdict_list_len_borrows_string_literal_key` | read | test setup/parity | 277 |
| `crates/sifr_codegen/src/lib_codegen_tests/iterators_and_generators_codegen_tests.rs:382` `test_generate_rust_test_collects_imports_from_emitted_code` | read | test setup/parity | 462, 467 |
| `crates/sifr_codegen/src/lib_codegen_tests/multi_module_stdlib_feature_tests.rs:4` `test_generate_rust_multi_with_metadata_infers_fs_feature_from_private_stdlib_source` | read | test setup/parity | 4, 38, 39, 43, 54, 71 |
| `crates/sifr_codegen/src/lib_codegen_tests/multi_module_stdlib_feature_tests.rs:76` `test_generate_rust_multi_omits_runtime_for_unused_private_stdlib_bridge` | read | test setup/parity | 76, 102, 103, 107, 118, 120 |
| `crates/sifr_codegen/src/lib_codegen_tests/multi_module_stdlib_feature_tests.rs:126` `public_stdlib_reexport_uses_transitive_private_signature_for_call_borrowing` | read | test setup/parity | 126, 172, 173, 177, 188 |
| `crates/sifr_codegen/src/lib_codegen_tests/performance_codegen_tests.rs:135` `defaultdict_set_membership_borrows_bucket_without_cloning` | read | test setup/parity | 136 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:11` `ordinary_class_codegen_skips_structural_impls` | read | test setup/parity | 24 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:99` `ordinary_union_codegen_skips_structural_impls_without_demand` | read | test setup/parity | 112 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:122` `direct_ordinary_union_gets_structural_impls_when_demanded` | read | test setup/parity | 138 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:146` `project_union_resolves_structural_members_from_their_defining_module` | read | test setup/parity | 166 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:177` `project_record_eligibility_resolves_nested_imported_members` | read | test setup/parity | 213 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:238` `project_root_record_keeps_qualified_structural_identity` | read | test setup/parity | 243 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:274` `named_single_file_record_keeps_unqualified_structural_identity` | read | test setup/parity | 277, 279 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:289` `imported_stdlib_record_gets_one_late_canonical_structural_impl` | read | test setup/parity | 289, 300, 301, 305, 317 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:328` `platform_integer_union_does_not_receive_structural_impls` | read | test setup/parity | 340 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:351` `project_structural_demand_enables_implicit_classes_across_modules` | read | test setup/parity | 366, 371 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_impl_demand_codegen_tests.rs:376` `test_project_root_record_keeps_qualified_structural_identity` | read | test setup/parity | 381 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_inheritance_codegen_tests.rs:7` `concrete_generic_child_flattens_parent_fields_for_structural_bridge` | read | test setup/parity | 58 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_stdlib_impl_codegen_tests.rs:12` `unused_import_only_stdlib_structural_impls_are_pruned_after_relocation` | read, enumerate | test setup/parity | 12, 21 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_stdlib_impl_codegen_tests.rs:35` `project_identity_with_stdlib_prefix_gets_no_origin_bypass` | read | test setup/parity | 35, 41, 43 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_stdlib_impl_codegen_tests.rs:71` `json_stdlib` | read | test setup/parity | 71, 75, 76, 80, 90 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_stdlib_impl_codegen_tests.rs:153` `imported_stdlib_structural_contracts_include_transitive_field_types_only` | read, enumerate | test setup/parity | 153, 154, 166, 167, 175, 179, 180, 188, 189, 199 |
| `crates/sifr_codegen/src/lib_codegen_tests/structural_stdlib_impl_codegen_tests.rs:217` `import_only_structural_registration_preserves_opaque_runtime_ownership` | read | test setup/parity | 225, 226, 237, 238, 240 |
| `crates/sifr_codegen/src/lib_codegen_tests/structured_lowering_codegen_tests.rs:4` `test_nested_stdlib_constructor_uses_canonical_nominal_path` | read | test setup/parity | 4, 30, 31 |
| `crates/sifr_codegen/src/lib_codegen_tests/support_assembly_codegen_tests.rs:5` `multi_module_support_has_one_private_owner_and_a_strict_size_budget` | read, enumerate | test setup/parity | 10 |
| `crates/sifr_codegen/src/lib_codegen_tests/support_assembly_codegen_tests.rs:62` `support_visibility_exposes_signature_types_but_not_private_implementation_types` | read | test setup/parity | 363, 373 |
| `crates/sifr_codegen/src/lib_codegen_tests/support_assembly_codegen_tests.rs:112` `support_imports_keep_implicit_format_captures_after_canonicalization` | read | test setup/parity | 363, 373 |
| `crates/sifr_codegen/src/lib_codegen_tests/support_assembly_codegen_tests.rs:154` `support_imports_drop_reverse_nominal_edges_of_pruned_helpers` | read | test setup/parity | 363, 373 |
| `crates/sifr_codegen/src/lib_codegen_tests/task_spawn_ownership_codegen_tests.rs:4` `test_thread_pool_executor_submit_reuses_blocking_task_substrate` | read | test setup/parity | 35 |
| `crates/sifr_codegen/src/lib_codegen_tests.rs:16` `trait_impl_fixture_stdlib_code` | read | test setup/parity | 16, 17, 18 |
| `crates/sifr_codegen/src/lib_emitter_state.rs:242` `new` | read | layered view + demanded record handles | 258, 276, 284, 285, 286, 287, 293, 294 |
| `crates/sifr_codegen/src/lib_emitter_state.rs:358` `collect_nested_fn_lexical_captures` | read, enumerate | layered view + demanded record handles | 369 |
| `crates/sifr_codegen/src/lib_emitter_structured_stmt.rs:8` `try_lower_structured_stmt_with_following` | read | layered view + demanded record handles | 89, 108, 148, 161, 193, 307, 315, 329, 367, 370, 494, 505 |
| `crates/sifr_codegen/src/lib_modules_and_codegen/deferred_codegen.rs:18` `deferred_codegen_result` | read, enumerate | layered view + demanded record handles | 20, 60, 61, 62, 63, 80, 81, 84 |
| `crates/sifr_codegen/src/lib_modules_and_codegen/deferred_codegen.rs:90` `inline_codegen_result` | read, enumerate | layered view + demanded record handles | 92, 100, 106, 136, 158, 159 |
| `crates/sifr_codegen/src/lib_modules_and_codegen.rs:69` `into_application` | read | layered view + demanded record handles | 75 |
| `crates/sifr_codegen/src/lib_modules_and_codegen.rs:113` `generate_rust_with_stdlib` | read | layered view + demanded record handles | 113, 114 |
| `crates/sifr_codegen/src/lib_modules_and_codegen.rs:118` `generate_rust_with_stdlib_for_module` | read | layered view + demanded record handles | 118, 120, 123, 125, 129, 130 |
| `crates/sifr_codegen/src/lib_modules_and_codegen.rs:136` `generate_stdlib_module_body` | read | layered view + demanded record handles | 136, 138 |
| `crates/sifr_codegen/src/lib_modules_and_codegen.rs:146` `generate_module` | read | layered view + demanded record handles | 149, 155 |
| `crates/sifr_codegen/src/lib_modules_and_codegen.rs:165` `generate_rust_with_stdlib_for_module_with_project_policy` | read, enumerate | layered view + demanded record handles | 165, 167, 185, 189, 224, 240, 249 |
| `crates/sifr_codegen/src/lib_modules_structural_policy.rs:6` `generate_rust_with_stdlib_for_module_with_structural_policy` | read | layered view + demanded record handles | 6, 8, 12, 14 |
| `crates/sifr_codegen/src/lib_project_codegen/imported_unions.rs:3` `register_imported_union_types` | read | layered view + demanded record handles | 6, 12, 22 |
| `crates/sifr_codegen/src/lib_project_codegen/tests.rs:69` `project_unions_have_one_crate_root_definition` | read | test setup/parity | 87 |
| `crates/sifr_codegen/src/lib_project_codegen/tests.rs:112` `main_owned_union_is_imported_from_the_crate_root` | read | test setup/parity | 123 |
| `crates/sifr_codegen/src/lib_project_codegen/tests.rs:139` `dotted_union_user_imports_the_crate_root_definition` | read | test setup/parity | 157 |
| `crates/sifr_codegen/src/lib_project_codegen/tests.rs:168` `root_prelude_combines_try_conversions_with_ordinary_union_traits` | read | test setup/parity | 190 |
| `crates/sifr_codegen/src/lib_project_codegen/tests.rs:210` `root_prelude_uses_crate_rooted_nominal_payload_paths` | read | test setup/parity | 220 |
| `crates/sifr_codegen/src/lib_project_codegen/tests.rs:240` `root_union_plan_distinguishes_non_class_nominal_identities` | read | test setup/parity | 297 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:29` `project_union_usage` | read, enumerate | layered view + demanded record handles | 31 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:161` `render_local_module_imports` | read, enumerate | layered view + demanded record handles | 164, 176 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:262` `register_imported_generic_classes` | read, enumerate, borrow/retain | layered view + demanded record handles | 263, 284, 285, 287 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:294` `generate_rust_multi_with_metadata` | read, enumerate, clone projection | layered view + demanded record handles | 296, 299, 301, 307, 310, 325, 327, 329, 330, 343, 364, 385, 388, 410, 415, 416, 418, 421, 422, 424, 478, 482, 501, 503, 530, 538, 540, 541 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:552` `generate_rust_multi` | read | layered view + demanded record handles | 553 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:565` `generate_project_with_deps` | read | layered view + demanded record handles | 568, 570 |
| `crates/sifr_codegen/src/lib_project_codegen.rs:578` `generate_project_with_deps_and_crates` | read | layered view + demanded record handles | 581, 594 |
| `crates/sifr_codegen/src/lib_project_codegen_opaque_import_tests.rs:9` `local_imports_bring_opaque_extension_traits_into_scope` | read | test setup/parity | 73 |
| `crates/sifr_codegen/src/lib_project_signatures.rs:52` `project_class_fields` | read, enumerate | layered view + demanded record handles | 55 |
| `crates/sifr_codegen/src/lib_project_signatures.rs:152` `module_class_fields` | read, enumerate | layered view + demanded record handles | 152 |
| `crates/sifr_codegen/src/lib_runtime_needs.rs:17` `replace_sync_channel_runtime_items` | read | layered view + demanded record handles | 36 |
| `crates/sifr_codegen/src/lib_test_project_codegen.rs:33` `generate_rust_test_project_with_metadata` | read, enumerate, clone projection | test setup/parity | 36, 42, 44, 47, 58, 60, 62, 63, 85, 91, 107, 138, 141, 143, 150, 170, 173, 175, 179, 184, 185, 187, 190, 191, 193, 244, 248, 266, 268, 322, 330, 331, 340 |
| `crates/sifr_codegen/src/method_call_emitter.rs:50` `is_generator_call` | read | layered view + demanded record handles | 54 |
| `crates/sifr_codegen/src/method_call_emitter.rs:66` `generic_generator_calls_use_the_canonical_function_name` | read | test setup/parity | 68 |
| `crates/sifr_codegen/src/module_constants.rs:21` `try_emit_lowered_module_constant_result` | read | layered view + demanded record handles | 44 |
| `crates/sifr_codegen/src/module_constants.rs:49` `try_emit_expression_module_constant` | read | layered view + demanded record handles | 60 |
| `crates/sifr_codegen/src/module_prescan.rs:63` `collect_import_metadata` | read, enumerate | layered view + demanded record handles | 77, 79 |
| `crates/sifr_codegen/src/module_prescan.rs:147` `collect_function_signature_metadata` | read, enumerate | layered view + demanded record handles | 154, 162, 182, 186 |
| `crates/sifr_codegen/src/operator_protocol_emitters.rs:492` `lower_operator_stmt_block_ir` | read | layered view + demanded record handles | 521 |
| `crates/sifr_codegen/src/operator_protocol_emitters.rs:702` `lower_operator_expr_ir` | read, enumerate | layered view + demanded record handles | 705, 712 |
| `crates/sifr_codegen/src/preamble/parallel_runtime.rs:7` `replace_parallel_runtime_items` | read, enumerate | layered view + demanded record handles | 31 |
| `crates/sifr_codegen/src/preamble/task_context_runtime.rs:56` `build_task_current_context_items` | read | layered view + demanded record handles | 57 |
| `crates/sifr_codegen/src/preamble/task_scope_offload_runtime.rs:68` `build_task_scope_process_items` | read | layered view + demanded record handles | 69, 71, 73 |
| `crates/sifr_codegen/src/preamble/task_scope_offload_runtime.rs:182` `scoped_process_body` | read | layered view + demanded record handles | 194 |
| `crates/sifr_codegen/src/project_constants.rs:4` `register_imported_constants` | read, enumerate | layered view + demanded record handles | 7, 10, 23 |
| `crates/sifr_codegen/src/project_constants.rs:29` `extend_project_constant_mappings` | read, enumerate | layered view + demanded record handles | 30, 56, 68, 72, 73, 85, 86 |
| `crates/sifr_codegen/src/project_stdlib_nominals/registry.rs:41` `corpus_repair_builtin_registration_preserves_identity` | read | test setup/parity | 89 |
| `crates/sifr_codegen/src/project_stdlib_nominals/registry.rs:102` `corpus_repair_builtin_registration_rejects_nonbuiltin_names` | read | test setup/parity | 129 |
| `crates/sifr_codegen/src/project_stdlib_nominals/relocation.rs:77` `relocate_project_stdlib_nominals` | read, enumerate | layered view + demanded record handles | 77, 92 |
| `crates/sifr_codegen/src/project_stdlib_nominals/relocation.rs:102` `relocate_project_stdlib_nominals_owned_by` | read, enumerate | layered view + demanded record handles | 102 |
| `crates/sifr_codegen/src/project_stdlib_nominals/relocation_tests.rs:18` `relocated_structural_contracts_have_one_shared_owner_across_importers` | read | test setup/parity | 22 |
| `crates/sifr_codegen/src/project_stdlib_nominals/relocation_tests.rs:51` `relocated_structural_contracts_do_not_capture_a_same_name_local_owner` | read | test setup/parity | 54 |
| `crates/sifr_codegen/src/project_stdlib_nominals/relocation_tests.rs:70` `relocated_structural_contracts_never_select_between_conflicting_bodies` | read | test setup/parity | 74 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:38` `project_stdlib_nominal_plan` | read, enumerate | layered view + demanded record handles | 38 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:132` `register_imported_structural_nominals` | read, enumerate | layered view + demanded record handles | 134, 139 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:161` `extract_project_stdlib_nominal_prelude` | read, enumerate | layered view + demanded record handles | 161, 164, 168 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:268` `register_transitive_stdlib_nominals` | read | layered view + demanded record handles | 268, 270, 274 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:496` `stdlib_error` | read | layered view + demanded record handles | 496 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:508` `same_basename_stdlib_errors_keep_distinct_nominal_declarations` | read | test setup/parity | 508, 514, 515 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:533` `same_basename_stdlib_errors_get_distinct_project_paths` | read | test setup/parity | 533, 534, 535, 538 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:556` `direct_crate_root_nominal_gets_a_project_path_without_other_nominals` | read | test setup/parity | 567 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:579` `builtin_error_union_keeps_its_shared_module_definition` | read | test setup/parity | 603 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:614` `task_scope_error_plan_includes_scope_failure_conversion` | read | test setup/parity | 645 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:659` `emitted_transitive_nominals_join_the_project_registry` | read | test setup/parity | 660, 661, 682 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:724` `builtin_registry_identity_does_not_replace_a_module_qualified_shadow` | read | test setup/parity | 754 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:768` `direct_io_error_kind_type_does_not_create_a_project_nominal` | read | test setup/parity | 783 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:795` `io_error_kind_handlers_do_not_create_dangling_project_nominals` | read | test setup/parity | 828 |
| `crates/sifr_codegen/src/project_stdlib_nominals.rs:840` `relocation_retains_local_child_conversion_into_shared_parent` | read | test setup/parity | 855 |
| `crates/sifr_codegen/src/project_structural_record_codegen.rs:4` `render_project_structural_record_prelude` | read | layered view + demanded record handles | 6 |
| `crates/sifr_codegen/src/project_structural_record_codegen.rs:72` `build_order_and_module_order_reuse_one_crate_root_layout` | read, enumerate | test setup/parity | 77 |
| `crates/sifr_codegen/src/project_structural_record_codegen.rs:98` `crate_root_body_does_not_import_its_local_structural_layout` | read | test setup/parity | 101 |
| `crates/sifr_codegen/src/project_structural_record_codegen.rs:116` `support_module_imports_its_crate_root_structural_layout` | read | test setup/parity | 121 |
| `crates/sifr_codegen/src/python_arrow_codegen_tests.rs:39` `arrow_bridge_producer_uses_resolved_runtime_target` | read, borrow/retain | test setup/parity | 47 |
| `crates/sifr_codegen/src/rust_interop_plan.rs:22` `cache_key_fragment` | read | layered view + demanded record handles | 24, 28 |
| `crates/sifr_codegen/src/stdlib_codegen_metadata.rs:63` `emission_view` | read | layered view + demanded record handles | 63, 64, 66 |
| `crates/sifr_codegen/src/stdlib_codegen_metadata.rs:71` `bootstrap_view` | read | layered view + demanded record handles | 73, 74, 75, 77 |
| `crates/sifr_codegen/src/stdlib_demand_plan.rs:8` `plan_demanded_stdlib_sources` | read, enumerate, clone projection | layered view + demanded record handles | 8, 9, 30, 44, 60, 76, 85, 90, 104, 120, 124, 130 |
| `crates/sifr_codegen/src/stdlib_demand_plan.rs:172` `public_import_roots_seed_matching_transitive_private_definitions` | read | test setup/parity | 173, 174, 178, 189, 197, 198 |
| `crates/sifr_codegen/src/stdlib_filter/implementation.rs:59` `seal_canonical_stdlib_names` | read, enumerate | layered view + demanded record handles | 59, 67, 72 |
| `crates/sifr_codegen/src/stdlib_filter/implementation.rs:132` `rewrite_canonical_stdlib_tokens` | read, enumerate | layered view + demanded record handles | 132, 158 |
| `crates/sifr_codegen/src/stdlib_filter/implementation.rs:174` `rewrite_rust_identifiers` | read | layered view + demanded record handles | 180 |
| `crates/sifr_codegen/src/stdlib_filter/implementation.rs:239` `filter_stdlib_ir_to_needed` | read | layered view + demanded record handles | 239, 243 |
| `crates/sifr_codegen/src/stdlib_filter/implementation.rs:258` `filter_canonical_stdlib_ir_to_needed` | read, enumerate | layered view + demanded record handles | 258, 267, 270, 272, 273 |
| `crates/sifr_codegen/src/stdlib_filter/implementation.rs:493` `parse_stdlib_ir_file` | read, enumerate | layered view + demanded record handles | 493 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:65` `filter_keeps_transitive_dependencies_in_item_order` | read | test setup/parity | 82 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:92` `filter_ignores_name_mentions_in_strings_and_comments` | read | test setup/parity | 103 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:110` `filter_tracks_type_level_dependencies_via_identifiers` | read | test setup/parity | 119 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:126` `canonical_filter_restores_nominal_refs_before_dependency_closure` | read | test setup/parity | 127, 128, 141 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:155` `filter_supports_enum_trait_static_and_pub_items` | read | test setup/parity | 179 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:190` `filter_supports_async_const_unsafe_fn_and_static_mut` | read | test setup/parity | 207 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:216` `filter_tracks_type_alias_dependencies_and_drops_unused_aliases` | read | test setup/parity | 228 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:237` `filter_avoids_false_positive_from_local_variable_name` | read | test setup/parity | 249 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:255` `filter_tracks_dependencies_used_in_macro_arguments` | read | test setup/parity | 272 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:281` `filter_keeps_all_items_for_needed_type_name` | read | test setup/parity | 296 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:304` `filter_keeps_every_implementation_of_a_needed_trait` | read | test setup/parity | 323 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:645` `canonical_stdlib_names_are_sealed_across_declarations_and_uses` | read | test setup/parity | 645, 654, 671 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:680` `canonical_stdlib_sealing_uses_exact_module_identity` | read | test setup/parity | 680, 686, 691, 692, 699, 704, 709, 710, 715 |
| `crates/sifr_codegen/src/stdlib_filter/tests.rs:724` `canonical_stdlib_sealing_preserves_external_qualified_path_segments` | read | test setup/parity | 724, 731, 732 |
| `crates/sifr_codegen/src/stdlib_import_signatures.rs:5` `register_imported_stdlib_metadata` | read, enumerate, clone projection | layered view + demanded record handles | 5, 8, 13, 14, 15, 20, 21, 22, 27, 28, 29, 36, 38, 40, 51, 60, 61, 66, 69, 83, 86 |
| `crates/sifr_codegen/src/stdlib_import_signatures.rs:101` `transitive_stdlib_signature` | read, enumerate | layered view + demanded record handles | 101, 102, 106, 109, 110 |
| `crates/sifr_codegen/src/stdlib_import_signatures.rs:124` `register_imported_stdlib_signature` | read, clone projection | layered view + demanded record handles | 124, 126, 131, 132, 137, 141, 147, 149, 153 |
| `crates/sifr_codegen/src/stdlib_import_signatures_tests.rs:17` `imported_metadata_preserves_aliases_methods_fields_generators_and_constants` | read, borrow/retain | test setup/parity | 38, 39, 48, 52, 53, 55, 57, 59, 65, 66, 67, 69, 77, 78, 80, 83, 85 |
| `crates/sifr_codegen/src/stdlib_import_signatures_tests.rs:91` `private_alias_registration_preserves_local_signatures_without_overwriting_origin` | read | test setup/parity | 92, 93, 97, 103, 109, 116, 117, 118 |
| `crates/sifr_codegen/src/stdlib_interop_demand/observation.rs:17` `observe_stdlib_interop_selection` | read | layered view + demanded record handles | 17 |
| `crates/sifr_codegen/src/stdlib_interop_demand.rs:31` `application_plan` | read, enumerate | layered view + demanded record handles | 32, 36, 37 |
| `crates/sifr_codegen/src/stdlib_interop_demand.rs:46` `select` | read, enumerate | layered view + demanded record handles | 46, 49, 60, 79, 121 |
| `crates/sifr_codegen/src/stdlib_interop_demand.rs:199` `resolve` | read, enumerate | layered view + demanded record handles | 211, 215 |
| `crates/sifr_codegen/src/stdlib_interop_demand.rs:410` `stdlib_module_roots` | read, enumerate | layered view + demanded record handles | 410 |
| `crates/sifr_codegen/src/stmt_support_emitter/assert_and_augassign.rs:4` `lower_exact_int_augassign_stmt_for_ir` | read | layered view + demanded record handles | 16 |
| `crates/sifr_codegen/src/stmt_support_emitter/assert_and_augassign.rs:83` `try_lower_structured_aug_assign_stmt` | read | layered view + demanded record handles | 96 |
| `crates/sifr_codegen/src/stmt_support_emitter/call_args_and_returns.rs:6` `adapt_plain_call_args_with_signature_for_ir` | read, enumerate | layered view + demanded record handles | 168, 175 |
| `crates/sifr_codegen/src/stmt_support_emitter/call_args_and_returns.rs:352` `lower_recursive_capture_arg_for_ir` | read | layered view + demanded record handles | 358 |
| `crates/sifr_codegen/src/stmt_support_emitter/call_args_and_returns.rs:470` `lower_return_value_expr_for_ir` | read | layered view + demanded record handles | 580 |
| `crates/sifr_codegen/src/stmt_support_emitter/call_args_and_returns.rs:586` `lower_rendered_expr_for_ir` | read | layered view + demanded record handles | 593, 617 |
| `crates/sifr_codegen/src/stmt_support_emitter/class_upcasts.rs:308` `render_ancestor_rust_type` | read | layered view + demanded record handles | 316 |
| `crates/sifr_codegen/src/stmt_support_emitter/class_upcasts.rs:379` `ancestor_paths_are_local_stdlib_or_crate_rooted_by_identity` | read | test setup/parity | 379, 390 |
| `crates/sifr_codegen/src/stmt_support_emitter/condition_lowering.rs:3` `lower_condition_expr_for_ir` | read | layered view + demanded record handles | 132, 137 |
| `crates/sifr_codegen/src/stmt_support_emitter/print_calls.rs:66` `try_lower_stmt_expr_statement_only` | read, enumerate | layered view + demanded record handles | 140 |
| `crates/sifr_codegen/src/stmt_support_emitter/stmt_block.rs:7` `try_lower_stmt_block_for_ir_inner` | read, enumerate | layered view + demanded record handles | 92, 143, 216, 244, 888 |
| `crates/sifr_codegen/src/stmt_support_emitter/stmt_expr_method_and_question_mark.rs:529` `lower_stmt_expr_for_ir` | read, enumerate | layered view + demanded record handles | 570 |
| `crates/sifr_codegen/src/structural_impl_codegen/stdlib_implementations.rs:15` `imported_classes` | read, enumerate | layered view + demanded record handles | 17, 20, 21, 28 |
| `crates/sifr_codegen/src/structural_impl_codegen.rs:17` `emit_imported_stdlib_structural_impls` | read, enumerate, clone projection | layered view + demanded record handles | 17, 20, 25, 35 |
| `crates/sifr_codegen/src/support_plan.rs:52` `from_emitter` | read, enumerate | layered view + demanded record handles | 79, 80 |
| `crates/sifr_codegen/src/support_plan.rs:107` `merge_project_module` | read, enumerate | layered view + demanded record handles | 116, 117, 118, 119 |
| `crates/sifr_codegen/src/support_plan.rs:139` `needs_support` | read | layered view + demanded record handles | 142 |
| `crates/sifr_codegen/src/support_plan.rs:147` `directly_used_stdlib_modules` | read | layered view + demanded record handles | 147, 148 |
| `crates/sifr_codegen/src/support_plan.rs:167` `render_support` | read | layered view + demanded record handles | 169, 171, 172, 175, 184, 185, 187, 189, 228, 229, 230, 237, 251, 287, 292, 326 |
| `crates/sifr_codegen/src/support_plan.rs:393` `render_stdlib_support` | read, enumerate | layered view + demanded record handles | 393, 395, 400, 404, 406, 411, 412, 414, 415, 435, 457 |
| `crates/sifr_codegen/src/support_plan.rs:480` `append_stdlib_dependencies` | read, enumerate | layered view + demanded record handles | 480, 482, 489, 492 |
| `crates/sifr_codegen/src/support_plan.rs:622` `single_file_user_error_suppresses_late_runtime_demand` | read | test setup/parity | 631 |
| `crates/sifr_codegen/src/support_plan.rs:638` `project_merge_does_not_promote_a_module_shadow_to_a_crate_veto` | read | test setup/parity | 644, 650, 651, 662 |
| `crates/sifr_codegen/src/union_type_helpers.rs:106` `collect_union_types` | read, enumerate | layered view + demanded record handles | 114, 119, 156, 184 |
| `crates/sifr_analysis/src/host/construction.rs:8` `open_project` | read, clone projection, borrow/retain | layered view + demanded record handles | 17 |
| `crates/sifr_analysis/src/host/construction.rs:23` `open_single_file` | read, borrow/retain | layered view + demanded record handles | 29 |
| `crates/sifr_analysis/src/host/construction.rs:46` `new_with_sql_profiles` | read, enumerate, borrow/retain | layered view + demanded record handles | 64 |
| `crates/sifr_analysis/src/host/file_access.rs:7` `path_for_file` | read | layered view + demanded record handles | 11 |
| `crates/sifr_analysis/src/host/file_access.rs:15` `stdlib_navigation` | read, borrow/retain | layered view + demanded record handles | 15, 16 |
| `crates/sifr_analysis/src/host/file_access.rs:18` `source_text_for_file` | read | layered view + demanded record handles | 19, 21 |
| `crates/sifr_analysis/src/host/generated_rust_preview_tests.rs:38` `generated_rust_preview_tracks_generated_support_source_map_entry` | read | test setup/parity | 66 |
| `crates/sifr_analysis/src/host/implementation.rs:661` `symbol_index` | read | layered view + demanded record handles | 675 |
| `crates/sifr_analysis/src/host/implementation.rs:685` `refresh_existing_symbol_index` | read | layered view + demanded record handles | 701 |
| `crates/sifr_analysis/src/host/implementation.rs:705` `locations_for_identifier_at` | read | layered view + demanded record handles | 716 |
| `crates/sifr_analysis/src/host/overlay_updates.rs:10` `open_project_with_overlays` | read, clone projection, borrow/retain | layered view + demanded record handles | 20 |
| `crates/sifr_analysis/src/host/overlay_updates.rs:30` `open_single_file_overlay` | read, clone projection, borrow/retain | layered view + demanded record handles | 41 |
| `crates/sifr_analysis/src/host/semantic_editor.rs:96` `hover_symbol_file` | read, enumerate | layered view + demanded record handles | 111 |
| `crates/sifr_analysis/src/host/semantic_editor_tests.rs:14` `hover_returns_semantic_function_and_binding_details` | read, borrow/retain | test setup/parity | 136, 152 |
| `crates/sifr_analysis/src/host/stdlib_navigation.rs:10` `refresh_stdlib_symbol_bucket` | read | layered view + demanded record handles | 10, 12, 14 |
| `crates/sifr_analysis/src/host/stdlib_navigation.rs:18` `stdlib_import_location_for_token` | read | layered view + demanded record handles | 18, 23, 34, 41, 47 |
| `crates/sifr_analysis/src/host/stdlib_navigation.rs:56` `stdlib_symbols_from_source_map` | read, enumerate | layered view + demanded record handles | 56, 57 |
| `crates/sifr_analysis/src/host/stdlib_navigation.rs:73` `stdlib_symbol_location_from_source_map` | read, enumerate | layered view + demanded record handles | 73, 79 |
| `crates/sifr_analysis/src/host/stdlib_navigation.rs:94` `stdlib_import_target` | read | layered view + demanded record handles | 94 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:11` `stdlib_interop_startup_editor_retained_snapshot_after_defs_projection` | read, enumerate, borrow/retain | test setup/parity | 11, 42 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:149` `assert_stdlib_import_resolves` | read, enumerate, borrow/retain | test setup/parity | 149 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:174` `single_file_analysis_resolves_sysroot_stdlib_imports` | read, borrow/retain | test setup/parity | 174, 185 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:189` `project_analysis_resolves_sysroot_stdlib_imports` | read, borrow/retain | test setup/parity | 189, 210 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:215` `analysis_source_map_tracks_public_and_private_sysroot_origins` | read, enumerate, borrow/retain | test setup/parity | 237 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:256` `stdlib_symbol_bucket_is_available_without_private_declarations` | read, enumerate, borrow/retain | test setup/parity | 256, 280, 285, 286 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:297` `definition_for_public_stdlib_import_lands_in_sysroot_source` | read, borrow/retain | test setup/parity | 297 |
| `crates/sifr_analysis/src/host/stdlib_tests.rs:334` `definition_inside_public_stdlib_can_link_to_private_declaration_file` | read, enumerate, borrow/retain | test setup/parity | 334, 343 |
| `crates/sifr_analysis/src/lib.rs:55` `tooling_sysroot_status` | read | layered view + demanded record handles | 58 |
| `crates/sifr_analysis/src/lib.rs:62` `tooling_sysroot_probe` | read | layered view + demanded record handles | 63 |
| `crates/sifr_analysis/src/symbols.rs:141` `replace_stdlib_symbols` | read | layered view + demanded record handles | 141 |
| `crates/sifr_analysis/src/symbols.rs:266` `stdlib_symbol_location` | read, enumerate | layered view + demanded record handles | 266 |
| `crates/sifr_analysis/src/symbols.rs:453` `symbol_index_records_workspace_package_and_stdlib_readiness` | read, enumerate | test setup/parity | 453 |
| `crates/sifr_lsp/src/analysis_workspace.rs:58` `restored_check_modules` | read, enumerate, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:79` `has_analysis` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:85` `open_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:104` `update_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:146` `synchronize_projects` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:180` `record_watcher_events` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:196` `load_diagnostics` | read, enumerate | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:209` `can_analyze_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:222` `file_maps_for_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:243` `workspace_symbols` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:264` `with_document` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:299` `uri_for` | read | layered view + demanded record handles | 304 |
| `crates/sifr_lsp/src/analysis_workspace.rs:311` `source_for` | read | layered view + demanded record handles | 315 |
| `crates/sifr_lsp/src/analysis_workspace.rs:324` `open` | read, enumerate, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:523` `with_host` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:550` `file_maps` | read | layered view + demanded record handles | 567 |
| `crates/sifr_lsp/src/analysis_workspace.rs:573` `uri_by_file` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:593` `workspace_symbols` | read | layered view + demanded record handles | 619 |
| `crates/sifr_lsp/src/analysis_workspace.rs:631` `open` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:681` `from_host` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:690` `with_host` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:717` `file_maps` | read | layered view + demanded record handles | 751 |
| `crates/sifr_lsp/src/analysis_workspace.rs:757` `workspace_symbols` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/analysis_workspace.rs:795` `workspace_root_for` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/capabilities.rs:139` `execute_command_provider_only_advertises_server_workspace_commands` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/commands.rs:29` `explain_diagnostic` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/commands.rs:60` `generated_rust_preview` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/compiled_identity.rs:2` `compiled_input_tokens` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:162` `workspace_symbol` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:191` `completion_item` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:294` `selection_range` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:329` `code_action` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:348` `code_action_data` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:363` `workspace_edit` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/conversion.rs:500` `source_origin_name` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/diagnostics.rs:60` `publish_all` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/diagnostics.rs:64` `reconcile_changes` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/diagnostics.rs:71` `publish_workspace` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/diagnostics.rs:194` `document_diagnostics` | read, enumerate | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/dxf_embedding_tests.rs:6` `production_workspace_preserves_context` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/notifications/mod.rs:9` `handle` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/notifications/mod.rs:41` `initialized` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/notifications/mod.rs:50` `publish_tooling_sysroot_diagnostic` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/notifications/mod.rs:67` `tooling_sysroot_notification` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/notifications/mod.rs:98` `workspace_did_change_configuration` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/notifications/mod.rs:116` `workspace_did_change_watched_files` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/python_declarations.rs:146` `python_declaration_snapshot` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/python_declarations.rs:364` `resolve_package_python_environment_inner` | read, enumerate | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/code_action.rs:8` `code_action` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/code_action.rs:41` `resolve` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/completion.rs:7` `completion` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/diagnostics.rs:15` `workspace_diagnostic` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/folding_range.rs:7` `folding_range` | read, enumerate | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/formatting.rs:9` `formatting` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/formatting.rs:33` `range_formatting` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/formatting.rs:61` `format_options` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/hover.rs:7` `hover` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/hover.rs:29` `signature_help` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/inlay_hint.rs:7` `inlay_hint` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:22` `handle` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:84` `sysroot_status` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:108` `sysroot_status_from_probe` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:130` `sysroot_status_success` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:179` `sysroot_status_failure` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:214` `document_position` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/mod.rs:228` `code_action_context_diagnostics` | read, enumerate | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/navigation.rs:32` `document_highlight` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/navigation.rs:49` `prepare_rename` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/navigation.rs:78` `rename` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/navigation.rs:101` `locations` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/selection_range.rs:7` `selection_range` | read, enumerate | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/semantic_tokens.rs:22` `tokens` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/symbols.rs:9` `document_symbol` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/symbols.rs:24` `workspace_symbol` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/type_hierarchy.rs:8` `prepare` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/requests/type_hierarchy.rs:34` `hierarchy` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session/tests/dx11_editor_tests.rs:198` `dx11_incremental_push_reconciles_multiple_documents_without_workspace_progress` | read, enumerate | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/project_ownership_tests.rs:5` `project_save_without_version_keeps_project_owner_current` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/project_ownership_tests.rs:68` `unmapped_project_file_does_not_create_standalone_project_fallback` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/project_ownership_tests.rs:143` `unmapped_project_file_does_not_fallback_after_unrelated_root_refresh` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/python_declaration_tests.rs:539` `workspace_member_python_requirements_are_validated` | read, enumerate | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/python_package_ownership_tests.rs:4` `sifr_only_package_does_not_adopt_an_ancestor_cargo_workspace` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/sql_editor_tests.rs:5` `sql_virtual_document_features_share_the_analysis_snapshot` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:8` `definition_request_for_stdlib_import_returns_sysroot_uri` | read | test setup/parity | 8, 9 |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:36` `definition_request_for_stdlib_call_returns_sysroot_uri` | read | test setup/parity | 36, 37 |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:64` `type_definition_request_for_stdlib_import_returns_sysroot_uri` | read | test setup/parity | 64, 65 |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:92` `hover_request_for_stdlib_call_reflects_installed_signature` | read | test setup/parity | 92, 93 |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:116` `completion_request_includes_public_stdlib_symbols_not_private_modules` | read, enumerate | test setup/parity | 116, 117 |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:445` `cli_sysroot_status` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:474` `workspace_root` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session/tests/sysroot_request_tests.rs:482` `open_stdlib_import_fixture` | read | test setup/parity | 482 |
| `crates/sifr_lsp/src/session.rs:49` `restored_check_modules` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:53` `ensure_document_analysis` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:65` `refresh_toolchain` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:76` `compiler_context` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:87` `with_compiler` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:125` `open_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:156` `apply_compacted_change` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:180` `save_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:195` `close_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:208` `record_watcher_events` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:221` `can_analyze_document` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:231` `load_diagnostics` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:235` `file_maps_for_uri` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:241` `workspace_symbols` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:258` `with_document_analysis` | read, borrow/retain | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/session.rs:521` `open_document_analysis_uses_unsaved_overlay_text` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session.rs:549` `changed_document_keeps_analysis_in_session_workspace` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/session.rs:662` `document_change_batch_compacts_before_analysis_update` | read | test setup/parity | indirect |
| `crates/sifr_lsp/src/settings.rs:5` `settings_from_initialize_params` | read | snapshot query (indirect) | indirect |
| `crates/sifr_lsp/src/settings.rs:20` `parse_workspace_settings` | read | snapshot query (indirect) | indirect |
| `crates/sifr/src/build_output.rs:187` `sysroot_dependency_plan` | read | layered view + demanded record handles | 189 |
| `crates/sifr/src/sysroot_cli.rs:36` `cmd_doctor` | read | layered view + demanded record handles | 112, 113, 115, 147 |
| `crates/sifr/src/sysroot_cli.rs:184` `print_sysroot` | read | layered view + demanded record handles | 199, 200, 201, 204, 205 |

<!-- dx5-sites:end -->
