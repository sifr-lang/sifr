## Review: Verification Taxonomy Cleanup

### Blocking findings

**1. CRITICAL — `validation_contracts/manifest.json` expects text that demos no longer print**
File: `verification/areas/project_workspace/data/validation_contracts/manifest.json`
- Line 75: expects `"project-workspace parity regression matrix demo:"` but `demos/mode_consistency/main.sifr:9` prints `"contract22_4 parity regression matrix demo:"`
- Line 233: expects `"single-file-project-graph-contract"` but `demos/graph_isolation/single_file/main.sifr:6` prints `"single-file-contract23_5"`
- Line 304: expects `"project graph and isolation regression matrix demo:"` but `demos/graph_isolation/main.sifr:8` prints `"contract23_5 graph and isolation regression matrix demo:"`

The contract assertion in `crates/sifr/tests/validation_contract_support/runner.rs:222` is a substring check on actual stdout. These three mismatches will fail `test_validation_contract_matrix` whenever the `project_workspace` area runs (cases `positive_project`, `single_file_layout_smoke`, `multi_file_import_closure_and_test`).

This passed your reported validation because:
- The `#[test]` is gated `#[ignore]` and only runs via verification area runners.
- `verification/profiles/create-pr.json` does **not** include `project_workspace` (line 63 onward — coverage_matrix, diagnostics, runtime_platform, algorithmic_compatibility, developer_tooling, generated_code_quality, performance, stdlib_parity only).
- `merge.json`, `nightly.json`, and `release.json` all do include it (`profiles/*.json:169-172`), so merge-gate CI will fail.

Fix: either restore the original text in the manifest, or update the three demo `print(...)` strings (and the matching `emitted.rs`/`idiomatic.rs`) to the new wording.

**2. Stale references to a renamed script in three internal docs**
Commit `11069afab` renamed `check_phase36_closeout.py` → `check_tooling_readiness.py`, but three docs still reference the old path:
- `internal_docs/typescript_go_architecture_transfer_bucketed_indexes.md:51-52`
- `internal_docs/typescript_go_architecture_transfer_lsp_latency_budgets.md:66-67`
- `internal_docs/typescript_go_architecture_transfer_lsp_cancellation_progress_watchdog.md:46-47`

These are validation-evidence lines (`python3 verification/areas/developer_tooling/check_phase36_closeout.py -> PASS`) and now point at a non-existent file. Either update to `check_tooling_readiness.py` or remove (and `check_phase36_closeout.py` still leaks the phase36 token even though the rename is done elsewhere).

### Non-blocking (cleanup gaps and guard scope)

**3. Guard regex misses `phase\d+` without a separator**
`TEXT_PATTERNS` only matches `phase[_-]…`, so `phase31`, `phase36`, `phase18` slip through. Remaining tracked leaks of this shape:
- `internal_docs/{tooling_analysis,tooling_verification,editor_integrations,vscode_extension,lsp_server}.md` line 3 — `status: phase36-contractXX.Y-...`
- `demos/{mut_sort,recursive_records,own_mut_updates,max_heap,dict_membership,paired_indices,tuple_assignment,normalized_fixtures,reverse_indices,sentinel_values,range_aliasing,…}/main.sifr` — `# Reference: phase31` or `# Reference: phase31_contract31<x>`
- Many demos still carry `# Source issue: phase{17,18,21,31}-…md` — these point at real (still-existing) `plans/issues/archive/phase…md` files, so the paths are valid. They're delivery taxonomy in spirit, but renaming requires renaming the underlying plan files (out of scope).

If you want broader coverage, extending the guard to `\b(?:phase|milestone|wave)\d+\b` would surface all of the above. Skip if you prefer to keep "phase18" only when it's a literal file path link.

**4. Surfaces not in `ACTIVE_ROOTS`**
- `.cursor/skills/phase-closure-loop/SKILL.md` — filename and body are entirely about phase/wave/milestone closure workflow (a tracked, non-plan workflow doc).
- `.cursor/commands/create-new-version.md:3,47` — still says "Phase 33". The companion `scripts/distribution/create_new_version.sh` was updated (Phase 33 → preview), so this doc is now out of sync with the script it documents.
- `lib/sifr/hashlib.sifr:140` — `# SHA3/SHAKE constructor placeholders for wave-level dependency-audit gating.` (stdlib comment, not in scanned roots).

Decide whether `.cursor/` and `lib/` should be in `ACTIVE_ROOTS`. They are tracked first-party surfaces.

**5. Editorial regressions from the rename**
- `scripts/distribution/create_new_version.sh:16` — `Plan or execute a preview release preview release.` — duplicated "preview release" after the substitution.
- `verification/areas/generated_code_quality/panic_inventory.md:3` — sentence now starts mid-clause: `historical diagnostic-panic inventory could not be located under …` (was "Phase 27's historical panic inventory…"). Needs a capitalized leading subject.

**6. Submodule (`editor_integrations/vscode`) has uncommitted edits**
The parent diff shows `editor_integrations: -dirty`. Inside the submodule:
- `editor_integrations/vscode/CHANGELOG.md` and `README.md` rewrite "Phase 36 extension scaffold" → "Contract 36 extension scaffold" (Phase 36 tooling contract → Contract 36 tooling contract). These are real renames but will not be committed by the parent repo. They need a separate commit in the submodule, plus a pointer bump in the parent.

### Things that look correct

- Guard scope expansion (`crates`, `demos`, `docs`, `internal_docs`, `editor_integrations`, `.github/workflows`, `scripts/distribution`, plus all the listed verification subareas) is coherent. The `reports` skip dropped, `node_modules` added — both deliberate and right.
- Allowlist (`WorkspaceTracePhase`, `SingleOwnerCompilerPhase`, `LintPhase`, `phase_plan`, `record_compiler_phase_trace`, etc.) correctly protects real compiler/lint APIs; `cargo` build and lint code is untouched (no `crates/` modifications in the diff).
- `corpus_manifest.json` rename from `concurrency-runtime-m7` (×7) → `concurrency-runtime-closeout` (×7) is consistent across the manifest, `POSITIVE_GROUPS`, `REQUIRED_GROUP_COUNTS`, and `CONCURRENCY_CLOSEOUT_DEMOS` in `generated_code_quality.py`; counts still total 96 entries.
- The 16 `git mv` renames under `internal_docs/typescript_go_architecture_transfer_*` are coherent (rename + content updated, `RM` status), and the matching `M14` / `M5` / etc. text was rewritten to "contract slice N" in-body.
- Demo `Reference:` / `print()` / `emitted.rs` / `idiomatic.rs` triplets for the renamed taxonomy (auto_detection, classes, mode_consistency, graph_isolation, branch_paths, control_flow_paths, etc.) are updated in sync — except for the three mismatches called out in finding 1.

### Recommendation

Block on findings 1 and 2; the others are quality follow-ups. The validation contract regressions are the only finding that breaks tests; everything else is residual taxonomy or doc hygiene.
