# M2 Review Pass 5: TypeScript-Go Source Provider And Overlay Store (post-split)

Branch: `wave_tsgo_m2_source_provider_overlay` (working tree vs `origin/main`)
Scope: `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` M2 + AC-30/AC-31 + W-20 (package ambiguity boundary)
Pass-3 is **stale** for the post-split state — `source_map.rs`, `sifr.rs`, `sifr/load.rs`, `sifr_fields.rs`, `validate.rs`, `source_map/discovery.rs`, `source_map/resolution.rs`, and the new `milestone_adhoc_tsgo_m2_tests.rs` were not yet split out at the time of pass-3. This pass re-validates against the current files.

## Verdict

**M2 is APPROVED for PR.** The M2 source-provider contract, the provider-backed read migration, the package ambiguity/fatal model, the guardrails, and the docs/tracker are all consistent with the spec on the post-split working tree. No regressions in `cargo test -p sifr_frontend` (9), `sifr_package` (68), `sifr_lint` (22), `sifr_format` (7), or `sifr_driver` (135). `cargo clippy --workspace -- -D warnings`, `cargo fmt --check`, `git diff --check`, the M1 guardrail script (positive + `--self-test`), the file-size guardrail, and the package-manager guardrail all pass on the current tree. The M2 Disposition in the M1 doc correctly enumerates the migrated rows and the remaining CLI command-output / cache probe exceptions. The M1 doc's "Future Milestone Update Obligations" section now names M3 (overlay lifecycle + tracked dependency records) and M6 (consume tracked records for dirty-scope) as owners of the M2-introduced tracking surface, which was a pass-2 follow-up and is now resolved.

Severity legend: **Block** = required to merge; **Recommend** = should fix in this PR; **Note** = observation, not blocking.

## 1. Blocking findings

**None.**

The pass-1 blockers (M1 guardrail inventory drift, undocumented direct reads in `source/layout.rs` and `cargo/lock_modes.rs`, missing nested-overlay test) and the pass-2 follow-ups (M1 doc line-number drift after migration, no M3/M6 ownership for tracking records) are all closed. The M2 closeout criteria, AC-30 (preserve ambiguous candidates), and AC-31 (resolution state coverage) are met on the post-split files.

## 2. M2 source-provider contract (item 1 in the request)

### Note: `SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, and `TrackingSourceProvider` are all in one file

