# Ad Hoc Phase: Pre-v1 Compatibility Removal

Status: in progress; Item 11 merged and ready for Item 12 on 2026-08-19

## Objective

Remove Sifr-owned fallback, legacy, alias, and transition paths before the
first compatibility promise.

The phase keeps one canonical contract for each language, stdlib, compiler,
tooling, verification, and installation surface.

The phase does not remove compatibility that an external protocol requires.
It also does not remove normal error handling or product fallback values.

## Exit State

The phase is complete when all these conditions are true:

- The public language has no `bigint` transition alias.
- Each stdlib operation has one canonical public name and signature.
- First-class types replace temporary helper modules and procedural adapters.
- Receiver rules do not infer or reinterpret syntax for source compatibility.
- Package manifests accept one source-root schema and one single-root layout.
- The installer accepts one canonical layout.
- Diagnostics do not retain old Sifr codes or old syntax recognizers.
- Lowering does not recognize hidden `__compat_*` names.
- Verification uses one profile schema and one execution model.
- E2E pass fixtures use assertions as the only runtime expectation model.
- Compiler services use one provider-backed source path.
- Code generation uses structured Rust types only.
- An executable guard rejects new Sifr-owned compatibility paths.
- External protocol compatibility remains intact and documented.

## Reason for This Phase

Sifr has no v1 compatibility obligation. Current architecture rules also
forbid new fallback and backward-compatibility paths without approval.

Several earlier phases kept temporary bridges. Later phases removed some
bridges but did not remove all consumers, aliases, schemas, and tests.

This phase removes that remaining debt in small, sequential items. Each item
migrates all repository consumers before it removes the old path.

## Source Evidence

The initial audit found these Sifr-owned compatibility surfaces:

| Area | Current evidence | Required disposition |
| --- | --- | --- |
| Exact integers | `Type::BigInt`, `SIFR-INT-0011`, `SIFR-TYPE-0006`, and quarantined fixtures | Remove the public transition and its dedicated diagnostics. |
| Stdlib names | Phase 7 kept old names as aliases for later removal | Select one name for each operation, then remove all aliases. |
| Binary APIs | `sifr.bytes` is a temporary wrapper over first-class `bytes` | Move required behavior to first-class APIs, then remove transition helpers. |
| Collections | Procedural list-backed set helpers remain for backward compatibility | Migrate consumers to first-class generic collections, then remove the helpers. |
| Hashing | String helpers coexist with bytes-native helpers | Keep one canonical bytes-native contract. |
| Sorted collections | `heapq` and `bisect` keep copy-returning compatibility APIs | Keep the canonical mutating APIs. |
| Receivers | Lowering retains inferred mutability and old `own self` interpretation | Lock explicit receiver rules, then remove compatibility inference. |
| Package manifests | Package and driver readers accept different source-root schemas and defaults | Keep `[source].root` and the `src/` default only. |
| Installation | Self-update accepts toolchain-root and flat layouts | Keep one toolchain-root layout. |
| Diagnostics | Production project paths emit old workspace codes beside canonical import codes | Keep canonical diagnostics only. |
| Hidden names | Two `__compat_sifr_*` recognizers remain after alias generation was removed | Remove the recognizers and their tests. |
| Verification | Profiles require `legacy_facade` and accept schema versions 1 and 2 | Keep one schema and one selected-area execution model. |
| E2E expectations | The harness accepts assertions and `# expect-stdout` directives | Keep assertion-based runtime checks only. |
| Source access | Provider APIs coexist with disk-backed and result-collapsing wrappers | Keep provider-backed APIs and structured resolution results. |
| Rust types | Structured `RustType` coexists with string-based `Type::rust_type()` | Complete structured conversion and remove the string path. |

Primary evidence files:

