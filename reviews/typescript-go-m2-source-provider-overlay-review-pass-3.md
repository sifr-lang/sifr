# M2 Source Provider & Overlay Store — Review Pass 2

Branch: `wave_tsgo_m2_source_provider_overlay` (diff vs `origin/main`)
Scope: `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` M2 row + AC-31

---

## 1. Blocking findings

**None.**

The two pass-1 blockers have both closed:

- **M1 guardrail drift (was: `check_typescript_go_m1_guardrails.py` failing).** The script now passes against the working tree, including `--self-test`. The one remaining direct read in `crates/sifr/src/check_and_package_commands.rs:579` (`Ok(config.cache_dir.join(key).is_file())`) is the documented CLI cache-probe exception, referenced in the M1 doc inventory.
- **"Direct production reads migrated or listed as exceptions" not met (was: `source/layout.rs:30`, `cargo/lock_modes.rs:46`, `ops/session_discovery.rs`, `ops/session_targets.rs`).** All four are now provider-backed: `validate_pure_marker_file_with_provider` (`source/layout.rs:36-43`), `validate_offline_source_availability_with_provider` (`cargo/lock_modes.rs:44-76`), `session_discovery.rs` and `session_targets.rs` all consume `provider.is_file`/`provider.read_file`/`provider.read_dir`.

Provider implementations, overlay metadata, and the package ambiguity model all match the M2 spec. Validations listed in the request all pass locally; no regressions in `cargo test -p sifr_driver` (144), `sifr_package` (68), `sifr_frontend` (9), `sifr_lint` (22), `sifr_format` (7), or `sifr` unit/integration (89 total).

---

## 2. Non-blocking findings

Ordered by severity within this section.

### 2.1 M1 doc inventory line numbers have drifted from current code
**File:** `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md`
The "Direct FS reads in production source" table references line numbers that no longer host direct reads after the M2 migration, e.g. `graph_cache_and_queries.rs:312/322/359`, `discovery.rs:90/118/180/332/574`, `imports/namespace_api.rs:34`. The M1 guardrail script's `REQUIRED_DOC_SNIPPETS` only checks that the *symbols* appear, not that the line numbers point at code that still does a direct read. The doc is therefore misleading but not incorrect.
**Suggested fix:** re-run the inventory against the post-M2 tree and update the table; the M1 doc's "Future Milestone Update Obligations" already establishes the convention.

### 2.2 M1 doc does not name M3/M6 as owners of `TrackingSourceProvider` records
**File:** `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md:11-15` ("Future Milestone Update Obligations")
The M2 update note says readers must consult `load_project_tracked` outputs, but does not state which later milestone is responsible for consuming the recorded `SourceDependency` stream (M3 `WorkspaceSession`/workspace snapshot work, or M6 file-watcher). The TrackingProvider already records FileRead / DirectoryRead / FileProbe / DirectoryProbe / Canonicalize / FailedLookup — the doc should pin the consumer so the next milestone's diff can be reviewed against a known contract.
**Suggested fix:** add one line in the update-obligations section naming the consumer milestone.

### 2.3 Visibility of `_with_provider` variants is inconsistent
**Files:**
- `crates/sifr_format/src/lib.rs:450-464` — `read_source_with_provider` is `pub`
- `crates/sifr_format/src/config.rs` — provider-backed, `pub`
- `crates/sifr_package/src/manifest/sifr.rs:60-73` — `load_with_provider` is `pub`
- `crates/sifr_package/src/manifest/validate.rs`, `source/layout.rs`, `cargo/lock_modes.rs` — provider variants are `pub(crate)` / crate-local
- `crates/sifr_driver/src/project/discovery.rs`, `workspace/mod.rs` — `pub(crate)` provider variants
- `crates/sifr_lint/src/{config,discovery,engine}.rs` — `pub(crate)` provider variants

The public surface is the `SourceProvider` trait itself, which is `pub` in `crates/sifr_frontend/src/source_provider.rs:61-67`. The `*_with_provider` functions are plumbing, not API. Pick one (recommend `pub(crate)`) and apply uniformly so the boundary is unambiguous to the next reader.
**Suggested fix:** standardize on `pub(crate)` for all `*_with_provider` plumbing functions; only `SourceProvider`, `OverlayDocument`, `SourceProviderError`, `SourceProviderErrorKind`, `SourceDirEntry`, `SourceDependency`, `SourceDependencyKind`, and the new `PackageImport*` types belong on the public surface.

### 2.4 `OverlaySourceProvider::read_dir` silent fallback deserves a TODO
**File:** `crates/sifr_frontend/src/source_provider.rs:174-260`
When an inner `read_dir` call returns `Err(_)`, the method returns an empty `Vec<SourceDirEntry>` rather than propagating the error. For `read_file`/`is_file` the provider does propagate. The asymmetry is intentional (so overlay additions from unsaved buffers still surface in the directory listing) but a future file-watcher (M6) will want a "dirty directory" signal. Add a one-line `// TODO(m6):` here so the M6 reviewer finds it.

### 2.5 `package_import_ambiguity_source_diagnostic` has no driver-level test
**File:** `crates/sifr_driver/src/project/package_discovery.rs:209-275`
The conversion from `PackageImportAmbiguity` → `sifr_diagnostics::Diagnostic` with `DiagnosticCode::IMPORT_AMBIGUOUS_SOURCE_MODULE` is untested at the driver layer. The package-layer test in `milestone_37_3_tests.rs` covers the source-map half, but the diagnostic-emission half (package id, cargo id, candidate paths, severity) is uncovered. This is M17 scope but closing it now would prevent silent regressions during M3.

---

## 3. Missing validation or test coverage

- **AC-31 negative check** — confirm that ambiguity does *not* also emit `SIFR-IMPORT-0005` (unresolved) or any `SIFR-PACKAGE-*` diagnostic. The current `import_resolution_result_preserves_ambiguous_candidates` test asserts the source-map state; an end-to-end check that the diagnostic stream contains exactly one `SIFR-IMPORT-AMBIGUOUS-SOURCE-MODULE` and nothing else would close the spec criterion tightly. **M17 scope; M2 is not blocking on this.**
- **Driver-level ambiguity diagnostic test** — see 2.5 above.
- **`OverlaySourceProvider` round-trip with a deep overlay** — the current test `overlay_provider_synthesizes_nested_directories` covers depth 1; a depth-2/3 case would protect against a future refactor that flattens `Path` comparisons. Not blocking.
- **CI workflow step for `check_typescript_go_m1_guardrails.py`** — the M1 doc says this script must be run, but no `.github/workflows` step enforces it. Recommended, not blocking for M2.

---

## 4. Verdict

**M2 is APPROVED for PR after the listed validations.**

The provider model, overlay metadata, provider-backed read migration, tracking taxonomy, and `PackageSourceMap` ambiguity/fatal representation all match the spec. The two pass-1 blockers are closed. Items 2.1–2.5 are non-blocking follow-ups — recommend opening them as follow-up issues against the owning milestones (M3 doc sync, M6 watcher, M17 diagnostic e2e) rather than holding this PR.

Ready to push the branch and open the PR.
