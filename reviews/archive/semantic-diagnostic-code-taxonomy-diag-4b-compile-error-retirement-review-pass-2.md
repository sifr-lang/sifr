# `milestone_diag_4b` slice 2 — `CompileError` retirement review (pass 2)

## Scope under review

Same scope as pass 1, re-reviewed against the user-reported fixes:

- Delete the public `CompileError` diagnostic abstraction from active driver/CLI Rust code.
- Move driver/CLI APIs to the existing transitional `CompilerDiagnostic` transport for this slice.
- Keep diagnostic identity explicit via active `SIFR-*` codes at construction.
- No fallback paths or compatibility aliases.

Pass 1 raised one BLOCKING issue (F1), one nice-to-have regression test gap (F2), one nice-to-have stdlib codegen-panic forwarding fix (F3), plus a non-blocking F4 noting the residual `CompilerDiagnostic` exposure already deferred to a later slice. The user reports F1, F2, F3, and a clippy `result_large_err` fix have all been applied.

I read the working-tree state of every changed file in `git status`, the full diff of the four files specifically called out in the user's note (`crates/sifr_driver/src/test_runner/orchestrator.rs`, `crates/sifr_driver/src/stdlib/bootstrap.rs`, `crates/sifr_driver/src/tests/test_runner.rs`, `crates/sifr_driver/src/diagnostics.rs`), and re-grepped the workspace to confirm no `CompileError` symbol remains and no parallel `INTERNAL_COMPILER_PANIC` reclassification site survived. I did not run `cargo` myself; the user reports `cargo fmt --check`, `git diff --check`, and the new targeted test all pass, with the broader suites already green pre-fix and re-running locally now.

## Summary

All blocking and nice-to-have items from pass 1 are addressed correctly and minimally. The orchestrator now forwards each inner `CompilerDiagnostic` and only mutates its `message`, the stdlib codegen-panic mapping does the same on a deboxed diagnostic, the new test pins `SIFR-TYPE-0002` survival through `run_tests`, and the panic-boundary `Result` is boxed only at the two sites that own `INTERNAL_COMPILER_PANIC` construction. No behavior regressions remain in the slice's active scope, and no new abstractions were introduced.

I would merge slice 2 in this state. Remaining notes are bookkeeping.

## Verification of pass 1 findings

### F1 — RESOLVED

[crates/sifr_driver/src/test_runner/orchestrator.rs:104](crates/sifr_driver/src/test_runner/orchestrator.rs:104)

The orchestrator now reads:

```rust
let diagnostics: Vec<CompilerDiagnostic> = errors
    .into_iter()
    .map(|mut error| {
        error.message = format!("[{}] {}", test_file.display(), error.message);
        error
    })
    .collect();
return Err(diagnostics);
```

This is exactly the minimal mutation pass 1 suggested. `errors` is `Vec<CompilerDiagnostic>` from `lower_frontend_module`, so `into_iter()` yields owned `CompilerDiagnostic` values; the closure rebinds `error` mutably, edits only `message`, and returns the same diagnostic. `code`, `url`, `severity`, `primary_span`, `related_spans`, `children`, `help`, and `suggestions` are all preserved.

Effect on the user-visible behavior pass 1 flagged:

- A type error in a buggy test module now keeps its `SIFR-TYPE-0002` (or whatever HIR lowering produced via `lowering_error_code_or_internal`).
- `is_internal_diagnostic` ([crates/sifr/src/main.rs:258](crates/sifr/src/main.rs:258)) returns `false` for `SIFR-TYPE-0002`, so `diagnostic_exit_code` correctly returns `EXIT_USER_DIAGNOSTIC` instead of `EXIT_INTERNAL_COMPILER_FAILURE`.
- `render_diagnostics` ([crates/sifr/src/main.rs:367](crates/sifr/src/main.rs:367)) routes through `diagnostic_label_for_code_str`, which falls into the `else` branch for any non-prefix-matched `SIFR-*` code and prints `type error: …` rather than `internal compiler error: …`.
- JSON output now carries `"code": "SIFR-TYPE-0002"` and the matching `"url": ".../SIFR-TYPE-0002"` for what is in fact a regular semantic diagnostic.