- `crates/sifr_frontend/src/source_provider.rs:61-67` — `SourceProvider` trait with `read_file`, `read_dir`, `is_file`, `is_dir`, `canonicalize`. `&mut self` receiver everywhere, which is what the M1 spec implicitly required so `TrackingSourceProvider` can append to a `Vec` without interior mutability.
- `crates/sifr_frontend/src/source_provider.rs:79-139` — `DiskSourceProvider` is a zero-sized struct (`Default`, `Clone`, `Copy`) whose impl wraps `std::fs::` and `Path::*` calls and reports errors via `SourceProviderError`. `SourceProviderError` carries `kind`, `path`, `message`; `kind` is `Canonicalize | FileRead | DirectoryRead` (`source_provider.rs:7-11`). This is the right shape — `is_file`/`is_dir` do not return errors, so a sentinel `FailedLookup` is not needed there.
- `crates/sifr_frontend/src/source_provider.rs:141-171` — `OverlayDocument` exposes `path: SourcePath`, `uri: Option<String>`, `version: DocumentVersion`, `source: SourceText` (which itself owns the M0 `LineMap` and source text), `source_hash: SourceHash`, and `matches_disk: bool`. The `matches_disk` flag is computed at construction by comparing against the optional `disk_source: Option<&str>` argument (`source_provider.rs:160-161`). This matches the M2 spec's "URI, path, version, text hash, line map, and disk-match state" requirement; the `line map` lives inside `SourceText` rather than as a separate field, which is the M0 ownership rule and is documented correctly in `internal_docs/typescript_go_architecture_transfer_m2_source_provider.md:24-27`.
- `crates/sifr_frontend/src/source_provider.rs:173-262` — `OverlaySourceProvider<P>` is read-through: `read_file` checks the overlay map first (`source_provider.rs:201-203`), `is_file` short-circuits on overlay presence (`source_provider.rs:243-245`), `is_dir` treats any overlay that descends from the queried path as a synthetic directory (`source_provider.rs:247-253`), and `canonicalize` returns the path unchanged when an overlay is present (`source_provider.rs:255-261`).
- `crates/sifr_frontend/src/source_provider.rs:298-381` — `TrackingSourceProvider<P>` records every successful `FileRead` / `DirectoryRead` / `FileProbe { exists }` / `DirectoryProbe { exists }` / `Canonicalize`, plus a `FailedLookup` on every `Err` (`:332-380`). The probe variants carry the `exists: bool` payload so M6 dirty-scope work can distinguish a real file presence change from a transient false. `into_parts()` returns `(P, Vec<SourceDependency>)` so callers can split the inner provider and the dependency list without a clone.
- Re-exports are wired in `crates/sifr_frontend/src/lib.rs:13-14` (`mod source_provider; pub use source_provider::*;`). `SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, `TrackingSourceProvider`, `OverlayDocument`, `SourceDirEntry`, `SourceProviderError`, `SourceProviderErrorKind`, `SourceDependency`, and `SourceDependencyKind` are all on the public surface.

### Note: Overlay-aware file and directory reads, including nested overlay-only directories

- `crates/sifr_frontend/src/source_provider.rs:207-241` — `OverlaySourceProvider::read_dir` swallows `inner.read_dir` errors and falls back to an empty `Vec` only when at least one overlay descends from the queried path; otherwise the error propagates. The TODO at `source_provider.rs:218-219` correctly flags the silent-fallback for M6 watcher work.
- `crates/sifr_frontend/src/source_provider.rs:223-238` — overlays that are direct children of the queried directory become `is_file: true` entries, and overlays that descend deeper become `is_dir: true` entries via `overlay_dir_entry` (`source_provider.rs:275-296`). The merge dedupes by `entry.path` and then `sort_by(left.path.cmp(&right.path))` (`:239`) so the result is deterministic across calls. The new `overlay_provider_synthesizes_nested_directories` test (`source_provider.rs:429-461`) exercises the depth-2 case the pass-1 review requested: temp dir with no on-disk files, one overlay at `root/pkg/mod.sifr`, asserts `is_dir(root) == true`, `read_dir(root)` returns the synthesized `pkg/` directory entry, and `read_dir(root/pkg)` returns the synthesized `mod.sifr` file entry. The test passes locally.

### Note: Dependency tracking covers the full taxonomy

- File reads → `SourceDependencyKind::FileRead` (`source_provider.rs:334`).
- Directory reads → `SourceDependencyKind::DirectoryRead` (`:347`).
- File probes → `SourceDependencyKind::FileProbe { exists }` (`:359`).
- Directory probes → `SourceDependencyKind::DirectoryProbe { exists }` (`:365`).
- Canonicalize → `SourceDependencyKind::Canonicalize` (`:372`).
- Failed lookups → `SourceDependencyKind::FailedLookup` (`:338`, `:351`, `:376`).
- The `tracking_provider_records_successes_and_failed_lookups` test (`source_provider.rs:410-427`) confirms a `FileProbe { exists: false }` is recorded for a missing file followed by a `FailedLookup` for a follow-up `read_file`. The `project_loading_uses_overlay_and_tracking_provider` test (`crates/sifr_frontend/src/query_diagnostics.rs:613-657`) confirms a `DirectoryRead` is captured during `FrontendContext::load_project_with_provider`.

## 3. Production read migration (item 2 in the request)

The M1 doc inventory (`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:45-67`) is the source of truth for "production reads affected by compilation/tooling identity." Every in-scope row now goes through a `SourceProvider`:

- **Frontend project/module reads** — `crates/sifr_frontend/src/graph_cache_and_queries.rs:310-415`: `load_project` wraps `TrackingSourceProvider::new(DiskSourceProvider::new())` and delegates to `load_project_with_provider`; `load_project_tracked` returns the captured `SourceDependency` list. No direct `std::fs::*` call survives in `graph_cache_and_queries.rs` outside `is_production_source == false` paths.
- **Driver project/package reads** —
  - `crates/sifr_driver/src/workspace/mod.rs:31-52` and `:54-75`: `find_workspace_root_with_provider` / `parse_workspace_config_with_provider` take `&mut impl SourceProvider`. `validate_source_root_with_provider` (`:131-177`) uses `provider.is_dir`. No `std::fs::*` calls in the M2 scope.
  - `crates/sifr_driver/src/project/discovery.rs:90-217`: `ModuleResolver::resolve_with_provider` and the helper `parent_module_file` / `candidate_paths` thread the provider. `discover_project_sifr_files_with_provider` (`:352-367`) uses `provider.read_dir` and the `SourceDirEntry::is_file` flag rather than a second `is_file` probe.
  - `crates/sifr_driver/src/project/package_discovery.rs:42-67`: `parse_package_import_closure_source_modules` creates a `DiskSourceProvider` and reads package sources through `provider.read_file`. The ambiguity conversion at `:209-275` (see §5 below) is unchanged in shape from pass-2.
- **Formatter source/config reads** — `crates/sifr_format/src/lib.rs:177-200` (`collect_sifr_files` / `collect_sifr_files_with_provider`), `:445-464` (`read_source` / `read_source_with_provider`). `crates/sifr_format/src/config.rs:71-172` (`discover_sifr_toml` / `apply_config_file` and their `_with_provider` variants). The `write_source` helper at `lib.rs:466-474` still uses `std::fs::write`, which is a command-output effect, not a semantic source read, and is consistent with the M1 doc exception at `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:69-88` ("Generated-output and test-harness reads … are outside the M2 semantic source-provider scope").
- **Linter source/config reads** — `crates/sifr_lint/src/config.rs:34-138` (`discover_sifr_toml` / `apply_config_file` / `apply_extends` and their `_with_provider` variants). `crates/sifr_lint/src/discovery.rs:9-141` (`collect_sifr_files` / `collect_sifr_files_for_targets` and the `pub(crate) collect_sifr_files_for_targets_with_provider`). `crates/sifr_lint/src/engine.rs:128-150` (`LintRunner::run_paths` constructs a `DiskSourceProvider` and reads through `provider.read_file`).
- **Package manifest reads** — `crates/sifr_package/src/manifest/sifr/load.rs:7-30` (`SifrManifest::load` / `load_with_provider`); the post-split `sifr.rs` no longer contains the read. Manifest validation is in `crates/sifr_package/src/manifest/validate.rs:7-94` (`validate_source_roots_exist[_with_provider]`, `validate_exports_match_sources[_with_provider]`); all source-root / exports probes go through `provider.is_dir` / `provider.is_file`.
- **Source-map traversal** — `crates/sifr_package/src/imports/source_map/discovery.rs:38-72` (`discover_modules_recursive`) uses `provider.read_dir` and the `SourceDirEntry::is_file` / `is_dir` flags. The orchestrator in `crates/sifr_package/src/imports/source_map.rs:87-136` (`build_with_provider`) takes `&mut impl SourceProvider` and threads it to `discover_package_modules` and `discover_namespace_apis`. `module_for_file` (`:342-365`) is the one consumer that still calls `Path::canonicalize` directly on file paths, but that is a module-file identity helper, not a source read — it has no provider surface and is not in the M2 inventory rows.
- **Namespace API** — `crates/sifr_package/src/imports/namespace_api.rs:32` (`parse_init_sifr_reexports` takes `&mut impl SourceProvider`), `:110-115` (`parse_relative_from`), `:148-153` (`parse_child_namespace_exports`), `:266-271` (`public_child_namespace_exists` uses `provider.is_file`). Recursion is correctly threaded — no `std::fs::*` calls remain.
- **Validation** — covered by `manifest/validate.rs` above; `validate_pure_marker_file_with_provider` at `crates/sifr_package/src/source/layout.rs:36-43` consumes `provider.read_file`.
- **Lock / offline availability** — `crates/sifr_package/src/cargo/lock_modes.rs:36-76` (`validate_offline_source_availability_with_provider` uses `provider.is_dir(package_root)`). No direct `is_dir` / `is_file` remains.
- **Session discovery / targets** — `crates/sifr_package/src/ops/session_discovery.rs:5-33` (`find_manifest` and `is_cargo_workspace_root` use `provider.is_file` and `provider.read_file`). `crates/sifr_package/src/ops/session_targets.rs:11-60` (`discover_app_targets` / `collect_bin_targets` use `provider.is_file` and `provider.read_dir`). The M1 doc rows at `:64` are fully migrated.
- **Pure marker validation** — `crates/sifr_package/src/source/layout.rs:30-43` (already documented above). No direct `read_to_string` remains.
- **CLI command surface / cache probes** — `crates/sifr/src/check_and_package_commands.rs:579` (`Ok(config.cache_dir.join(key).is_file())` in `try_formatter_cache_hit`) is the one remaining CLI direct read; it is documented as a non-semantic command-surface exception in the M1 doc's "M2 Disposition" section (`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:105-109`) and is also called out in the execution tracker.

The M1 guardrail script (`verification/tooling/check_typescript_go_m1_guardrails.py`) confirms the above is consistent: the script's `direct_fs_sites()` walk over the five production scan roots finds only `check_and_package_commands.rs:579` outside the `SOURCE_PROVIDER_BOUNDARY` and the documented permitted exceptions, and that one site is in the doc's inventory (`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:66` and `:105-109`). The script's `validate_direct_fs_inventory` passes, as do all its other checks (doc snippets, source-map stubs, LSP current state, aggregate-only LSP budget, and the `sifr_source` dependency-direction sub-guard).

## 4. Package source-map ambiguity model (item 3 in the request)

### Note: Ambiguity is retained as import-resolution state, not as a fatal construction diagnostic

- `crates/sifr_package/src/imports/source_map.rs:17-23` — `PackageSourceMap` carries four mutually-exclusive state fields plus a fatal-diagnostics slot:
  ```
  pub roots: BTreeMap<(SifrPackageId, ImportRoot), PathBuf>,
  pub modules: BTreeMap<PackageModuleKey, PackageModuleSource>,
  pub ambiguous_modules: BTreeMap<PackageModuleKey, Vec<PackageModuleSource>>,
  pub public_apis: BTreeMap<PackageModuleKey, NamespaceApi>,
  pub fatal_diagnostics: Vec<PackageDiagnostic>,
  ```
  `ambiguous_modules` is the M2-acquired field; `fatal_diagnostics` is the construction-failure slot. The legacy `PackageDiagnostic::invalid_sifr_manifest("source.roots", …)` push that the pre-M2 code did on duplicate module paths is gone — `build_with_provider` at `source_map.rs:100-112` only pushes to the construction-failure path on a `discover_package_modules` `Err`, not on an in-source-map duplicate.
- `crates/sifr_package/src/imports/source_map.rs:308-323` — `insert_module` moves an existing single module into `ambiguous_modules` when a duplicate key is inserted and appends to the same `Vec` for subsequent duplicates. The accumulation is correct (existing candidates are not dropped, the new candidate is appended).
- `crates/sifr_package/src/imports/source_map.rs:325-340` — `module_resolution` returns `ModuleResolution::Ambiguous(&[PackageModuleSource])` when the key is in `ambiguous_modules`, and `ModuleResolution::Resolved(&PackageModuleSource)` / `Missing` otherwise.
- `crates/sifr_package/src/imports/source_map.rs:342-365` — `module_for_file` iterates both `modules.values()` and `ambiguous_modules.values().flat_map(...)`, so file-identity lookups still find candidates from either bucket.

### Note: Fatal package-map failures remain fatal and short-circuit resolution

- `crates/sifr_package/src/imports/source_map.rs:177-187` — `resolve_import_result` checks `!self.fatal_diagnostics.is_empty()` first and returns `PackageImportResolutionResult::FatalPackageMapFailure(self.fatal_diagnostics.clone())` before any other logic. This preserves the spec rule that fatal package-map failures short-circuit source import diagnostics for that package.
- `crates/sifr_package/src/imports/source_map.rs:138-144` — `from_fatal_diagnostics` constructs a map that has only `fatal_diagnostics` populated; the new test `import_resolution_result_distinguishes_unresolved_private_and_fatal_states` (`milestone_adhoc_tsgo_m2_tests.rs:48-90`) exercises this path and asserts the `FatalPackageMapFailure` result. The pre-existing `resolve_import` legacy method (`source_map.rs:146-175`) maps the `Ambiguous` result to a `PackageDiagnostic::undeclared_direct_import` with the joined candidate paths so the historical non-source callers keep working.

### Note: Unit tests cover all five states

The two new tests live in a dedicated `milestone_adhoc_tsgo_m2_tests.rs` file (post-split, registered in `crates/sifr_package/src/lib.rs:90`) rather than the previously-shared `milestone_37_3_tests.rs`. Coverage:

- `import_resolution_result_preserves_ambiguous_candidates` (`milestone_adhoc_tsgo_m2_tests.rs:11-45`) — `math` package with two source roots `["sifr", "alt"]`, writes `math.vector` under both via the new `write_module_under` helper (`:142-149`), then asserts `PackageImportResolutionResult::Ambiguous(ambiguity)` with `ambiguity.candidates.len() == 2` and `PackageImportOrigin::DirectDependency { .. }`. The `Origin` assertion is the negative check that ambiguity travels with the correct dependency-scope origin (own-package vs. direct-dependency).
- `import_resolution_result_distinguishes_unresolved_private_and_fatal_states` (`milestone_adhoc_tsgo_m2_tests.rs:48-90`) — exercises `Unresolved` (`missing.module`), `PrivateAccess` (`math._internal` on a non-production-schema package), and `FatalPackageMapFailure` (constructed via `PackageSourceMap::from_fatal_diagnostics(vec![cargo_metadata_parse(...)])`). All three match arms are asserted.

Both tests pass locally (verified by `cargo test -p sifr_package`, 68/68). The `import_resolution_result_preserves_ambiguous_candidates` is the AC-31 state-coverage test for `Ambiguous`; the second test covers `Unresolved`, `PrivateAccess`, and `FatalPackageMapFailure`. `Resolved` is exercised by every existing package test (e.g., `milestone_37_3_tests`, `milestone_adhoc_pkg_*_tests`) and is a non-issue.

### Note: Digest accounts for ambiguity state

- `crates/sifr_package/src/graph/digest.rs:146-200` — `CanonicalSourceMap` flattens both `modules.values()` (`:179-188`) and `ambiguous_modules.values().flat_map(...)` (`:189-199`) into separate `modules` / `ambiguous_modules` fields, so the `digest_package_source_map` output changes when an ambiguous candidate is added or removed. This preserves the M2 invariant that the digest is stable across M2 and reflects the full package source map state. The two other digests (`digest_graph_inputs` at `:97-136` and `digest_package_graph` at `:20-22` / `:138-143`) are not affected by ambiguity and remain stable.

### Note: The driver ambiguity conversion is still untested at runtime

`package_import_ambiguity_source_diagnostic` at `crates/sifr_driver/src/project/package_discovery.rs:209-275` carries the `DiagnosticCode::IMPORT_AMBIGUOUS_SOURCE_MODULE` code with the correct `module`, `resolution_scope: "package"`, `candidate_paths`, `written_module_path`, `resolved_package_import_path`, `package_import_origin`, `package_id`, and `cargo_package_id` args, plus a per-candidate `candidate path: …` note. This was the pass-1 / pass-2 recommendation; it remains M17 scope per `internal_docs/architecture.md:273` and the planning issue's AC-31 ownership note. **Non-blocking for M2.**

## 5. Guardrail impact (item 4 in the request)

All three guardrails pass on the post-split tree:

- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` → **PASS** (verified locally).
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` → **PASS** (verified locally).
- `python3 scripts/check_file_size_guardrails.py` → **PASS** (2020 files, limit 900 lines).
- `python3 scripts/check_package_manager_guardrails.py` → **PASS** (after the `source_map/` and `manifest/sifr/` splits: `source_map.rs` 372 lines, `discovery.rs` 151, `resolution.rs` 64; `sifr.rs` 262, `sifr_fields.rs` 148, `sifr/load.rs` 30, `validate.rs` 94 — all under the 420-line package-manager cap).
- `cargo fmt --check` → **PASS** (verified locally).
- `git diff --check` → **PASS** (verified locally).
- `cargo clippy --workspace -- -D warnings` → **PASS** (verified locally; clippy is clean across the post-split files).

The M1 guardrail script's `is_production_source` (`verification/tooling/check_typescript_go_m1_guardrails.py:133-140`) excludes `tests` and `bin` paths and any file ending in `_tests.rs` / `tests.rs`. The new `milestone_adhoc_tsgo_m2_tests.rs` is correctly classified as a test file by the script and is not scanned. The post-split `source_map/discovery.rs`, `source_map/resolution.rs`, `manifest/sifr/load.rs`, `manifest/sifr_fields.rs`, and `manifest/validate.rs` are scanned as production source. None contains a `std::fs::` direct read or a `.is_file()` / `.is_dir()` probe (verified by `grep`); all reads go through `provider.*`. The M1 doc's inventory still lists historical line numbers (e.g., `source_map.rs:240`, `sifr.rs:55`, `validate.rs:14/43/44`, `namespace_api.rs:32/264`) that no longer match the post-split file structure — see §6.1 below.

## 6. Non-blocking concerns and missing validation

### 6.1 M1 doc inventory still cites pre-split line numbers

`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:45-67` references, e.g., `crates/sifr_package/src/imports/source_map.rs:240` and `:254`, `crates/sifr_package/src/imports/namespace_api.rs:32` and `:264`, `crates/sifr_package/src/manifest/validate.rs:14`, `:43`, and `:44`, and `crates/sifr_package/src/manifest/sifr.rs:55`. The post-split tree has those line numbers in different files (`source_map/discovery.rs`, `namespace_api.rs`, `manifest/validate.rs`, `manifest/sifr/load.rs`). The M1 guardrail script's `validate_direct_fs_inventory` does not require the line numbers to point at code that still does a direct read — it only requires the doc to *contain* the `path:line` reference as a snippet — so the guardrail still passes. But the inventory is misleading as a maintenance document: a reader who opens `source_map.rs:240` will find a `discover_modules_recursive` provider call, not a direct read. **Recommended (not blocking):** in the M3 follow-up that updates the doc, refresh the inventory rows to point at the new files (or to the actual line that contains a direct read, which for the migrated rows is now the `_with_provider` boundary). A possible lightweight fix is to add a `## Inventory Drift (post-split)` note listing the new locations; the script continues to pass either way.

