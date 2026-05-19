# Phase 37 Execution: Cargo-Backed Sifr Package Coordination

Phase plan: `internal_docs/phases/37_package_management.md`

## Status

- [x] milestone_37_1: Cargo metadata and Sifr manifest linking
- [x] milestone_37_2: Package graph, scoped imports, and multiple versions
- [x] milestone_37_3: Package-aware source compilation
- [x] milestone_37_4: Cargo commands, lock modes, and backend trust
- [x] milestone_37_5: Workspaces, filters, and tooling
- [ ] milestone_37_6: Packaging, publishing, and vendoring
- [ ] milestone_37_7: Validation, docs, and guardrails

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

- [x] Claude milestone review pass 1: `reviews/phase37-m37-1-review-pass-1.md`
- [x] Claude milestone review final READY

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

- [x] Claude milestone review pass 1: `reviews/phase37-m37-2-review-pass-1.md`
- [x] Claude milestone review final READY

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

- [x] Claude milestone review pass 1: `reviews/phase37-m37-3-review-pass-1.md`
- [x] Claude milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2144

## milestone_37_4: Cargo Commands, Lock Modes, And Backend Trust

Branch: `phase37-m37-4-cargo-ops-lock-trust`

Scope:

- Add Cargo command plans for metadata, fetch, build, package, publish, vendor, add, remove, and update.
- Model locked/offline/frozen command arguments and mutation restrictions.
- Validate offline/frozen Sifr package source availability with `SIFR-PACKAGE-0104`.
- Map Cargo command and private-source credential failures to `SIFR-PACKAGE-0101` and `SIFR-PACKAGE-0105`.
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

- [x] Claude milestone review pass 1: `reviews/phase37-m37-4-review-pass-1.md`
- [x] Claude milestone review final READY

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

- [x] Claude milestone review pass 1: `reviews/phase37-m37-5-review-pass-1.md`
- [x] Claude milestone review final READY

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

- [x] Claude milestone review pass 1: `reviews/phase37-m37-6-review-pass-1.md`
- [x] Claude milestone review final READY

PR:

- https://github.com/sifr-lang/sifr/pull/2147