- `verification/areas/core_language/data/integer_model/implementation_inventory.md`
- `plans/phases/07_stdlib_parity.md`
- `plans/issues/archive/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
- `plans/issues/archive/milestone_stdlib_classes.md`
- `plans/issues/archive/ad-hoc-class-field-mutating-receiver-place-semantics.md`
- `plans/phases/14_codegen_architecture.md`
- `internal_docs/distribution_pipeline.md`
- `docs/package_management.md`
- `crates/sifr_package/src/manifest/production.rs`
- `crates/sifr_driver/src/workspace/mod.rs`
- `crates/sifr_driver/src/project/discovery.rs`
- `crates/sifr_driver/src/project/compile_order.rs`
- `crates/sifr_package/src/imports/source_map.rs`
- `crates/sifr/tests/e2e_support/harness_model.rs`
- `internal_docs/typescript_go_architecture_transfer_guardrails.md`
- `verification/schemas/profile.schema.json`

## Definitions

This phase uses the following terms:

- **Compatibility path:** A path that accepts an older Sifr name, shape,
  schema, layout, diagnostic code, or behavior.
- **Transition path:** A temporary path that connects an old Sifr model to a
  canonical Sifr model.
- **External compatibility:** Support for a published protocol, file format,
  operating system, editor protocol, or dependency contract.
- **Normal fallback:** A documented product value that applies when data is
  absent. Examples include configuration defaults and translation chains.
- **Canonical contract:** The only supported Sifr-owned name, shape, schema,
  layout, or behavior after an item merges.

## Locked Decisions

1. The phase adds no deprecation period, warning period, migration mode, or
   compatibility flag.
2. Each removal item migrates repository consumers and removes the old path in
   the same implementation PR.
3. A removed alias does not remain as an undocumented import or private-name
   re-export.
4. A canonical API preserves useful behavior from a broader old alias before
   the alias is removed.
5. If behavior is not part of the canonical contract, the item adds a direct
   compile-time diagnostic or removes the behavior.
6. The phase does not keep old names only because demos, tests, or audit
   fixtures use them.
7. Historical records under `plans/issues/archive/` remain unchanged unless a
   broken current link requires a correction.
8. Negative tests can prove that an unsupported schema fails. They must not
   implement a reader for that schema.
9. One top-level composition root can construct `DiskSourceProvider`. Lower
   layers must receive a provider or a captured session.
10. Code generation must fail through a structured compiler error for an
    unsupported type. It must not use a string renderer as a fallback.
11. The phase keeps exact-integer runtime storage that canonical `int` needs.
    It removes only the public `bigint` type split and its transition logic.
12. Each item obeys the phase-closure loop. One session owns one unfinished
    item, one worktree, one branch, and one Git index.

## Retained External and Product Contracts

The following surfaces are not Sifr pre-v1 compatibility debt:

- DLPack legacy and versioned capsules.
- LSP UTF-16 position defaults for clients that omit an encoding.
- Cargo metadata normalization and dependency-edge handling.
- Cargo semantic-version resolution.
- IPC protocol-version negotiation inside the documented support range.
- Operating-system and architecture portability paths.
- Cancellation hooks that preserve cleanup and task termination.
- Configuration fallback values.
- Translation and locale fallback chains.
- Vendored dependency compatibility code.
- Support for documented external file formats and protocol versions.
- Phase 40 release-channel bootstrap evidence and its `legacy-index` options.
- The `RuleStatus::Deprecated` lint lifecycle state.

The phase must not remove these contracts. Item 0 records an exact retained
list so that later broad scans do not misclassify them.

## Rejected-Only Legacy Forms

Some code recognizes old Sifr forms only to emit a targeted error. These paths
do not execute old behavior.

The phase still removes Sifr-specific migration residue from these paths. The
canonical parser or resolver will emit the normal unsupported-form diagnostic.

This rule applies to:

- old Sifr stdlib module names and their replacement table,
- old Rust interop decorator keys,
- old workspace diagnostic-code pages,
- migration-only messages that name abandoned schemas or layouts.

Generic unsupported-version tests remain in scope. They do not name or parse
an abandoned Sifr contract.

## Scope

### In scope

- Public language aliases and transition-only types.
- Public stdlib naming aliases and behavioral compatibility helpers.
- Repository demos, fixtures, docs, snapshots, and generated baselines that
  use removed surfaces.
- Compiler recognition of hidden or old internal names.
- Sifr-owned profile, report, and installation schemas with dual readers.
- Test-only compatibility helpers that preserve obsolete compiler APIs.
- Internal API wrappers that keep pre-session or string-based compiler paths.
- Guardrails that prevent compatibility residue from returning.

### Out of scope

- New language or stdlib features.
- New CPython source-compatibility requirements.
- Compatibility for unpublished Sifr releases.
- External protocol or file-format removal.
- Changes to vendored dependencies.
- Unrelated compiler correctness work.
- Unrelated performance work.
- A migration command or source codemod.
- Automatic repair of user projects outside this repository.

CPython parity does not retain duplicate Sifr APIs. Remove a CPython-shaped
duplicate when it exposes an operation that has a canonical Sifr name.

Keep CPython parity semantics when the canonical Sifr operation requires those
semantics. Item 0 records the classification for each surface.

## Execution Rules

Items execute in the listed order. Do not start an item until all earlier
items are merged and the phase document records their evidence.

Each implementation item uses this lifecycle:

1. Read this phase document and the item boundary.
2. Make sure that the worktree and Git index contain no unexpected changes.
3. Record the exact base SHA and the item branch.
4. Change only the current item.
5. Run focused tests during implementation.
6. Run the file-size and HIR guardrails before review.
7. Run `scripts/run_all_tests.sh --profile create-pr` before the draft PR.
8. Request one exact-SHA implementation review through the phase-closure loop.
9. Apply valid blocking findings in one batch.
10. Run the affected tests again after implementation changes.
11. Run `scripts/run_all_tests.sh` once on the final candidate.
12. Merge only after validation and review cover the same candidate SHA.
13. Record the PR, merge SHA, validation, review, and deferred work here.
14. Stop the session after the item record is complete.

Documentation-only planning changes use documentation checks and
`git diff --check`. They do not run compiler gates.

Before a long Cargo gate, inspect free disk space and the private target size.
Apply the cleanup rule from `AGENTS.md` only to the current worktree.

## Sequential Item Plan

### Item 0: Canonical Contract and Inventory Lock

ID: `pre_v1_compat_0_contract`

Purpose: Define every canonical contract before the first deletion.

Scope:

- Add a checked-in inventory for every live compatibility surface.
- Record the old surface, canonical replacement, behavior differences,
  consumers, owner item, and retained or removed status.
- Resolve conflicts between Phase 7 naming and current architecture docs.
- Record the receiver contract for `self`, `mut self`, `own self`, and
  `own mut self`.
- Record the exact retained external-compatibility list.
- Classify Phase 40 `legacy-index` options as current release contracts.
- Classify `RuleStatus::Deprecated` as current lint lifecycle metadata.
- Record current counts for forbidden names, wrappers, and schemas.
- Add an inventory checker with a self-test under the verification tree.

Required stdlib classifications:

- `canonical`: one supported public surface.
- `merge-then-remove`: the old surface has behavior that the canonical surface
  must gain before removal.
- `remove`: an exact alias or obsolete helper.
- `distinct`: two operations have different approved semantics.

Acceptance criteria:

- [x] Every audited surface has one classification.
- [x] Every removal row names exactly one implementation item.
- [x] No row uses `unknown`, `later`, or an unowned follow-up.
- [x] The inventory distinguishes public exports from private implementation
      imports.
- [x] The receiver contract has no source-compatibility interpretation.
- [x] The retained list includes DLPack and LSP protocol requirements.
- [x] The retained list names the Phase 40 and lint lifecycle owners.
- [x] The checker fails for an unowned compatibility row.
- [x] The checker self-test proves each rejection class.

Validation:

- `git diff --check`
- The new inventory checker
- The inventory checker self-test
- Documentation structure checks

### Item 1: Remove the Public `bigint` Transition

ID: `pre_v1_compat_1_bigint`

Purpose: Make canonical `int` the only exact source-level integer type.

Scope:

- Remove `bigint` from annotation and builtin-name resolution.
- Remove `Type::BigInt` and all mixed `int` or `bigint` branches.
- Remove transition conversions and representation-specific code generation.
- Remove `SIFR-INT-0011` and transition-only `SIFR-TYPE-0006` behavior.
- Rewrite useful fixtures to canonical `int` or explicit fixed-width types.
- Remove the bigint quarantine file after its final consumer disappears.
- Update user docs, architecture docs, diagnostic docs, snapshots, and demos.
- Preserve the arbitrary-precision storage that canonical `int` requires.

Acceptance criteria:

- [x] User source cannot resolve `bigint` as a type or runtime type object.
- [x] `Type::BigInt` has no production or test occurrence.
- [x] Transition-only diagnostics and docs are absent.
- [x] Exact `int` arithmetic keeps its documented range and safety behavior.
- [x] Fixed-width conversions keep explicit checked boundaries.
- [x] No bigint transition fixture or quarantine entry remains.

Focused validation:

- Type-system tests for exact integers and fixed-width conversions
- Frontend warning and diagnostic tests
- Codegen tests for exact integers
- Integer-model e2e suites
- Generated-code panic scans

### Item 2: Canonicalize Numeric and Random Stdlib Names

ID: `pre_v1_compat_2_numeric_random_names`

Purpose: Keep one public name for each numeric and random operation.

Scope:

- Apply the Item 0 decision for `abs_val`, `fabs`, `pow_val`, and `pow`.
- Apply the Item 0 decision for other imported math names with public wrappers.
- Canonicalize `random_int` or `randint`.
- Canonicalize `random_float` or `random`.
- Canonicalize `random_uniform` or `uniform`.
- If the old export remains live, canonicalize `random_choice` or `choice`.
- Preserve module-state behavior through the canonical random API.
- Migrate all docs, demos, fixtures, snapshots, and audit sources.
- Delete `demos/stdlib_aliases/` and its `stdlib_naming` reference.
- Remove old exports and their compiler manifest entries.

Acceptance criteria:

- [x] Each numeric operation has one public name.
- [x] Each random operation has one public name.
- [x] Removed random calls cannot bypass the canonical module state.
- [x] The public export inventory contains no removed name.
- [x] Repository Sifr sources use only canonical names.

Focused validation:

- Math and random stdlib unit tests
- Stateful random e2e suites
- Stdlib export and bootstrap tests
- Stdlib parity area suites

### Item 3: Canonicalize Platform, Time, System, Environment, and Date Names

ID: `pre_v1_compat_3_runtime_names`

Purpose: Remove intrinsic-shaped public names from runtime-information modules.

Scope:

- Canonicalize `platform_*` and CPython-style platform wrappers.
- Canonicalize `time_now`, `time_format`, `time`, and `strftime`.
- Canonicalize `get_args`, `sys_*`, and their public `sifr.sys` wrappers.
- Canonicalize `env_get`, `env_set`, `env_unset`, and public environment APIs.
- Canonicalize `UTC` and `utc`.
- Migrate repository consumers and documentation.
- Keep private `_sifr.*` intrinsic names only behind private aliases.
- Remove public re-exports of implementation names.

Acceptance criteria:

- [x] Each operation has one public name.
- [x] Private intrinsic names are not importable from public modules.
- [x] Environment default behavior remains explicit and typed.
- [x] Date and timezone APIs expose one UTC value.
- [x] Repository Sifr sources use only canonical names.

Focused validation:

- Platform, time, sys, env, and datetime e2e suites
- Public stdlib export tests
- Installed sysroot recertification

### Item 4: Canonicalize Text and Structured-Data Names

ID: `pre_v1_compat_4_text_data_names`

Purpose: Remove duplicate names without removing broader old behavior.

Scope:

- Canonicalize `re_*` helpers and public regex operations.
- Keep an architecture-approved name for keyword-conflicting regex operations.
- Canonicalize `json_loads` and `loads`.
- Move supported `json_dumps` input behavior into the canonical JSON surface.
- Remove `json_dumps_value*` aliases after canonical behavior is complete.
- Canonicalize `toml_loads` and `loads`.
- Canonicalize `base64_*` and `b64*` operations.
- Canonicalize `fnmatch_filter` and `filter`.
- Apply Item 0 decisions to `html_*`, `calendar_*`, URL helper pairs, and
  similar same-module duplicates.
- Migrate all repository consumers before each old export is removed.

Acceptance criteria:

- [x] Each operation has one public name and one documented signature.
- [x] Canonical JSON serialization preserves every approved input type.
- [x] Regex keyword conflicts have one documented Sifr spelling.
- [x] No old name remains through a private import re-export.
- [x] Public docs and examples use canonical names only.

Focused validation:

- Regex, JSON, TOML, Base64, fnmatch, HTML, calendar, and URL e2e suites
- Parse-safety and integer-boundary suites
- Stdlib parity area suites
- Public export inventory checks

### Item 5: Remove Binary and Hashing Compatibility Helpers

ID: `pre_v1_compat_5_binary_hashing`

Purpose: Make first-class `bytes` and bytes-native hashing authoritative.

Scope:

- Move approved `sifr.bytes` behavior to first-class `bytes` methods or
  canonical non-transition helpers.
- Remove wrappers that duplicate `bytes.decode`, `bytes.from_hex`,
  `bytes.from_ints`, and `bytes(size)`.
- Decide the final owner for byte search and prefix operations.
- Remove legacy `list[int]` conversion paths outside explicit construction.
- Remove `HashObject.update(str)`.
- Remove `hashlib.new(str)` and other string compatibility entrypoints.
- Keep bytes-native hashing and explicit text encoding.
- Migrate all demos, fixtures, and stdlib consumers.
- Remove private intrinsics that become unreachable.

Acceptance criteria:

- [x] Typed binary APIs use `bytes` for storage and transport.
- [x] Text reaches hashing only through explicit encoding.
- [x] No compatibility wrapper duplicates a first-class `bytes` operation.
- [x] No typed binary API accepts `list[int]` as its canonical input.
- [x] Private binary intrinsics have at least one canonical live consumer.

Focused validation:

- Bytes type-system and codegen tests
- Binary file and conversion e2e suites
- Hashlib and crypto e2e suites
- Stdlib native-adapter reachability checks
- Generated-code panic scans

### Item 6: Remove Collection and Sorted-Insert Compatibility APIs

ID: `pre_v1_compat_6_collections`

Purpose: Make first-class generic collections and mutating algorithms canonical.

Scope:

- Remove `new_set`, `set_from_list`, `set_add`, `set_contains`, `set_remove`,
  `set_len`, `set_union`, and `set_intersection`.
- Migrate consumers to first-class generic set operations.
- Remove list-backed set intrinsics that become unreachable.
- Remove `heapq` copy-returning compatibility helpers.
- Remove `bisect` copy-returning compatibility helpers.
- Keep mutating `heapq` and `bisect` operations.
- Review private max-heap helpers separately as CPython parity, not old Sifr
  source compatibility.
- Migrate demos, fixtures, docs, and snapshots in the same PR.

Acceptance criteria:

- [x] Public set operations use first-class `set[T]`.
- [x] No list-backed set compatibility helper remains.
- [x] `heapq` exposes one mutation model.
- [x] `bisect` exposes one insertion model.
- [x] Removed intrinsics are absent from manifests and codegen registries.

Focused validation:

- Generic collection lowering and codegen tests
- Collections, heapq, and bisect e2e suites
- Algorithmic compatibility representative suites
- Stdlib manifest and native-adapter checks

### Item 7: Remove Remaining Public Alias Residue

ID: `pre_v1_compat_7_stdlib_residue`

Purpose: Close every stdlib row that earlier items did not own.

Scope:

- Recompute the public export inventory from compiled stdlib sources.
- Remove same-operation aliases found after Items 2 through 6.
- Resolve URL `parse` or `parse_url` and `build` or `build_url` pairs.
- Apply architecture-approved names for Rust and Sifr keyword conflicts.
- Remove aliases that enter public modules through unrenamed `_sifr` imports.
- Add a guard that rejects a new public implementation-name re-export.
- Mark approved distinct operations with explicit semantic evidence.

Acceptance criteria:

- [x] The Item 0 stdlib inventory has no open row.
- [x] Each same-operation public export group has one member.
- [x] Each approved distinct pair has different tested semantics.
- [x] The export guard detects direct and imported aliases.
- [x] Public docs match the compiled export inventory.

Focused validation:

- Complete stdlib export inventory
- Stdlib bootstrap and installed-sysroot tests
- Full stdlib parity area
- Export guard self-test

### Item 8: Enforce Explicit Receiver Semantics

ID: `pre_v1_compat_8_receivers`

Purpose: Remove receiver interpretation that exists only for source compatibility.

Dependency: Item 0 must contain the approved receiver contract.

Scope:

- If syntax omits the required receiver convention, remove inferred mutable
  `self` acceptance.
- Remove the compatibility mapping from `own self` to `SharedBorrow`.
- Remove the compatibility mapping from `own mut self` to `MutableBorrow`.
- Implement the approved owned-receiver meaning or reject unsupported forms.
- Require explicit receiver syntax in protocols and implementations.
- Update stdlib classes, demos, fixtures, algorithmic corpora, and docs.
- Classify `report_legacy_name_conflicts` as current ownership behavior.
- Rename it to describe same-binding conflicts without the `legacy` term.
- Keep compiler-synthesized constructor storage rules separate.

Acceptance criteria:

- [x] Receiver syntax maps directly to one receiver convention.
- [x] Method-body analysis does not change the declared convention.
- [x] Mutating a shared receiver produces one source diagnostic.
- [x] Protocol conformance compares explicit receiver conventions.
- [x] Owned receiver behavior has native runtime coverage.
- [x] Repository Sifr sources use explicit canonical receiver syntax.
- [x] Current same-binding conflict reporting has no compatibility name.

Focused validation:

- Parser and AST receiver tests
- Lowering receiver-analysis and place tests
- Protocol conformance tests
- Codegen receiver tests
- Class and ownership e2e suites
- Complete algorithmic compatibility lane

### Item 9: Canonicalize the Package Manifest and Source Layout

ID: `pre_v1_compat_9_package_layout`

Purpose: Keep one package manifest schema and one single-root layout.

Scope:

- Keep `[source].root` as the only source-root key.
- Keep one relative source root and use `src/` as its default.
- Remove `[source].roots` from the package and driver readers.
- Remove the package `sifr/` default and the driver `.` default.
- Replace driver source-root collections with one source root.
- Remove driver multi-root resolution and its ambiguity messages.
- Remove the schema selection that depends on the presence of `roots`.
- Reject `[exports].modules` and `[[bin]]` in all package manifests.
- Remove `parse_exports`, the manifest `exports` field, and
  `validate_exports_match_sources`.
- Derive the canonical `ImportRoot` value from the package name only.
- Retain `ImportRoot` as an internal namespace type, not a manifest reader.
- Remove the dedicated not-production diagnostics for old manifest tables.
- Use the normal unsupported-field diagnostic for those tables.
- Remove the related `SIFR-PACKAGE-0701` and `SIFR-PACKAGE-0711` docs,
  fixtures, registry rows, and generated catalog data.
- Retain `SIFR-WORKSPACE-0001` through `SIFR-WORKSPACE-0004`.
- Update these codes for one `[source].root` and its validation errors.
- Migrate package and project-workspace fixtures to `src/__init__.sifr`.
- Migrate driver workspace tests to the single-root schema and default.
- Remove `test_package_cli_check_explicit_file_falls_back_for_legacy_workspace_manifest`.
- Update package tests, snapshots, examples, and public documentation.
- Remove the Layout Migration section from `docs/package_management.md`.
- Do not add a manifest converter, source-root fallback, or warning period.

Acceptance criteria:

- [x] Every package and workspace reader accepts `[source].root` only.
- [x] Each reader accepts at most one relative source root.
- [x] Each reader defaults to `src/` when `[source]` is absent.
- [x] No crate reads `[source].roots` or handles multiple source roots.
- [x] The manifest parser rejects `[exports].modules` and `[[bin]]`.
- [x] Import roots come from canonical package names only.
- [x] Workspace source-root diagnostics describe `[source].root` only.
- [x] All repository package fixtures use the canonical source layout.
- [x] Public package documentation describes one single-root layout.

Focused validation:

- `sifr_package` manifest and source-map tests
- `sifr_driver` workspace, discovery, and diagnostic tests
- CLI mode-resolution tests
- Package public-API tests
- Project-workspace verification suites
- Package-management documentation checks

### Item 10: Remove the Flat Installation Layout

ID: `pre_v1_compat_10_install_layout`

Purpose: Keep one toolchain-root installation layout.

Scope:

- Keep `<sysroot>/bin/sifr` as the canonical binary location.
- Remove acceptance of `<sysroot>/sifr`.
- Remove flat-layout receipt validation and tests.
- Remove flat-layout installer branches and documentation.
- Make old flat receipts fail through the normal unmanaged-install diagnostic.
- Keep current release metadata strict. Do not add a receipt converter.

Acceptance criteria:

- [x] Install, update, and uninstall use one binary layout.
- [x] Self-update accepts one binary and sysroot relationship.
- [x] Public installation docs show one layout.
- [x] No migration or conversion path exists for a flat receipt.
- [x] Release archive and installer tests use the canonical layout.

Focused validation:

- Self-update receipt tests
- Distribution-release installer suites
- Sysroot-release suites
- Public installation documentation checks

### Item 11: Remove Legacy Diagnostics and Rejection Residue

ID: `pre_v1_compat_11_diagnostics`

Purpose: Keep one diagnostic family for each current compiler error.

Scope:

- Remove `SIFR-WORKSPACE-0101` through `SIFR-WORKSPACE-0104`.
- Migrate production project emitters to canonical `SIFR-IMPORT-*` codes.
- Replace `ResolutionError::to_diagnostic` with a canonical span-less builder.
- Use an `<unknown>` primary location for a span-less root-module error.
- Keep resolution-scope, tried-path, and candidate-path notes.
- Emit `SIFR-IMPORT-0007` for span-less project import cycles.
- Remove the old production constants and rendering harness rows.
- Remove old diagnostic docs and index entries.
- Update `workspace_unresolved_import` and `workspace_ambiguous_import`.
- Update root-module, namespace-collision, and project-cycle fixtures.
- Remove migration-only stdlib module replacement tables.
- Replace old Rust interop key recognition with canonical decorator validation.
- Remove `diagnostic_legacy_display` and `legacy_diagnostic_display` helpers.
- Replace old-shape fixtures with generic unsupported-form fixtures where useful.

Acceptance criteria:

- [x] Each active compiler error has one diagnostic code.
- [x] Project root-module and cycle errors emit `SIFR-IMPORT-*` only.
- [x] Span-less import diagnostics use canonical codes and `<unknown>`.
- [x] No active diagnostic title or message describes a Sifr form as legacy.
- [x] Old stdlib names do not receive a migration suggestion table.
- [x] Old Rust decorator keys do not use a special parser branch.
- [x] Test helpers render diagnostics through the production renderer.
- [x] Diagnostic catalogs and public docs contain no removed code.

Focused validation:

- Diagnostic registry and catalog checks
- Import-resolution tests
- Project-workspace root-module and cycle tests
- Rust interop declaration tests
- CLI rendering and `--explain` tests
- Documentation error-code link checks

### Item 12: Remove Hidden Compatibility-Name Recognition

ID: `pre_v1_compat_12_hidden_names`

Purpose: Remove dead lowering acceptance for hidden alias names.

Scope:

- Remove `__compat_sifr_sync_` prefix stripping.
- Remove `__compat_sifr_concurrent_` prefix stripping.
- Remove related tests, fixtures, and comments.
- Search lowering and codegen for all remaining `__compat_*` identifiers.
- If a live internal representation has canonical behavior, rename it.
- Add a production-tree guard for removed hidden prefixes.

Acceptance criteria:

- [x] Lowering does not strip or recognize hidden compatibility prefixes.
- [x] No compiler-generated source emits a removed prefix.
- [x] The guard excludes archives and rejects production or fixture residue.
- [x] Canonical task and blocking APIs retain their behavior.

Focused validation:

- Task-scope lowering tests
- Blocking-executor lowering tests
- Async and runtime-platform e2e suites
- Hidden-prefix guard self-test

### Item 13: Canonicalize the Verification Runner

ID: `pre_v1_compat_13_verification`

Purpose: Keep one profile schema and one execution authority.

Dependency: The active Phase 40 owner must release or coordinate every shared
profile, runner, workflow, and release-governance path before this item starts.

Scope:

- Make profile schema version 2 the only accepted profile version.
- Remove `legacy_facade` from profile schemas and profile files.
- Make `selected_areas`, `toolchain_steps`, and `guardrail_steps` authoritative.
- Remove `legacy-facade` execution mode and its default.
- Remove duplicate profile-plan fields and schema-version 1 output.
- Remove `--hardening-summary` from the common adapter and area runners.
- Make structured result JSON the only report input.
- Update workflows, release governance, docs, self-tests, and fixtures.
- Remove legacy report-line parsers after all producers are gone.
- Remove `extract_expect_stdout` and `# expect-stdout` recognition.
- Convert the LeetCode audit generator to assertion-based runtime checks.
- Remove harness self-tests for the old directive.
- Update the e2e harness description in `internal_docs/architecture.md`.

