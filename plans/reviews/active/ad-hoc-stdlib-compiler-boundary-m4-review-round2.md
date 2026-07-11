# M4 Review — Round 2

## Verdict

**SATISFIED**

Round 1's only soft coverage gap — the loss of the structured-intrinsic
nested-argument and typed-method-call-argument variants — is now closed with two
retained-ID tests that both pass locally. The permanent manifest row description
no longer uses delivery-plan language, the retained-set arithmetic still totals
17 typed IDs, and every deletion, guard, and behavior claim from round 1
continues to hold in the current working-tree diff.

## Restored structured-intrinsic coverage — CONFIRMED

- `crates/sifr_codegen/src/lib_codegen_tests/structured_intrinsic_codegen_tests.rs`
  now defines `module_with_expr` + `bytes_with_size` helpers and two `#[test]`
  functions:
  - `structured_intrinsic_supports_nested_intrinsic_arguments` uses
    `CompilerIntrinsicId::TestAssertEqual` with two nested
    `CompilerIntrinsicId::BytesWithSize` arguments and asserts the rendered
    source contains `assert_eq!(`, contains
    `bytes(size) requires a non-negative size` (the checked-body path in
    `crates/sifr_codegen/src/intrinsics/registry/bytes.rs:242`), and reports
    `expr_structured > 0`. This single test covers both the round-1
    "bare intrinsic call as expression statement" and "nested intrinsic call
    argument" variants because the outer `TestAssertEqual` is the bare
    expression-statement while its two `BytesWithSize` args are the nested
    intrinsic arguments.
  - `structured_intrinsic_supports_typed_method_call_arguments` uses
    `CompilerIntrinsicId::BytesFromHex` with a `HirExpr::MethodCall`
    (`"A0".lower()`) argument and asserts the rendered source contains
    `.to_lowercase()` and `u8::from_str_radix(pair_str, 16)` (from
    `crates/sifr_codegen/src/intrinsics/registry/bytes.rs:206`), and reports
    `expr_structured > 0`. This recovers the round-1 typed-method-call-arg
    variant.
- Both tests were executed locally:
  `cargo test -p sifr_codegen --lib structured_intrinsic` — `2 passed; 0 failed`.

## Deletion completeness — CONFIRMED (unchanged)

