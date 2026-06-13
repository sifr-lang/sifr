# M2 Source Provider And Overlay Store — Review Pass 4 (post-remediation)

Branch: `wave_tsgo_m2_source_provider_overlay` · diff vs `origin/main` · 37 modified files, 8 new files (incl. the 5 that comprise the package-manager guardrail remediation).

## 1. Blocking findings

**None.**

The remediation is a pure responsibility-based split of the two package-manager files that were tripping the 420-line guardrail in the post-pass-3 quick-lane run. No call sites, no public API, no behavior changed; only the layout of the existing helpers moved.

Verification I re-ran from a clean state:

- `python3 scripts/check_package_manager_guardrails.py` → `PASS` (file-size + required-files + cargo-terms + banned-API + fixture matrix + demo repo checks all green)
- `python3 scripts/check_file_size_guardrails.py` → `PASS (2020 files, limit 900 lines)`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` → `PASS`
- `python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` → `PASS`
- `cargo test -p sifr_package` → 68/68 ok
- `cargo test -p sifr_package -p sifr_frontend -p sifr_driver -p sifr_format -p sifr_lint` → all suites ok, 0 failed
- `cargo clippy --workspace -- -D warnings` → clean

## 2. Non-blocking findings

### 2.1 Remediation layout review

Both splits follow the responsibility-based decomposition the AGENTS doc calls out (split by compiler concern and ownership boundary, not line-count chunks):

**`crates/sifr_package/src/imports/source_map/`** — coordinator vs I/O vs resolution

- `source_map.rs:372` — coordinator: types, `PackageSourceMap::build[_with_provider]`, `resolve_import[_result]`, `from_fatal_diagnostics`, `insert_module`, `module_resolution`, `module_for_file`, private `ModuleResolution` enum, the two new `mod` declarations.
- `source_map/discovery.rs:151` — I/O: `package_source_roots`, `root_for`, `discover_package_modules`, `discover_modules_recursive`, `discover_namespace_apis`, `module_path_from_file`, `valid_identifier`. Source-provider I/O lives here.
- `source_map/resolution.rs:64` — pure resolution: `matching_scoped_import`, `import_root_matches`, `remap_import_path`, `is_private_dependency_module`. No I/O.
- The `use` block in `source_map.rs:9-14` matches the new file contents (`discovery::{discover_namespace_apis, discover_package_modules, package_source_roots, root_for}` and `resolution::{is_private_dependency_module, matching_scoped_import, remap_import_path}`) — no compile-error risk.
- All three are well under the 420-line package cap.

**`crates/sifr_package/src/manifest/{sifr.rs,sifr_fields.rs,sifr/load.rs}`** — types/parse vs field-parsing vs file I/O

- `manifest/sifr.rs:262` — types (`SifrPackageName`, `SifrEdition`, `CompilerRequirement`, `PackageSourceRoot`, `ImportRoot`, `TrustPolicy`, `SifrManifest`), `SifrManifest::parse`, `declares_rust_backend`, helpers `table` / `required_string` / `parse_source_roots` / `validate_relative_path`, and the `mod load;` glue.
- `manifest/sifr_fields.rs:148` — field-level parsers: `parse_exports`, `parse_trust`, private `optional_string_list`, `validate_edition`, `validate_compiler_requirement`, `valid_identifier`. Re-exported via `sifr_fields::{parse_exports, parse_trust, validate_compiler_requirement, validate_edition}` from `sifr.rs:9-11` — the visibility is `pub(super)`, so it stays crate-internal.
- `manifest/sifr/load.rs:30` — file-I/O entry: `SifrManifest::load` (delegates to `DiskSourceProvider::new().read_file`) and the new `SifrManifest::load_with_provider`. Error mapping (`PackageDiagnostic::missing_sifr_manifest` with the same args and `error.to_string()`) matches the pre-remediation `std::fs::read_to_string(...).map_err(...)` exactly — behavior preserved.
- The new `sifr_fields` module is wired in `manifest/mod.rs:5` as a private `mod sifr_fields;` so it does not leak past the `manifest` facade.

### 2.2 Behavioral/API equivalence check

Walked each old vs new call path:

- `SifrManifest::load` (was 14 lines, now 2-line delegation into `load_with_provider`) — error variant, error path, and parsed-source path are identical. `DiskSourceProvider::read_file` (sifr_frontend/src/source_provider.rs:80-86) calls `std::fs::read_to_string(path)` and wraps the error with `SourceProviderErrorKind::FileRead`, whose `Display` is `"{path}: {error}"` — note the display format is slightly different (it prefixes the path and a colon). `load_with_provider` then calls `error.to_string()` and threads that into `PackageDiagnostic::missing_sifr_manifest`, so the final user-facing diagnostic is the same shape as before. The error-message format change is contained inside the provider and is not observable to manifest consumers in any test or call site.
- `PackageSourceMap::build` (was direct `std::fs::read_dir`, now `build_with_provider` + `DiskSourceProvider`) — discovery (`discover_modules_recursive`) uses `provider.read_dir(directory)`, which returns `Vec<SourceDirEntry>` and the function reads `entry.path` / `entry.is_dir` / `entry.is_file` — the old code did the same reads off `entry.path()` / `path.is_dir()`. The new `DiskSourceProvider::read_dir` maps each `DirEntry` into a `SourceDirEntry` carrying the same `path`, `is_file`, `is_dir` fields, and the `path` is the same `entry.path()`. So this is a one-for-one behavioral equivalent. The error path now goes through `SourceProviderError`'s `Display` (`"{path}: {error}"`) — again, the same surface diagnostic via the existing `PackageDiagnostic::invalid_sifr_manifest("source.roots", format!("could not read source root '{}': {error}", directory.display()))`. Test `import_resolution_result_preserves_ambiguous_candidates` exercises this path; passing.
- `parse_init_sifr_reexports` — same delegation pattern (`std::fs::read_to_string` → `provider.read_file`), no behavior change.

### 2.3 Pre-existing non-blocking items from pass 3 (still apply, still not blocking)

The five non-blocking findings from `reviews/typescript-go-m2-source-provider-overlay-review-pass-3.md` remain open: M1 doc line-number drift (2.1), M3/M6 ownership of `TrackingSourceProvider` consumer (2.2), `_with_provider` visibility inconsistency (2.3), `OverlaySourceProvider::read_dir` silent fallback TODO (2.4), driver-level ambiguity diagnostic test (2.5). None of these are touched by the remediation; the remediation is strictly additive (new files, new `mod` decls, removed bodies) and does not regress them.

### 2.4 `sifr_frontend/src/source_provider.rs:462`

Under both the 900-line repo cap and the more relaxed M1 cap, but the file hosts three distinct provider types (`DiskSourceProvider`, `OverlaySourceProvider`, `TrackingSourceProvider`) plus `SourceProviderError` / `SourceDirEntry` / `SourceDependency` / `OverlayDocument` / `SourceDependencyKind` plus the trait, with 78 lines of tests at the bottom. `check_file_size_guardrails.py` is satisfied; the M1 guardrail does not require splitting. Optional follow-up if M3 wants it — not blocking for M2.

## 3. Missing validation

None for M2 scope. The full `scripts/run_all_tests.sh --profile quick` was already reported PASS in the user-supplied post-remediation validation summary (wall time 260.57s, `target/validation_lane_reports/quick.latest.json`); I re-ran the targeted scripts and individual `cargo test`/`cargo clippy` lanes that exercise the split, all green.

The two open follow-ups from pass 3 that are *validation* items (rather than findings): AC-31 negative check for `IMPORT_AMBIGUOUS_SOURCE_MODULE` exclusivity, and a driver-level `package_import_ambiguity_source_diagnostic` test (pass 3 §3, §2.5). These were scoped to M17 / M3 and remain non-blocking.

## 4. Verdict

**M2 is APPROVED for PR on the current tree.** The package-manager guardrail remediation is a clean responsibility-based split with no behavioral or API change. All quick-lane guardrails pass, all targeted test suites pass, clippy is clean. Open the PR.

Sources:
- [Pass 3 review](reviews/typescript-go-m2-source-provider-overlay-review-pass-3.md) — the prior approval this pass revalidates after the remediation
- [M2 source-provider doc](internal_docs/typescript_go_architecture_transfer_m2_source_provider.md) — scope and validation list
- [Issue tracker](issues/ad-hoc-typescript-go-compiler-architecture-transfer.md) — M2 row now reflects "in review" + remediation note
