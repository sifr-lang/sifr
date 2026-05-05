# Review: `milestone_diag_4b` slice 1 — phase-mapping retirement (pass 2 retry)

- Branch: `codex/diag-4b-phase-mapping-retirement`
- Phase issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
- Pass 1 review: [reviews/semantic-diagnostic-code-taxonomy-diag-4b-phase-mapping-retirement-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4b-phase-mapping-retirement-review-pass-1.md)
- Slice scope (unchanged): delete the public `CompilePhase` enum and the `phase` field from `CompileError`; remove phase-derived panic-boundary plumbing; preserve the legacy human labels (`parse error`, `type error`, `codegen error`, `build error`, `internal compiler error`) by deriving them from the active `DiagnosticCode` family. Retiring `CompileError` itself and the `sifr_driver` re-exports stays out of scope.
- This is a re-run of pass 2 against the current uncommitted diff after pass 1 fixes were applied (the previous pass 2 attempt timed out before writing a file).

Validation evidence the implementer reported for the pass 1 fix-ups:
- `cargo fmt`
- `cargo test -p sifr_driver --lib --tests`

Earlier validation for the same slice before pass 1 fixes:
- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr --test e2e test_e2e_fail`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`

I independently re-ran:
- `cargo fmt --check` (clean).
- `cargo test -p sifr_driver --lib --tests` — 101 passed, 0 failed; the new `test_compile_error_labels_are_derived_from_diagnostic_codes` and the renamed `test_run_codegen_with_boundary_reports_string_panic_as_internal_compiler_panic` / `..._reports_non_string_payload_as_internal_compiler_panic` all pass.

I did not re-run the `sifr` e2e suite or `--workspace` clippy; AGENTS.md still puts `scripts/run_all_tests.sh` on the user before merge.

## Verdict

Pass 1's three required findings (F1, F2, F4) are addressed. F3 is informational and the implementer has agreed to call it out in the PR description.

The slice is mechanically correct, focused, and ready to merge from my perspective. There are a couple of low-priority cosmetic observations below; none block the slice. The next slice (residual `CompileError` retirement) can build on this without sitting on top of dead public surface.

## Pass 1 finding-by-finding verification

### F1. Duplicated label resolution between driver and CLI — ✅ resolved

- The driver still owns the canonical helper but now exposes both shapes:
  - [crates/sifr_driver/src/diagnostics.rs:194-216](../crates/sifr_driver/src/diagnostics.rs:194)
    - `compile_error_label_for_code(code: DiagnosticCode)` for typed callers (used by `Display for CompileError` at [diagnostics.rs:183-192](../crates/sifr_driver/src/diagnostics.rs:183) and the new label test).
    - `compile_error_label_for_code_str(code: &str)` does the actual matching, against `DiagnosticCode::INTERNAL_COMPILER_PANIC.code()`, `STDLIB_BOOTSTRAP_FAILURE.code()`, `STDLIB_CACHE_FAILURE.code()` and the family prefixes.
- Both are re-exported from the driver: [crates/sifr_driver/src/lib.rs:23-28](../crates/sifr_driver/src/lib.rs:23).
- The CLI now consumes the `&str` variant in `render_compile_errors`:
  - [crates/sifr/src/main.rs:11-16](../crates/sifr/src/main.rs:11) imports `compile_error_label_for_code_str`.
  - [crates/sifr/src/main.rs:367-388](../crates/sifr/src/main.rs:367) — for any `diagnostic.code` starting with `SIFR-`, the label is delegated to the driver helper; non-SIFR codes still fall through to severity-based labels.
- The hard-coded strings `"SIFR-INTERNAL-0001"`, `"SIFR-STDLIB-0003"`, `"SIFR-STDLIB-0004"` and the parallel prefix branches are gone from the CLI.

Renumbering or family relabeling now requires only one edit, in `compile_error_label_for_code_str`. F1's regression mode (driver/CLI silently disagreeing on labels) is closed.

### F2. No tests pin down the new label contract — ✅ resolved

