# Phase 37 Execution: Cargo-Backed Sifr Package Coordination

Phase plan: `internal_docs/phases/37_package_management.md`

## Status

- [x] milestone_37_1: Cargo metadata and Sifr manifest linking
- [ ] milestone_37_2: Package graph, scoped imports, and multiple versions
- [ ] milestone_37_3: Package-aware source compilation
- [ ] milestone_37_4: Cargo commands, lock modes, and backend trust
- [ ] milestone_37_5: Workspaces, filters, and tooling
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
