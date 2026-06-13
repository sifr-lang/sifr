# Ad-hoc Owned-Crate Clippy Cleanup Execution

## Scope

Owned crates only:
- `sifr_hir`
- `sifr_codegen`
- `sifr_driver`
- `sifr`
- `sifr_type_system`

Out of scope:
- `sifr_python_ast`
- `sifr_python_parser`
- third-party or vendored `ruff`-derived code

## Part 1: `sifr_hir`

Status: completed

To-do:
- [x] Replace wildcard imports in `src/lower/mod.rs`
- [x] Replace wildcard imports in `src/stdlib/mod.rs`
- [x] Replace `use super::*` in `src/lower/*.rs` and `src/stdlib/*.rs`
- [x] Fix `format_push_string` in `src/cfg.rs`
- [x] Capture part-specific source inventory
- [x] Run targeted validation
- [x] Verify relevant user-facing path/demo
- [x] Record validation evidence
- [x] Record PR / merge link(s)

Notes:
- Source inventory before edits showed wildcard imports in `lower/` and `stdlib/`, plus `push_str(&format!(...))` in `src/cfg.rs`.
- Source inventory after edits no longer shows wildcard imports or `push_str(&format!(...))` in `sifr_hir`.
- Targeted validation passed with `cargo clippy -p sifr_hir --message-format short -- -D warnings` and `cargo test -p sifr_hir --lib`.
- Test-only explicit imports were fixed in `src/lower/expressions.rs` after the unit-test pass surfaced missing symbols.
- PR: https://github.com/sifr-lang/sifr/pull/1099

## Part 2: `sifr_codegen`

Status: completed

To-do:
- [x] Replace remaining wildcard imports in crate code and tests
- [x] Fix surfaced pedantic warnings with mechanical refactors only
- [x] Evaluate `#![allow(dead_code)]` in `src/lib.rs`
- [x] Capture part-specific source inventory
- [x] Run targeted validation
- [x] Verify relevant user-facing path/demo
- [x] Record validation evidence
- [x] Record PR / merge link(s)

Notes:
- Source inventory before edits showed wildcard imports in `src/lib_codegen_tests.rs` and crate-level `#![allow(dead_code)]` in `src/lib.rs`.
- Wildcard imports in `src/lib_codegen_tests.rs` have been replaced with explicit imports.
- Mechanical fixes included unused-`self` removal, `let_and_return` cleanup, `if let`/`let...else` simplifications, format-string inlining, clone cleanups, and restructuring `IrRuntimeImportNeeds` to avoid excessive booleans.
- `#![allow(dead_code)]` was removed temporarily, but restoring it was necessary because it masks a large pre-existing dormant-code surface outside this warning slice. No new suppression was introduced.
- Targeted validation passed with `cargo clippy -p sifr_codegen --message-format short -- -D warnings`.
- Smoke tests passed with `cargo test -p sifr_codegen simple_function_codegen --lib` and `cargo test -p sifr_codegen generate_rust_multi_exports_non_main_items --lib`.
- Test-only explicit imports were fixed in `src/lib_codegen_tests.rs` after the targeted test run surfaced missing symbols.
- PR: https://github.com/sifr-lang/sifr/pull/1099

## Part 3: residual owned crates

Status: completed

To-do:
- [x] Clear remaining warnings in `sifr_driver`
- [x] Clear remaining source-level `format_push_string` cases in `sifr`
- [x] Clear remaining warnings in `sifr_type_system`
- [x] Capture part-specific source inventory
- [x] Run targeted validation
- [x] Verify relevant user-facing path/demo
- [x] Record validation evidence
- [x] Record PR / merge link(s)

Notes:
- Replaced `push_str(&format!(...))` in `crates/sifr/src/main.rs` and `crates/sifr/tests/e2e.rs`.
- No wildcard-import inventory was found in `sifr_driver`, `sifr`, or `sifr_type_system`.
- Residual Clippy fixes in `crates/sifr/src/main.rs` included `let...else`, explicit `Ok(())`, inline format args, borrowing panic payload by reference, and boolean simplification.
- Residual owned-crate lint gate passed with `cargo clippy -p sifr -p sifr_driver -p sifr_type_system --message-format short -- -D warnings`.
- `cargo test -p sifr_driver test_compile_hello_world` passed as a focused driver smoke test.
- CLI user-path validation passed with:
  - positive: `cargo run -q -p sifr -- check demos/milestone_enums_demo.sifr`
  - negative: `cargo run -q -p sifr -- check demos/milestone_generics_impl_demo.sifr` -> expected frontend type error
- PR: https://github.com/sifr-lang/sifr/pull/1099

## Phase-wide Quality Gates

- No new Clippy suppressions
- No fallback, migration, or legacy compatibility code
- Root-cause fixes only
- No regressions to panic-safety, emitted runtime safety, diagnostics stability, recovery ordering, or exit-code behavior
- Validation evidence recorded before merge

## Validation Log

- `cargo fmt --all` -> pass
- Rust toolchain restored locally; `rustc -vV` no longer fails
- `cargo clippy -p sifr_hir --message-format short -- -D warnings` -> pass
- `cargo test -p sifr_hir --lib` -> pass
- `cargo clippy -p sifr_codegen --message-format short -- -D warnings` -> pass
- `cargo test -p sifr_codegen simple_function_codegen --lib` -> pass
- `cargo test -p sifr_codegen generate_rust_multi_exports_non_main_items --lib` -> pass
- `cargo clippy -p sifr -p sifr_driver -p sifr_type_system --message-format short -- -D warnings` -> pass
- `cargo clippy -p sifr_type_system -p sifr_hir -p sifr_codegen -p sifr_driver -p sifr --message-format short -- -D warnings` -> pass
- `cargo test -p sifr_driver test_compile_hello_world` -> pass
- `cargo run -q -p sifr -- check demos/milestone_enums_demo.sifr` -> pass (`no errors found`)
- `cargo run -q -p sifr -- check demos/milestone_generics_impl_demo.sifr` -> expected negative-path failure (`type error: return type mismatch: expected 'T', got 'T | None'`)
- `scripts/run_all_tests.sh --profile quick` -> pass

## Review Log

- Pending
