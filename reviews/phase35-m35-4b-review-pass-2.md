

## Phase 35 m35.4b Review Pass 2 - Delivered

**SATISFIED** - No blockers identified.

### Focus Areas Reviewed

#### 1. `sifr_syntax::parse_module_suite` AST-only parser path

- **Implementation**: `parse_module_suite` (line 171-176 in `sifr_syntax/src/lib.rs`) delegates to `parse_module_raw` and extracts only the AST suite via `Parsed::into_suite`, avoiding token/trivia collection overhead
- **Routing verified**: All callers use `sifr_frontend::parse_source` → `sifr_syntax::parse_module_suite`:
  - CLI (`sifr/src/main.rs:153`): Detects project-mode entrypoints via `has_local_project_imports`
  - Driver test support (`sifr_driver/src/tests/support.rs:5`): Test helper
  - `sifr_frontend` (`lib.rs:348`): Canonical parse entrypoint
  - Driver project discovery (`project/discovery.rs:419`): `parse_import_closure_source_modules`

#### 2. CLI/project/test routing through `sifr_frontend`

- **check/run/emit**: `RootedEntrypointPlan::from_entrypoint` → `parse_source` or `parse_import_closure_source_modules` → `collect_project_hir_source_modules` → `compile_module_hir_with_source`
- **test**: `run_tests` → `build_test_runner_project` → `parse_import_closure_source_modules` → `compile_module_hir_with_source`
- **Driver frontend layer** (`frontend/api.rs`): Delegates to `sifr_frontend` (no duplicate semantics)
- **Deleted migration shims** (pass-1 confirmed): `module_lowering.rs`, `parser_diagnostics.rs`, `exports.rs` removed

#### 3. `verification/performance/run_benchmarks.py` RSS measurement

- **Primary path**: `/usr/bin/time -l` (Darwin) or `-v` (Linux) via `timed_command()` wrapper
- **Fallback**: `resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss` with platform-specific normalization
- **Logic**: `parse_peak_rss` extracts from stderr → if None, use RUSAGE fallback (lines 416-419)
- **Correctness**: Both Darwin and Linux output formats handled; fallback covers cases where `/usr/bin/time` unavailable

#### 4. Split-brain/frontend ownership remains sound

- **Guardrail**: No `sifr_python_parser`/`ruff_python_parser` direct use outside `sifr_syntax`/`sifr_frontend`/`sifr_hir`/`tests`
- **Verified**:
  - `check_split_brain_guardrail.py`: PASS
  - `check_split_brain_guardrail.py --self-test`: PASS
  - No driver/CLI migration allowlist exists

### Validation Results

| Check | Result |
|-------|--------|
| Split-brain guardrail | PASS |
| `cargo check` (sifr_syntax, sifr_frontend, sifr_driver, sifr) | PASS |
| `cargo clippy` (pedantic) | PASS |
| `cargo test -p sifr_frontend --lib` | 3/3 PASS |
| `cargo test -p sifr_driver project_build_check` | 16/16 PASS |
| Benchmark self-test | PASS |
| `sifr check demos/borrowed_builtins/main.sifr` | "no errors found" |
| `sifr emit demos/branch_paths/main.sifr` | Generates valid Rust |
