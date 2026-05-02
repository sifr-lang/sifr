# Review: `milestone_diag_9` slice 3 — project/test span rendering

## Summary

The slice completes project-mode and test-runner HIR diagnostic span rendering by threading parsed source text and display paths through module discovery and lowering. All five review concerns are satisfied.

---

## 1. Production fallback / spanless path in project build/check/emit and `sifr test` file-discovered lowering

**Finding: Clean. No blockers.**

### Entry point path (`build/entrypoint.rs`)

`RootedEntrypointPlan::from_entrypoint` for `RootedEntrypoint::Project`:
- `parse_import_closure_source_modules` is called (not the old `parse_import_closure_modules`).
- `collect_project_hir_source_modules` is called (not the old `collect_project_hir_modules`).
- Source context is always provided via `FrontendSourceContext { display_path, source }` in `collect_project_hir_source_modules`.

`compile_single_frontend_module_with_source` (used for the single-file CLI path) also always receives a `FrontendSourceContext`, including in `entrypoint.rs` single-file mode at line 157–162.

### `lowering_error_to_diagnostic` fallback shape

`module_lowering.rs:86–89` — when `source_context` is `Some` AND `primary_range` is `Some`, it takes the `diagnostic_with_source_range` path. Otherwise it falls back to `diagnostic_with_code(...)` which creates a spanless diagnostic.

This fallback is **only reachable** when:
- (a) `source_context` is `None` — only for the test-only `lower_frontend_module` (guarded `#[cfg(test)]`) which has no source context, OR
- (b) `primary_range` is `None` — only when HIR lowering emits a diagnostic without a range.

Case (a) is test-only. Case (b) is the rangeless HIR path retained for later source-context slices per slice 2's design note. Neither represents a production fallback for project or test-runner diagnostics.

### Test runner (`test_runner/orchestrator.rs`)

`build_test_runner_project`:
- `parse_import_closure_source_modules` (not the old `parse_import_closure_modules`).
- `collect_project_hir_source_modules` for support modules.
- Each test module is lowered with `lower_frontend_module_with_source` and a `FrontendSourceContext` from `parsed.display_path` / `parsed.source`.

No spanless fallback exists in the test-runner production path.

---

## 2. Source text / display path ownership cleanliness; no ad hoc span math

**Finding: Clean. No blockers.**

### `ParsedProjectModule` — clean owned storage

`discovery.rs` defines `ParsedProjectModule` with owned fields:
```rust
pub(crate) struct ParsedProjectModule {
    pub(crate) suite: Vec<Stmt>,     // owned AST
    pub(crate) source: String,       // owned source text
    pub(crate) display_path: String, // owned display path
}
```

`parse_import_closure_source_modules` populates these from filesystem reads:
```rust
let source = std::fs::read_to_string(&path).map_err(...)?;
parsed_modules.insert(module_name, ParsedProjectModule {
    suite,
    source,
    display_path: path.display().to_string(),
});
```

### Flow to lowering with correct lifetimes

`project/frontend.rs:collect_project_hir_source_modules`:
```rust
let result = lower_frontend_module_with_source(
    module_name,
    &parsed_module.suite,
    &external_defs,
    FrontendDiagnosticStyle::ModulePrefixed,
    Some(FrontendSourceContext {
        display_path: &parsed_module.display_path,
        source: &parsed_module.source,
    }),
)?;
```

The `FrontendSourceContext` borrows from `ParsedProjectModule` which lives in `parsed_modules: HashMap<String, ParsedProjectModule>` on the stack of `collect_project_hir_source_modules`. That function's result (`ProjectLowering`) only stores `HirModule` (no source text), so the lifetimes are properly contained.

### No ad hoc span math anywhere in the changed surface

`lowering_error_to_diagnostic` routes entirely through:
- `diagnostic_with_source_range` for ranged+contexted diagnostics — uses `SourceMap::register_source` and `SourceSpan::new` only.
- `diagnostic_with_code` for rangeless/internal diagnostics — zero span construction.

No character-offset arithmetic, no `line/col` reconstruction from raw bytes, no span cloning/merging.

---

## 3. Module rename-to-main preserves source context correctly

**Finding: Clean. No blockers.**

`entrypoint.rs:189–192`:
```rust
if main_module_name != "main" {
    if let Some(entry_module) = parsed_modules.remove(&main_module_name) {
        parsed_modules.insert("main".to_string(), entry_module);
    }
}
```

`ParsedProjectModule` carries its own `source` and `display_path` (the actual file path on disk). The rename only changes the hash-map key from `main_module_name` (e.g., `"app"`) to `"main"`. The `entry_module.source` and `entry_module.display_path` are unchanged.

