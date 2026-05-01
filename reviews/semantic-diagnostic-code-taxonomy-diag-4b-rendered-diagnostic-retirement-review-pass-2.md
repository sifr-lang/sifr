# `milestone_diag_4b` slice 3 — `RenderedDiagnostic` retirement review (pass 2)

## Scope under review

Pass 2 of the slice that retires the custom driver `CompilerDiagnostic` transport and carries `sifr_diagnostics::RenderedDiagnostic` through the driver and CLI APIs directly. The pass-1 review at [reviews/semantic-diagnostic-code-taxonomy-diag-4b-rendered-diagnostic-retirement-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4b-rendered-diagnostic-retirement-review-pass-1.md) raised one BLOCKING finding (F1: stale check-json baselines) and three NIT-level observations (F2: deliberate `help_count` semantic change, F3: duplicated helpers + asymmetric `cfg(test)`, F4: stub `message_template`/`args` polluting every diagnostic). Pass 2 reports the following remediation:

- Blessed five `check-json.stderr.txt` baselines to the canonical `RenderedDiagnostic` JSON shape.
- Updated the paired `check-human.stderr.txt` baselines to use the code-derived label that `diagnostic_label_for_code_str` now emits in `render_diagnostics`.
- Added [crates/sifr/src/main.rs:663](crates/sifr/src/main.rs:663) `test_json_diagnostic_format_uses_canonical_rendered_schema` to lock the canonical wire format.
- Updated the Phase 27 schema spec at [internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26](internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26) from `primary_span`/`related_spans` to `spans`/`message_template`/`args`.
- Scoped the legacy display helpers to `#[cfg(test)]` in both crates.

I read every file in the working tree's `git status`, the full diff (`git diff HEAD`), the canonical model and render definitions in `crates/sifr_diagnostics/src/{codes.rs,model/mod.rs,render/mod.rs}`, the entire `crates/sifr/tests/verification/` baseline tree, the validation-lane manifest at `verification/validation_lanes/manifest.json`, and the pass-1 review. I did not run any cargo or harness commands; the user reports the four validation invocations listed in the request all succeed (`cargo check -p sifr_driver -p sifr`, `cargo test -p sifr_driver --lib --tests`, `--bless`/non-`--bless` runs of `python3 scripts/run_verification_hardening.py --suite diagnostics --suite project`, and the new schema-shape unit test).

## Summary

Pass 1 BLOCKING F1 is fully resolved. The five JSON baselines are now structurally equivalent to what `serde_json::to_string_pretty(&Vec<RenderedDiagnostic>)` will emit at runtime, the human baselines are aligned with the code-derived labels that `render_diagnostics` writes for any `SIFR-`-prefixed code, and the new schema-shape unit test future-proofs the CLI's wire format against accidental drift back to the bespoke envelope. The pass-1 NIT-level observations are addressed where appropriate (F3's symmetry, the Phase 27 spec) and the residual ones (F2's `help_count` semantic change, F4's `args`/`message` desync after orchestrator path-prefixing) remain accurate descriptions of the new behavior, not regressions, and continue to be the right shape for the deferred `DiagnosticSink`-direct migration.

I find **no remaining blocking issues**. Two NIT observations are recorded below for the PR description and one for the next reviewer of the `DiagnosticSink`-direct migration; none are actionable in this slice.

## Findings

### F1 (pass 1) — RESOLVED — JSON baselines now match the canonical envelope

All five files now serialize the active `RenderedDiagnostic` schema:

- [crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt:1](crates/sifr/tests/verification/diagnostics/decimal_invalid_literal/baselines/check-json.stderr.txt:1)
- [crates/sifr/tests/verification/project/missing_import_reports_error/baselines/check-json.stderr.txt:1](crates/sifr/tests/verification/project/missing_import_reports_error/baselines/check-json.stderr.txt:1)
- [crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-json.stderr.txt:1](crates/sifr/tests/verification/project/workspace_unresolved_import/baselines/check-json.stderr.txt:1)
- [crates/sifr/tests/verification/project/workspace_ambiguous_import/baselines/check-json.stderr.txt:1](crates/sifr/tests/verification/project/workspace_ambiguous_import/baselines/check-json.stderr.txt:1)
- [crates/sifr/tests/verification/project/workspace_malformed_manifest/baselines/check-json.stderr.txt:1](crates/sifr/tests/verification/project/workspace_malformed_manifest/baselines/check-json.stderr.txt:1)

