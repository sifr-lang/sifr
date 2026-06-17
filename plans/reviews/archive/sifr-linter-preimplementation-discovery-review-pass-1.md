

I have now reviewed all primary files, current implementations, existing review artifacts, Ruff sources, and validation tooling. Here is my pre-implementation discovery review.

---

# Pre-Implementation Discovery Review: Sifr Production Linter Phase

## Summary

**No blockers remain.** The phase is implementation-ready. The following findings are precision items for M1 to encode correctly — none prevent starting, all must be addressed before the phase closes.

---

## Findings By Severity

### MEDIUM — Manifest Encoding Precision (M1)

**1. Manifest schema has one undefined field and one underspecified validation**

The audit manifest schema (lines 265–275) defines `accepted_sifr_config_keys` and `rejected_ruff_config_keys` but never populates the `rejected_ruff_config_keys` array. The validation obligation (line 280) references it. M1 must add the rejected config keys to the manifest or remove the field.

**Recommended edit location:** `issues/ad-hoc-production-grade-sifr-linter.md`, add to manifest schema after line 275:

```markdown
- `rejected_sifr_config_keys`: array of Sifr lint config keys that must fail if accepted as configuration.
```

**2. Suppression-gate manifest `updated_by_milestone` field is underspecified**

The schema defines `updated_by_milestone` but never specifies:
- Format (string like `"m36.5"` or numeric like `5`?)
- What constitutes a valid value
- Whether M1 should populate it with `"m1"` or leave it absent until the first change

**Recommended edit location:** `issues/ad-hoc-production-grade-sifr-linter.md`, add to suppression-gate schema after `updated_by_milestone`:

```markdown
- `updated_by_milestone`: string milestone identifier, e.g., `"m1"` for Milestone 1. Absent until the first gate change.
```

### MEDIUM — Compile-Time Gate Mechanism (M1/M3 Contract)

**3. W-8 references a compile-time gate but does not name the Rust API path**

AC-13 says the gate is mechanically enforced through Rust types and that syntax/HIR/workspace rule modules must depend on the parser-aware suppression API at compile time. W-8 references this but does not name the specific API. The `check_linter_reuse_rules.py` validation check in M1 (line 549) says it will validate the suppression-gate manifest path, schema, and state — but it does not say it will verify that syntax/HIR/workspace modules import the required API.

The M1 `check_linter_reuse_rules.py` scope (lines 548–558) covers `cargo tree` forbidden dependency checks, manifest schema validation, and Ruff filesystem-directory coverage. It does not include a Rust-source-level API-dependency check.

**Recommended edit location:** `issues/ad-hoc-production-grade-sifr-linter.md`, add to M1 scope after line 557:

```
- verify that any Sifr rule module whose `suppression_complexity` is not `physical-line` imports `sifr_lint::suppression::ParserAwareSuppressions` through a positive import assertion in `check_linter_reuse_rules.py`
```

This makes the compile-time gate verifiable at M1, not deferred to M3.

### MEDIUM — Diagnostic Class Not Present in Current Code

**4. The typed `Hard` vs `Policy` diagnostic class is in the phase plan but not in the current `RenderedDiagnostic` schema or LSP handlers**

AC-6 requires that hard vs policy diagnostic class is present in analysis/LSP diagnostic data and code-action gating. AC-9 and the review gate for M6 (line 712) confirm hard diagnostics cannot be auto-fixed or suppressed. The phase requires typed class enforcement (lines 414–421) with the explicit statement "string-prefix checks such as `SIFR-LINT-*` are not sufficient."

However:
- `sifr_diagnostics::RenderedDiagnostic` does not carry a `DiagnosticClass` enum
- `sifr_analysis/src/queries.rs` does not include `DiagnosticClass`
- `sifr_lsp/src/requests/code_action.rs` gates code actions by calling `host.code_actions()`, which routes through `sifr_analysis`, which routes through `sifr_lint`. There is no LSP-layer check for `DiagnosticClass::Policy` before offering a suppression code action.
- `check_lsp_split_brain.py` does not verify hard-vs-policy class enforcement in code actions

This is a Phase plan requirement that has no current implementation. The phase correctly identifies it as AC-6, but the current `check_lsp_split_brain.py` and `check_tooling_dependency_boundaries.py` do not cover this specific check.

**Recommended edit location:** `issues/ad-hoc-production-grade-sifr-linter.md`, add to M6 scope at line 703:

```
- verify that `check_lsp_split_brain.py` or `check_linter_reuse_rules.py` fails if LSP code-action handlers offer suppression actions for `Hard` class diagnostics
```

**Recommended edit location:** `scripts/run_all_tests.sh`, after the linter reuse contract check is wired in (after phase M7):

```bash
python3 "${SCRIPT_DIR}/../verification/tooling/check_linter_diagnostic_class.py"
python3 "${SCRIPT_DIR}/../verification/tooling/check_linter_diagnostic_class.py" --self-test
```

This is not a blocker for M1. The phase correctly defers the typed class implementation to M6 where the fix engine ships. But the guardrail to enforce it must be planned now.

### LOW — CLI Implementation Gap (M2)

