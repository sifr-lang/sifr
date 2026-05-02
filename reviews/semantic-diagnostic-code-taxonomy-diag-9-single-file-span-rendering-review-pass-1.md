# milestone_diag_9 slice 2 — Review Pass 1

## Verdict: SATISFIED

The slice is well-contained, correctly implements the canonical SourceMap rendering path, and introduces no fallback or compatibility behavior.

---

## 1. Single-file HIR primary_range diagnostics render through SourceMap

**Status: Satisfactory**

`module_lowering.rs:97-123` — `diagnostic_with_source_range` function:

```rust
fn diagnostic_with_source_range(
    message: String,
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(source_context.display_path, source_context.source);
    let span = SourceSpan::new(source_id, range);
    let diagnostic = DiagnosticBuilder::source(code, code.declared_severity(), span)
        .message_template("{message}")
        .arg("message", DiagnosticArg::String(message))
        .build();
    let mut sink = DiagnosticSink::new();
    sink.emit_error(diagnostic);
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        // internal error paths below
    }
}
```

This is the canonical path: `SourceMap` + `DiagnosticBuilder::source` + `DiagnosticSink::emit_error` + `sifr_diagnostics::render::render_sink`. The resulting `RenderedDiagnostic.spans` carry `line`, `column`, `end_line`, `end_column` populated by the renderer.

The call site at `module_lowering.rs:85-86` is gated on both `source_context` (Some) and `primary_range` (Some), so the source-map path is only taken when both are available:

```rust
if let (Some(context), Some(range)) = (source_context, primary_range) {
    return diagnostic_with_source_range(message, code, context, range);
}
```

**Driver test** (`single_file_frontend.rs:252-270` — `test_check_reports_primary_span_for_ranged_hir_diagnostic`):
Asserts `file=Some("main")`, `line=Some(2)`, `column=Some(8)`, `end_line=Some(2)`, `end_column=Some(9)` for `SIFR-FLOW-0005` on `if 1:\n`.

**E2E fixture** (`elif_condition_numeric_truthiness.sifr`):
`# expect-error[col=10]: SIFR-FLOW-0005` — asserts column 10, which is the start of `1` on line 6 (`elif 1:`).

This confirms the end-to-end column assertion for the elif condition path.

---

## 2. Canonical sifr_diagnostics rendering, not ad hoc span math

**Status: Satisfactory**

No manual line/column computation is present. The renderer output is used directly. The internal error paths (see review item 4) catch renderer failures as compiler bugs, not user-visible degradation.

---

## 3. Project/multi-file intentionally preserved for later source-context slices

**Status: Satisfactory**

`entrypoint.rs:166-196` — `RootedEntrypointPlan::from_entrypoint` distinguishes two paths:

- **Single-file** (`RootedEntrypoint::SingleFile`, line 151-163): calls `compile_single_frontend_module_with_source(..., FrontendSourceContext { display_path: "main", source }, ...)` — the new path carrying source context.
- **Project** (`RootedEntrypoint::Project`, line 166-197): calls `collect_project_hir_modules(...)` which calls `compile_frontend_modules(...)` → `lower_frontend_module(...)` with **no source context** (line 36 passes `None`) — preserving existing rangeless behavior for multi-file.

This is explicitly called out in the issue tracker: "Project/multi-file source context remains intentionally deferred to later milestone_diag_9 slices."

---

## 4. Internal-error path for invalid renderer output

**Status: Satisfactory**

`module_lowering.rs:114-121`:

```rust
Ok(_) => crate::diagnostics::diagnostic_with_code(
    "internal compiler error: frontend diagnostic renderer emitted an unexpected diagnostic count",
    DiagnosticCode::INTERNAL_COMPILER_PANIC,
),
Err(error) => crate::diagnostics::diagnostic_with_code(
    format!("internal compiler error: invalid frontend diagnostic span: {error:?}"),
    DiagnosticCode::INTERNAL_COMPILER_PANIC,
),
```

