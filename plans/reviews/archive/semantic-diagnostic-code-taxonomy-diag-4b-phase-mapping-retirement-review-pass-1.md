# Review: `milestone_diag_4b` slice 1 — phase-mapping retirement (pass 1)

- Branch: `codex/diag-4b-phase-mapping-retirement`
- Phase issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
- Slice scope: delete the public `CompilePhase` enum and the `phase` field from `CompileError`; remove phase-derived panic-boundary plumbing; preserve the legacy human labels (`parse error`, `type error`, `codegen error`, `build error`, `internal compiler error`) by deriving them from the active `DiagnosticCode` family.
- Out of scope (next slices): retiring `CompileError` itself, removing `sifr_driver` re-exports of `sifr_diagnostics` types.

Validation evidence the implementer reported:
- `cargo fmt --check`
- `git diff --check`
- `cargo test -p sifr_driver --lib --tests`
- `cargo test -p sifr --test e2e test_e2e_fail`
- `cargo test -p sifr -- --skip test_e2e_pass`
- `cargo clippy --workspace -- -D warnings`

I did not re-run these — this review is a code/diff read, not an independent test run. The user is responsible for `scripts/run_all_tests.sh` per AGENTS.md before merge.

## Verdict

The slice is correctly scoped and mechanically clean. No source file in the workspace still references `CompilePhase` ([crates/sifr_driver/src/diagnostics.rs:26](../crates/sifr_driver/src/diagnostics.rs:26), confirmed by `grep -rn "CompilePhase\|\.phase" crates/`). The 30+ call sites that previously passed a phase to `CompileError::with_code` have all been updated, and no caller still touches a `phase` field. The new label helper preserves legacy human labels for parser/type/codegen/build/workspace/stdlib/internal codes.

Behavioral changes are user-visible but small, and on balance they are corrections rather than regressions. No e2e fixture or test depends on the old labels.

There are four findings to address before declaring the slice complete; none block merging the cleanup, but two of them should be fixed in this slice rather than deferred. Details below.

## What was checked

- Removal site: `CompilePhase` enum, `CompileError.phase` field, `compile_error_label_for_code` helper, updated `Display` ([crates/sifr_driver/src/diagnostics.rs](../crates/sifr_driver/src/diagnostics.rs)).
- Public re-exports: [crates/sifr_driver/src/lib.rs:23-27](../crates/sifr_driver/src/lib.rs:23).
- All `CompileError::with_code` and `CompileError { ... }` construction sites in `crates/sifr_driver/src/{frontend,project,build,stdlib,test_runner,workspace}` and in [crates/sifr/src/main.rs](../crates/sifr/src/main.rs).
- Panic-boundary plumbing in `run_codegen_with_boundary` ([crates/sifr_driver/src/diagnostics.rs:239](../crates/sifr_driver/src/diagnostics.rs:239)) and CLI `run_with_panic_boundary` ([crates/sifr/src/main.rs:244](../crates/sifr/src/main.rs:244)).
- Driver tests: [tests/diagnostics.rs](../crates/sifr_driver/src/tests/diagnostics.rs), [tests/panic_boundary.rs](../crates/sifr_driver/src/tests/panic_boundary.rs), [tests/single_file_frontend.rs](../crates/sifr_driver/src/tests/single_file_frontend.rs).
- CLI render path: `render_compile_errors` ([crates/sifr/src/main.rs:367](../crates/sifr/src/main.rs:367)) and `compile_error_exit_code` ([crates/sifr/src/main.rs:262](../crates/sifr/src/main.rs:262)).
- Code-string mapping vs. canonical constants: [crates/sifr_diagnostics/src/codes.rs:99-126](../crates/sifr_diagnostics/src/codes.rs:99).
- Doc updates: [internal_docs/diagnostic_emission_inventory.md](../internal_docs/diagnostic_emission_inventory.md), [internal_docs/phases/22_frontend_mode_parity_hardening.md](../internal_docs/phases/22_frontend_mode_parity_hardening.md), [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md).
- Static checks: `grep -rn "CompilePhase" crates/` returns nothing in source; `grep -rn "\.phase" crates/` returns no `CompileError`-related accesses; `grep -rn "compile_error_label_for_code" crates/` shows the helper is defined and re-exported but no external caller invokes it.

Note: I did not check the `sifr_python_parser` submodule or `verification/` because the slice only touches driver/CLI.

## Findings

### F1. `CompileError` label resolution is duplicated between driver and CLI (action required)

The driver introduced a new helper:

- [crates/sifr_driver/src/diagnostics.rs:194-217](../crates/sifr_driver/src/diagnostics.rs:194)
  ```rust
  pub fn compile_error_label_for_code(code: DiagnosticCode) -> &'static str {
      if code == DiagnosticCode::INTERNAL_COMPILER_PANIC { "internal compiler error" }
      else if code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE
           || code == DiagnosticCode::STDLIB_CACHE_FAILURE { "build error" }
      else { compile_error_label_for_code_str(code.code()) }
  }
  ```

The CLI re-implements the same rules with hard-coded code strings:

- [crates/sifr/src/main.rs:372-389](../crates/sifr/src/main.rs:372)
  ```rust
  let label = match diagnostic.code.as_str() {
      "SIFR-INTERNAL-0001" => "internal compiler error",
      "SIFR-STDLIB-0003" | "SIFR-STDLIB-0004" => "build error",
      ...
  };
  ```

This is dual-source-of-truth logic. Today the rules agree, but:

- If `SIFR-INTERNAL-0001` / `SIFR-STDLIB-0003` / `SIFR-STDLIB-0004` are renumbered in [crates/sifr_diagnostics/src/codes.rs](../crates/sifr_diagnostics/src/codes.rs), the driver helper continues to work (it uses enum equality) but the CLI silently breaks.
- Future label policy changes (e.g., adding a `SIFR-OWN-` family with its own label) require touching both files.
- The driver re-exports `compile_error_label_for_code` publicly ([crates/sifr_driver/src/lib.rs:24](../crates/sifr_driver/src/lib.rs:24)) but no external caller uses it. Making the CLI route through the helper would consume that export and remove the duplication.

Recommendation (pick one):
1. Expose `compile_error_label_for_code_str(&str) -> &'static str` (currently `fn` private) or rename the existing helper to take `&str`, and have `render_compile_errors` call it.
2. Replace the CLI's hard-coded strings with `DiagnosticCode::INTERNAL_COMPILER_PANIC.code()` / `STDLIB_BOOTSTRAP_FAILURE.code()` / `STDLIB_CACHE_FAILURE.code()` so the canonical-code string is the source of truth, even if the matching logic stays inline.

Either fix should land in this slice — the divergence is a regression in maintainability that this slice introduced, and the helper without a caller is dead public surface.

### F2. No tests pin down the new label contract (action recommended)

The slice's stated purpose is "preserve existing human labels through canonical diagnostic-code families rather than phases." That contract has no unit tests. Specifically:

- `compile_error_label_for_code` (driver) has no test coverage.
- `CompileError`'s updated `Display` impl has no test coverage (no caller in this repo invokes `to_string()` on `CompileError` either, so the impl is unverified end-to-end).
- The CLI's inline match in `render_compile_errors` has no unit test that asserts the emitted `<label>: <message>` text.

Existing tests assert the diagnostic *code* (e.g. [tests/diagnostics.rs:8-18](../crates/sifr_driver/src/tests/diagnostics.rs:8) and the rewritten panic-boundary tests), which is correct but does not lock down the label-preservation guarantee. A handful of cheap table-driven tests would prevent silent label regressions when codes are added/renamed:

- `INTERNAL_COMPILER_PANIC` → `internal compiler error`
- `STDLIB_BOOTSTRAP_FAILURE`, `STDLIB_CACHE_FAILURE` → `build error`
- `STDLIB_UNSUPPORTED_SURFACE`, `STDLIB_ARGUMENT_TYPE_MISMATCH` → `type error`
- `WORKSPACE_MALFORMED_MANIFEST`, `WORKSPACE_UNRESOLVED_IMPORT`, `WORKSPACE_IMPORT_CYCLE` → `build error`
- `PARSE_EXPECTED_TOKEN_OR_RECOVERY` → `parse error`
- `CODEGEN_BACKEND_FAILURE` → `codegen error`
- `BUILD_MATERIALIZATION_FAILURE`, `BUILD_RUSTC_OR_CARGO_FAILURE`, `BUILD_TEMP_WORKSPACE_FAILURE`, `BUILD_CARGO_MANIFEST_FAILURE`, `BUILD_ARTIFACT_MISSING` → `build error`
- `TYPE_MISMATCH` → `type error`

This is the sort of regression detection that's much cheaper to add now than to track down later when a downstream fixture starts failing.

### F3. CLI render path emits a different label for internal panics than before (intentional change — flag in PR description)

This is not a bug, but it should be called out so reviewers see it.

Old behavior, prior to this slice:
- `CompileError::Display`: `"codegen error: ..."` (when caller passed `CompilePhase::Codegen`) or `"build error: ..."` (when caller passed `CompilePhase::Build`) — even though `code` was always `INTERNAL_COMPILER_PANIC`.
- CLI `render_compile_errors`: code starts with `SIFR-INTERNAL-` matched no branch and fell through to severity-derived `"error: ..."`.

New behavior:
- Both `Display` and CLI render emit `"internal compiler error: ..."` for `INTERNAL_COMPILER_PANIC`.

Effect on user-visible output:
- Internal panics now render with a more descriptive label and the two output paths now agree (an improvement; previously `Display` and the CLI disagreed for the same error).
- Stdlib bootstrap failures previously rendered as `"parse error: [stdlib:X] ..."` or `"type error: [stdlib:X] ..."` depending on which sub-step failed. They now uniformly render as `"build error: [stdlib:X] ..."`. This is consistent with the user's stated coherence question — bootstrap is build infrastructure — but it's a label change.

No fixture or test in `crates/sifr/tests/e2e/` or `crates/sifr_driver/src/tests/` asserts these labels, so no test breaks. Worth one line in the PR description.

### F4. Test name drift in `panic_boundary.rs` (action: rename or accept)

After the change, [tests/panic_boundary.rs:4](../crates/sifr_driver/src/tests/panic_boundary.rs:4) reads:

```rust
fn test_run_codegen_with_boundary_reports_string_panic_as_codegen_error() {
    ...
    assert_eq!(err.code, DiagnosticCode::INTERNAL_COMPILER_PANIC);
```

The function name asserts "codegen error" but the body asserts `INTERNAL_COMPILER_PANIC`. The name is now misleading. Either:
- Rename to `..._reports_string_panic_as_internal_compiler_panic` and the same for the non-string-payload sibling.
- Accept the cosmetic drift; the assertions are correct.

I'd rename — the test names show up in CI logs and `cargo test` filters, and the inconsistency will mislead future readers.

## Coherence checks the user explicitly asked about

### Stdlib classification

- `STDLIB_UNSUPPORTED_SURFACE` (`SIFR-STDLIB-0001`) and `STDLIB_ARGUMENT_TYPE_MISMATCH` (`SIFR-STDLIB-0002`) are emitted from HIR builtin lowering ([crates/sifr_hir/src/lower/builtin_calls.rs](../crates/sifr_hir/src/lower/builtin_calls.rs)) for *user code* that calls unsupported stdlib surfaces or passes wrong-typed args. They fall into the `else` branch of `compile_error_label_for_code_str` → `"type error"`. This is coherent: from the user's perspective, those are semantic errors in user code.
- `STDLIB_BOOTSTRAP_FAILURE` (`SIFR-STDLIB-0003`) and `STDLIB_CACHE_FAILURE` (`SIFR-STDLIB-0004`) are constructed in [crates/sifr_driver/src/stdlib/bootstrap.rs](../crates/sifr_driver/src/stdlib/bootstrap.rs) and [crates/sifr_driver/src/stdlib/cache.rs](../crates/sifr_driver/src/stdlib/cache.rs) and represent compiler-internal failures while loading or caching the stdlib. The helper special-cases them to `"build error"`. This is also coherent: those failures are part of the build/load infrastructure, not user-source semantic checking. The previous behavior of labeling them `"parse error"` or `"type error"` based on the failing sub-step was misleading because the user can't act on a parse error in the embedded stdlib — it's always a compiler/build issue from their perspective.

### Removing `CompilePhase` from the public re-export

- The phase plan explicitly states: "Sifr is not production-released yet. This phase intentionally does not preserve compatibility for the existing diagnostic surface" ([issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:8](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:8)).
- The `milestone_diag_4b` DoD includes "`CompilePhase` is not a public diagnostic display source" ([:1111](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:1111)). Removing it from `pub use diagnostics::{ ... }` satisfies that DoD line.
- I did not search outside this repo, but no in-tree consumer references `sifr_driver::CompilePhase`. The removal is acceptable for this pre-1.0 compiler.

### Stale phase-derived paths in Rust source

- `grep -rn "CompilePhase\|\.phase" crates/` reports zero matches in source. Other matches are unrelated (`/* two-phase compilation */` comments, fixture names like `phase-five`).
- `grep -rn "{phase}: {message}" .` finds no occurrences. The phase-derived `Display` format string is gone.

### Doc updates

The doc updates are sufficient for the slice:
- [internal_docs/diagnostic_emission_inventory.md:84](../internal_docs/diagnostic_emission_inventory.md:84) and [:117](../internal_docs/diagnostic_emission_inventory.md:117) describe that `CompilePhase` is removed and labels are code-derived. The per-surface table rows for `frontend/api.rs`, `frontend/module_lowering.rs`, `project/compile_order.rs`, `project/frontend.rs`, and `crates/sifr/src/main.rs` are updated to drop the `CompilePhase::*` source descriptions. Construction count for `crates/sifr/src/main.rs` is updated `3 → 1`, which matches the diff (only the panic-boundary site remains; the previous three CLI sites passed `CompilePhase` but their construction was the panic boundary).
- [internal_docs/phases/22_frontend_mode_parity_hardening.md:43](../internal_docs/phases/22_frontend_mode_parity_hardening.md:43) drops the literal `{phase}: {message}` format and replaces it with "code-derived human labels".
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:11) and [:70](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:70) advance the wave/slice tracker.
- The issue body retroactively rewords the original "implementation root cause" paragraph into past tense ([:213-220](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:213)). Reasonable.

What is *not* updated, and which I think is fine:
- `internal_docs/architecture.md` and `internal_docs/roadmap.md` do not reference `CompilePhase` and do not need touching.
- Historical references in `reviews/archive/`, `reviews/*-review-pass-*.md`, and `issues/archive/*` should remain; they are time-stamped commentary on prior states.

## Other observations (non-blocking)

- `stdlib/bootstrap.rs:203-207` ([here](../crates/sifr_driver/src/stdlib/bootstrap.rs:203)) and `test_runner/orchestrator.rs:107-111` ([here](../crates/sifr_driver/src/test_runner/orchestrator.rs:107)) construct `CompileError` via raw struct literal:
  ```rust
  CompileError {
      code: e.code,
      message: format!("[stdlib:{module_name}] {}", e.message),
  }
  ```
  Equivalent to `CompileError::with_code(format!(...), e.code)`. Both forms work; the rest of the migration uses `with_code`. Cosmetic only — feel free to leave.
- The orchestrator's previous behavior was buggy: it overrode forwarded `error.phase` to `CompilePhase::TypeCheck` regardless of whether the underlying error was an `INTERNAL_COMPILER_PANIC` (from a `LoweringError` with no canonical code). The new code drops the phase override and preserves the inner `error.code`. So an internal panic surfacing through HIR lowering of a test module now correctly labels as "internal compiler error" rather than masquerading as "type error". Silent improvement.
- Module-lowering tests at [frontend/module_lowering.rs:90-117](../crates/sifr_driver/src/frontend/module_lowering.rs:90) already cover the "uncoded `LoweringError` is reclassified as `INTERNAL_COMPILER_PANIC` with a synthetic message" path — those tests survived the migration unchanged because they assert on `code`, not phase.

## Summary of recommended actions

Required before I'd consider the slice complete:
1. **F1** — Eliminate the duplicated label-resolution logic between `compile_error_label_for_code` and the CLI's inline match. Either route the CLI through a `&str`-taking helper, or replace the hard-coded `"SIFR-INTERNAL-0001"` / `"SIFR-STDLIB-0003"` / `"SIFR-STDLIB-0004"` strings with `DiagnosticCode::*.code()`.

Recommended:
2. **F2** — Add unit tests for `compile_error_label_for_code` covering each family the helper claims to preserve.
3. **F4** — Rename the two `..._reports_string_panic_as_codegen_error` / `..._reports_non_string_payload` tests so they don't claim "codegen error" while asserting `INTERNAL_COMPILER_PANIC`.

Informational:
4. **F3** — In the PR description, call out that internal-panic and stdlib-bootstrap rendering labels changed (more descriptive; no test or fixture relies on the old labels).

Once F1, F2, F4 are addressed, this slice is ready and the next slice (residual `CompileError` retirement) can proceed without sitting on top of a transient public helper that nobody calls.