This restores parity with the inventory entry at [internal_docs/diagnostic_emission_inventory.md:101](internal_docs/diagnostic_emission_inventory.md:101) ("forwarded frontend diagnostics retain original identity") and with the slice's own scope rule.

The two surrounding panic-boundary call sites (lines 79 and 123) now use `vec![*error]` to deref the new `Box<CompilerDiagnostic>` return type from `run_codegen_with_boundary`. That correctly produces a `Vec<CompilerDiagnostic>` of length 1 carrying the boundary's `INTERNAL_COMPILER_PANIC` diagnostic — which is the correct identity for those sites and not affected by F1's no-reclassification rule.

### F2 — RESOLVED

[crates/sifr_driver/src/tests/test_runner.rs:319](crates/sifr_driver/src/tests/test_runner.rs:319)

`test_run_tests_frontend_type_errors_use_single_path_prefix` is updated with the two assertions pass 1 asked for:

```rust
assert!(
    errors.iter().any(|error| error.code == DiagnosticCode::TYPE_MISMATCH.code()),
    "test module frontend diagnostics should preserve semantic code identity: {errors:?}"
);
assert!(
    errors.iter().all(|error| error.code != DiagnosticCode::INTERNAL_COMPILER_PANIC.code()),
    "test module frontend diagnostics must not be reclassified as internal compiler failures: {errors:?}"
);
```

Both assertions go through `DiagnosticCode::*.code()` rather than stringly comparing magic constants, matching the explicit-identity rule. The existing pre-fix assertions (path prefix present, no double prefix `] [test_bad] return type mismatch`) remain. The fixture is an inert two-file project with `helper.sifr` valid and `test_bad.sifr` returning `"bad"` from a function declared `-> int`, which deterministically reaches `lower_frontend_module` with the `Bare` style and emits a `TYPE_MISMATCH` lowering error. So the test exercises exactly the orchestrator path F1 regressed.

I did not find a dedicated test for the parallel "uncoded lowering error" branch (`uncoded` arm in `lowering_error_to_diagnostic`), but that branch is already covered by the unit test [crates/sifr_driver/src/frontend/module_lowering.rs:101](crates/sifr_driver/src/frontend/module_lowering.rs:101) (`codeless_lowering_error_is_internal_compiler_diagnostic`), which is unrelated to the orchestrator's reclassification regression and predates this slice. No additional coverage is needed.

### F3 — RESOLVED

[crates/sifr_driver/src/stdlib/bootstrap.rs:202](crates/sifr_driver/src/stdlib/bootstrap.rs:202)

The codegen panic-boundary mapping for stdlib modules is now:

```rust
.map_err(|e| {
    let mut diagnostic = *e;
    diagnostic.message = format!("[stdlib:{module_name}] {}", diagnostic.message);
    vec![diagnostic]
})?;
```

This deboxes the `Box<CompilerDiagnostic>` returned by `run_codegen_with_boundary`, mutates only the message, and re-wraps in a `Vec<CompilerDiagnostic>`. Identity, severity, span, children, help, and suggestions all flow through. Behavior is preserved relative to pre-slice-2 because the boundary still produces `INTERNAL_COMPILER_PANIC`, but the site is now contract-safe: if the boundary's contract ever broadens to forward stdlib-specific failure codes, this site silently inherits the new identity instead of stamping over it.

The other three "stdlib bootstrap" mapping sites in this file ([lines 32, 49, 66](crates/sifr_driver/src/stdlib/bootstrap.rs:66)) still construct fresh `STDLIB_BOOTSTRAP_FAILURE` diagnostics. That is intentional and explicitly justified at lines 62-65 in the source comment ("Even if `e.code` is `Some(_)`, stdlib lowering failures collapse to bootstrap failures from the caller's perspective, not user-facing semantic diagnostics"). The pass 1 review only flagged the codegen-panic site, and the other three sites are correct under the slice's identity rule.

### F4 — UNCHANGED — out of scope