Each baseline now contains the new top-level fields `message_template: "{message}"`, `args: { "message": { "kind": "string", "value": "..." } }`, and `spans: []`, and no longer carries `primary_span`/`related_spans`. The order of fields in each baseline (`code`, `severity`, `message`, `message_template`, `args`, `url`, `spans`, `children`, `help`, `suggestions`) matches the `serde` declaration order at [crates/sifr_diagnostics/src/render/mod.rs:24](crates/sifr_diagnostics/src/render/mod.rs:24), so `serde_json::to_string_pretty` will reproduce them deterministically.

The companion `check-human.stderr.txt` baselines were updated to flip the leading label from `error:` to `type error:` (decimal) and `build error:` (workspace/import) — these are the labels `diagnostic_label_for_code_str` returns at [crates/sifr_driver/src/diagnostics.rs:122](crates/sifr_driver/src/diagnostics.rs:122) for `SIFR-DECIMAL-*`, `SIFR-WORKSPACE-*`, and the panic-boundary code, and `render_diagnostics` chooses the code-derived label for any `SIFR-`-prefixed code at [crates/sifr/src/main.rs:403](crates/sifr/src/main.rs:403). The diagnostic message bodies, exit codes, and severity remain intact.

I confirmed there are no other check-json baselines in the repository that still reference `primary_span`/`related_spans`. `find crates/sifr/tests/verification -name "check-json.stderr.txt"` returns exactly the five files above, and a tree-wide grep for `primary_span`/`related_spans` in `*.txt`/`*.json` is empty under `crates/`. The `pr` lane's other hardening suites (`fixedbugs`, `crashes`, `oss-curated`) do not surface check-json baselines that would lock onto the bespoke schema.

### F1.a — RESOLVED — schema regression-guard test present

[crates/sifr/src/main.rs:663](crates/sifr/src/main.rs:663) `test_json_diagnostic_format_uses_canonical_rendered_schema` constructs a single diagnostic via the CLI's private `diagnostic_with_code` helper, runs it through `serde_json::to_value`, and asserts the four invariants the slice cares about: `message_template`, `args`, and `spans` are present, and the retired `primary_span` and `related_spans` keys are absent. Two minor observations:

- The test serializes via `serde_json::to_value` rather than `serde_json::to_string_pretty`. Both routes use the same `Serialize` impl on `RenderedDiagnostic`, so the field set is identical; the JSON-pretty path the CLI uses cannot diverge without a corresponding change to the canonical model. The choice is fine.
- The test does not exercise `apply_diagnostic_recovery_limits` or the recovery-limit summary path, so it does not catch a hypothetical future regression in which a summary collapse drops or renames a field. The five blessed baselines provide cross-check coverage for the non-empty-group case via the harness, and the recovery-limit unit tests at [crates/sifr_driver/src/tests/diagnostics.rs:143](crates/sifr_driver/src/tests/diagnostics.rs:143) cover the summary collapse semantically. Acceptable.

The test is the right shape for a unit-level future-proof guard.

### F1.b — RESOLVED — Phase 27 spec text is current

[internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26](internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26) now lists the canonical structured-diagnostic schema as `code`, `severity`, `message`, `message_template`, structured `args`, `url`, `spans`, `children`, `help`, plus optional structured suggestions. That matches the `RenderedDiagnostic` definition in `crates/sifr_diagnostics/src/render/mod.rs:24` exactly. Other occurrences of `primary_span`/`related_spans` in `issues/ad-hoc-...` describe the internal HIR `SourceDiagnostic` builder shape (e.g. lines 411–412 reference the internal `SourceDiagnostic.primary_span: SourceSpan`, lines 722/727/1124/1134 talk about HIR-side requirements), which is a separate, internal concept from the user-visible JSON envelope and is still accurate. No further doc edits are needed.

### F3 (pass 1) — RESOLVED for the asymmetry sub-bullet — `cfg(test)` is now symmetric