- `grep -rn "counter_from_list|counter_get|counter_most_common|counter_total|counter_values|counter_keys|counter_items|counter_increment|_defaultdict_new_impl|_defaultdict_get_impl|_defaultdict_set_impl"`
  across `crates/`, `stdlib/`, `internal_docs/`, and `scripts/` returns only the
  eleven residue strings inside
  `scripts/check_stdlib_native_intrinsic_allowlist.py:50-60` (the residue
  guard's own tuple). The only other match anywhere in the repo is a locally
  named variable `counter_total` inside
  `verification/areas/performance/tools/run_integer_model_readiness_perf.py`,
  which is unrelated to the deleted intrinsics and outside the guard's scan
  roots — no false positive risk.
- `grep -rn "CounterFromList|CounterGet|CounterMostCommon|CounterTotal|CounterValues|CounterKeys|CounterItems|CounterIncrement"`
  across `crates/` returns zero matches; the eight enum arms are fully removed
  from `crates/sifr_ir/src/hir_nodes.rs:118-136` including their
  `declaration_name` and `from_declaration_name` matches.
- Lowerer files `crates/sifr_codegen/src/intrinsics/registry/collections/{counter_defaultdict_intrinsics.rs,set_and_list_intrinsics.rs}`
  are deleted from the working tree.
- Fallback signatures for the eleven residues are removed from
  `crates/sifr_retained_intrinsics/src/collections_bytes_time.rs:111` (net −106
  lines).
- Registry-only Counter test replaced with negative-list test
  `collections_bridge_helpers_are_not_intrinsics` in
  `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:336` — passes
  locally (`1 passed`).
- Serialized bridge deletion confirmed in
  `stdlib/_sifr/collections.sifr:38-40`, `stdlib/sifr/collections.sifr:1-3`,
  and `crates/sifr_stdlib/src/collections.rs:73` (public wrappers and the
  `defaultdict_helpers_preserve_default_json_behavior` test both removed).
- `StdlibFeature::SerdeJson` no longer emits direct `serde`/`serde_json`
  dependency specs in
  `crates/sifr_stdlib_manifest/src/features/dependency_plan.rs:274`;
  `internal_docs/stdlib_retained_compiler_intrinsics.toml:118` still lists
  only the six non-collection retained direct dependency packages.

## Live typed-defaultdict registry ownership — CONFIRMED

- `crates/sifr_codegen/src/intrinsics/registry/collections.rs` shrinks to two
  shared classifier helpers and one in-file test:
  - `is_defaultdict_storage_alias` matches `__sifr_defaultdict_{int,list,set}`.
  - `is_collection_defaultdict_storage_alias` matches only
    `__sifr_defaultdict_{list,set}` — correctly excluding `int` from the
    append/add paths.
  - `typed_defaultdict_alias_classification_excludes_legacy_serialized_helpers`
    asserts inclusion and exclusion cases.
- Re-exported from `crates/sifr_codegen/src/intrinsics/registry.rs:14-16` and
  consumed by four downstream sites — all four call the classifier through
  `crate::intrinsics::is_(collection_)defaultdict_storage_alias`:
  `intrinsic_method_emitters/builtin_core_methods.rs:81,90`,
  `intrinsic_method_emitters/collection_methods.rs:547`,
  `lower_expr/leaves_and_plain_calls.rs:388`,
  `stmt_support_emitter/performance_lowering_gate.rs:161`.
- `cargo check -p sifr_codegen --lib` finished cleanly in the working tree.

## Manifest accuracy — CONFIRMED

- `sifr.collections::typed_defaultdict_language_semantics`
  (`internal_docs/stdlib_retained_compiler_intrinsics.toml:50-58`) replaces the
  legacy `_sifr.collections::counter_defaultdict` row: `state =
  "retained-by-design"`, `registry_files = ["collections.rs"]`, no
  `exact_intrinsics`, `declaration_files = ["stdlib/sifr/collections.sifr"]`,
  and `reason` now describes typed defaultdict language semantics + generic
  `Counter[T]` in checked Sifr source + retained registry alias
  classification. No delivery-plan wording remains.
- `sifr.bytes::primitive_constructors`
  (`internal_docs/stdlib_retained_compiler_intrinsics.toml:77-89`) replaces
  `_sifr.bytes::first_class_constructors`: keeps the three retained IDs
  `bytes_from_hex`, `bytes_from_integers`, `bytes_with_size` and drops the
  M3-migrated `bytes_to_hex_strict`. Reason accurately describes the
  public-wrapper → primitive-construction HIR path and the separate strict-hex
  bridge routing.
- `scripts/check_stdlib_manifest_schema.py:40-44` adds both
  `_sifr.bytes::first_class_constructors` and
  `_sifr.collections::counter_defaultdict` to
  `PLANNED_REARCHITECTURE_DELETIONS`, so `_validate_final_transitions` accepts
  their removal from the `origin/main` base manifest without demoting them to
  `closing` first.
- Live check runs pass:
  - `python3 scripts/check_stdlib_manifest_schema.py` → PASS
    (surfaces=10, schema_version=2, final_state=retained-by-design).
  - `python3 scripts/check_stdlib_manifest_schema.py --self-test` → PASS.
  - `python3 scripts/check_stdlib_native_intrinsic_allowlist.py` → PASS
    (exact_intrinsics=17, registry_files=8, preamble_files=9,
    fallback_signature_modules=17, retained_direct_dependency_packages=6,
    direct_runtime_roots=2).
  - `python3 scripts/check_stdlib_native_intrinsic_allowlist.py --self-test` →
    PASS.
  - `python3 scripts/check_stdlib_bootstrap_ordering.py` → PASS
    (private=30, public=61, public_edges=15) and self-test → PASS.
- Retained-set arithmetic still totals 17 typed IDs: `_sifr.fs` 2 +
  `generated-test-glue` 7 + `_sifr.encoding::utf8_bytes_string_glue` 4 +
  `sifr.bytes::primitive_constructors` 3 + `_sifr.task::language_runtime_glue`
  1 = 17, matching the guard's `exact_intrinsics=17`.

## Guard scope — CONFIRMED

- `DELETED_COLLECTION_RESIDUE_ROOTS` covers all six deletion loci:
  `crates/sifr_ir/src/hir_nodes.rs`, `REGISTRY_ROOT`,
  `crates/sifr_retained_intrinsics/src`,
  `crates/sifr_stdlib/src/collections.rs`,
  `stdlib/_sifr/collections.sifr`, and `stdlib/sifr/collections.sifr`.
- Directory roots iterate only `*.rs`; the two `.sifr` roots are treated as
  single files. Verified by inspection: the surviving Counter method names in
  `stdlib/sifr/collections.sifr` (`get`, `total`, `most_common`, `keys`,
  `values`, `items`, `increment`, `from_list`) do NOT share substrings with
  any of the eleven `counter_`-prefixed / `_defaultdict_*_impl` residues, so
  the checked Sifr source produces no false positives.
- No raw-name bypass remains: no `HirExpr::Call { func: "counter_*" }` or
  `func: "_defaultdict_*_impl"` construction survives in codegen/lowering, and
  the only surviving `__sifr_defaultdict_*` occurrences are the typed
  storage-alias strings inside the shared classifier — the intended
  language-lowering path.

## Source Counter/defaultdict behavior — CONFIRMED

- `stdlib/sifr/collections.sifr:44-248` implements generic `Counter[T:
  Hashable]` and module-level `from_list[T: Hashable]` entirely in checked
  Sifr source over `dict[T, int]`. No serialized JSON path remains.
- Typed `defaultdict` construction stays a compiler/type-system intercept in
  `sifr_lowering::lower::mod_impl::explicit_defaultdict_bindings` (unchanged
  by M4), routed to language storage aliases `__sifr_defaultdict_{int,list,set}`
  which the classifier in `intrinsics/registry/collections.rs` gates.
- New E2E fixture
  `crates/sifr/tests/e2e/pass/collections_boundary_ownership.sifr` exercises
  `Counter[str]` via `from_list`, `defaultdict(int|list|set)`, and each of the
  three public bytes wrappers; it is registered in
  `verification/areas/core_language/data/create_pr_e2e_manifest.json`.

## Checked bytes wrapper proof — CONFIRMED

- `crates/sifr_driver/src/stdlib/stateless_private_codegen_tests.rs:245-272`
  now proves the public `sifr.bytes` module emits:
  1. Function bodies `fn bytes_from_hex(`, `fn bytes_from_ints(`, and
     `fn bytes_with_size(`.
  2. Their in-source error strings —
     `u8::from_str_radix(pair_str, 16)`, `byte out of range at index`, and
     `bytes(size) requires a non-negative size` — which are produced by the
     compiler intrinsic bodies in
     `crates/sifr_codegen/src/intrinsics/registry/bytes.rs:206,242,335`.
  3. That `bytes_from_hex`, `bytes_from_integers`, and `bytes_with_size` are
     NOT present in `compiled.code.intrinsic_names["sifr.bytes"]`, confirming
     the public surface exposes checked source bodies rather than raw
     intrinsic dispatch — precisely the M4 requirement.
- The pre-existing `_sifr.bytes` private-declaration assertions remain
  intact; strict-hex still routes through the checked `_sifr.bytes` Rust
  bridge declaration, matching the updated `reason` text.

## Non-blocking notes

1. `crates/sifr_codegen/src/lib_codegen_tests.rs:39-42` declares
   `mod structured_intrinsic_codegen_tests;` without a `#[cfg(test)]`
   attribute and declares
   `mod structured_lowering_codegen_tests;` with two adjacent
   `#[cfg(test)] #[cfg(test)]` attributes. Because the parent
   `mod lib_codegen_tests;` is itself gated on `#[cfg(test)]` in
   `crates/sifr_codegen/src/lib.rs:115`, both anomalies are functionally inert
   (`cargo check -p sifr_codegen` succeeds and both tests run under
   `cargo test`), but the module list is cosmetically inconsistent with its
   siblings. Purely a stylistic cleanup — recommend restoring the single
   `#[cfg(test)]` per sibling module in a small follow-up.

2. Round 1 note 2 (imprecise `declaration_files` pointer on
   `sifr.collections::typed_defaultdict_language_semantics`) still applies:
   `stdlib/sifr/collections.sifr` does not declare `defaultdict` — the
   identifier is intercepted at import time by
   `sifr_lowering::lower::mod_impl::explicit_defaultdict_bindings`. The
   round-2 `reason` acknowledges this by naming the compiler/type-system
   ownership, and phase plan M6 owns precise cross-crate file enumeration.
   Non-blocking for M4; carry into M6.

3. Round 1 note 3 (phase doc `plans/phases/06_stdlib_architecture.md:285,301,
   304,323,336` still describes deleted `counter_from_list/get/most_common`
   intrinsics as design) still holds. M6's task list owns phase doc updates;
   the residue guard does not scan `plans/`. Non-blocking for M4.

4. Round 1 note 4 (residue-guard `_deleted_collection_residue_failures`
   assumes each of the six roots exists — a later M5 deletion of
   `sifr_retained_intrinsics/src` would raise `FileNotFoundError`) still
   holds. Non-blocking for M4; M5 must adjust when it deletes that crate.

## Validation gaps to run in the create-PR gate

Static review plus the local checks below confirm structure, semantics,
guards, and coverage. The authoritative
`scripts/run_all_tests.sh --profile create-pr` pass reported in the review
prompt (crate tests 126,439 ms / 600,000 ms, runtime platform 58,139 ms /
120,000 ms, E2E 31,402 ms / 600,000 ms, 130/130 E2E fixtures) is the
merge-gating evidence. Also run before merge:

- `cargo check -p sifr_ir -p sifr_codegen -p sifr_retained_intrinsics -p sifr_stdlib -p sifr_stdlib_manifest -p sifr_driver -p sifr` (affected crates).
- `cargo test -p sifr_codegen` (full codegen suite including the two restored
  `structured_intrinsic_codegen_tests` and the renamed
  `collections_bridge_helpers_are_not_intrinsics`).
- Typed-defaultdict lowering filter suite plus the two Counter/defaultdict
  ownership filters.
- `cargo test -p sifr_driver stateless_collections_codegen_tests stateless_private_codegen_tests`.
- `cargo test -p sifr_stdlib_manifest` and `-p sifr_retained_intrinsics`.
- Native runs for `collections_boundary_ownership`, `generic_counter_int`,
  `generic_counter_bigint`, `generic_counter_custom_class`, and
  `defaultdict_len_and_deque`.
- All three stdlib guards main + `--self-test`.

## Local verification performed in this review

- `python3 scripts/check_stdlib_manifest_schema.py` — PASS.
- `python3 scripts/check_stdlib_manifest_schema.py --self-test` — PASS.
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py` — PASS.
- `python3 scripts/check_stdlib_native_intrinsic_allowlist.py --self-test` — PASS.
- `python3 scripts/check_stdlib_bootstrap_ordering.py` — PASS.
- `python3 scripts/check_stdlib_bootstrap_ordering.py --self-test` — PASS.
- `cargo check -p sifr_codegen --lib` — cleanly finished.
- `cargo test -p sifr_codegen --lib structured_intrinsic` — 2 passed.
- `cargo test -p sifr_codegen --lib collections_bridge_helpers_are_not_intrinsics`
  — 1 passed.
- Repo-wide grep for the eleven deleted residues finds no matches outside the
  residue-guard's own tuple entries.

## Summary

M4's Collections Residue Removal remains complete, semantically preserved, and
appropriately guarded. Round 1's soft coverage gap is closed with two
retained-ID structured-intrinsic tests that pass locally, and the manifest
row's permanent language is now taxonomy-clean. No blocking findings — only
the four non-blocking notes above. Proceed to merge on the reported
authoritative gate pass.
