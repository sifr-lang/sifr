# `milestone_diag_4b` slice 2 — `CompileError` retirement review (pass 1)

## Scope under review

The uncommitted slice 2 changes on branch `codex/diag-4b-compile-error-retirement`:

- Delete the public `CompileError` diagnostic abstraction from active driver/CLI Rust code.
- Move driver/CLI APIs to the existing transitional `CompilerDiagnostic` transport for this slice.
- Keep diagnostic identity explicit via active `SIFR-*` codes at construction.
- No fallback paths or compatibility aliases.

The next slice may retire the residual `CompilerDiagnostic` renderer/re-export surface in favor of `DiagnosticSink` directly. This review treats that as out of scope and only judges slice 2 against its stated goals.

I read the full uncommitted diff for every file in `git status` plus the surrounding code on disk where needed. I did not run validation; the user reported `cargo check -p sifr_driver -p sifr`, `cargo fmt --check`, `git diff --check`, `cargo test -p sifr_driver --lib --tests`, and `cargo test -p sifr --test e2e test_e2e_fail` all pass.

## Summary

The mechanical surface migration is mostly clean: `CompileError` is gone, `CompilerDiagnostic::with_code` replaces it as the construction entry point, the public re-exports in `crates/sifr_driver/src/lib.rs` are pruned, and the renamed label helpers (`diagnostic_label_for_code`, `diagnostic_label_for_code_str`) are coherent. No fallback shims or compatibility aliases were introduced.

However, the slice contains **one behavioral regression** that violates the explicit slice requirement to "keep diagnostic identity explicit via active `SIFR-*` codes at construction": the test-runner orchestrator now reclassifies *every* test-module lowering diagnostic as `SIFR-INTERNAL-0001`. This changes both the user-facing label and the CLI exit code for a common case (a type error in a test file). I would not merge slice 2 with this regression in place.

## Findings

### F1 — BLOCKING — Test-runner orchestrator drops the inner diagnostic code

[crates/sifr_driver/src/test_runner/orchestrator.rs:104](crates/sifr_driver/src/test_runner/orchestrator.rs:104)

Before this slice, when `lower_frontend_module` returned errors for a test module, the orchestrator preserved each error's `code` field while prefixing the message with the test file path:

```rust
.map(|error| CompileError {
    code: error.code,
    message: format!("[{}] {}", test_file.display(), error.message),
})
```

After the migration, the same site is:

```rust
.map(|error| {
    CompilerDiagnostic::with_code(
        format!("[{}] {}", test_file.display(), error.message),
        DiagnosticCode::INTERNAL_COMPILER_PANIC,
    )
})
```

`CompilerDiagnostic::with_code(..., INTERNAL_COMPILER_PANIC)` constructs a brand-new diagnostic with hardcoded code `SIFR-INTERNAL-0001`, discarding `error.code`. `lower_frontend_module` returns diagnostics whose `code` can be any active `SIFR-TYPE-*`, `SIFR-NAME-*`, `SIFR-OWN-*`, `SIFR-CALL-*`, etc. (whatever HIR produced via `lowering_error_code_or_internal`). All of those identities are now silently rewritten to `SIFR-INTERNAL-0001` once the orchestrator forwards them.

User-visible impact through `sifr test`:

- The CLI label resolver branches on `code.starts_with("SIFR-")` and routes to `diagnostic_label_for_code_str`, which maps `SIFR-INTERNAL-0001` to `"internal compiler error"`. So a plain user-authored type error in a test file now renders as `internal compiler error: [test_file] type mismatch ...` instead of `type error: ...`.
- `is_internal_diagnostic` ([crates/sifr/src/main.rs:258](crates/sifr/src/main.rs:258)) returns `true` for `SIFR-INTERNAL-0001`, so `diagnostic_exit_code` returns `EXIT_INTERNAL_COMPILER_FAILURE` instead of `EXIT_USER_DIAGNOSTIC`. A user with a buggy test source now triggers the "internal compiler failure" exit path the panic-boundary contract is supposed to be reserved for.
- JSON-format output now ships `"code": "SIFR-INTERNAL-0001"` and `"url": ".../SIFR-INTERNAL-0001"` for what is in fact a regular semantic diagnostic.

This directly contradicts both:

- The slice's own task scope ("Keep diagnostic identity explicit via active SIFR-* codes at construction"), and
- The inventory entry for this file at [internal_docs/diagnostic_emission_inventory.md:101](internal_docs/diagnostic_emission_inventory.md:101): "test orchestration failure and frontend error forwarding | `BUILD-*` for orchestration; **forwarded frontend diagnostics retain original identity**."

Suggested fix: forward the inner `CompilerDiagnostic` and only mutate the message. With `errors.into_iter()` we already own each diagnostic, so:

```rust
.map(|mut error| {
    error.message = format!("[{}] {}", test_file.display(), error.message);
    error
})
```

That preserves `code`, `url`, `severity`, `primary_span`, `related_spans`, `children`, `help`, and `suggestions` — all currently dropped by the `with_code(...)` reconstruction. It also matches the way slice 1 framed the orchestrator: "frontend error forwarding" rather than "internal-compiler reclassification."

