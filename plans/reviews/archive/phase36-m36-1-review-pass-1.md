

## Phase 36 m36.1 Review — Production Tooling Contract Lock

### SATISFIED

---

### Files Reviewed

| Category | Files |
|---|---|
| New docs (5) | `tooling_analysis.md`, `lsp_server.md`, `vscode_extension.md`, `editor_integrations.md`, `tooling_verification.md` |
| New contracts (2) | `lsp_protocol_matrix.json`, `vscode_extension_contract.json` |
| New guardrails (4) | `check_tooling_contract_lock.py`, `check_tooling_dependency_boundaries.py`, `check_lsp_split_brain.py`, `check_vscode_extension_contract.py` |
| New placeholder (1) | `check_vscode_extension.py` |
| Execution trackers (2) | `phase36-developer-tooling-execution.md`, `phase36-vscode-extension-production-execution.md` |
| Script wiring (1) | `scripts/run_all_tests.sh` |

---

### Findings (ordered by severity)

**NONE.** No issues found at any severity level.

---

### Positive Evidence by Focus Area

**1. Contract lock completeness**

All 7 contract artifacts exist and agree. Crate names (`sifr_analysis`, `sifr_format`, `sifr_lint`, `sifr_lsp`) are locked in the phase doc execution status block, `lsp_protocol_matrix.json`, `tooling_analysis.md`, `tooling_verification.md`, and `lsp_server.md`. VS Code repo boundary (`sifr-lang/sifr-vscode`) is locked in the phase doc, `vscode_extension_contract.json`, `vscode_extension.md`, and `tooling_verification.md`.

**2. LSP protocol matrix coverage**

`lsp_protocol_matrix.json` covers all 38 required methods from the phase contract (lines 48–88). All 6 required workspace commands are present (lines 89–96). All 4 required settings are present (lines 42–47). All 3 diagnostics modes are present (line 17). Semantic token legend contains all 14 categories from the contract (lines 18–34). All 5 code action kinds are present (lines 35–41). All 8 unsupported surfaces are present including `notebookDocumentSync`, `python.importResolution`, and `python.environmentDiscovery` (lines 97–107). Every entry has positive and negative coverage plus an owner mapping.

**3. VS Code extension contract**

`vscode_extension_contract.json` correctly locks `separate-repository` boundary, `sifr-lang/sifr-vscode` full name, `sifr-lang.sifr-vscode` extension id, `^1.90.0` min engine, `sifr` default command with `["lsp", "--stdio"]` args, all 8 required commands, all 5 required settings, and 12 forbidden behavior entries. `check_vscode_extension_contract.py` validates the main-repo contract in m36.1 and fails with an actionable message if the extension checkout is missing when Phase 36 extension validation is active — it does not silently skip before m36.7. The `activation_milestone` field correctly records `"36.7"` for extension repo validation.

**4. Guardrail quality**

Each guardrail has a meaningful self-test:
- `check_tooling_contract_lock.py --self-test` removes `textDocument/completion` from the matrix and verifies the check fails.
- `check_tooling_dependency_boundaries.py --self-test` seeds `ty_python_semantic` in a temp file and verifies detection.
- `check_lsp_split_brain.py --self-test` seeds `HirModule` and `lower_module(` in a temp file and verifies detection.
- `check_vscode_extension_contract.py --self-test` removes `sifr.runCheck` from a temp package.json and verifies failure.

`check_tooling_contract_lock.py` validates crate names, diagnostics modes, token legend, settings, methods with owner/coverage requirements, commands, and unsupported surface presence — it is not merely a file-existence check.

`check_lsp_split_brain.py` correctly excludes `AnalysisHost::generated_rust_preview` from pattern matching (allowed snippet), checks `crates/sifr_lsp/` if and only if it exists (no false positive on missing crate), and scans line-by-line for fine-grained error reporting.

`check_tooling_dependency_boundaries.py` limits scanning to `{crates,editors,vscode,packages}/**` and skips `target/`/`.git` correctly. File extensions are restricted to `.rs`, `.toml`, `.json`, `.ts`, `.js`, `.mjs`, `.cjs` — no false positives on `.md` or `.py` containing pattern strings.

**5. `scripts/run_all_tests.sh` wiring**

The new "Developer Tooling Checks" block (lines 123–131) wires all 6 commands with self-tests. Commands are identical between local validation and CI. The block runs in the main lane (not profile-gated), consistent with other Phase 35 guardrails.

**6. Tracker accuracy**

`issues/phase36-developer-tooling-execution.md` records all 7 scope items as `[x]` complete with accurate validation evidence including the transient build timeout and successful rerun. The branch name, artifact locations, and checklist state are correct.

`issues/phase36-vscode-extension-production-execution.md` correctly states the separate-repo decision made in m36.1, the non-negotiable boundary, the sequential position (m36.1 before m36.7 before m36.8), and the PR sequence mapping to `milestone_36_1`. The cross-repository validation section accurately describes the main-repo contract check behavior.

**7. Split-brain analysis**

No split-brain loopholes found:
- `sifr_lsp` has no existing implementation to audit, so `check_lsp_split_brain.py` correctly passes when the crate is absent. The pattern set will catch direct semantic paths when the crate is added in m36.5.
- `check_tooling_dependency_boundaries.py` scans the full crate workspace, not just tooling paths, catching forbidden deps in any production code.
- VS Code contract correctly forbids `pythonLanguageServerFallback`, `pyrightFallback`, `ruffServerFallback`, `tyServerFallback`.
- Editor integrations doc locks the shared rule that all integrations delegate to `sifr lsp --stdio`.

**8. JSON contract validity**

Both JSON files pass `json.tool`. The matrix has `schema_version: 1`, `phase: "36"`, `milestone: "36.1"`, `validation_stage: "contract-lock"` consistently. The LSP matrix covers LSP 3.17, stdio transport, and the exact `sifr lsp --stdio` launch command.

---

### No Issues Identified

- No missing required methods, commands, settings, or semantic token categories.
- No false positives in guardrail scripts (self-tests pass).
- No CI-only behavior introduced.
- No silent skip of extension validation before m36.7 — the contract check requires `--require-extension-repo` flag to activate that path.
- No false negatives: `check_lsp_split_brain.py` patterns cover `sifr_python_parser`, `ruff_python_parser`, `parse_unchecked`, `parse_module_with_diagnostics`, `lower_module(`, `lower_frontend_module`, `type_check`, `HirModule`, `sifr_codegen::` — all paths that would constitute split-brain violations.
- No incorrect file paths in guardrail scripts.
- No gaps in unsupported surfaces coverage.
- No milestone sequencing risks in the contracts.

---

### Validation Already Confirmed by User

All validation commands listed in the request passed:
- All 4 guardrail checks + self-tests: PASS
- `json.tool` on both JSON contracts: PASS
- `py_compile` on all Python checks: PASS
- `cargo fmt --check`: PASS
- `git diff --check`: PASS
- `scripts/run_all_tests.sh --profile quick`: PASS on rerun

---

### m36.1 Can Proceed to PR

The contract lock is complete, consistent, and self-verifying. All required artifacts are present, correctly structured, and wired into local validation. The phase contract (`36_developer_tooling_and_ecosystem_hooks.md`) is updated to reflect the locked state. Execution trackers accurately reflect completed work. No blockers remain for opening the m36.1 PR.