Acceptance criteria:

- [ ] The profile loader accepts schema version 2 only.
- [ ] Profiles contain no `legacy_facade` key.
- [ ] One field selects each verification area and suite.
- [ ] Area runners emit one structured result format.
- [ ] No report parser consumes a legacy summary line.
- [ ] No harness code or generator produces or accepts `# expect-stdout`.
- [ ] E2E runtime expectations use Sifr `assert` statements only.
- [ ] Create-PR, merge, nightly, and release profiles preserve their coverage.
- [ ] Phase 40 release qualification uses the canonical model.

Focused validation:

- Verification runner self-tests
- Profile schema and assignment checks
- Every area adapter self-test
- E2E harness behavior and dependency-plan tests
- Algorithmic-compatibility LeetCode audit tests
- Create-PR and merge profile dry plans
- Full create-PR and merge gates

### Item 14: Remove Obsolete Source and Package API Wrappers

ID: `pre_v1_compat_14_source_api`

Purpose: Keep provider-backed source access and structured package results.

Scope:

- Inventory all wrappers that construct `DiskSourceProvider` internally.
- Move disk construction to approved CLI or session composition roots.
- Make frontend, formatter, linter, and package APIs receive a provider or
  captured session.
- Migrate all pre-session callers.
- Remove paired `foo` and `foo_with_provider` APIs after caller migration.
- Remove `PackageSourceMap::resolve_import`.
- Migrate package tests to `resolve_import_result` and its result variants.
- Preserve ambiguity, access, unresolved, and fatal states without collapsing.
- Update `internal_docs/typescript_go_architecture_transfer_source_provider.md`.
- Keep `DiskSourceProvider` as the canonical disk implementation.
- Add a dependency-direction guard for provider construction.