`CompilerDiagnostic` remains the public driver/CLI transport. The CLI continues to host one direct construction site in `run_with_panic_boundary` at [crates/sifr/src/main.rs:251](crates/sifr/src/main.rs:251). The `pub use diagnostics::{…}` block in [crates/sifr_driver/src/lib.rs:23](crates/sifr_driver/src/lib.rs:23) still re-exports `CompilerDiagnostic`, `DiagnosticChild`, `DiagnosticSpan`, `DiagnosticSuggestion`, `RelatedSpan`, `Severity`, `SuggestionKind`, `apply_diagnostic_recovery_limits`, `diagnostic_label_for_code`, and `diagnostic_label_for_code_str`. This is exactly what the slice scope says it should be; recorded for the next reviewer's bookkeeping, not as an actionable item.

## Verification of additional clippy fix (pass 1 did not flag this)

The user reports `clippy::result_large_err` was addressed by boxing only the direct panic-boundary `Err` diagnostics. I confirmed:

- [crates/sifr_driver/src/diagnostics.rs:213](crates/sifr_driver/src/diagnostics.rs:213) — `run_codegen_with_boundary` now returns `Result<T, Box<CompilerDiagnostic>>`.
- [crates/sifr/src/main.rs:244](crates/sifr/src/main.rs:244) — `run_with_panic_boundary` returns `Result<T, Box<CompilerDiagnostic>>`.

These are the only two `Box<CompilerDiagnostic>` occurrences in the workspace (`git grep -n "Box<CompilerDiagnostic>"`). Every consumer either re-deboxes via `*error` (orchestrator at lines 79 and 123, stdlib bootstrap at line 203, the rest of `crates/sifr_driver/src/build/*` via the panic-boundary helpers — all spot-checked) or uses `Box`'s deref coercion to read fields directly (CLI's `cmd_*` functions render through `&[*internal]` slices; panic-boundary tests at [crates/sifr_driver/src/tests/panic_boundary.rs:10](crates/sifr_driver/src/tests/panic_boundary.rs:10) and [:20](crates/sifr_driver/src/tests/panic_boundary.rs:20) read `err.code` / `err.message` directly).

Boxing only at the two panic-boundary surfaces is the minimal scope for the lint and avoids touching the much wider `Result<…, Vec<CompilerDiagnostic>>` family that the rest of the driver returns — `Vec` already has a small stack footprint, so the lint does not fire there. Good scoping.

## Re-checked invariants from pass 1

I re-grepped the workspace to confirm no regressions slipped in alongside the fixes:

- `git grep -n "CompileError\b" -- crates/` returns zero matches.
- `git grep -n "compile_error_label_for_code\|compile_errors_to_diagnostics\|compile_error_exit_code\|is_internal_compile_error" -- crates/` returns zero matches.
- `git grep -nE "with_code\(.*INTERNAL_COMPILER_PANIC" -- crates/` matches only programmer-invariant assertion sites (entrypoint shape mismatch, missing `main` HIR module, missing parsed test module, and the two `*_with_panic_boundary` constructors), not user-diagnostic reclassification sites.
- `INTERNAL_COMPILER_PANIC` accessor via `DiagnosticCode::INTERNAL_COMPILER_PANIC.code()` is used in the renamed CLI helper [crates/sifr/src/main.rs:259](crates/sifr/src/main.rs:259), in the new test [crates/sifr_driver/src/tests/test_runner.rs:361](crates/sifr_driver/src/tests/test_runner.rs:361), and in panic-boundary tests, all going through the canonical accessor instead of stringly comparing magic constants — same explicit-identity rule as pass 1.
- The `Display for CompilerDiagnostic` impl, the label table in `test_diagnostic_labels_are_derived_from_diagnostic_codes`, the `apply_diagnostic_recovery_limits` grouping behavior, the `OnceLock<Result<StdlibCompiled, Vec<CompilerDiagnostic>>>` cache and its no-fallback-rebuild test, and the renamed `lowering_error_to_diagnostic` helper are all unchanged from pass 1's "what looks correct" list.

## Recommended action

None blocking. Slice 2 matches its stated scope and the no-fallback pre-1.0 direction. Once the user's locally re-run validation finishes (`scripts/run_all_tests.sh --profile quick` or the equivalent broader suite), this is ready to land.

The next slice (retiring the residual `CompilerDiagnostic` renderer/re-export surface in favor of `DiagnosticSink` directly) can proceed against this baseline; the inventory at [internal_docs/diagnostic_emission_inventory.md](internal_docs/diagnostic_emission_inventory.md) and the issue checklist at [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:71](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:71) already reflect the slice-2 endpoint.