The driver's `diagnostic_legacy_display` at [crates/sifr_driver/src/diagnostics.rs:108](crates/sifr_driver/src/diagnostics.rs:108) is now gated on `#[cfg(test)]`, matching the CLI's `legacy_diagnostic_display` at [crates/sifr/src/main.rs:291](crates/sifr/src/main.rs:291). All four call sites (driver `tests/diagnostics.rs`, driver `tests/single_file_frontend.rs`, driver `tests/project_build_check.rs`, and the six CLI consistency tests in `crates/sifr/src/main.rs`) live under `#[cfg(test)]`, so neither helper is dead code in non-test builds. The remaining sub-observation from pass 1 — that `diagnostic_with_code` is duplicated in CLI and driver because of `pub(crate)` visibility — is unchanged in pass 2 and remains a NIT to defer until a fourth call site appears under `crates/sifr/`.

### F2 (pass 1) — UNCHANGED, deliberate — `compact_severity_summary.help_count` semantics

[crates/sifr/src/main.rs:299](crates/sifr/src/main.rs:299) still counts `diagnostic.help.is_some()` rather than the retired `Severity::Help`. Pass 1 already classified this as a deliberate semantic change rather than a bug; pass 2 does not modify it. The renamed snapshot test `test_compact_renderer_snapshot_multi_severity_group_order` at [crates/sifr/src/main.rs:1438](crates/sifr/src/main.rs:1438) flips the third diagnostic from `Severity::Help` to `Severity::Note` and continues to assert `1 help item(s)` as the count derived from the warning's `Some("remove the assignment")` help text. Coherent. Worth a one-line note in the PR description so future readers do not interpret it as a no-op rename.

### F4 (pass 1) — UNCHANGED — `args["message"]` desyncs from `message` after orchestrator path-prefixing and after recovery-limit summary collapse

[crates/sifr_driver/src/test_runner/orchestrator.rs:107](crates/sifr_driver/src/test_runner/orchestrator.rs:107) still mutates only `error.message`, leaving `error.args["message"]` set to the original frontend message. Symmetrically, [crates/sifr_driver/src/diagnostics.rs:86](crates/sifr_driver/src/diagnostics.rs:86) updates `summary.message` to the `... +N more similar diagnostics` string while leaving `summary.args["message"]` set to the message of `group[0]`. Both are no-ops today because `render_diagnostics`, `render_compact_diagnostics`, and the JSON serializer all read `diagnostic.message` directly and never re-render from `message_template` + `args`. They will become real desyncs the day a renderer starts re-formatting from the template. Not actionable in this slice; the next reviewer of the `DiagnosticSink`-direct migration should re-evaluate when the stub `message_template = "{message}"` is replaced with a real template, since at that point both mutations need to update `args` too.

### N1 — NIT — severity now derives from `code.declared_severity()` rather than always being `Severity::Error`

This is a pass-1-noted construction detail re-examined here for completeness. Both the driver helper at [crates/sifr_driver/src/diagnostics.rs:40](crates/sifr_driver/src/diagnostics.rs:40) and the CLI helper at [crates/sifr/src/main.rs:103](crates/sifr/src/main.rs:103) set `severity: code.declared_severity()`. The retired `CompilerDiagnostic::with_code` always set `severity: Severity::Error`. Every `DiagnosticCode` constant referenced by `diagnostic_with_code` call sites in the driver and CLI today (`PARSE_EXPECTED_TOKEN_OR_RECOVERY`, `TYPE_MISMATCH`, `CODEGEN_BACKEND_FAILURE`, `BUILD_*`, `WORKSPACE_*`, `STDLIB_BOOTSTRAP_FAILURE`, `STDLIB_CACHE_FAILURE`, `INTERNAL_COMPILER_PANIC`) is declared `Severity::Error` at `crates/sifr_diagnostics/src/codes.rs:9`–`126`, so the resulting severity is identical to the prior hard-coded value for every production path. The behavior change only becomes user-visible if a `LoweringError` arrives at `lowering_error_to_diagnostic` carrying a Warning- or Note-severity code — but HIR currently routes Warning/Note diagnostics through `lowering_result.warnings` and `lowering_result.reveal_types` (rendered to stderr separately by `emit_frontend_diagnostics`), not through `LoweringError`. So no current call path produces a non-error forwarded severity. The change is forward-correct (matches declared semantics) and worth flagging in the PR description so anyone reviewing the diff does not treat the swap as a no-op.