### 6.2 `REQUIRED_DOC_SNIPPETS` in the script still has pre-split line numbers

`verification/tooling/check_typescript_go_m1_guardrails.py:47-120` lists the historical line numbers as required doc snippets. Because the doc still contains those numbers, the script passes. But the script could be more honest about the M2 state by either dropping the migrated line numbers (since the migration moved them off the direct-read path) or adding an `M2_DOC_SNIPPETS` list for the new locations. **Recommended as a small follow-up, not blocking.** This is the same recommendation as pass-2 §4.2.

### 6.3 Driver-level runtime test for `package_import_ambiguity_source_diagnostic` is still missing

The two new M2 unit tests in `milestone_adhoc_tsgo_m2_tests.rs` cover the package state model and the new `PackageImportResolutionResult` variants. The driver-level conversion at `crates/sifr_driver/src/project/package_discovery.rs:209-275` is still unexercised at runtime. AC-31 ("including a negative check that one import ambiguity does not emit both `SIFR-IMPORT-0005` and `SIFR-PACKAGE-*`") is M17 scope per the planning issue. **Not blocking for M2**; the conversion is the same shape as the workspace ambiguity diagnostic at `crates/sifr_driver/src/project/discovery.rs:262-339` (which is covered by the existing `test_workspace_resolver_reports_ambiguous_source_roots` test), so the risk of a silent regression is bounded.