**5. The current `sifr lint` CLI has no Ruff-compatible surface; the phase plan locks a full contract before M2 starts**

Current `sifr lint` CLI (lines 257–261 in `cli_model_and_entrypoint.rs`):
```rust
Lint {
    /// Input .sifr file or directory
    path: PathBuf,
}
```

The lint entrypoint (lines 707–715 in `check_and_package_commands.rs`) calls `sifr_lint::lint_path` with only `explicit_target` and `LintOptions::default()`.

The phase locks 47 CLI surface rows (lines 332–381) including `--select`, `--ignore`, `--output-format`, `--show-files`, `--statistics`, stdin handling, per-file ignores, and fix flags. None of these exist in the current CLI.

The phase already acknowledges this: "W-10: `sifr lint` currently has a placeholder single-path CLI while Ruff's lint command has a production command surface." This is not a blocker. The contract is locked before implementation starts. M1 encodes the manifest; M2 implements the CLI.

**Recommended edit location:** No change needed. This is already correctly documented in the phase. M2 implementation must use `lint_cli_parity.json` as the authoritative source of truth.

### LOW — Lint Config in sifr.toml (M2)

**6. The phase defines `[lint]` in `sifr.toml` but the current `sifr_format::config` only handles `[format]`**

The formatter config (`crates/sifr_format/src/config.rs`) implements `effective_format_config` and `FormatConfigOverrides`. The lint config design (lines 436–470) mirrors the formatter config pattern. There is no `sifr_lint::config` module yet.

The `check_linter_reuse_rules.py` verification in M1 (line 554) checks that accepted lint config keys appear in the manifest with allowed dispositions. This requires the manifest to include the planned `[lint]` config keys as `accepted_sifr_config_keys`.

**Recommended edit:** When encoding `lint_cli_parity.json` in M1, also encode the planned `[lint]` section config keys in `ruff_rule_config_audit.json` as `accepted_sifr_config_keys`. Keys include `select`, `extend-select`, `ignore`, `fixable`, `unfixable`, `unsafe-fixes`, `include`, `exclude`, `extend-exclude`, `respect-gitignore`, `force-exclude`, `preview`, `extend-safe-fixes`, `extend-unsafe-fixes`, `per-file-ignores`, `extend-per-file-ignores`, and `extend`. These must appear as `adapt` disposition in the audit manifest so M2 can implement them from an approved row.

### LOW — Missing Validation Tooling Manifests Directory

**7. `verification/tooling/linter_manifests/` does not exist and `check_linter_reuse_rules.py` does not exist**

These are M1 deliverables. No change needed to the phase — these are correctly listed in M1 scope. However, M1 implementation should use the formatter-phase manifests as a structural template since `verification/tooling/formatter_phase_manifests/` does not exist either (the formatter manifests live elsewhere or are embedded in the phase plan).

To avoid structural uncertainty, M1 should create:
- `verification/tooling/linter_manifests/` directory
- `verification/tooling/linter_manifests/ruff_rule_config_audit.json`
- `verification/tooling/linter_manifests/lint_cli_parity.json`
- `verification/tooling/linter_manifests/suppression_gate.json`
- `verification/tooling/check_linter_reuse_rules.py`

### LOW — Phase-36 LSP Lint Integration Not Yet Wire-Through

**8. The phase requires `sifr_analysis` and `sifr_lsp` to use `sifr_lint` diagnostics, but the wire-through is not implemented**

Current `sifr_analysis/src/host.rs` calls `sifr_lint` in the diagnostics path. `sifr_lsp/src/diagnostics.rs` calls `host.diagnostics()` which routes through analysis. This is already structurally correct.

However, the code action handler (`sifr_lsp/src/requests/code_action.rs`) does not verify that the diagnostic being acted on is `Policy` class before offering suppression. The phase correctly identifies this as a M6 requirement, not a M1 gap. No action needed.

### LOW — File-Size Guardrail Not Verified for Lint CLI Modules

**9. The current lint CLI implementation lives in `cli_model_and_entrypoint.rs` (886 lines), `formatter_cli.rs` (66 lines), and `check_and_package_commands.rs` (716 lines)**

The `cli_model_and_entrypoint.rs` exceeds the 900-line hand-maintained-file guardrail at 886 lines. The linter addition to this file (adding the full `Lint` variant and its processing) will push it over the limit before M2 is complete.

**Recommended edit location:** `issues/ad-hoc-production-grade-sifr-linter.md`, add to M2 scope:

```
- split the lint CLI processing from `check_and_package_commands.rs` into a dedicated `lint_cli.rs` module once the lint command surface exceeds the file-size guardrail threshold; use the existing HIR lowering module layout as the structural example
```

This is not blocking. The file is currently under 900 lines. M2's lint CLI expansion will make the split necessary.

### LOW — Exit Code 3 Documentation Gap

**10. The phase defines exit code 3 for "internal compiler/linter failure" but the panic boundary in `run_with_panic_boundary` converts panics to diagnostics with code `INTERNAL_COMPILER_PANIC`, which currently exits 1 (EXIT_USER_DIAGNOSTIC)**