## Slice-goal verification

- ✓ `git grep -n "CompilerDiagnostic\b" crates/ --include='*.rs'` returns zero matches.
- ✓ `git grep -nE "\bRelatedSpan\b|\bDiagnosticSuggestion\b|\bSuggestionKind\b|\bDiagnosticChild\b" crates/sifr_driver/ crates/sifr/ --include='*.rs'` returns zero matches.
- ✓ `git grep -nE "Severity::Help" crates/ --include='*.rs'` matches only `crates/sifr_diagnostics/src/render/presentation.rs:213` and `:276`, both referencing the canonical `ChildSeverity::Help` variant on `RenderedDiagnosticChild` — not the retired top-level `Severity::Help`. The driver/CLI surface is fully clean.
- ✓ `git grep -nE "primary_span|related_spans" crates/sifr_driver/ crates/sifr/src/` matches only the new local `fn primary_span(diagnostic: &RenderedDiagnostic)` helper and its caller in `apply_diagnostic_recovery_limits`, plus the negative-assertion lines in the new schema test. No other references survive.
- ✓ `pub use diagnostics::{...}` at [crates/sifr_driver/src/lib.rs:23](crates/sifr_driver/src/lib.rs:23) is reduced to `apply_diagnostic_recovery_limits, diagnostic_label_for_code, diagnostic_label_for_code_str, CompileResult, CompileResultFull` only. The CLI now imports `DiagnosticArg`, `DiagnosticCode`, `DiagnosticSpan`, `RenderedDiagnostic`, and `Severity` directly from `sifr_diagnostics`.
- ✓ Active `SIFR-*` identity is preserved at every construction site (`build/{api,entrypoint,materialize,workspace}.rs`, `frontend/{api,module_lowering}.rs`, `project/{compile_order,discovery,frontend}.rs`, `stdlib/{bootstrap,cache}.rs`, `test_runner/{execution,orchestrator}.rs`, `workspace/mod.rs`, `crates/sifr/src/main.rs`). Every call passes an explicit `DiagnosticCode::*`, never derives a code from a message prefix.
- ✓ Slice-2 test-runner identity invariant test at `crates/sifr_driver/src/tests/test_runner.rs:355`/`:361` is unchanged and still asserts `error.code == DiagnosticCode::TYPE_MISMATCH.code()` and `!= DiagnosticCode::INTERNAL_COMPILER_PANIC.code()` after the orchestrator path-prefix mutation.
- ✓ Stdlib-cache no-fallback-rebuild test at `crates/sifr_driver/src/stdlib/cache.rs:46` is rewritten to use the new helper (`crate::diagnostics::diagnostic_with_code`) but otherwise unchanged in scope — same `STDLIB_CACHE_FAILURE` sentinel, same single-build counter assertion.
- ✓ Recovery-limit thresholds remain `MAX_TOP_LEVEL_DIAGNOSTICS = 50` and `MAX_SIMILAR_DIAGNOSTICS_PER_GROUP = 5`. The grouping key migrates from `(severity_rank, code, message, primary_span.file)` to `(severity_rank, code, message, primary_span(d).and_then(span.file))` where `primary_span` is the new helper that scans `spans` for `is_primary == true`. Behavior is preserved because all current production diagnostics ship empty `spans`, so `primary_span(d)` returns `None` and the file component of the key collapses to `None`, identical to the prior behavior.
- ✓ Span-clearing on the summary diagnostic moves from clearing `primary_span = None; related_spans.clear()` to `summary.spans.clear()`. With empty spans (the production case), this is a no-op, identical to the prior behavior.
- ✓ Code-derived label table at `crates/sifr_driver/src/diagnostics.rs:122` and the table-driven label test at `crates/sifr_driver/src/tests/diagnostics.rs:81` are unchanged. The label test now asserts equality against `crate::diagnostics::diagnostic_legacy_display(...)` instead of `Display::to_string`, which is a name-only swap (the body of `diagnostic_legacy_display` reproduces the exact text the retired `Display` impl produced).
- ✓ Compact rendering: `render_compact_diagnostics` keeps the same group key, the same severity ordering, and only swaps `&diagnostic.primary_span` for `diagnostic.spans.iter().find(|span| span.is_primary)`. With empty spans (production case), behavior is identical.
- ✓ No fallback paths or compatibility shims. The slice is pure replacement; there is no transitional `Severity::Help`-to-something fallback, no `if old { ... } else { ... }` bridge, and no message-prefix code derivation anywhere.