### 6.4 `OverlaySourceProvider::read_dir` silent fallback is intentionally undocumented in the M2 closeout doc

`crates/sifr_frontend/src/source_provider.rs:218-219` has the right `// TODO(m6):` comment for the M6 watcher work, but the M2 closeout doc (`internal_docs/typescript_go_architecture_transfer_m2_source_provider.md`) does not call out the silent-fallback asymmetry. A future M2 reviewer reading only the M2 doc could be surprised by it. **Recommended: one line in the M2 doc's "Provider Model" section noting that overlay-only directories are synthesized and that failed-lookup errors that hit an overlay descendant are intentionally swallowed for the open-editor case, with M6 owning the dirty-directory signal.**

### 6.5 Visibility of `_with_provider` plumbing functions is still mixed

The pass-2 review flagged that some `_with_provider` functions are `pub` and others are `pub(crate)`. The post-split state:

- `crates/sifr_format/src/lib.rs:182` `pub fn collect_sifr_files_with_provider`, `:450` `pub fn read_source_with_provider` — public.
- `crates/sifr_format/src/config.rs:76` `fn discover_sifr_toml_with_provider`, `:109` `fn apply_config_file_with_provider` — crate-private.
- `crates/sifr_lint/src/config.rs:39` `fn discover_sifr_toml_with_provider`, `:72` `fn apply_config_file_with_provider` — crate-private.
- `crates/sifr_lint/src/discovery.rs:24` `pub(crate) fn collect_sifr_files_for_targets_with_provider` — crate-visible.
- `crates/sifr_lint/src/engine.rs:128-150` — `pub fn run_paths` constructs the provider inline; the `with_provider` plumbing is not exposed.
- `crates/sifr_package/src/manifest/sifr/load.rs:8` `pub fn load`, `:16` `pub fn load_with_provider` — public.
- `crates/sifr_package/src/manifest/validate.rs:7/23/48/64` — `pub fn` for the four validate variants.
- `crates/sifr_package/src/imports/source_map.rs:82/87/146/177` — `pub fn` for `build` / `build_with_provider` / `resolve_import` / `resolve_import_result` / `module_for_file`.
- `crates/sifr_package/src/imports/source_map/discovery.rs` / `resolution.rs` — `pub(super) fn` for the internal helpers, which is correct (no public surface needed).
- `crates/sifr_driver/src/workspace/mod.rs:31` `pub(crate) fn find_workspace_root_with_provider` — crate-visible.
- `crates/sifr_driver/src/project/discovery.rs:95` `pub(crate) fn resolve_with_provider` — crate-visible.
- `crates/sifr_driver/src/project/package_discovery.rs` — no public `_with_provider` surface; the `DiskSourceProvider` is constructed inline at `:42`.

