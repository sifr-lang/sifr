# Phase 37 Execution: Cargo-Backed Sifr Package Coordination

Phase plan: `internal_docs/phases/37_package_management.md`

## Status

- [ ] milestone_37_1: Cargo metadata and Sifr manifest linking
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

- Pending.