## Test coverage

- The new `test_json_diagnostic_format_uses_canonical_rendered_schema` schema-shape test (`crates/sifr/src/main.rs:663`) addresses the optional pass-1 ask to lock the wire format outside the verification baselines. It uses `INTERNAL_COMPILER_PANIC` for stable identity and serializes through `serde_json::to_value`; the assertions are precisely the four schema-drift signals (`message_template`/`args`/`spans` present, `primary_span`/`related_spans` absent).
- The five blessed verification baselines provide end-to-end harness coverage of the canonical JSON shape across all real driver code families used today (`SIFR-DECIMAL-*`, `SIFR-WORKSPACE-0001`, `SIFR-WORKSPACE-0101`, `SIFR-WORKSPACE-0102`).
- All six CLI consistency tests (`test_compile_entrypoint_error_consistency_for_*`, `test_check_entrypoint_*`) and `test_check_entrypoint_consistency_with_helper_type_mismatch_in_project_mode` now run their byte-equality comparisons through `legacy_diagnostic_display`. That helper reproduces the exact output the retired `Display` impl produced (`{label}: {message}`), so the consistency contract is unchanged.
- Driver tests in `tests/diagnostics.rs` migrate to a new local `test_diagnostic`/`primary_test_span` helper pair that constructs canonical `RenderedDiagnostic` values without depending on `diagnostic_with_code`'s behavior. The seven test functions assert the same invariants as before.
- I did not find any deleted test coverage. The `+test_json_diagnostic_format_uses_canonical_rendered_schema` is net new; everything else in the diff is mechanical migration.

## Doc and inventory updates

- [internal_docs/diagnostic_emission_inventory.md:8](internal_docs/diagnostic_emission_inventory.md:8) and the surface section at line 84 now correctly state that both `CompileError` and `CompilerDiagnostic` have been deleted and that driver/CLI APIs carry `sifr_diagnostics::RenderedDiagnostic`.
- The inventory's per-file row at line 87 (`crates/sifr_driver/src/diagnostics.rs`) names `diagnostic_with_code` correctly and notes that it constructs canonical rendered diagnostics.
- The inventory's "Manual sites" table at line 108 renames the column to `RenderedDiagnostic` with the same 9/2 split (CLI/driver-tests).
- The Phase 27 schema bullet at [internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26](internal_docs/phases/27_diagnostics_error_recovery_and_stability_contract.md:26) is now accurate.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:72](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:72) still flags slice 3 as `[ ] in progress`, which is correct until the PR merges.

## Recommended action

- **Resolved**: the BLOCKING pass-1 finding is fully addressed. No remaining blockers.
- **Optional**: in the PR description, call out the two deliberate-but-not-mechanical changes as a courtesy to future readers — (a) `compact_severity_summary.help_count` is now derived from `help.is_some()` rather than the retired `Severity::Help` (F2 above), and (b) constructed-diagnostic severity now derives from `code.declared_severity()` rather than the prior hard-coded `Severity::Error` (N1 above). Both are no-ops in production today, but the diff alone reads like a no-op rename when it is in fact a definition change.
- **Optional / next slice**: when the deferred `DiagnosticSink`-direct migration replaces the stub `message_template = "{message}"` with a real template, audit the orchestrator's `error.message = format!(...)` path-prefix at `test_runner/orchestrator.rs:107` and the summary-collapse `summary.message = format!(...)` at `diagnostics.rs:86` for `args` updates (F4 carry-over). Not actionable in this slice.

This slice is ready to merge after the PR-description courtesy notes are added (or without them at the author's discretion). The Rust migration is complete and behavior-preserving, the verification baselines now match the canonical shape, the new unit test guards against schema drift, and the docs are consistent.