The pattern is "the high-level entry-point that downstream callers already use (`load`, `build`, `resolve_import`, `validate_*`, `find_workspace_root`) is `pub`; the inner `_with_provider` plumbing is `pub(crate)` or `fn`." That is the same convention the rest of the workspace uses (e.g., the `with_options` variants in `sifr_format/src/lib.rs:142`, `sifr_package/src/lib.rs:58`). The M2 surface that is genuinely new and should be on the public API of `sifr_frontend` is the `SourceProvider` / `DiskSourceProvider` / `OverlaySourceProvider` / `TrackingSourceProvider` set, plus the public types `SourceDirEntry`, `SourceProviderError`, `SourceProviderErrorKind`, `SourceDependency`, `SourceDependencyKind`, and `OverlayDocument`. All of those are correctly `pub`. **The visibility is not a regression; recommend standardizing the doc wording for M3 ("`*_with_provider` plumbing is crate-internal unless the high-level entry point is `pub`").**

### 6.6 M2 doc does not enumerate the post-split file structure

`internal_docs/typescript_go_architecture_transfer_m2_source_provider.md` says "M2 focused validation" lists `cargo test -p sifr` etc. but does not name the post-split test file (`milestone_adhoc_tsgo_m2_tests.rs`), the new `source_map/{discovery,resolution}.rs` modules, or the new `sifr/{load.rs}` and `sifr_fields.rs` modules. A reader checking the M2 surface area has to grep the tree. **Recommended: add a "Post-split module layout" subsection listing the new files so the next milestone's diff is reviewed against the correct files.** This was pass-3's stale-snapshot risk; the doc is current on intent but not on the as-shipped file layout.