Acceptance criteria:

- [ ] Lower compiler layers do not construct `DiskSourceProvider`.
- [ ] Each source operation records dependencies through the active provider.
- [ ] Formatter, linter, package, CLI, and LSP paths share provider semantics.
- [ ] No compatibility wrapper remains for a pre-session caller.
- [ ] No test-only wrapper collapses package resolution result variants.
- [ ] Package tests use the production structured-result API.
- [ ] Overlay and snapshot behavior remains deterministic.

Focused validation:

- Source-provider unit tests
- Frontend mode-parity suites
- Formatter and linter path tests
- Package source-map and offline tests
- Package public-API tests
- LSP snapshot and stale-result suites
- Source-crate dependency-direction guard

### Item 15: Remove String-Based Rust Type Rendering

ID: `pre_v1_compat_15_rust_type`

Purpose: Make structured `RustType` the only codegen type representation.

Scope:

- Complete `sifr_type_to_rust_type` for every supported `Type` variant.
- Add structured field-type conversion where field rules differ.
- Replace every production `.rust_type()` call.
- Remove `Type::rust_type()` and `Type::rust_type_for_struct_field()`.
- Remove `RustType::Named(ty.rust_type())` fallback branches.
- Replace fallback panics with structured codegen errors at approved boundaries.
- Update snapshots, emitted Rust fixtures, and codegen statistics.
- Add a guard that rejects string-based type rendering.

Acceptance criteria:

- [ ] Production code contains no `.rust_type()` call.
- [ ] Codegen has a total structured mapping for supported types.
- [ ] Unsupported types produce one structured compiler error.
- [ ] Rust imports and generic nesting come from structured nodes.
- [ ] Generated Rust snapshots preserve required output semantics.
- [ ] The guard detects direct and indirect string-renderer restoration.

Focused validation:

- Type-system query tests
- Complete codegen unit suite
- Generated Rust snapshot tests
- E2E pass suite
- Generated-code quality suites
- Codegen raw-code and structured-lowering guards

### Item 16: Final No-Compatibility Guard and Phase Closure

ID: `pre_v1_compat_16_closure`

Purpose: Prove the final repository has no unowned Sifr compatibility path.

Scope:

- Regenerate the Item 0 inventory from the final tree.
- Remove temporary removal rows and keep the retained external list.
- Add one executable guard for forbidden production residue.
- Scan source, stdlib, verification, workflows, docs, demos, and fixtures.
- Exclude historical archives, vendored code, generated files, and external
  protocol terms from false-positive scans.
- Exclude the Phase 40 `legacy-index` options and lint lifecycle metadata.
- Update `internal_docs/architecture.md` with the final canonical contracts.
- Update public docs for all breaking pre-v1 changes.
- Record all item PRs, SHAs, validation, and review evidence.
- Run the phase closure checks without a new broad implementation review.

Required forbidden classes:

- public `bigint` type support,
- public stdlib same-operation aliases,
- list-backed set compatibility helpers,
- copy-returning heapq and bisect compatibility helpers,
- Sifr-owned legacy schema readers,
- `[source].roots` readers, multi-root handling, and non-`src/` defaults,
- manifest-level export and binary-table readers,
- `legacy_facade` and legacy summary lines,
- legacy `# expect-stdout` harness expectations,
- `__compat_sifr_*` recognition,
- legacy workspace diagnostic codes,
- pre-session source wrappers,
- result-collapsing package API wrappers that only tests use,
- string-based Rust type rendering,
- flat installation layout support.

Acceptance criteria:

- [ ] Every removal row from Item 0 is closed.
- [ ] Every retained row names an external or current product contract.
- [ ] The final guard passes and its self-test rejects each forbidden class.
- [ ] The complete create-PR profile passes.
- [ ] The complete merge gate passes once on the final candidate.
- [ ] Item-level exact-SHA review evidence covers all implementation changes.
- [ ] Architecture and public docs match the final source tree.
- [ ] This phase document records the final handoff and closure state.

Closure validation:

- `scripts/run_all_tests.sh --profile create-pr`
- `scripts/run_all_tests.sh`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `python3 scripts/check_file_size_guardrails.py`
- The final no-compatibility guard and self-test
- Documentation structure and error-code link checks

