# Ad Hoc Phase: Pre-v1 Compatibility Removal

Status: in progress; Item 1 merged and ready for Item 2 on 2026-08-18

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

- [ ] Each numeric operation has one public name.
- [ ] Each random operation has one public name.
- [ ] Removed random calls cannot bypass the canonical module state.
- [ ] The public export inventory contains no removed name.
- [ ] Repository Sifr sources use only canonical names.

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

- [ ] Each operation has one public name.
- [ ] Private intrinsic names are not importable from public modules.
- [ ] Environment default behavior remains explicit and typed.
- [ ] Date and timezone APIs expose one UTC value.
- [ ] Repository Sifr sources use only canonical names.

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

- [ ] Each operation has one public name and one documented signature.
- [ ] Canonical JSON serialization preserves every approved input type.
- [ ] Regex keyword conflicts have one documented Sifr spelling.
- [ ] No old name remains through a private import re-export.
- [ ] Public docs and examples use canonical names only.

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

- [ ] Typed binary APIs use `bytes` for storage and transport.
- [ ] Text reaches hashing only through explicit encoding.
- [ ] No compatibility wrapper duplicates a first-class `bytes` operation.
- [ ] No typed binary API accepts `list[int]` as its canonical input.
- [ ] Private binary intrinsics have at least one canonical live consumer.

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

- [ ] Public set operations use first-class `set[T]`.
- [ ] No list-backed set compatibility helper remains.
- [ ] `heapq` exposes one mutation model.
- [ ] `bisect` exposes one insertion model.
- [ ] Removed intrinsics are absent from manifests and codegen registries.

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

- [ ] The Item 0 stdlib inventory has no open row.
- [ ] Each same-operation public export group has one member.
- [ ] Each approved distinct pair has different tested semantics.
- [ ] The export guard detects direct and imported aliases.
- [ ] Public docs match the compiled export inventory.

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

- [ ] Receiver syntax maps directly to one receiver convention.
- [ ] Method-body analysis does not change the declared convention.
- [ ] Mutating a shared receiver produces one source diagnostic.
- [ ] Protocol conformance compares explicit receiver conventions.
- [ ] Owned receiver behavior has native runtime coverage.
- [ ] Repository Sifr sources use explicit canonical receiver syntax.
- [ ] Current same-binding conflict reporting has no compatibility name.

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

- [ ] Every package and workspace reader accepts `[source].root` only.
- [ ] Each reader accepts at most one relative source root.
- [ ] Each reader defaults to `src/` when `[source]` is absent.
- [ ] No crate reads `[source].roots` or handles multiple source roots.
- [ ] The manifest parser rejects `[exports].modules` and `[[bin]]`.
- [ ] Import roots come from canonical package names only.
- [ ] Workspace source-root diagnostics describe `[source].root` only.
- [ ] All repository package fixtures use the canonical source layout.
- [ ] Public package documentation describes one single-root layout.

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

- [ ] Install, update, and uninstall use one binary layout.
- [ ] Self-update accepts one binary and sysroot relationship.
- [ ] Public installation docs show one layout.
- [ ] No migration or conversion path exists for a flat receipt.
- [ ] Release archive and installer tests use the canonical layout.

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

- [ ] Each active compiler error has one diagnostic code.
- [ ] Project root-module and cycle errors emit `SIFR-IMPORT-*` only.
- [ ] Span-less import diagnostics use canonical codes and `<unknown>`.
- [ ] No active diagnostic title or message describes a Sifr form as legacy.
- [ ] Old stdlib names do not receive a migration suggestion table.
- [ ] Old Rust decorator keys do not use a special parser branch.
- [ ] Test helpers render diagnostics through the production renderer.
- [ ] Diagnostic catalogs and public docs contain no removed code.

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

- [ ] Lowering does not strip or recognize hidden compatibility prefixes.
- [ ] No compiler-generated source emits a removed prefix.
- [ ] The guard excludes archives and rejects production or fixture residue.
- [ ] Canonical task and blocking APIs retain their behavior.

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
| 2. Numeric and random names | pending | — | — | — | — | — |
| 3. Runtime-information names | pending | — | — | — | — | — |
| 4. Text and structured-data names | pending | — | — | — | — | — |
| 5. Binary and hashing helpers | pending | — | — | — | — | — |
| 6. Collection helpers | pending | — | — | — | — | — |
| 7. Remaining stdlib residue | pending | — | — | — | — | — |
| 8. Receiver semantics | pending | — | — | — | — | — |
| 9. Package manifest and source layout | pending | — | — | — | — | — |
| 10. Installation layout | pending | — | — | — | — | — |
| 11. Diagnostics and rejection residue | pending | — | — | — | — | — |
| 12. Hidden compatibility names | pending | — | — | — | — | — |
| 13. Verification runner and e2e expectations | pending | — | — | — | — | Phase 40 handoff required. |
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