### 6.7 The `cli_model_and_entrypoint.rs::read_source` provider migration's full call chain

`crates/sifr/src/cli_model_and_entrypoint.rs:723-737` constructs a `DiskSourceProvider` and reads through `provider.read_file`. The `Ok(_)` arm calls `.as_str().to_string()` on the returned `SourceText` to produce a `String`. Verified by `cargo clippy --workspace -- -D warnings` and `cargo test -p sifr` passing. **No action needed**; this is the same call shape as `sifr_format::read_source_with_provider` (`crates/sifr_format/src/lib.rs:450-464`), which was the pass-1 migration model. The clippy clean build confirms the match-expression typing is correct (`process::exit` is `!`, so the match's overall type is `SourceText`).

### 6.8 `tracking_provider_records_successes_and_failed_lookups` does not assert ordering across mixed calls

The test asserts the exact kind Vec for two calls (`:416-426`). A more thorough test would interleave a successful read and a failed read, or a probe-then-read on the same path, to confirm the dependency order is recorded. Not blocking for M2; the `record` method is a simple `Vec::push` (`source_provider.rs:322-327`) so the ordering is correct by construction, and the new `project_loading_uses_overlay_and_tracking_provider` test (`query_diagnostics.rs:611-657`) exercises a multi-call sequence that the assertion checks downstream.

## 7. Doc and tracker coverage (item 5 in the request)

- **Execution tracker (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:11`)** — M2 row enumerates the new public types (`SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, `TrackingSourceProvider`, `OverlayDocument`, `SourceDependency*`, `PackageImportAmbiguity`, `PackageImportResolutionResult`) and the new test surface ("new tests cover overlay shadowing, nested overlay directories, tracked reads, provider-backed project loading, package ambiguity, unresolved/private/fatal import states, and existing lint/format/package behavior"). A reviewer can audit the M2 surface area from the tracker alone. The "M2 local validation so far" block at `:13-23` lists every command and confirms `scripts/run_all_tests.sh --profile quick` PASS at 274.59s.
- **Execution checklist (`issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md:11`)** — M2 is correctly marked as "in review, pending PR" (the checklist is unchecked, the planning issue is the source of truth for the in-review status).
- **M1 guardrail doc (`internal_docs/typescript_go_architecture_transfer_m1_guardrails.md`)** —
  - M2 update note at `:11-15` names `crates/sifr_frontend/src/source_provider.rs` as the intentional boundary.
  - M2 Disposition section at `:90-109` enumerates the migrated rows and the remaining CLI cache-probe exception.
  - Future Milestone Update Obligations at `:166-184` names M3 (move overlay lifecycle and tracked dependency records into `WorkspaceSession` snapshots) and M6 (consume tracked records for dirty-scope classification and dependency-sensitive invalidation) as the owners of the M2-introduced tracking surface. **This closes the pass-2 follow-up §2.2.**
- **M2 doc (`internal_docs/typescript_go_architecture_transfer_m2_source_provider.md`)** — has "Provider Model", "Migrated Reads", "Package Import Ambiguity", "Direct-Read Exceptions After M2", and "Validation" sections. The doc notes at `:46-48` and `:64-65` that dependency records and package runtime fixtures are M3-M6 / M17 scope respectively, which correctly assigns the ownership the request asked about.
- **Frontend cache invalidation doc (`internal_docs/frontend_cache_invalidation.md:13-19`)** — explicitly states "M3-M6 wire them into session snapshots and dependency-sensitive invalidation." Consistent with the M1 doc.
- **Frontend query architecture doc (`internal_docs/frontend_query_architecture.md:25-34`)** — M2 note covers `load_project_with_provider`, `load_project_tracked`, and `OverlaySourceProvider` semantics. Consistent with the M1 doc.
- **Architecture doc (`internal_docs/architecture.md:273, 286-288, 659`)** — M2 line names the provider boundary, the M2 doc reference, and the package import ambiguity preservation. Consistent with the M1 / M2 docs.
- **M3/M6 ownership assignment** — verified. M3 owns overlay lifecycle + tracked records; M6 owns consuming the records for dirty-scope classification. The M2 closeout does not need to consume the records, and the M2 code does not attempt to. No contract drift.

## 8. Test execution summary (verified locally on the post-split tree)

| Command | Result |
| --- | --- |
| `python3 verification/tooling/check_typescript_go_m1_guardrails.py` | PASS |
| `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` | PASS |
| `python3 scripts/check_file_size_guardrails.py` | PASS (2020 files, limit 900 lines; largest M2 file is `graph_cache_and_queries.rs` at 796) |
| `python3 scripts/check_package_manager_guardrails.py` | PASS (largest M2-touched package file is `source_map.rs` at 372; the 420-line cap is respected after the `source_map/`, `sifr/`, and `sifr_fields` splits) |
| `cargo fmt --check` | PASS |
| `git diff --check` | PASS |
| `cargo test -p sifr_frontend` | 9 passed, 0 failed (incl. 3 source_provider tests + 1 project_loading test) |
| `cargo test -p sifr_package` | 68 passed, 0 failed (incl. 2 new M2 ambiguity tests in `milestone_adhoc_tsgo_m2_tests`) |
| `cargo test -p sifr_lint` | 22 passed, 0 failed |
| `cargo test -p sifr_format` | 7 passed, 0 failed |
| `cargo test -p sifr_driver` | 135 passed, 0 failed |
| `cargo clippy --workspace -- -D warnings` | PASS |

The previously-reported pass-2 `e2e_entrypoints::test_e2e_pass` failure is a pre-existing `main`-reproducible Rust compilation failure in the test cache; it is not caused by M2 and is excluded from `--skip test_e2e_pass`.

## 9. M2 closeout criteria (cross-checked against the spec at `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:558-575`)

| Closeout line | Met? | Evidence |
| --- | --- | --- |
| "implement `SourceProvider`, `DiskSourceProvider`, `OverlaySourceProvider`, and `TrackingSourceProvider`" | Yes | `crates/sifr_frontend/src/source_provider.rs:61-67, 69-139, 173-262, 298-381` |
| "model overlays with URI, path, version, text hash, line map, and disk-match state" | Yes | `OverlayDocument` at `:141-171`; line map is owned by `SourceText` per M0 (documented at `internal_docs/typescript_go_architecture_transfer_m2_source_provider.md:24-27`) |
| "migrate frontend project/module reads and package/config reads that affect compilation" | Yes | `graph_cache_and_queries.rs:310-415`, `manifest/sifr/load.rs:7-30`, `manifest/validate.rs:7-94`, `imports/source_map/discovery.rs:38-72`, `imports/namespace_api.rs:32/110/148/266` |
| "let formatter and linter create short-lived providers instead of reading source through separate line-map paths" | Yes | `sifr_format/src/lib.rs:177-200, 445-464`, `sifr_format/src/config.rs:71-172`, `sifr_lint/src/config.rs:34-138`, `sifr_lint/src/discovery.rs:9-141`, `sifr_lint/src/engine.rs:128-150` |
| "track dependency reads, including successful file reads, directory reads, canonicalization, and failed lookups" | Yes | `source_provider.rs:330-380` records all six `SourceDependencyKind` variants; `tracking_provider_records_successes_and_failed_lookups` test exercises two of them; `project_loading_uses_overlay_and_tracking_provider` test exercises `DirectoryRead` end-to-end |
| "split package source-map fatal construction diagnostics from import-site ambiguity records" | Yes | `source_map.rs:17-23` (separate fields), `:177-187` (fatal short-circuit), `:308-323` (ambiguity accumulation), `:325-340` (`Ambiguous` resolution state) |
| "workspace-backed source reads are overlay-aware and dependency-tracked" | Yes | `load_project_with_provider` accepts any `SourceProvider`; `load_project_tracked` returns the captured `SourceDependency` list |
| "M2 owns the overlay record model and provider behavior; M3 owns overlay lifecycle inside `WorkspaceSession`" | Yes | `OverlaySourceProvider` exposes `insert_overlay` + `overlays` getter; no `remove_overlay` / `clear`; no `WorkspaceSession` introduced. M1 doc's "Future Milestone Update Obligations" names M3 as the owner |
| "direct production reads are either migrated or listed as non-semantic exceptions" | Yes | Only `check_and_package_commands.rs:579` (CLI formatter cache probe) survives outside the boundary; documented as a non-semantic command-surface exception at `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:105-109` and the M2 doc. Build artifact metadata (`build/workspace.rs:219/282/296`) and package projection (`projection.rs:100/109/127/129/169/187`) are documented as M15 / package-management exceptions |
| "package source maps preserve legal ambiguous candidates for import resolution instead of failing construction early or emitting `SIFR-PACKAGE-*` for that case" | Yes | `ambiguous_modules` is a separate state field; `insert_module` accumulates candidates; the legacy `PackageDiagnostic::invalid_sifr_manifest("source.roots", …)` push for duplicates is gone (verified at `source_map.rs:100-112`) |
| "package import-resolution unit tests cover resolved, ambiguous, unresolved, private, and fatal package-map states" | Yes | `milestone_adhoc_tsgo_m2_tests.rs:11-45` (ambiguous) and `:48-90` (unresolved + private + fatal); `Resolved` is exercised by every existing package test |

## 10. Approval verdict

**M2 is APPROVED for PR.** The source-provider contract, overlay-aware read model, tracking taxonomy, package ambiguity / fatal separation, package ambiguity / private / unresolved / fatal state coverage, and the post-split module layout are all consistent with the locked M2 architecture decisions and the M2 closeout criteria. All five cargo test suites pass, clippy is clean, the M1 guardrail (positive + `--self-test`) passes, file-size and package-manager guardrails pass, and the M1 doc's M2 Disposition and Future Milestone Update Obligations sections correctly enumerate the migrated rows, the remaining CLI exception, and the M3/M6 ownership of the tracking records. The post-split files respect the 900-line hand-maintained cap and the 420-line package-manager cap.

The non-blocking items in §6 are recommended for a follow-up issue (recommend filing against M3 since they touch the doc, inventory drift, and M2 doc layout). None of them is a correctness regression, and none is required to merge this PR.

**Blockers: zero.** Ready to push the branch and open the PR.