Current `cli_model_and_entrypoint.rs` line 827:
```rust
fn is_internal_diagnostic(error: &RenderedDiagnostic) -> bool {
    error.code == DiagnosticCode::INTERNAL_COMPILER_PANIC.code()
}
fn diagnostic_exit_code(errors: &[RenderedDiagnostic]) -> i32 {
    if errors.iter().any(is_internal_diagnostic) {
        EXIT_INTERNAL_COMPILER_FAILURE  // 3
    } ...
}
```

This is already correct for hard compiler diagnostics. The lint path (`cmd_lint` in `check_and_package_commands.rs` line 287) uses `render_diagnostics` which delegates to `diagnostic_exit_code`, so exit code 3 propagates correctly for internal failures. The phase does not need to change anything here.

### LOW — Editor Settings for Lint Are Planned But Not Wired

**11. The phase (lines 78–82) defines `sifr.lint.enable` as a required LSP setting, but the current `sifr_lsp/src/settings.rs` (lines 39–44) parses `lint_enable` and the LSP passes it to diagnostics mode, but the lint runner does not check `lint_enable` before running**

Current `sifr_lsp/src/diagnostics.rs` uses `DiagnosticsMode` to gate whether diagnostics are published. The `lint_enable` setting is parsed but not connected to the lint runner — lint diagnostics currently run regardless of this setting.

This is not a blocker for M1. The M7 scope (line 721) requires updating LSP/editor integration docs for lint diagnostics and settings. The wiring should be completed in M7.

### INFORMATIONAL — Phase Relationship Clarity

**12. The phase is marked "ad hoc" and the execution checklist item "Linter CLI parity manifest created" is correctly marked undone since this is the planning phase**

The relationship between this ad-hoc phase and Phase 36 is clear: Phase 36 completed the tooling contract, formatter foundation, LSP server, editor integrations, and VS Code extension. This phase builds the production linter on top of that foundation. No inconsistency between `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md` and this linter phase was found. The reuse strategy (`internal_docs/tooling_reuse_strategy.md`) is consistent with the Ruff reuse matrix in the phase plan.

---

## Items Confirmed Complete — No Issues

1. **Ruff reuse boundary is sound.** The phase correctly rejects `ruff_linter::rules::*`, `ruff_python_semantic`, Python plugin config blocks, Python version targets, Ruff `noqa`, Ruff `Rule` IDs, notebook support, and Python import resolution. The `adapt` vs `reference-only` vs `reject` taxonomy is applied consistently.

2. **Parser-aware suppression gate is correctly specified.** The `physical_line_only` to `parser_aware` transition is defined, the M3 milestone implements the transition, M5 cannot add non-physical-line rules unless the gate is closed, and `ParserAwareSuppressions` is referenced by name. The one precision improvement (Finding 3 above) is to name the exact Rust API path in W-8.

3. **Rule-family and config-surface audit is complete.** All 58 Ruff rule-family directories and all config surfaces are classified. No row reads "figure it out during implementation." The `extend-ignore` deprecated spelling is explicitly rejected. The manifest validation obligations are complete.

4. **CLI parity contract is complete.** All 47 Ruff `check` surfaces have rows. Hidden Ruff compatibility flags (e.g., `--no-fix`, `--no-show-fixes`) are explicitly rejected or adapted. The `--statistics` mutual-exclusion conflicts are documented. Exit codes 0/1/2/3 are precise. Output formats `concise`, `full`, `json` are M2 scope; extended formats are future-phase.

5. **Fix engine requirements are specified.** W-6 lists applicability, edit isolation, conflict resolution, source-map tracking, and idempotence. The phase correctly defers fix-capable rules to M6 after M3 parser-aware suppression and M4 phase-gated engine.

6. **Suppression syntax is Sifr-owned.** `# sifr: ignore[rule-id]` is canonical. Blanket suppressions are forbidden. Unknown and unused suppressions produce diagnostics. This matches the current implementation in `crates/sifr_lint/src/lib.rs`.

7. **Hard diagnostics remain unsuppressible.** This is enforced in the phase quality contract (AC-1, AC-5, AC-14, AC-15) and the implementation review gates.

8. **All previous review pass findings are applied.** Passes 1-5 for rule/config audit and passes 1-3 for CLI parity are all recorded in the execution tracker (lines 60-74) with evidence of resolution.

---

## Verdict

**The phase is implementation-ready. No blockers remain.**

The phase correctly defers the typed diagnostic class enforcement to M6 (Finding 4), the full lint CLI to M2 (Finding 5), lint TOML config to M2 (Finding 6), the manifest directory to M1 (Finding 7), and LSP lint-enable wiring to M7. All of these are planned, not forgotten.

The three precision improvements (Findings 1-3: manifest field population, suppression-gate `updated_by_milestone` format, and W-8 API path naming) are minor encoding details that M1 should address as part of encoding the manifests, not blockers.

M1 should start immediately and encode all three manifests, create `check_linter_reuse_rules.py` with the positive and negative self-tests, initialize the suppression gate manifest, and run `git diff --check` before opening the PR.