- New table-driven test at [crates/sifr_driver/src/tests/diagnostics.rs:20-54](../crates/sifr_driver/src/tests/diagnostics.rs:20) covers exactly the matrix recommended in pass 1 plus a `CompileError::Display` round-trip:

  | Code | Asserted label |
  | --- | --- |
  | `INTERNAL_COMPILER_PANIC` | `internal compiler error` |
  | `STDLIB_BOOTSTRAP_FAILURE` | `build error` |
  | `STDLIB_CACHE_FAILURE` | `build error` |
  | `STDLIB_UNSUPPORTED_SURFACE` | `type error` |
  | `STDLIB_ARGUMENT_TYPE_MISMATCH` | `type error` |
  | `WORKSPACE_MALFORMED_MANIFEST` | `build error` |
  | `WORKSPACE_UNRESOLVED_IMPORT` | `build error` |
  | `WORKSPACE_IMPORT_CYCLE` | `build error` |
  | `PARSE_EXPECTED_TOKEN_OR_RECOVERY` | `parse error` |
  | `CODEGEN_BACKEND_FAILURE` | `codegen error` |
  | `BUILD_MATERIALIZATION_FAILURE` / `BUILD_RUSTC_OR_CARGO_FAILURE` / `BUILD_TEMP_WORKSPACE_FAILURE` / `BUILD_CARGO_MANIFEST_FAILURE` / `BUILD_ARTIFACT_MISSING` | `build error` |
  | `TYPE_MISMATCH` | `type error` |

- For each row the test also constructs a `CompileError` and asserts `to_string()` equals `format!("{label}: message")`, which exercises the previously-untested `Display` impl end-to-end.
- I verified `cargo test -p sifr_driver --lib --tests test_compile_error_label` passes.

This locks down both the helper output and the `Display`-based render shape against future code-family additions, matching pass 1's recommendation.

### F3. Internal-panic / stdlib-bootstrap label change — ℹ️ informational

The implementer has acknowledged this will be called out in the PR description. No test or fixture relies on the prior labels (verified again: `grep -rn "type error:\|parse error:\|codegen error:\|build error:" crates/sifr/tests/e2e/fail crates/sifr_driver/src/tests` returns nothing). No code action needed.

### F4. Test name drift in `panic_boundary.rs` — ✅ resolved

[crates/sifr_driver/src/tests/panic_boundary.rs](../crates/sifr_driver/src/tests/panic_boundary.rs):

- `test_run_codegen_with_boundary_reports_string_panic_as_codegen_error` → `..._as_internal_compiler_panic` ([panic_boundary.rs:5](../crates/sifr_driver/src/tests/panic_boundary.rs:5)).
- `test_run_codegen_with_boundary_reports_non_string_payload` → `..._reports_non_string_payload_as_internal_compiler_panic` ([panic_boundary.rs:15](../crates/sifr_driver/src/tests/panic_boundary.rs:15)).

The function-name now matches what the body asserts (`assert_eq!(err.code, DiagnosticCode::INTERNAL_COMPILER_PANIC)`), so CI logs and `cargo test` filters won't mislead future readers.

## Other things I checked this pass

### Diff scope

- `git diff --stat` covers 22 files: the source removal of `CompilePhase` plus its callers in driver and CLI, three doc updates (`diagnostic_emission_inventory.md`, `phases/22_frontend_mode_parity_hardening.md`, the diag taxonomy issue), and the four test files. No unrelated edits sneaked in.
- `grep -rn "CompilePhase\|phase: " crates/ --include="*.rs"` returns no results — the type and the field are fully retired from compiled Rust source.

### Internal usage of the `_str` helper

- `compile_error_label_for_code_str` is used both by the driver (`compile_error_label_for_code` delegates to it) and by the CLI render path; `compile_error_label_for_code` is used by the driver `Display` impl and the new test. Both helpers therefore have callers — no transient dead public surface from the F1 fix.
- The CLI's `if diagnostic.code.starts_with("SIFR-")` guard at [main.rs:372](../crates/sifr/src/main.rs:372) is safe even though every `CompileError`-derived `CompilerDiagnostic` already starts with `SIFR-`: tests construct synthetic `CompilerDiagnostic` values directly, and the guard preserves the historic severity-based fallthrough for non-SIFR codes. Not a regression.

### Doc / status alignment

- [internal_docs/diagnostic_emission_inventory.md:84](../internal_docs/diagnostic_emission_inventory.md:84) — narrative is updated to "carries an active `DiagnosticCode`, and legacy human labels are derived from canonical code families."
- [:88-102](../internal_docs/diagnostic_emission_inventory.md:88) — surface table rows for `frontend/api.rs`, `frontend/module_lowering.rs`, `project/compile_order.rs`, `project/frontend.rs`, and `crates/sifr/src/main.rs` no longer mention `CompilePhase::*`. The `crates/sifr/src/main.rs` count moves from `3 → 1`, which matches the diff (the panic boundary is the only remaining CLI construction site).
- [:117](../internal_docs/diagnostic_emission_inventory.md:117) — "Phase-derived `CompilePhase` mapping and display labels" is correctly reframed as "removed; … human labels are code-derived".
- [internal_docs/phases/22_frontend_mode_parity_hardening.md:43](../internal_docs/phases/22_frontend_mode_parity_hardening.md:43) — replaces the literal `{phase}: {message}` formatter description with "code-derived human labels".
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11) and [:70](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:70) advance the wave/slice tracker to `milestone_diag_4b` and mark slice 1 in progress. The retroactive past-tense rewording at [:213-220](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:213) is consistent.
- DoD targets at [:1110-1111](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1110) (`No public diagnostic code is assigned from CompilePhase`, `CompilePhase is not a public diagnostic display source`) are now satisfied.