### F2 — NICE-TO-HAVE — No regression test catches F1

No test in `crates/sifr_driver/src/tests/test_runner.rs` exercises the failure path through `build_test_runner_project`. All four existing tests (`test_run_tests_resolves_local_imports_and_constants`, `test_run_tests_resolves_dotted_local_support_modules`, `test_run_tests_reuses_cached_workspace_for_unchanged_project`, `test_run_tests_invalidates_cached_workspace_when_sources_change`) write valid Sifr and assert success. There is no test that writes, e.g., a test module containing `assert plus_one(BASE) == "wrong"` and asserts that the resulting `Vec<CompilerDiagnostic>` includes a `SIFR-TYPE-*` error rather than `SIFR-INTERNAL-0001`.

That gap is what allowed F1 to pass `cargo test -p sifr_driver --lib --tests`. When fixing F1, please add a unit test along the lines of "test_runner_forwards_lowering_codes_for_buggy_test_module" that asserts at least one returned diagnostic still carries the original active code. This also future-proofs the orchestrator against the next migration to `DiagnosticSink`.

### F3 — NICE-TO-HAVE — Stdlib codegen panic mapping hardcodes a code instead of forwarding

[crates/sifr_driver/src/stdlib/bootstrap.rs:200](crates/sifr_driver/src/stdlib/bootstrap.rs:200)

Before:

```rust
.map_err(|e| {
    vec![CompileError {
        code: e.code,
        message: format!("[stdlib:{module_name}] {}", e.message),
    }]
})?;
```

After:

```rust
.map_err(|e| {
    vec![CompilerDiagnostic::with_code(
        format!("[stdlib:{module_name}] {}", e.message),
        DiagnosticCode::INTERNAL_COMPILER_PANIC,
    )]
})?;
```

`run_codegen_with_boundary` is the only producer here and it can only return `SIFR-INTERNAL-0001` ([crates/sifr_driver/src/diagnostics.rs:213](crates/sifr_driver/src/diagnostics.rs:213)), so the migration is *currently* behavior-preserving. It's only a "nice-to-have" because:

- It makes the stdlib site re-derive a contract that already lives at the panic boundary; if the boundary's contract ever broadens (e.g., to forward stdlib-specific failure codes), this site silently regresses.
- The same mechanical fix as F1 (`mut e; e.message = format!(...); e`) is shorter and forwards the boundary's identity.

I'd take it but I would not block on it. If kept as-is, no further action needed.

### F4 — non-blocking — Slice scope vs. residual `CompilerDiagnostic` exposure (already acknowledged)

This is explicitly out of scope per the task ("the next slice may retire the remaining `CompilerDiagnostic` renderer/re-export surface in favor of `DiagnosticSink` directly"). I am only noting the current state for completeness:

- `CompilerDiagnostic` is still publicly exported from `sifr_driver` ([crates/sifr_driver/src/lib.rs:23](crates/sifr_driver/src/lib.rs:23)) and is still the transport for every driver API (`build`, `build_project`, `check`, `compile`, `run_tests`, `find_workspace_root`, etc.).
- The CLI continues to construct `CompilerDiagnostic` directly via `run_with_panic_boundary` in [crates/sifr/src/main.rs:251](crates/sifr/src/main.rs:251), so the CLI still owns one `INTERNAL_COMPILER_PANIC` construction site.
- `apply_diagnostic_recovery_limits`, `diagnostic_label_for_code`, `diagnostic_label_for_code_str`, `Severity`, `SuggestionKind`, `DiagnosticSpan`, `RelatedSpan`, `DiagnosticChild`, and `DiagnosticSuggestion` all remain exported.

This is exactly what the slice scope says it should be. Recording for the next reviewer's bookkeeping, not as an actionable item.

## What looks correct

