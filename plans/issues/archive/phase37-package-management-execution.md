# Phase 37 Execution: Cargo-Backed Sifr Package Coordination

Phase plan: `internal_docs/phases/37_package_management.md`

## Status

- [x] milestone_37_1: Cargo metadata and Sifr manifest linking
- [x] milestone_37_2: Package graph, scoped imports, and multiple versions
- [x] milestone_37_3: Package-aware source compilation
- [x] milestone_37_4: Cargo commands, lock modes, and backend trust
- [x] milestone_37_5: Workspaces, filters, and tooling
- [x] milestone_37_6: Packaging, publishing, and vendoring
- [x] milestone_37_7: Validation, docs, and guardrails
- [x] milestone_37_7 follow-up: Organization demo repository templates
- [x] milestone_37_7 follow-up 2: Organization demo repository subrepos

## milestone_37_7 follow-up 2: Organization Demo Repository Subrepos

Branch: `phase37-demo-subrepos`

Scope:

- Create and populate the real `sifr-lang/sifr-demo-*` repositories.
- Convert `verification/package_management/demo_repositories/sifr-demo-*` from checked-in files to git submodules.
- Keep the local guardrail validating `.gitmodules`, checked-out required files, trust declarations, Git tag references, lockfile shape, alias coverage, and workspace shape.
- Validate the subrepos directly with Cargo metadata/check commands before merging the main-repo submodule conversion.

Validation:

- [x] `cargo check` in `sifr-demo-http`
- [x] `cargo check` in `sifr-demo-app`
- [x] `cargo check --workspace` in `sifr-demo-workspace`
- [x] `cargo metadata --locked --format-version 1` in `sifr-demo-json`
- [x] `cargo metadata --locked --format-version 1` in `sifr-demo-http`
- [x] `cargo metadata --locked --format-version 1` in `sifr-demo-test-support`
- [x] `cargo metadata --locked --format-version 1` in `sifr-demo-app`
- [x] `cargo metadata --locked --format-version 1` in `sifr-demo-workspace`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `cargo fmt --check`
- [x] `cargo test -p sifr_package phase37_demo_subrepos_cover_required_org_repos`
- [x] `cargo test -p sifr_package`
- [x] `scripts/run_all_tests.sh --profile quick`
  - Final post-fix run passed with advisories only: warm wall-time budget exceeded (`2098.66s`) and e2e group skew high (`9.5x`); e2e `67/67`, `cache_hits=12/12`, no failures.
- [x] agent follow-up review final READY
  - Review artifacts: `reviews/phase37-demo-subrepos-review-pass-1.md`, `reviews/phase37-demo-subrepos-review-pass-2.md`, `reviews/phase37-demo-subrepos-review-pass-3.md`
  - Verdict: READY; no blockers.

PR:

- https://github.com/sifr-lang/sifr/pull/2151

## milestone_37_7 follow-up: Organization Demo Repository Templates

Branch: `phase37-demo-repository-templates`

Scope:

- Add checked-in publishable source templates for the Phase 37 organization demo repositories:
  - `sifr-demo-json`
  - `sifr-demo-http`
  - `sifr-demo-test-support`
  - `sifr-demo-app`
  - `sifr-demo-workspace`
- Add `verification/package_management/phase37_demo_repositories.json` as the manifest for required files, tags, validations, and repository identities.
- Extend `scripts/check_package_manager_guardrails.py` so the demo repository templates stay present and preserve pure marker, Rust-backed trust, alias, lockfile, and workspace coverage.
- Add closeout test coverage for the demo repository manifest.
- Update Phase 37 and traceability docs with the local template source of truth.

Validation:

- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `cargo fmt --check`
- [x] `cargo test -p sifr_package`
- [x] `scripts/run_all_tests.sh --profile quick`
  - Passed with advisories only: warm wall-time budget exceeded (`1870.35s`) and e2e group skew high (`9.5x`); no failures.
- [x] agent follow-up review final READY
  - Review artifacts: `reviews/phase37-demo-repositories-review-pass-1.md`, `reviews/phase37-demo-repositories-review-pass-2.md`
  - Verdict: READY; no blockers.

PR:

- https://github.com/sifr-lang/sifr/pull/2150

## milestone_37_1: Cargo Metadata And Sifr Manifest Linking

Branch: `phase37-m37-1-package-foundation`

Scope:

- Add `crates/sifr_package` with facade types and Cargo adapter boundaries.
- Parse and validate `sifr.toml`.
- Parse and validate `[package.metadata.sifr]` from Cargo metadata.
- Derive selected Cargo package graph from normalized `cargo metadata`.
- Classify Sifr packages vs backend Rust crates.
- Validate pure Sifr marker targets.
- Add package diagnostics.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `cargo test -p sifr -- --skip test_e2e_pass`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - Passed with warm wall-time budget advisory on the first cold-ish rebuild after adding `crates/sifr_package`.
- [x] Post-clippy-fix targeted validation:
  - `cargo fmt --check`
  - `cargo clippy -p sifr_package -- -D warnings`
  - `cargo test -p sifr_package`
  - `cargo test -p sifr_lsp --lib`
- [ ] Post-clippy-fix quick rerun:
  - Aborted after reproducing an idle `rustdoc` hang in unchanged `cargo test -p sifr_lsp` doctests. The standalone `cargo test -p sifr_lsp --lib` passes; changed-code gates above pass.

Reviews:

- [x] agent milestone review pass 1: `reviews/phase37-m37-1-review-pass-1.md`
- [x] agent milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2142

## milestone_37_2: Package Graph, Scoped Imports, And Multiple Versions

Branch: `phase37-m37-2-scoped-imports`

Scope:

- Parse Cargo `resolve.nodes[].deps[]` edges and keep dependency rename identity.
- Derive Sifr package edges from exact resolved Cargo package ids instead of package-name guesses.
- Build per-package direct dependency import scopes.
- Allow the same export root to resolve to different package instances in different scopes.
- Reject duplicate import roots inside one package scope with `SIFR-PACKAGE-0201`.
- Support `[package.metadata.sifr.aliases]` for same-scope multiple-version imports.
- Add package-instance type identity model and `SIFR-PACKAGE-0204` diagnostics.
- Include resolve edges and import scopes in deterministic graph digests.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `python3 scripts/check_diagnostic_docs_sync.py`
- [x] `python3 scripts/check_diagnostic_code_coverage.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - Passed; warm wall-time budget exceeded on a cold-ish rebuild (`2445.68s`), no test failures.

Reviews:

- [x] agent milestone review pass 1: `reviews/phase37-m37-2-review-pass-1.md`
- [x] agent milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2143

## milestone_37_3: Package-Aware Source Compilation

Branch: `phase37-m37-3-package-source-map`

Scope:

- Build `PackageSourceMap` from `SifrPackageGraph` package source roots.
- Discover `.sifr` modules, including package directory `__init__.sifr` paths.
- Resolve package-aware imports against own package sources first, then direct dependency scopes.
- Remap alias import roots to dependency export roots.
- Reject transitive or undeclared package imports with `SIFR-PACKAGE-0202`.
- Reject dependency private module access with `SIFR-PACKAGE-0203`.
- Expose source-map models for later CLI/LSP integration.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `python3 scripts/check_diagnostic_docs_sync.py`
- [x] `python3 scripts/check_diagnostic_code_coverage.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - Passed; warm wall-time budget exceeded on a cold-ish rebuild (`2493.27s`), no test failures.

Reviews:

- [x] agent milestone review pass 1: `reviews/phase37-m37-3-review-pass-1.md`
- [x] agent milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2144

## milestone_37_4: Cargo Commands, Lock Modes, And Backend Trust

Branch: `phase37-m37-4-cargo-ops-lock-trust`

Scope:

- Add Cargo command plans for metadata, fetch, build, package, publish, vendor, add, remove, and update.
- Model locked/offline/frozen command arguments and mutation restrictions.
- Validate offline/frozen Sifr package source availability with `SIFR-PACKAGE-0104`.
- Map Cargo command and private-source credential failures to redacted `SIFR-PACKAGE-0101`.
- Validate direct backend Rust crate trust policy with `SIFR-PACKAGE-0301` and stale trust entries with `SIFR-PACKAGE-0305`.
- Extend package build cache inputs with Cargo lock, metadata, graph, source-map, Sifr metadata/source, compiler, target, profile, feature, and selector digests.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `python3 scripts/check_diagnostic_docs_sync.py`
- [x] `python3 scripts/check_diagnostic_code_coverage.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - First run timed out while building the frontend query benchmark helper.
  - Warm rerun passed; warm wall-time budget exceeded (`2025.69s`), no test failures.

Reviews:

- [x] agent milestone review pass 1: `reviews/phase37-m37-4-review-pass-1.md`
- [x] agent milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2145

## milestone_37_5: Workspaces, Filters, And Tooling

Branch: `phase37-m37-5-workspace-filters-tooling`

Scope:

- Select Sifr-capable Cargo workspace members from normalized Cargo metadata.
- Reject explicit Rust-only package selection with `SIFR-PACKAGE-0102`.
- Reject Rust-only workspace members that directly depend on Sifr packages with `SIFR-PACKAGE-0106`.
- Detect duplicate workspace import roots with `SIFR-PACKAGE-0602`.
- Add Turborepo-style package filters for package, dependency closure, dependent closure, dependents-only, and negation.
- Map changed paths to owning Sifr packages and report unmappable paths with `SIFR-PACKAGE-0603`.
- Add read-only outdated query source classification and unsupported-source diagnostics with `SIFR-PACKAGE-0604`.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `python3 scripts/check_diagnostic_docs_sync.py`
- [x] `python3 scripts/check_diagnostic_code_coverage.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - Passed; warm wall-time budget exceeded (`2658.70s`) and group skew advisory reported, no test failures.