`collect_project_hir_source_modules` looks up modules by key in `compile_order` (derived from the suites map), then retrieves `parsed_modules.get(module_name.as_str())`. For the renamed entry it finds the correct `ParsedProjectModule` via the `"main"` key and passes the unchanged `display_path` and `source` to `FrontendSourceContext`.

The renamed module's `display_path` remains the original file path (e.g., `"/path/to/cases/app.sifr"`), which is the correct display path for diagnostics.

---

## 4. Diagnostics are canonical `RenderedDiagnostic` with active codes; no old phase/code fallback

**Finding: Clean. No blockers.**

### `diagnostic_with_code` — canonical builder

`diagnostics.rs:28–50`:
```rust
pub(crate) fn diagnostic_with_code(...) -> RenderedDiagnostic {
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),  // intentionally empty for spanless path
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}
```

Always produces a fully-formed `RenderedDiagnostic` with an active code and no phase bridging.

### `diagnostic_with_source_range` — canonical span construction

`module_lowering.rs:98–124`:
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
        ...
    }
}
```

Uses canonical `SourceMap` / `DiagnosticBuilder` / `DiagnosticSink` pipeline. The error branch (lines 115–122) is an internal compiler panic diagnostic, not a fallback to an old phase code.

### `diagnostic_legacy_display` — test-only, never in production paths

`diagnostics.rs:109–115`:
```rust
#[cfg(test)]
pub(crate) fn diagnostic_legacy_display(diagnostic: &RenderedDiagnostic) -> String { ... }
```

Used only in:
- `tests/project_build_check.rs:343,347`
- `tests/single_file_frontend.rs:143,147`

These are all in test files, not production code. The `#[cfg(test)]` attribute gates it out of production binaries entirely.

### All diagnostic construction sites use active codes

Every call to `diagnostic_with_code` in the changed files passes a concrete active `DiagnosticCode` from the registry (e.g., `DiagnosticCode::FLOW_INVALID_CONDITION_TYPE`, `DiagnosticCode::TYPE_MISMATCH`, `DiagnosticCode::BUILD_MATERIALIZATION_FAILURE`, etc.). No phase-derived pseudo-codes, no legacy strings.

---

## 5. Tests cover project and test-runner span rendering for ranged HIR diagnostics

**Finding: Covered. No blockers.**

### `project_build_check.rs:53–77` — `test_check_project_reports_primary_span_for_ranged_hir_diagnostic`

- Creates a project with `if 1: pass` in `main.sifr`.
- Calls `check_project(&main_file)` to exercise project-mode lowering.
- Finds the `FLOW_INVALID_CONDITION_TYPE` diagnostic.
- Asserts the primary span:
  - `file` ends with `"main.sifr"` (correct file identity)
  - `line == Some(2)`, `column == Some(8)`, `end_line == Some(2)`, `end_column == Some(9)` — precise range for `1` in `if 1`.

### `test_runner.rs:339–396` — `test_run_tests_frontend_type_errors_use_single_path_prefix`

- Creates a test project with `if 1: pass` in `test_bad.sifr`.
- Calls `run_tests(&test_dir)` to exercise test-runner lowering.
- Validates:
  - All error messages contain `test_bad.sifr` (correct file attribution).
  - No double-prefixing with `] [test_bad]` pattern.
  - At least one error has `FLOW_INVALID_CONDITION_TYPE` code.
  - **That error has a primary span pointing at `test_bad.sifr` with `line == Some(4)`** — covers the test-runner primary span path.
  - No error is reclassified as `INTERNAL_COMPILER_PANIC`.

### No gaps identified

Both project-mode and test-runner paths are directly exercised with assertions on:
- file identity (`ends_with` check)
- line/column/end_line/end_column values
- active code presence (not internal/legacy)
- correct message attribution (no double-prefixing)

---

## Validation evidence reviewed

User confirmed locally run:
- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_driver test_check_project_reports_primary_span_for_ranged_hir_diagnostic -- --nocapture`
- `cargo test -p sifr_driver test_run_tests_frontend_type_errors_use_single_path_prefix -- --nocapture`
- `cargo test -p sifr_driver test_project_and_test_discovery_share_import_closure_membership -- --nocapture`
- `cargo clippy -p sifr_driver --no-deps -- -D warnings`

Additional `cargo clippy -p sifr_driver --all-targets --no-deps -- -D warnings` findings are pre-existing and outside this slice.

---

## Blocker assessment

**No blockers. Reviewer is satisfied.**

The slice correctly threads source text and display paths through project and test-runner discovery → lowering, renders HIR primary ranges through the canonical `SourceMap` pipeline, and produces clean `RenderedDiagnostic` values with active diagnostic codes throughout.

The only intentional spanless fallback exists for (a) the `#[cfg(test)]`-only `lower_frontend_module` path and (b) rangeless HIR diagnostics retained for future source-context slices — neither represents a production fallback left in the project or test-runner paths.