## Dependency Sequence

```text
Item 0  Canonical contract and inventory
  -> Item 1  bigint removal
  -> Item 2  numeric and random names
  -> Item 3  runtime-information names
  -> Item 4  text and structured-data names
  -> Item 5  binary and hashing helpers
  -> Item 6  collection helpers
  -> Item 7  remaining stdlib residue
  -> Item 8  receiver semantics
  -> Item 9  package manifest and source layout
  -> Item 10 installation layout
  -> Item 11 diagnostics and rejection residue
  -> Item 12 hidden compatibility names
  -> Item 13 verification runner and e2e expectations
  -> Item 14 source and package API wrappers
  -> Item 15 structured Rust types
  -> Item 16 final guard and closure
```

The sequence is strict. Item 13 also needs an ownership handoff from the active
Phase 40 work because both phases touch verification and release paths.

## Validation Ownership Matrix

| Changed surface | Minimum focused owner |
| --- | --- |
| Parser, type system, or lowering | Core language area plus affected crate tests |
| Code generation | `sifr_codegen`, generated-code quality, and affected e2e suites |
| Public stdlib source | Stdlib parity, bootstrap, installed sysroot, and affected e2e suites |
| Package manifest or source layout | `sifr_package` plus project-workspace suites |
| Verification profiles or runner | Runner self-tests and all changed profile plans |
| E2E runtime expectations | Harness tests plus affected corpus generators |
| Installation or release | Distribution-release and sysroot-release areas |
| Diagnostics | Diagnostics area, docs links, CLI rendering, and affected source fixtures |
| Source provider | Frontend parity, tooling, package, and LSP snapshot suites |
| Documentation only | Documentation checks and `git diff --check` |

The current item owns regressions that its changes cause. Pre-existing or
external errors remain with their existing owner.

## Review Contract

Each implementation PR receives one exact-SHA review through the required
phase-closure workflow.

The review request must contain:

- the exact base and candidate SHAs,
- all changed paths,
- the current item scope,
- the current item acceptance criteria,
- focused and gate validation evidence,
- prior blocking findings for a remediation review.

Only an in-scope omission or regression can block the item. Suggestions and
pre-existing problems become separate follow-up work.

If the same blocking finding returns twice, stop and request adjudication. If a
second review finds a new mechanism defect, stop and revise the item scope.

## Risk Controls

### Broader behavior under an old alias

An old alias can accept more input types than its proposed replacement.
Item 0 records that difference. The owning item moves approved behavior before
it removes the alias.

### Public leakage from private imports

A public module can re-export an unrenamed `_sifr` import. Export inventory
checks use compiled module metadata, not text search alone.

### External compatibility false positives

Terms such as `legacy` and `fallback` also describe external protocols and
normal product behavior. Item 0 records retained rows with exact owners.

### Active Phase 40 overlap

Phase 40 currently owns release profiles and qualification paths. Item 13 does
not start without an ownership handoff or an approved coordination record.

### Large internal migrations

Source-provider and Rust-type migrations touch many callers. Each item keeps
one responsibility and uses focused compilation throughout implementation.

### Repository corpus drift

Removed source forms can remain in demos or external corpora. Each removal item
migrates its repository consumers before the old path disappears.

## Progress Ledger