### Stale phase-derived runtime paths

- `grep -rn "{phase}: {message}\|CompilePhase::Build\|CompilePhase::Parse\|CompilePhase::TypeCheck\|CompilePhase::Codegen" .` finds nothing in source.
- The `crates/sifr_driver/src/tests/project_build_check.rs` match for `phase` is the unrelated string `"phase-five"` inside a `toml` decoder test fixture.

### Validation re-run

- `cargo fmt --check` — clean.
- `cargo test -p sifr_driver --lib --tests` — 101 passed, 0 failed. The new label test runs in `tests::diagnostics::test_compile_error_labels_are_derived_from_diagnostic_codes`; the panic-boundary tests run under `tests::panic_boundary::*`. No regressions visible.

## Minor non-blocking observations

These are cosmetic — feel free to leave for a follow-up. None affect correctness or test signal.

1. **Stale `.expect_err` text in `panic_boundary.rs`.** [panic_boundary.rs:9](../crates/sifr_driver/src/tests/panic_boundary.rs:9) and [:19](../crates/sifr_driver/src/tests/panic_boundary.rs:19) still say `"panic should be converted into a codegen error"`. The test names were correctly updated to mention `internal_compiler_panic`, but the panic message strings inside the bodies still describe the converted error as a "codegen error". The function under test is still `run_codegen_with_boundary` so it's not strictly wrong, but it would read better as `"panic should be converted into an internal compiler panic"` to align with the test name and the asserted code. Single-line edit if you want to land it; otherwise harmless.

2. **`compile_error_label_for_code(DiagnosticCode)` has no external callers.** Only `Display for CompileError` and the new test use it; the CLI uses the `&str` variant. The pass 1 review flagged transient public helpers without callers. With the F1 fix, both helpers exist for a reason — the typed variant is the natural public API for code-aware callers — so this is more of a preference than a defect. Since the next slice retires `CompileError` as a public abstraction (and these helpers go with it), I would not bother trimming the typed variant now; it's deleted in the follow-up regardless.

3. **CLI `if diagnostic.code.starts_with("SIFR-")` guard.** In production all `CompileError`-derived `CompilerDiagnostic` values carry an `SIFR-*` code, so the `else` severity-derived branch in `render_compile_errors` is currently only reachable from the renderer's own snapshot tests that construct `CompilerDiagnostic` literals. That's fine and arguably good defensive coding, but worth noting if a follow-up cleanup wants to simplify the renderer once `CompilerDiagnostic` is retired.

## Coherence checks (re-confirmed)

- **Stdlib classification:** `STDLIB_UNSUPPORTED_SURFACE` / `STDLIB_ARGUMENT_TYPE_MISMATCH` (user-facing) → `type error`; `STDLIB_BOOTSTRAP_FAILURE` / `STDLIB_CACHE_FAILURE` (compiler infra) → `build error`. The new test pins both behaviors.
- **`CompilePhase` removal from public re-export:** the issue's pre-1.0 no-compatibility decision applies. No in-tree consumer references `sifr_driver::CompilePhase`.
- **Stale phase-derived paths:** none in source; doc strings either updated or describe historical context (e.g., the `issues/archive/*` files, which intentionally remain frozen).
- **`CompileError` outer transport:** still present, still re-exported, deliberately preserved for the next slice. The slice scope explicitly excludes its retirement.

## Required actions

None. Pass 1 findings F1, F2, and F4 are resolved; F3 is acknowledged for the PR description.

## Recommended (optional) follow-ups

- Rename the two `.expect_err("panic should be converted into a codegen error")` strings in `panic_boundary.rs` to match the renamed test names. Trivially small.
- Consider whether the next slice can also collapse the CLI's `starts_with("SIFR-")` guard, once `CompilerDiagnostic` is retired and renderer tests are updated.

Once the implementer is satisfied with the PR description (F3 callout) and the optional cosmetics, I see no blockers to merging slice 1 and proceeding to the residual `CompileError` retirement slice.