Two distinct failure modes, both emit `INTERNAL_COMPILER_PANIC`. Neither is a silent fallback or user-facing error that would mask data corruption. Both are clearly labeled as internal compiler errors.

---

## 5. Tests are sufficient

**Status: Satisfactory**

### Driver unit test

`single_file_frontend.rs:252-270` — `test_check_reports_primary_span_for_ranged_hir_diagnostic`:
- Uses `check(source)` (the single-file path)
- Finds `FLOW_INVALID_CONDITION_TYPE` diagnostic
- Asserts primary span: `file`, `line`, `column`, `end_line`, `end_column` all set correctly

### E2E expect-error with col=10

`elif_condition_numeric_truthiness.sifr:1`:
```
# expect-error[col=10]: SIFR-FLOW-0005
```
Line 6 is `elif 1:`. Column 10 (1-indexed) is the `1` token. This validates the elif path specifically, complementing the existing `if_condition_numeric_truthiness.sifr` fixture.

### Existing tests preserved

`module_lowering.rs` unit tests (`coded_lowering_error_uses_active_diagnostic_code`, `codeless_lowering_error_is_internal_compiler_diagnostic`) were updated to pass `None` for the new `source_context` parameter and continue to pass.

---

## 6. Missing updates, regressions, or simplifications

**None identified.**

- `Cargo.toml` adds `ruff_text_size` dependency to `sifr_driver` — required for `TextRange` in `diagnostic_with_source_range`. No other crates are affected.
- No changes to `sifr` CLI binary or entrypoint surface.
- `project/mod.rs` re-exports `compile_frontend_modules` for `#[cfg(test)]` — unchanged in intent, only updated import paths.
- No changes to `sifr_hir` (slice 1 already added `primary_range` to `LoweringError`).
- No fixture deletion, no fallback paths, no compatibility shims introduced.

---

## 7. Summary of changes by file

| File | Change |
|------|--------|
| `Cargo.toml` | Added `ruff_text_size` workspace dep |
| `src/frontend/mod.rs` | Re-exports `lower_frontend_module_with_source`, `FrontendSourceContext` |
| `src/frontend/module_lowering.rs` | New `FrontendSourceContext`, new `lower_frontend_module_with_source`, new `diagnostic_with_source_range` using canonical SourceMap renderer, updated `lowering_error_to_diagnostic` to call renderer when both context and range are present |
| `src/project/frontend.rs` | New `compile_single_frontend_module_with_source` that passes `Some(source_context)` |
| `src/project/mod.rs` | Re-exports `compile_single_frontend_module_with_source` |
| `src/build/entrypoint.rs` | Single-file path switched to `compile_single_frontend_module_with_source` with `FrontendSourceContext`; project path unchanged |
| `src/tests/single_file_frontend.rs` | New `test_check_reports_primary_span_for_ranged_hir_diagnostic` |
| `e2e/fail/elif_condition_numeric_truthiness.sifr` | Changed `# expect-error: SIFR-FLOW-0005` to `# expect-error[col=10]: SIFR-FLOW-0005` |

---

## Validation confirmation

Local validation was run and passed (per user-provided results):
- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_driver test_check_reports_primary_span_for_ranged_hir_diagnostic -- --nocapture`
- `cargo test -p sifr_driver frontend::module_lowering -- --nocapture`
- `cargo test -p sifr --test e2e test_e2e_fail -- elif_condition_numeric_truthiness --nocapture`
- `cargo clippy -p sifr_driver --no-deps -- -D warnings`
- `scripts/run_all_tests.sh --profile quick` (wall_time=714.40s)

---

## Conclusion

The slice correctly wires HIR `primary_range` through the canonical `sifr_diagnostics` SourceMap renderer into `RenderedDiagnostic.spans` for the single-file frontend path, preserves existing rangeless project diagnostics for later slices, and provides sufficient test coverage. No fallback paths, no compatibility shims, no regressions. Ready for PR.