- The deletion of `CompileError`, `compile_errors_to_diagnostics`, and `compile_error_label_for_code{,_str}` is clean. No `pub use OldName as NewName`, no inherent-impl shim, no transitional wrapper. The required `CompilerDiagnostic::with_code(message, DiagnosticCode)` constructor is the single construction surface in the driver and CLI.
- `CompilerDiagnostic::with_code` correctly stamps `code: code.code().to_string()` and `url: format!("https://sifr.sh/docs/errors/{code}")` so identity is set explicitly at construction. There is no message-prefix code inference left in the driver — `workspace/mod.rs`, `project/discovery.rs`, `project/compile_order.rs`, `build/materialize.rs`, `build/workspace.rs`, `build/entrypoint.rs`, `frontend/api.rs`, `frontend/module_lowering.rs`, `stdlib/bootstrap.rs`, `test_runner/execution.rs`, and `test_runner/orchestrator.rs` (modulo F1) all pass an explicit `DiagnosticCode` at every call site. I checked all 50 `CompilerDiagnostic::with_code` call sites under `crates/sifr_driver/src` and `crates/sifr/src`.
- `apply_diagnostic_recovery_limits` now takes `&[CompilerDiagnostic]` directly; the CLI's previous `apply_diagnostic_recovery_limits(&compile_errors_to_diagnostics(errors))` chain collapses to `apply_diagnostic_recovery_limits(errors)` ([crates/sifr/src/main.rs:368](crates/sifr/src/main.rs:368)). Grouping keys (`severity`, `code`, `message`, `file`) and limits are unchanged.
- `Display for CompilerDiagnostic` ([crates/sifr_driver/src/diagnostics.rs:158](crates/sifr_driver/src/diagnostics.rs:158)) uses `diagnostic_label_for_code_str(&self.code)`. Because `code` is now `String`, the helper accepts `&str` and returns the same labels the previous `Display for CompileError` produced. Test coverage in [crates/sifr_driver/src/tests/diagnostics.rs:23](crates/sifr_driver/src/tests/diagnostics.rs:23) (`test_diagnostic_labels_are_derived_from_diagnostic_codes`) walks the same code-to-label table that existed before.
- `String == &'static str` comparisons in the renamed assertions (`tests/panic_boundary.rs`, `tests/project_graph.rs`, `tests/single_file_frontend.rs`, `crates/sifr/src/main.rs:259`) use `DiagnosticCode::*.code()` — i.e., they go through the canonical accessor instead of stringly comparing magic constants. That matches the slice's "explicit identity" rule.
- `crates/sifr/tests/e2e.rs` correctly sheds `compile_errors_to_diagnostics` because `CompiledFailure { code: String, message: String }` can be built directly from `CompilerDiagnostic`. No baseline expectation file is touched, which matches the slice scope (no diagnostic-content changes, only transport rename).
- The internal helper rename `lowering_error_to_compile_error` → `lowering_error_to_diagnostic` ([crates/sifr_driver/src/frontend/module_lowering.rs:30](crates/sifr_driver/src/frontend/module_lowering.rs:30)) is coherent with the public surface change. The test-only `to_diagnostic()` round-trip is correctly removed because `CompilerDiagnostic::with_code` already produces the renderer-ready envelope.
- `crates/sifr_driver/src/stdlib/cache.rs` now stores `OnceLock<Result<StdlibCompiled, Vec<CompilerDiagnostic>>>` and the cache tests (`test_get_or_init_stdlib_cache_reuses_successful_compilation`, `test_get_or_init_stdlib_cache_reuses_error_without_fallback_rebuild`) construct `CompilerDiagnostic` directly. No fallback rebuild — the test's stated invariant ("reuses_error_without_fallback_rebuild") matches the slice's no-fallback rule.
- The doc updates in [internal_docs/diagnostic_emission_inventory.md](internal_docs/diagnostic_emission_inventory.md) are consistent: the snapshot count is replaced with a clear statement that `CompileError` has been deleted and `CompilerDiagnostic` is the transitional transport. The "Mechanisms to remove" table swaps the workspace prefix-classifier row to "removed before milestone_diag_4b" and rewrites the phase-derived label row to refer to "active code strings", matching the actual code state.
- The issue checklist correctly marks slice 2 as in-progress at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:71](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:71).

## Things I deliberately did not flag

- **`CompilerDiagnostic` is still a public abstraction.** The slice scope explicitly defers this; tagged in F4 only for context.
- **`Display for CompilerDiagnostic` falls back to `"type error"` for any non-`SIFR-*` code.** This is the same behavior `Display for CompileError` had before slice 1, and changing it is out of scope.
- **TODO comments at `crates/sifr_driver/src/frontend/api.rs` lines 19/37 and `crates/sifr_driver/src/project/discovery.rs` lines 393 and 421** referencing "diag_4a slice 2" parse-code refinement. These TODOs predate this slice and the slice does not claim to address them.
- **`crates/sifr/src/main.rs` still hosts a panic-boundary helper that constructs a `CompilerDiagnostic` directly.** Inventory ([line 102](internal_docs/diagnostic_emission_inventory.md:102)) acknowledges this stays for now and is targeted by a later cleanup.
- **`is_internal_diagnostic` rename** (`is_internal_compile_error` → `is_internal_diagnostic`) and `compile_error_exit_code` → `diagnostic_exit_code`. Cosmetic, scope-aligned, no behavior change.

## Recommended action

1. Fix F1 by mutating the inner `CompilerDiagnostic` instead of reconstructing it. This is a 4-line change.
2. Add a test under `crates/sifr_driver/src/tests/test_runner.rs` that asserts the orchestrator forwards an active semantic code (e.g., `SIFR-TYPE-0002`) for a buggy test module. (F2.)
3. Optionally apply the same minimal-mutation pattern at [crates/sifr_driver/src/stdlib/bootstrap.rs:200](crates/sifr_driver/src/stdlib/bootstrap.rs:200) (F3).
4. Re-run the listed local validation set after F1; the existing `cargo test -p sifr_driver --lib --tests` would not have surfaced this regression on its own.

Once F1 is addressed and a regression test is in place, the slice matches the stated scope and the no-fallback pre-1.0 direction.
