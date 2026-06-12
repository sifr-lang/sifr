# M1 Review — milestone_stdlib_boundary_1 (Create `sifr_stdlib` Contract Crate)

Phase contract: [ad-hoc-stdlib-ir-lowering-boundary-refactor.md](../issues/ad-hoc-stdlib-ir-lowering-boundary-refactor.md)
Execution checklist: [ad-hoc-stdlib-ir-lowering-boundary-refactor-execution.md](../issues/ad-hoc-stdlib-ir-lowering-boundary-refactor-execution.md)

## Verdict

READY

## Scope verification

Compared the uncommitted workspace diff against the M1 scope and "definition of done" clauses in the phase contract.

- `crates/sifr_stdlib` exists, workspace-registered (`Cargo.toml` adds the member and path dep ahead of `sifr_hir`, matching the dependency direction).
- Intrinsic signature modules (`collections_bytes_time.rs`, `crypto_regex_uuid.rs`, `io_json.rs`, `math_test.rs`, `platform_misc.rs`, `sys_fs.rs`) moved verbatim from `crates/sifr_hir/src/stdlib/` into `crates/sifr_stdlib/src/`; `git diff` shows byte-identical bodies (no behavior drift).
- Embedded source inventory moved into `crates/sifr_stdlib/src/sources.rs` as `pub const STDLIB_SOURCES: &[StdlibSource]`, replacing `crates/sifr_driver/src/stdlib/registry.rs::STDLIB_FILES`. All 49 modules and their lexicographic order match the old registry. `include_str!` paths correctly retargeted (`../../../lib/sifr/*.sifr`).
- `crates/sifr_driver/src/stdlib/bootstrap.rs` now iterates `sifr_stdlib::STDLIB_SOURCES` and calls `sifr_stdlib::get_intrinsic_module`; driver still owns bootstrap orchestration. `registry.rs` and its `mod registry` reference are removed.
- `crates/sifr_hir/src/lib.rs` no longer declares a `stdlib` module. `crates/sifr_hir/src/lower/mod_impl.rs:265` switches from `crate::stdlib::get_intrinsic_module` to `sifr_stdlib::get_intrinsic_module`. `sifr_hir/Cargo.toml` adds the `sifr_stdlib` workspace dep.
- Repo-wide grep confirms no surviving `sifr_hir::stdlib` references; remaining `crate::stdlib::*` paths are all internal `sifr_driver::stdlib` submodules (`frontend/api.rs`, `stdlib/bootstrap.rs`, `stdlib/cache.rs`, `test_runner/orchestrator.rs`) — expected and unrelated.
- `crates/sifr_stdlib/Cargo.toml` depends only on `sifr_type_system`. No edge to `sifr_lowering`/`sifr_hir`/`sifr_frontend`/`sifr_codegen`/`sifr_driver`/`sifr_package`/`sifr_analysis`/`sifr_lsp`/CLI — satisfies the locked dependency rule (contract §5 and exit gate).
- No compatibility shim left behind in `sifr_hir` (no re-exported `pub mod stdlib`). Aligns with the "no permanent shims" rule.
- File-size guardrail: largest moved file is `math_test.rs` at 548 lines, well under the 900-line cap. New `sources.rs` is 205 lines. No 900+ files introduced.

## Validation evidence

Recorded in execution checklist (verified line-by-line against the contract's M1 validation list):

- `cargo check -p sifr_stdlib` — PASS
- `cargo test -p sifr_stdlib` — PASS (covers four new positive tests: known intrinsic resolves, unknown intrinsic is `None`, source inventory contains `sifr.json`, and the `_sifr.*` vs `sifr.*` classification negative case)
- `cargo tree -p sifr_stdlib --depth 5` — PASS; no forbidden edges
- `cargo test -p sifr -- stdlib` — PASS
- `cargo test -p sifr_hir name_import_diagnostics_tests` — PASS (lowering still resolves intrinsic imports via the new crate)
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/stdlib_test.sifr` — PASS (positive fixture)
- `cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/import_intrinsic.sifr` — expected `SIFR-IMPORT-0001` (negative fixture)
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/stdlib_io_consolidated.sifr` — PASS
- `scripts/run_all_tests.sh --profile create-pr` — PASS (advisory: warm wall-time budget exceeded; non-blocking)

This satisfies the contract's "at least one positive validation and one negative validation for the boundary" requirement (intrinsic-module lookup positive/negative inside `sifr_stdlib`'s own tests, plus the positive/negative e2e fixtures).

## Blocking findings

None.

## Non-blocking notes (defer to a follow-up cleanup, not required for M1 merge)

1. `scripts/check_file_size_guardrails.py:15` — `LEGACY_HIR_GUARDRAIL_PATHS` still enumerates `crates/sifr_hir/src/stdlib/{mod,io_json,math_test,collections_bytes_time,sys_fs,crypto_regex_uuid,platform_misc}.rs`. The self-test asserts only that these path strings would be categorized as Rust sources, so it continues to pass without the files existing. The intent of the list (legacy paths that must remain *guarded against monolithic regrowth*) is now stale because those paths no longer belong to the workspace. Consider trimming when M2 lands.
2. `scripts/check_hir_maintainability_guardrails.py:12` — `BANNED_MONOLITHS` still contains `crates/sifr_hir/src/stdlib.rs`. The check still passes (the file does not and cannot exist), but the entry is moot post-M1. Same suggestion: clean up during M2 or a later milestone.
3. `reviews/ad-hoc-stdlib-boundary-m1-review-1.md` was an empty placeholder before this review was written; that's fine — populated now. Ensure the execution checklist's "Review Artifacts" section gets this file's link appended when M1 is recorded as merged.
4. The `scripts/run_all_tests.sh --profile create-pr` warm wall-time budget advisory is noted; it is an advisory only and not a regression caused by M1 (the changes are mechanical moves with no new compute).

## Additional validation required

None beyond what is already in the execution ledger. The scope is purely mechanical (verbatim moves with one import-path edit in lowering and one in driver), all listed positive/negative validations have been run, dependency-direction is verified by `cargo tree`, and the "no behavior changes / no shims" rules are visibly satisfied by the diff.

Cleared to open the M1 PR.