| Item | Status | PR | Merge SHA | Validation | Review | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 0. Canonical contract and inventory | merged | [#3237](https://github.com/sifr-lang/sifr/pull/3237) | `4234ced2d405809b8523315b619ceff48c23132e` | Candidate `b518c4cfabbf1b7c5dd2bd8772cdb8aa52228f69`: inventory checker and mutation self-test, documentation structure, file-size and HIR guardrails, Python compile, and diff hygiene passed; no compiler files changed, so Sifr gates were omitted. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3237#issuecomment-5325904790): SATISFIED, no blocking findings. | 61 classified surfaces, 42 owned removals, 13 explicitly retained external/current contracts, and 12 reproducible baseline counts locked. |
| 1. Public `bigint` transition | merged | [#3242](https://github.com/sifr-lang/sifr/pull/3242) | `54a1af60896ec216a5ce1838b50bf1cdd3ebb607` | Candidate `87b9e0ea7ee7ccb49a6a064c07491c1a425e94da`: type-system, lowering, codegen, runtime-int, driver, exact-int, heapq, fixed-width, diagnostics, generated-code panic, formatting, file-size, HIR, and residue checks passed. The one create-PR and one merge gate both stopped on the same verification-taxonomy failure reproduced at base `5285d7cb6d8df3338d2964f5af4308c94b9a3f48`; Item 1 changed none of the reported files. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3242#issuecomment-5326605073): SATISFIED, no blocking findings. | Removed the public transition type and diagnostics while preserving arbitrary-precision `int`; recorded inherited integer-model and verification follow-ups below. |
| 2. Numeric and random names | merged | [#3246](https://github.com/sifr-lang/sifr/pull/3246) | `b5c90b9c56a3fc5e0dfd067d1b4d356fbc4db8a0` | Initial candidate `3008253603e2d68fbbe806b990d8daeebb51749f`: driver stdlib (71 passed, 2 ignored), lowering (986 passed, 1 ignored), codegen (1021 passed), stdlib, canonical export, stateful random, math/random e2e, parity, demo, inventory, formatting, file-size, HIR, and residue checks passed. The pass suite completed 684 of 685 fixtures; its host-timing cleanup failure passed in immediate isolation. A separate pre-existing protocol diagnostic/CFG fail fixture remained out of scope. The one create-PR gate stopped at inherited verification-taxonomy failures. After demo-only remediation, four emitted artifacts byte-matched live emission, all affected Sifr and idiomatic demos ran, and the one merge gate on final candidate `bbeee29689ddf4f5ed2c3c0f84955992c1e8f46e` stopped at the same inherited taxonomy failure; neither gate was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3246#issuecomment-5327168534): NOT SATISFIED for stale demo companions; [one remediation review](https://github.com/sifr-lang/sifr/pull/3246#issuecomment-5327211679): SATISFIED, no blocking findings. | Removed duplicate numeric/random public names, preserved stateful canonical random behavior, taught bootstrap/lowering/codegen to honor private function aliases, migrated consumers, and deleted the alias demo. |
| 3. Runtime-information names | merged | [#3250](https://github.com/sifr-lang/sifr/pull/3250) | `c96d0b5048cd4f9916a2bc6995f6fd356ddbdbf0` | Candidate `b15f3b443ae3252a8ed7c505eff789672d57e372`: focused compiled-export coverage, 11 affected e2e fixtures, sys/process runtime checks, six regenerated Sifr demos, eight idiomatic companions, full stdlib parity and self-test, driver stdlib (72 passed, 2 ignored), `sifr_stdlib` (56 passed), `sifr_sysroot` (11 passed), stdlib manifest (39 passed), installed/source sysroot equivalence, broad CLI tests, formatting, file-size, HIR, inventory, and residue checks passed. The one create-PR and one merge gate stopped at the inherited verification-taxonomy failure assigned to Item 16; neither gate was repeated. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3250#issuecomment-5327677808): SATISFIED, no blocking findings. | Removed intrinsic-shaped runtime-information exports, kept private aliases behind canonical wrappers, preserved typed environment defaults, migrated consumers, and removed the duplicate UTC constructor. |
| 4. Text and structured-data names | merged | [#3254](https://github.com/sifr-lang/sifr/pull/3254) | `4bdb89b741a4d6cfbf8a4bcc3b8bb6d676d75501` | Candidate `b98864c82c61f42909b3395aae7714cc53215937`: 27 focused e2e fixtures, affected Sifr and idiomatic demos, fresh generated baselines, codegen filter shadowing, driver stdlib (73 passed, 2 ignored), codegen (1023 passed), `sifr_stdlib` (56 passed), `sifr_sysroot` (11 passed), stdlib manifest (39 passed), broad CLI tests, installed/source sysroot equivalence, formatting, file-size, HIR, compatibility inventory, full module parity, and complexity checks passed. The aggregate parity runner's two failures were inherited bare-namespace and Python import-path issues. The one create-PR and one merge gate stopped at the inherited verification-taxonomy failure assigned to Item 16; neither gate was repeated. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3254#issuecomment-5328750688): SATISFIED, no blocking findings. | Removed the duplicate text/data names, preserved the broad canonical JSON input contract, kept regex keyword handling under `Pattern.is_match`, privatized implementation imports, migrated consumers, and fixed imported/user-defined `filter` shadowing. |
| 5. Binary and hashing helpers | merged | [#3260](https://github.com/sifr-lang/sifr/pull/3260) | `55ba0d3855eac637106ffb02df8455d0aa764f95` | Initial candidate `3d34ad6289096f7588a75dda8689eb19b4f1ad62`: focused bytes/hash/gzip e2e, module parity, codegen (1023 passed), lowering (986 passed, 1 ignored), driver stdlib (73 passed, 2 ignored), stdlib/manifest suites, broad non-e2e CLI tests, installed/source sysroot equivalence, generated-code panic and demo suites, native-adapter reachability, formatting, Clippy, file-size, HIR, compatibility inventory, and residue checks passed. Remediation candidate `0d6b6519806afc86fd7c57d9b1472d96ff65db88`: codegen (1023 passed), driver stdlib (73 passed, 2 ignored), effectful-receiver and in-memory-stream native runs, strict emitted-output freshness, all generated-code panic variants, formatting, Clippy, file-size, HIR, driver maintainability, manifest, and inventory guards passed. The one create-PR profile and one merge gate each stopped at the inherited verification-taxonomy failure plus the newly surfaced stale `sifr_stdlib/bytes` coverage classification assigned to Item 7; neither gate was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3260#issuecomment-5330429482): NOT SATISFIED for repeated bytes-receiver evaluation and list-backed `BytesIO`; [one remediation review](https://github.com/sifr-lang/sifr/pull/3260#issuecomment-5330822648): SATISFIED, both blockers fixed and no new blocking defect. | Removed the `sifr.bytes` transition module and stale string-hash paths, made first-class bytes operations and bytes-native hashing canonical, migrated gzip and `BytesIO` to bytes storage, evaluated bytes-method receivers once, migrated all consumers, and removed unreachable private intrinsics. |
| 6. Collection helpers | merged | [#3264](https://github.com/sifr-lang/sifr/pull/3264) | `a7e62e19e725c81be61f72c9d62efce1b5c09278` | Candidate `9d2cecd719dd619f172aa3bddfce5cfc2ad1367c`: codegen (1,025 passed), driver stdlib (73 passed, 2 ignored), stdlib manifest (30 unit and 10 integration passed), seven focused native collection/bisect/heapq fixtures, private max-heap parity evidence, 12 algorithmic representatives, affected Sifr and idiomatic demos, nine byte-exact generated companions, module inventory, complexity/resource inventory, compatibility inventory self-test, formatting, Clippy on the identical pre-rebase Item 6 diff, file-size, HIR, and diff hygiene passed. A broad CLI run passed 109 unit tests before the inherited protocol diagnostic/CFG fail-fixture defect. The one create-PR profile and one merge gate on the exact candidate both stopped at the inherited taxonomy conflict, the Item 7-owned stale `sifr_stdlib/bytes` feature row, and concurrent static-class-adapter taxonomy matches; the removed `collections` feature was no longer reported, and neither gate was repeated. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3264#issuecomment-5331452183): SATISFIED, no blocking findings. | Replaced copied `list[int]` set helpers with first-class generic sets, removed the private Rust adapter and Cargo feature, kept one mutating heapq/bisect model, retained separately tested CPython max-heap helpers, and migrated all current consumers and records. |
| 7. Remaining stdlib residue | merged | [#3270](https://github.com/sifr-lang/sifr/pull/3270) | `1f29687f590eee416a70b03e331e98324fbbf4eb` | Final candidate `2a1eee336ac531f8ab566499a940aede461e5ac7`: codegen (1,027 passed), frontend/type-system/lowering, driver stdlib and private re-export suites, installed/source sysroot checks, focused bytes/bisect/fnmatch/statistics/regex/time/stream/hash runs, module parity, export inventory, all 246 generated demo companions, formatting, Clippy, file-size, HIR, manifest, coverage, and native-adapter checks passed. Full parity also passed complexity and all 411 algorithmic fixtures; its remaining package-context and audit-import failures are assigned to Item 13. The one create-PR profile on reviewed candidate `ed8e173c76f7b21e95d2d47d1964441a69a95146` and one merge gate on the final candidate stopped at the governed verification-taxonomy conflict; neither gate was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3270#issuecomment-5333132488): NOT SATISFIED for an unregistered freshness guard and stale current paths; [one remediation review](https://github.com/sifr-lang/sifr/pull/3270#issuecomment-5333223660): NOT SATISFIED for further instances of the same stale-path blocker, which were corrected mechanically without a prohibited third review. Its two newly identified freshness-runner mechanisms are recorded for Item 13. | Removed the remaining same-operation aliases and dead intrinsics, centralized private/export policy, hardened cross-module private aliases and collision handling, fixed canonical `map` shadowing and stdlib resource ownership, published the compiled public API, reconciled all demo companions, and installed the standing freshness guard. |
| 8. Receiver semantics | merged | [#3279](https://github.com/sifr-lang/sifr/pull/3279) | `94c1dae1009f9419f2f2239cbe84f1438f7c85d1` | Candidate `8db8b12f24da88bb60e9178f70fefde43e631558`: parser/frontend/type-system, receiver lowering and place analysis, Rust/Python interop lowering, codegen (1,027 passed), protocol diagnostics, driver stdlib, manifest/native-adapter, sysroot, 186-fixture core-language audit, explicit owned-mutable native run, one-diagnostic shared-mutation rejection, all 411 algorithmic fixtures, generated docs, demo freshness, formatting, Clippy, file-size, HIR, inventory, and residue checks passed. Full e2e passed 685 of 686 fixtures; the sole generated-Rust scoping failure is recorded below. The one create-PR profile and one merge gate both stopped at the inherited verification-taxonomy conflict before compiler tests; neither gate was repeated. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3279#issuecomment-5334374077): SATISFIED, no blocking findings. | Made all four source receiver spellings explicit through HIR and Rust codegen, rejected body-inferred mutation and fixed-trait convention drift, migrated repository and companion corpus sources, and merged companion [leetcode #45](https://github.com/sifr-lang/leetcode/pull/45) at `ebd0f2194c0fcb950a3fad3ce1e5b41a8ac8e123`. |
| 9. Package manifest and source layout | merged | [#3283](https://github.com/sifr-lang/sifr/pull/3283) | `22f0b2c2f25c8c517f463c44f86f62b9e7cbdab9` | Final candidate `42167ac27d7c6167aed80726a202cbaf0022533f`: package (141 passed with the unavailable external demo checkout excluded), driver (525 passed before base integration), CLI mode resolution, package public API, project-workspace parity, graph isolation, 14 baseline variants, diagnostics, LSP smoke, docs generation, formatting, Clippy, file-size, HIR, and residue checks passed. Base integration preserved the concurrent structural-default work; its native package run and 25 affected driver tests passed. The one create-PR and one merge gate stopped at the inherited verification-taxonomy conflict; neither gate was repeated. | [Initial exact-SHA review](https://github.com/sifr-lang/sifr/pull/3283#issuecomment-5335116462) found one stale documentation cross-reference; [one remediation review](https://github.com/sifr-lang/sifr/pull/3283#issuecomment-5335027285) on `c85fd4552b553db053aa721d46649775eea28a33` was SATISFIED. The later base merge was tested and received no prohibited third review. | Kept one `src/`-default source root, removed manifest exports/bin readers and multi-root resolution, derived import roots from package names, migrated repository fixtures, and removed PACKAGE-0701/0711. The external `sifr-demo-json` checkout remained unavailable. |
| 10. Installation layout | merged | [#3285](https://github.com/sifr-lang/sifr/pull/3285) | `5c94968db425414808856ec521960da6580a538a` | Candidate `f99442a9cc5a19d4c92c4551eba9e06f28228a46`: self-update receipt (16 passed), self-update (54 passed), receipt rules, all 56 representative distribution-release variants, canonical installed-stdlib boundary, formatting, Clippy, file-size, HIR, docs, inventory, and residue checks passed. The broad CLI run had only the inherited protocol diagnostic mismatch. The sysroot boundary then hit the inherited external static-adapter dependency path. The one create-PR and one merge gate stopped at the inherited verification-taxonomy conflict; neither gate was repeated. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3285#issuecomment-5335392607): SATISFIED, no blocking findings. | Kept only `<sysroot>/bin/sifr`, removed flat receipt discovery and acceptance, rejected non-canonical installer relationships, and documented one install layout without a converter. |
| 11. Diagnostics and rejection residue | merged | [#3287](https://github.com/sifr-lang/sifr/pull/3287) | `e71174f31587b9f84d0051c8c0a9309c66b3a59c` | Final candidate `d9ecd3c4f7cbe353d13b4e970f0f199f99082eec`: diagnostics registry/docs rules, diagnostics crate, project root and cycle tests, workspace baselines, canonicalization contract and self-test, runtime-platform golden, Rust interop declarations, stdlib import policy, CLI rendering and explain, formatting, Clippy, file-size, HIR, inventory, docs, JSON, and residue checks passed. The e2e fail suite reached only the inherited protocol diagnostic mismatch; later changed fixtures passed directly. The one create-PR gate on initial candidate `2b1b4347edc5457747cae77f9ccc65deea5ddc70` and one merge gate on the final candidate stopped at the inherited verification-taxonomy conflict; neither gate was repeated. | [Initial and remediation exact-SHA Opus evidence](https://github.com/sifr-lang/sifr/pull/3287#issuecomment-5335839622): the initial review found two blockers; the one remediation review was SATISFIED with no blockers. | Removed workspace diagnostic codes 0101–0104 and IMPORT-0009, kept canonical span-less import diagnostics, removed migration suggestion tables and old Rust-key handling, migrated fixtures and catalogs, and removed the legacy render helpers. |
| 12. Hidden compatibility names | merged | [#3289](https://github.com/sifr-lang/sifr/pull/3289) | `e3d4bf656a6992cc712911723c215d409c26e175` | Candidate `a3f75bbf4a9a5a1705da392dcbeda9f84a0b9524`: 25 focused task, blocking, sendability, and IPC lowering tests; all 140 create-PR e2e fixtures; 11 runtime-platform golden cases with one capability-gated skip; all 14 developer-tooling static variants; Clippy, formatting, file-size, HIR, inventory, residue, and diff checks passed. The full lowering crate had only one inherited class-diagnostic assertion failure in an untouched path. The one create-PR and one merge gate stopped at the inherited verification-taxonomy conflict assigned to Item 16; neither gate was repeated. | [Exact-SHA Opus review](https://github.com/sifr-lang/sifr/pull/3289#issuecomment-5336012950): SATISFIED, no blocking findings. | Removed both hidden-prefix stripping mechanisms, made task, synchronization, IPC, and blocking checks use canonical names directly, and registered a production/fixture residue guard with a mutation self-test. |
| 13. Verification runner and e2e expectations | active | — | — | — | — | Phase 40 handoff required before implementation. |
| 14. Source and package API wrappers | pending | — | — | — | — | — |
| 15. Structured Rust types | pending | — | — | — | — | — |
| 16. Final guard and closure | pending | — | — | — | — | — |

### Deferred reviewer follow-up

| Source item | Finding | Owner | Disposition |
| --- | --- | --- | --- |
| Item 0 | Strengthen exact owner-token, placeholder, private-import classification, and narrow-path enforcement when the temporary inventory becomes the final no-compatibility guard. | `pre_v1_compat_16_closure` | Non-blocking checker hardening; do not alter the approved Item 0 candidate. |
| Item 0 | Align the internal `Owned` receiver vocabulary with the source-level `own self` / `own mut self` distinction during receiver implementation. | `pre_v1_compat_8_receivers` | Non-blocking terminology follow-up. |
| Item 0 | Wire the final compatibility guard into governed verification profiles. | `pre_v1_compat_16_closure` | Non-blocking CI integration follow-up. |
| Item 1 | Exact integers stored through container/codegen paths still have a pre-existing `SifrInt` versus `i64` representation gap. | `plans/phases/13_type_system_completion.md` integer-model owner | Out of scope for public-alias removal; preserve as integer-model hardening work. |
| Item 1 | Exact-integer exponentiation accepts a pre-existing literal exponent above `u32::MAX` and truncates it at the Rust cast. | `plans/phases/13_type_system_completion.md` integer-model owner | Out of scope for public-alias removal; add an explicit exponent-bound diagnostic in integer-model hardening work. |
| Item 1 | The shared annotation resolver gives unsupported `isinstance` type objects a generic annotation diagnostic. | `pre_v1_compat_11_diagnostics` | Consider a purpose-built unsupported-runtime-type-object diagnostic while removing rejection residue. |
| Item 1 | TypeVar bound-name recognition duplicates the canonical bound-name set. | `plans/phases/13_type_system_completion.md` type-system owner | Consolidate the known bound names in later type-system maintainability work. |
| Item 1 | The verification-taxonomy checker rejects the compatibility inventory and architecture records, failing both Sifr gates identically at the Item 1 base and candidate SHAs. | `pre_v1_compat_16_closure` | Treat as inherited infrastructure debt and reconcile the final compatibility guard with governed taxonomy checks. |
| Item 2 | Private `_sifr.*` alias emission is implemented for direct function calls only; aliased classes, constants, and first-class function values need explicit coverage before they are used. | `pre_v1_compat_7_stdlib_residue` | Harden the alias mechanism and add focused coverage when closing remaining stdlib residue. |
| Item 2 | Registering original private callable signatures can overwrite a same-named signature from another `_sifr.*` module. | `pre_v1_compat_7_stdlib_residue` | Use collision-safe registration and add a conflicting-name test during stdlib residue closure. |
| Item 2 | The intrinsic registry still lists non-live `random_choice`, and private `abs_val` is dead after canonicalization. | `pre_v1_compat_7_stdlib_residue` | Remove dead intrinsic residue under the item already assigned to the final stdlib sweep. |
| Item 2 | Canonical `pow` currently leaves an unused emitted wrapper because intrinsic routing inlines `powf`; the extended-stdlib idiomatic demo's compact RNG arithmetic does not cover extreme `i64` bounds. | `pre_v1_compat_7_stdlib_residue` | Reconcile wrapper/intrinsic ownership and decide whether the hand-authored demo should model full-range arithmetic during the stdlib residue sweep. |
| Item 3 | The migrated time subset compares two wall-clock samples monotonically even though the system clock can move backward. | `pre_v1_compat_7_stdlib_residue` | Replace the theoretical flaky assertion with positive-sample coverage during the final stdlib sweep. |
| Item 3 | The stdlib parity inventory token `time(` is also a substring of several other calls. | `pre_v1_compat_7_stdlib_residue` | Tighten the canonical `time()` evidence token when reconciling the complete export inventory. |
| Item 3 | Public underscore privacy relies on export retention; `sifr.heapq` exceptions and the duplicate frontend policy need explicit reconciliation. | `pre_v1_compat_7_stdlib_residue` | Consolidate and directly test the approved private-export policy in the final stdlib sweep. |
| Item 3 | Current stdlib architecture and parity records still name the obsolete `lib/sifr/*.sifr` source path. | `pre_v1_compat_7_stdlib_residue` | Correct current documentation paths while making public docs match the final compiled export inventory. |
| Item 3 | Regenerated demo companions contained substantial pre-existing emission drift beyond the public-name edits. | `pre_v1_compat_7_stdlib_residue` | Reconcile remaining stdlib demo artifacts during the final stdlib sweep without reopening Item 3. |
| Item 4 | Two migrated CPython fixture header comments are mechanically worded, and several fixtures duplicate a local `has_match` adapter instead of covering `Pattern.is_match` directly. | `pre_v1_compat_7_stdlib_residue` | Clean the fixture prose and consolidate canonical regex consumer coverage during the final stdlib sweep. |
| Item 4 | JSON keeps a private `_loads_impl` to avoid current flat generated-name collisions while TOML internal callers can use public `loads` directly. | `pre_v1_compat_7_stdlib_residue` | Reconcile the redundant internal shape after the final public-export and collision policy is explicit. |
| Item 4 | User-defined `map` still has the same pre-existing name-based plain-call lowering risk that Item 4 removed for `filter`. | `pre_v1_compat_7_stdlib_residue` | Route builtin `map` exclusively through typed iterator HIR and add shadowing coverage during the final stdlib sweep. |
| Item 4 | `sifr.base64` still imports first-class byte transition helpers without private aliases. | `pre_v1_compat_5_binary_hashing` | Completed in Item 5: Base64 now uses first-class bytes operations and the transition modules are removed. |
| Item 4 | `sifr.json` and `sifr.tomllib` retain unaliased cross-module imports that can appear in compiled public metadata. | `pre_v1_compat_7_stdlib_residue` | Apply the final compiled-export policy to non-intrinsic cross-module imports. |
| Item 4 | The full stdlib parity runner reports inherited bare `math` namespace references, checks the package-scoped `m16_raw_api` demo as a repository-root single file without its manifest policy, and has an `audit_fixtures.py` import-path failure. | `pre_v1_compat_7_stdlib_residue` and `pre_v1_compat_13_verification` | Correct the demo namespace residue in Item 7; correct package-context execution and the audit runner environment in Item 13; do not reopen Item 4. |
| Item 5 | `hashlib.file_digest` does not close its native handle when the binary read raises. | `pre_v1_compat_7_stdlib_residue` | Add structured close-on-error ownership while completing the final stdlib residue sweep; the current path returns a typed error and does not panic. |
| Item 5 | The bytes-native `BytesIO.write_bytes` implementation is behaviorally correct but performs full-buffer slicing and concatenation per write; `getvalue` also retains an infallible `Result` shape and pre-existing closed-stream behavior. | `pre_v1_compat_7_stdlib_residue` | Reconcile final in-memory binary stream performance and API shape without restoring the removed `list[int]` storage path. |
| Item 5 | Holding the bound bytes receiver across argument evaluation can expose a Rust borrow conflict when an argument mutably borrows the same source place. | `pre_v1_compat_8_receivers` | Add explicit overlap coverage and reconcile evaluation-order borrowing with the final receiver-semantics contract. |
| Item 5 | Bytes effectful-receiver coverage pins `hex` and `find`, while `count` and `contains` have only rendered single-occurrence checks; the new prefix/hex diagnostic shapes lack dedicated fail fixtures. | `pre_v1_compat_7_stdlib_residue` and `pre_v1_compat_11_diagnostics` | Complete canonical bytes consumer coverage in Item 7 and diagnostic coverage in Item 11. |
| Item 5 | Two migrated bytes fail fixtures drifted from their named parity anchors, and the random-hashing demo hardcodes bytes instead of visibly encoding its text payload. | `pre_v1_compat_7_stdlib_residue` and `pre_v1_compat_11_diagnostics` | Restore direct canonical consumer intent during the stdlib and diagnostic residue sweeps. |
| Item 5 | The retained-intrinsic schema treats the old `sifr.bytes::primitive_constructors` identifier as a planned deletion although the surface was re-owned under first-class bytes. | `pre_v1_compat_16_closure` | Remove permanent transition-guard slack when the temporary inventory becomes the final compatibility guard. |
| Item 5 | Coverage-matrix readiness still classifies the deleted `sifr_stdlib/bytes` Cargo feature, so both one-shot gates report a stale feature classification in addition to inherited taxonomy debt. | `pre_v1_compat_7_stdlib_residue` | Remove the exact stale `sifr_stdlib` feature row while reconciling the complete stdlib export and feature inventory; do not alter legitimate external `bytes` crate classifications. |
| Item 6 | A length-only empty `set()` loses its source type before Rust inference, so two focused fixtures seed and remove one typed element to test emptiness. | `plans/phases/13_type_system_completion.md` type-preservation owner | Preserve the explicit typed workaround here; make source annotations constrain emitted empty generic constructors in later type-system/codegen hardening. |
| Item 6 | Regenerated companions expose pre-existing all-demo freshness drift and duplicated `PartialOrd` bounds; one unrelated unused heapq import group also remains in `generic_stdlib`. | `pre_v1_compat_7_stdlib_residue` | Reconcile current stdlib companions, add standing freshness evidence, remove unused imports, and deduplicate generated bounds without reopening Item 6. |
| Item 6 | Fully removed inventory rows cannot express zero consumers because the temporary schema requires a non-empty existing-path list. | `pre_v1_compat_16_closure` | Let the final inventory/guard represent closed removal rows without placeholder consumers. |
| Item 6 | Both one-shot gates also matched taxonomy terms introduced by the concurrent static-class-adapter phase. | active static-class-adapter owner and `pre_v1_compat_16_closure` | Coordinate the governed taxonomy contract with that active phase; do not absorb its source vocabulary into Item 6. |
| Item 7 remediation review | The standing demo-freshness guard hardcodes `target/debug/sifr` instead of the repository binary resolver, so non-default `CARGO_TARGET_DIR` and preselected-binary configurations fail closed rather than selecting or rebuilding the configured compiler. | `pre_v1_compat_13_verification` | Route the guard through `verification/areas/common/sifr_binary.py` while reconciling verification-runner execution; the second Item 7 review found this new mechanism defect, so no third review runs. |
| Item 7 remediation review | Adding the 246-demo freshness sweep to `core_guardrails` consumes substantial time without measured budget headroom, and declarative `guardrail_steps` metadata does not name it. | `pre_v1_compat_13_verification` | Measure the combined core-guardrail step, adjust create-PR/merge budgets if required, and align profile metadata during verification-runner closure; do not rerun the consumed Item 7 create-PR gate. |
| Item 8 validation | `hashlib.file_digest` compiles sequential `try` blocks with bindings that escape their generated Rust scopes. | `plans/phases/21_traversal_completeness_and_control_flow_correctness.md` | The full e2e run isolated this pre-existing control-flow/codegen defect after 685 passing fixtures; keep it outside receiver-semantics work. |
| Item 8 review | Consuming Rust/Python interop declarations still normalize a non-owned source receiver to `Owned` instead of diagnosing missing `own` syntax. | `pre_v1_compat_16_closure` | Reconcile or reject the remaining internal normalization when the final guard proves that no unowned compatibility path remains; do not alter the approved Item 8 candidate. |
| Item 8 review | Static slot ABI identity collapses `Owned` and `OwnedMutable`, while structural method identity distinguishes them. | `plans/phases/39_rust_interop.md` | The current ABI is correct because both forms pass `self` by value; preserve this as an interop identity-model note if receiver mutability becomes ABI-significant. |
| Item 9 review | `SIFR-WORKSPACE-0102` remains registered after Item 9 removed the project multi-root emitter. | `pre_v1_compat_11_diagnostics` | Remove the legacy code, placeholder representative, catalog, and docs during the diagnostics sweep. |
| Item 9 review | Duplicate-import-root and duplicate-Sifr-name diagnostics fixtures now exercise the same package-name-derived mechanism. | `pre_v1_compat_11_diagnostics` | Consolidate redundant diagnostic coverage when removing the legacy workspace diagnostic family. |
| Item 10 review | `default_manifest_path` retains an unreachable non-`bin` fall-through after receipt validation made the canonical relationship mandatory. | `pre_v1_compat_16_closure` | Remove the dead branch when the final guard proves that no flat-layout residue remains. |
| Item 10 review | Existing README and self-update examples use `$HOME/bin`, which is canonical but makes `$HOME` the sysroot. | `pre_v1_compat_16_closure` | Reconcile all public installation examples with the preferred dedicated toolchain root during final documentation closure. |
| Item 10 review | The installer creates directories before it rejects a mismatched sysroot relationship, so rejected input can leave empty directories. | distribution installer hardening owner | Preserve as non-blocking side-effect cleanup outside compatibility removal. |
| Item 10 review | The temporary compatibility inventory still records the flat-layout row and its old baseline count. | `pre_v1_compat_16_closure` | Close the row and remove the temporary count when the final inventory becomes the permanent guard. |
| Item 11 initial review | The production cycle fallback emits span-less `SIFR-IMPORT-0007` without the canonical cycle args, notes, or help. | `pre_v1_compat_16_closure` | Route the fallback through the canonical span-less builder during final diagnostics and guard closure. |
| Item 11 initial review | Span-less and source-backed cycle paths format `cycle_edges` with different separators. | `pre_v1_compat_16_closure` | Use one canonical structured-argument format during final diagnostics closure. |
| Item 11 initial review | The span-less namespace-collision branch lacks direct coverage. | `pre_v1_compat_16_closure` | Add a focused no-span collision test with canonical args and candidate notes. |
| Item 11 initial review | Removed-stdlib e2e fixtures retain many old-surface filenames while asserting the same generic diagnostic. | `pre_v1_compat_13_verification` | Consolidate redundant generic unsupported-import coverage while canonicalizing the e2e harness and expectations. |
| Item 11 initial review | `diagnostic_label_for_code_str` remains exported after its legacy display consumer was removed. | `pre_v1_compat_14_source_api` | Remove the unused public wrapper during compiler API consolidation. |
| Item 11 initial review | Active review records still describe `SIFR-IMPORT-0009` as the current HTTP rejection path. | `pre_v1_compat_16_closure` | Correct current review-record links and statements during final documentation closure without rewriting archives. |
| Item 11 remediation review | Lowering-emitted `SIFR-IMPORT-0002` diagnostics omit the declared `module` arg, so recovery capping can depend on fixture order. | `pre_v1_compat_16_closure` | Populate the canonical arg and add cap-independent coverage. This was a new mechanism defect in the second review, so Item 11 received no third review. |
| Item 11 remediation review | The canonicalization checker retains an unused `forbidden_codes` parameter after all callers moved to prefix checks. | `pre_v1_compat_13_verification` | Remove the dead checker branch during verification-runner closure. |
| Item 12 review | The hidden-prefix guard scans `crates/` and `verification/` but not the other production and documentation roots. | `pre_v1_compat_16_closure` | Broaden the final no-compatibility guard after the temporary inventory is removed. Current residue scans prove those roots are clean. |
| Item 12 review | The hidden-prefix self-test proves archive exclusion by placing the archive outside all scan roots, not by exercising an explicit archive rule. | `pre_v1_compat_16_closure` | Make archive exclusion explicit in the final guard and add a direct mutation case. |

## Phase Completion Record

Complete this section after Item 16 merges:

- Final status:
- Final merge SHA:
- Final create-PR profile:
- Final merge gate:
- Final guard result:
- Retained external contracts:
- Deferred out-of-scope work:
- Exact next action:
