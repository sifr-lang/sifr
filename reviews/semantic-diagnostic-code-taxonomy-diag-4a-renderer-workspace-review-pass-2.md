# Review: milestone_diag_4a — Renderer + Workspace-Inference Slice (Pass 2)

Branch: `codex/semantic-diagnostics-diag-4a` (uncommitted working tree on top of `73b4e32c`)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Pass-1 review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-renderer-workspace-review-pass-1.md)

## Verdict

**Pass-1 actionable findings are addressed; the slice is mergeable subject to one explicit deferral and a handful of polish items.** The five touched-but-uncoded sites called out in pass-1 R3 now carry `INTERNAL_COMPILER_PANIC` or `BUILD_MATERIALIZATION_FAILURE`. Materialize cargo failures now route through `BUILD_RUSTC_OR_CARGO_FAILURE`, restoring symmetry with the test-runner cargo path. The workspace source-root code selection is now driven by a typed `SourceRootErrorKind` enum, removing the message-string match. Presentation tests cover internal (no-primary-span) diagnostics and a mixed Error/Warning/Note compact summary. Driver diagnostics tests no longer positively assert retired phase-bucket codes.

The legacy `CompileError::diagnostic_code` bridge still maps every unmigrated phase to a **retired** code (`SIFR-PARSE-0001`, `SIFR-TYPE-0001`, `SIFR-CODEGEN-0001`, `SIFR-BUILD-0001`). The implementer's note proposes deferring the repointing/deletion to the later HIR-transport / `TypeCheck` wave. **For this slice that deferral is acceptable** — see the dedicated section below — provided the deferral is recorded explicitly in the issue and the next slice is the one that lands HIR transport with fixture re-keying.

## Pass-1 finding fidelity

| Pass-1 finding | Status in pass 2 | Evidence |
| --- | --- | --- |
| R1 — bridge emits retired codes | **Deferred (acceptable)** | [diagnostics.rs:125-138](../crates/sifr_driver/src/diagnostics.rs:125) retains the 4-arm fallback with a transitional comment. Tests no longer pin retired codes through `with_code` calls — see N1 below. |
| R2 — `materialize.rs` cargo failures | ✓ **Fixed** | [materialize.rs:144-158](../crates/sifr_driver/src/build/materialize.rs:144) splits `build_error` (filesystem → `BUILD_MATERIALIZATION_FAILURE`) from `cargo_build_error` (cargo → `BUILD_RUSTC_OR_CARGO_FAILURE`). [materialize.rs:117](../crates/sifr_driver/src/build/materialize.rs:117) and [materialize.rs:124](../crates/sifr_driver/src/build/materialize.rs:124) route through `cargo_build_error`. Symmetric with [test_runner/execution.rs:131-137](../crates/sifr_driver/src/test_runner/execution.rs:131). |
| R3 — touched-but-uncoded sites | ✓ **5 of 6 fixed** | [build/entrypoint.rs:208-221](../crates/sifr_driver/src/build/entrypoint.rs:208) → `INTERNAL_COMPILER_PANIC`. [project/discovery.rs:395-401](../crates/sifr_driver/src/project/discovery.rs:395) → `BUILD_MATERIALIZATION_FAILURE`. [project/frontend.rs:27-32](../crates/sifr_driver/src/project/frontend.rs:27) → `INTERNAL_COMPILER_PANIC`. [test_runner/orchestrator.rs:88-97](../crates/sifr_driver/src/test_runner/orchestrator.rs:88) → `INTERNAL_COMPILER_PANIC`. The orchestrator forwarder at [orchestrator.rs:107-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:107) is intentionally left as struct literal — see N2 below. |
| R4 — workspace source-root match | ✓ **Fixed** | [workspace/mod.rs:177-208](../crates/sifr_driver/src/workspace/mod.rs:177) introduces `SourceRootErrorKind::{Escapes, Invalid, NotDirectory}`, with `code()` and `reason()` methods, and replaces the message-string match with typed dispatch. Existing baselines and `test_source_roots_reject_escape_absolute_empty_missing_and_file_paths` still pass because the `reason()` strings preserve the substrings the tests look for. |
| R5 — single-bucket parse mapping | **Deferred (acceptable, no TODO marker)** | Three sites still funnel every Ruff parse failure into `PARSE_EXPECTED_TOKEN_OR_RECOVERY` — [discovery.rs:402-426](../crates/sifr_driver/src/project/discovery.rs:402), [frontend/api.rs:21-38](../crates/sifr_driver/src/frontend/api.rs:21), [stdlib/bootstrap.rs:33-50](../crates/sifr_driver/src/stdlib/bootstrap.rs:33). No `TODO(diag_4a)` marker was added. Pass-1 R5 explicitly accepted this if the parser-transport wave is next; if it isn't, a one-line marker is still cheap insurance. |
| R6 — renderer test coverage | ✓ **Fixed** | [presentation.rs:294-336](../crates/sifr_diagnostics/src/render/presentation.rs:294) adds a single multi-purpose test covering internal (no-primary-span) diagnostics, a mixed Error/Warning/Note compact summary header, and `help` propagation through `render_sink_human`. The "two diagnostics share `(severity, code, message_template)` but both are internal" grouping case is *not* covered — minor, tracked as N3 below. |
| R7 — `is_internal_compile_error` prefix fallback | ✓ **Effectively neutralized** | Every production site that emits `"internal compiler panic during ..."` now also carries `INTERNAL_COMPILER_PANIC` (see [main.rs:244-258](../crates/sifr/src/main.rs:244) and [diagnostics.rs:253-266](../crates/sifr_driver/src/diagnostics.rs:253)). The prefix branch in [main.rs:260-265](../crates/sifr/src/main.rs:260) is now dead code in production but is exercised by the synthetic-error test at [main.rs:1212-1224](../crates/sifr/src/main.rs:1212). See N4. |
| R8 — `compile_order` cycle phase change | ✓ **Unchanged from pass 1** | [compile_order.rs:191-197](../crates/sifr_driver/src/project/compile_order.rs:191) keeps the new `WORKSPACE_IMPORT_CYCLE` assignment. |

## Decision: is deferring the bridge repoint and `TypeCheck` deletion acceptable for this slice?

**Acceptable, but the deferral is *not* yet recorded honestly in the issue file.** Two facts are in tension:

1. The issue's plan explicitly says diag_4a deletes the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` arm so any missed TypeCheck path becomes a build/validation failure rather than silently using a fallback ([issue line 1124, slice description](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md#L1124)).
2. The slice keeps the arm intact at [diagnostics.rs:132-137](../crates/sifr_driver/src/diagnostics.rs:132) because production fixtures (e.g. [decimal_invalid_literal/baselines/check-json.stderr.txt](../crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt) emits `SIFR-TYPE-0001`) and 80+ e2e fail fixtures depend on the bridge until HIR `LoweringError` carries a structured code.

Bundling the bridge deletion with HIR transport in the next slice is the right scoping call — re-keying the baselines without simultaneously moving HIR diagnostics off `LoweringError` would create churn and a transient state where multiple TYPE bucket codes are emitted. The slice is internally consistent: every site this slice *did* touch is now coded, so the bridge is doing strictly less work than before.

What's missing is a visible audit trail. Concretely:

- The issue's "Started milestone_diag_4a slice 1" bullet at [issue line 34](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md#L34) does not mention that the `TypeCheck => SIFR-TYPE-0001` deletion (a stated diag_4a deliverable) is being moved to a later slice. Without that bullet, a reviewer comparing the merged slice against the issue plan will read it as scope drift rather than scope splitting.
- The transitional comment at [diagnostics.rs:128-131](../crates/sifr_driver/src/diagnostics.rs:128) says "Transitional legacy bridge for unmigrated non-workspace paths in diag_4a" but does not name the specific later slice that owns the deletion (e.g. "removed in `diag_4a` slice 2 once HIR `LoweringError` carries codes"). The comment will rot if the next slice's name changes.

**Recommendation:** before the slice 1 PR opens, edit the issue to add an explicit deferred-item bullet ("`TypeCheck => SIFR-TYPE-0001` bridge deletion + fixture re-keying deferred to slice 2 alongside HIR transport") and tighten the comment in `diagnostics.rs` to point at that slice. No code change beyond comments is required for this slice to merge.

## New findings introduced or exposed by pass-2 changes

### N1 — Driver diagnostics test now exercises the *active* code paths only (good, but R7 dead-code branch is now only test-covered)

[tests/diagnostics.rs:7-19](../crates/sifr_driver/src/tests/diagnostics.rs:7) now asserts `SIFR-PARSE-0002` (active) instead of `SIFR-PARSE-0001` (retired). [tests/diagnostics.rs:21-41](../crates/sifr_driver/src/tests/diagnostics.rs:21) asserts `SIFR-TYPE-0002` and `SIFR-CODEGEN-0002` (both active). [tests/diagnostics.rs:71-81](../crates/sifr_driver/src/tests/diagnostics.rs:71) confirms an explicit `BUILD_TEMP_WORKSPACE_FAILURE` code wins over a workspace-shaped message. This is exactly what pass-1 recommended.

The `apply_diagnostic_recovery_limits` tests at [tests/diagnostics.rs:84-129](../crates/sifr_driver/src/tests/diagnostics.rs:84) still construct `CompilerDiagnostic` literals with hard-coded `"SIFR-TYPE-0001"` strings. That is fine — those tests exercise the recovery limiter on synthetic input and are not making a claim about which code the bridge produces. Worth a one-line comment if a future contributor mistakes them for bridge regressions, but not a blocker.

### N2 — `test_runner/orchestrator.rs` forwarder still uses struct literal with `code: error.code` (low)

[test_runner/orchestrator.rs:107-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:107):

```rust
let compile_errors: Vec<CompileError> = errors
    .into_iter()
    .map(|error| CompileError {
        message: format!("[{}] {}", test_file.display(), error.message),
        phase: CompilePhase::TypeCheck,
        code: error.code,
    })
    .collect();
```

`error.code` is always `None` today because [module_lowering.rs:23-44](../crates/sifr_driver/src/frontend/module_lowering.rs:23) emits `CompileError::new(...)` with no code. The forwarding is therefore inert until HIR LoweringError gains a code, which is the deferred work. Pass-1 already noted this is acceptable scope-wise.

What was not picked up in pass 2: the lack of a one-line comment explaining that `code: error.code` is forwarding for forward-compatibility, not a wired path. A comment like `// Preserve None until HIR LoweringError carries a DiagnosticCode (next diag_4a slice).` keeps the next reviewer from assuming the forward is live.

The same shape exists in [stdlib/bootstrap.rs:198-203](../crates/sifr_driver/src/stdlib/bootstrap.rs:198), but there `e.code` is `Some(INTERNAL_COMPILER_PANIC)` (set by [run_codegen_with_boundary](../crates/sifr_driver/src/diagnostics.rs:253)), so the forward is *live* and the struct literal is genuinely necessary. A short comment distinguishing the two would help.

### N3 — Compact-grouping test does not pin the "two internals share a key" case (low)

[presentation.rs:294-336](../crates/sifr_diagnostics/src/render/presentation.rs:294) covers internal diagnostics rendered through `render_sink_human` and the mixed-severity summary. It does not cover the case where two internal diagnostics share `(severity, code, message_template)` and therefore should land in one compact group keyed on `primary_display_file: None`. This was called out in pass-1 R6 as a small addition. The current `CompactKey::from_diagnostic` already handles this correctly via `primary_span(...).and_then(|span| span.file.clone())` returning `None`, but absent a test, a future change to that key derivation could silently flip the behavior. Consider adding a 5-line test that emits two internal diagnostics with identical code and template and asserts the compact output contains a single `(x2)` entry with no `at` location lines.

### N4 — `is_internal_compile_error` prefix fallback is now production-dead

[main.rs:260-265](../crates/sifr/src/main.rs:260):

```rust
fn is_internal_compile_error(error: &CompileError) -> bool {
    if error.code == Some(DiagnosticCode::INTERNAL_COMPILER_PANIC) {
        return true;
    }
    error.message.starts_with("internal compiler panic during ")
}
```

I traced every production site that emits `"internal compiler panic during ..."`:

- [main.rs:422](../crates/sifr/src/main.rs:422), [main.rs:445](../crates/sifr/src/main.rs:445), [main.rs:478](../crates/sifr/src/main.rs:478), [main.rs:530](../crates/sifr/src/main.rs:530) — all go through `run_with_panic_boundary` which sets `INTERNAL_COMPILER_PANIC` ([main.rs:244-258](../crates/sifr/src/main.rs:244)).
- [build/project_codegen.rs:66](../crates/sifr_driver/src/build/project_codegen.rs:66), [build/entrypoint.rs:246](../crates/sifr_driver/src/build/entrypoint.rs:246), [stdlib/bootstrap.rs:193](../crates/sifr_driver/src/stdlib/bootstrap.rs:193), [test_runner/orchestrator.rs:78,121](../crates/sifr_driver/src/test_runner/orchestrator.rs:78) — all go through `run_codegen_with_boundary` which sets `INTERNAL_COMPILER_PANIC` ([diagnostics.rs:253-266](../crates/sifr_driver/src/diagnostics.rs:253)).

So the prefix branch can never fire from production code. The only thing exercising it is [main.rs:1212-1224](../crates/sifr/src/main.rs:1212), which manually constructs a `CompileError::new(...)` with the prefix string. That test was reasonable when the prefix was the canonical signal; with the structured code present, deleting the prefix branch and updating the test to construct a `CompileError::with_code(..., INTERNAL_COMPILER_PANIC)` would remove dead code. Not a correctness blocker — the existing path still classifies correctly — but worth doing in the same wave as the bridge repoint.

### N5 — `INTERNAL_COMPILER_PANIC` is paired with `CompilePhase::TypeCheck` at one site (cosmetic)

[build/entrypoint.rs:215-221](../crates/sifr_driver/src/build/entrypoint.rs:215) attaches `INTERNAL_COMPILER_PANIC` to `CompilePhase::TypeCheck`. The structured code is what `is_internal_compile_error` and the JSON output use, so behavior is correct. The legacy `Display` for this error renders `"type error: internal error: frontend lowering missing 'main' module"` ([diagnostics.rs:221-231](../crates/sifr_driver/src/diagnostics.rs:221)), which reads oddly for an internal-invariant violation. Pairing it with `CompilePhase::Build` would render as `"build error: ..."` and match the sibling site at [build/entrypoint.rs:208-212](../crates/sifr_driver/src/build/entrypoint.rs:208). One-line change, cosmetic only.

### N6 — `compile_errors_to_diagnostics` legacy renderer still classifies severity by code prefix (low)

[main.rs:372-417](../crates/sifr/src/main.rs:372) inside `render_compile_errors` sniffs `diagnostic.code.starts_with("SIFR-PARSE-")` etc. to choose the human-format label. With the bridge still emitting `SIFR-PARSE-0001` for unmigrated parse paths, this classifier still produces sensible labels. Once the bridge is repointed in the next slice, codes like `SIFR-INTERNAL-0001` will fall into the catch-all `match diagnostic.severity` arm and render as `error:` (lowercase). That is fine, but a future contributor might be tempted to add a `SIFR-INTERNAL-` arm here. The slice scope is unchanged ("legacy CLI renderer untouched"), so this is just flagging a near-future surface — no action this slice.

## Out-of-scope drift check (negative findings — *good*)

- No HIR file under `crates/sifr_hir/` is modified.
- `CompilePhase::TypeCheck` enum variant and `Display` arm are retained at [diagnostics.rs:34-39, 217-225](../crates/sifr_driver/src/diagnostics.rs:34).
- `apply_diagnostic_recovery_limits` and `compile_errors_to_diagnostics` are unchanged in signature.
- 91 fail fixtures and the two decimal verification baselines are untouched. The decimal baseline still emits `SIFR-TYPE-0001` via the deferred bridge, which is the documented transitional state.
- The `Severity::Help` variant on the driver-side `Severity` enum and the help-aware compact renderer in `main.rs` are unchanged. The new presentation-layer compact summary uses three categories (Error/Warning/Note) because `sifr_diagnostics::Severity` has no `Help` variant — that is by design.
- `LoweringError`, `LoweringResult`, and `LoweringOutcome` are untouched.

## Validation

The implementer's reported run is the right scope for this slice:

- `cargo fmt --check`
- `python3 scripts/check_diagnostic_schema_sync.py`
- `python3 scripts/check_diagnostic_docs_sync.py`
- `cargo test -p sifr_diagnostics`
- `cargo test -p sifr_driver diagnostics`
- `cargo test -p sifr --no-run`
- `cargo clippy -p sifr_diagnostics -p sifr_driver -p sifr -- -D warnings`

Because the renderer is not wired into the CLI in this slice (the JSON / compact baselines still come from `compile_errors_to_diagnostics`), no e2e baseline regeneration is required. Running `scripts/run_all_tests.sh --profile quick` once before merge is still recommended to confirm the workspace verification fixtures (`workspace_unresolved_import`, `workspace_ambiguous_import`, `workspace_malformed_manifest`) still match — the workspace baseline JSONs depend on `with_code` continuing to emit the active workspace identity, and that is what the slice rewires.

## Sequencing recommendations for slice 2

1. Repoint the legacy bridge so unmigrated paths emit *active* codes (`Build → BUILD_MATERIALIZATION_FAILURE`, `Codegen → CODEGEN_BACKEND_FAILURE`, `Parse → PARSE_EXPECTED_TOKEN_OR_RECOVERY`, `TypeCheck → INTERNAL_COMPILER_PANIC` until HIR transport lands), then delete the `TypeCheck => SIFR-TYPE-0001` arm as the issue requires. Re-key the affected verification and e2e baselines in the same slice so the test surface and the bridge land together.
2. Migrate HIR `LoweringError` to carry a `DiagnosticCode`, then drop the inert `code: error.code` forward in [orchestrator.rs:107-118](../crates/sifr_driver/src/test_runner/orchestrator.rs:107) and [module_lowering.rs:23-44](../crates/sifr_driver/src/frontend/module_lowering.rs:23) to use `CompileError::with_code` directly.
3. Delete the `is_internal_compile_error` prefix fallback (N4) and update the synthetic test to use `CompileError::with_code(..., INTERNAL_COMPILER_PANIC)`.
4. Pair `CompilePhase::Build` with the second `INTERNAL_COMPILER_PANIC` site in `entrypoint.rs` for `Display` consistency (N5).
5. Add the small "two internals share a compact key" test (N3) and consider splitting Ruff parse failures into the `SIFR-PARSE-0002..0009` buckets (R5) — these can land independently.

## Summary

Pass 2 cleanly addresses every actionable pass-1 finding except the explicit deferral of the bridge repoint and `TypeCheck => SIFR-TYPE-0001` deletion. That deferral is acceptable as a slicing decision because re-keying the affected baselines belongs in the same slice as the HIR `LoweringError` transport. Before merge, the issue file should be updated with a one-line bullet documenting the deferral and the in-code transitional comment should name the slice that owns the deletion. Six minor follow-ups (N2–N6 plus the parse-bucket TODO marker from R5) are easy wins for the next slice but none are correctness blockers for this one.