Reviews:

- [x] agent milestone review pass 1: `reviews/phase37-m37-5-review-pass-1.md`
- [x] agent milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2146

## milestone_37_6: Packaging, Publishing, And Vendoring

Branch: `phase37-m37-6-package-publish-vendor`

Scope:

- Validate package archive entries for required Sifr metadata and `.sifr` source files.
- Reject archive traversal paths before publish/package delegation.
- Plan `sifr package --dry-run` as Cargo package plus Cargo publish dry-run after Sifr validation.
- Delegate publish and vendor behavior through Cargo-compatible command plans.
- Keep publish/package diagnostics credential-redaction ready.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `python3 scripts/check_diagnostic_docs_sync.py`
- [x] `python3 scripts/check_diagnostic_code_coverage.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - First run timed out while building the frontend query benchmark helper.
  - Warm rerun passed; warm wall-time budget exceeded (`3118.71s`) and group skew advisory reported, no test failures.
  - Final post-review rerun passed; warm wall-time budget exceeded (`4013.09s`) and group skew advisory reported, `e2e cache_hits=12/12`, no test failures.

Reviews:

- [x] agent milestone review pass 1: `reviews/phase37-m37-6-review-pass-1.md`
- [x] agent milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2147

## milestone_37_7: Validation, Docs, And Guardrails

Branch: `phase37-m37-7-validation-docs-guardrails`

Scope:

- Add the Phase 37 package-management fixture matrix for pure Sifr packages, Rust-backed packages, workspaces, path/Git/registry dependencies, multiple-version graphs, aliases, and publishing.
- Tighten `scripts/check_package_manager_guardrails.py` so closeout coverage remains enforced.
- Complete `crates/sifr_package/DEPENDENCY_AUDIT.md`, `TRACEABILITY.md`, and `FEATURES.md`.
- Update public package-management docs, CLI docs, architecture, roadmap, and Phase 37 status.
- Keep uv/Python interop documented as future work outside Phase 37 core.

Validation:

- [x] `cargo fmt --check`
- [x] `cargo clippy -p sifr_package -- -D warnings`
- [x] `cargo test -p sifr_package`
- [x] `python3 scripts/check_package_manager_guardrails.py`
- [x] `python3 scripts/check_diagnostic_docs_sync.py`
- [x] `python3 scripts/check_diagnostic_code_coverage.py`
- [x] `scripts/run_all_tests.sh --profile quick`
  - First two runs timed out while building the frontend query benchmark helper.
  - Direct helper build completed locally in about `2m50s`; the next quick rerun passed.
  - Passing run reported warm wall-time budget exceeded (`3483.63s`) and group skew advisory, no test failures.
- [x] `scripts/run_all_tests.sh`
  - Several warm PR-lane attempts reached the performance budget subset and failed only on timing thresholds while the host had unrelated Spotlight/syspolicyd/backup load; no package, diagnostics, tooling, generated-code, or cargo test failures occurred before those performance budget stops.
  - The command benchmark drift is tracked in https://github.com/sifr-lang/sifr/issues/2148 with a time-bound Phase 37.7 waiver for the affected check-command budgets only.
  - Final PR-lane run passed; performance passed on retry with unchanged thresholds. Reported wall-time and e2e group-skew advisories only: `wall_time=4571.90s`, `e2e cache_hits=0/19`, `hardening variants=28`, `blocking_failures=0`.

Reviews:

- [x] agent milestone review pass 1
- [x] agent milestone review final READY
  - Review artifacts: `reviews/phase37-m37-7-review-pass-1.md`, `reviews/phase37-m37-7-review-pass-2.md`
  - Verdict: READY; no blocking findings.
- [x] agent full Phase 37 implementation review final READY
  - Review artifact: `reviews/phase37-full-implementation-review-pass-1.md`
  - Verdict: READY; no blocking findings.

PR:

- https://github.com/sifr-lang/sifr/pull/2149
