

Based on my comprehensive review of all seven documents, here's the verdict:

---

## VERDICT: NOT READY

The documentation is architecturally complete and well-structured. However, both phases are `status: planned` and unimplemented. Phase 35 must complete before Phase 36 begins. Additionally, one cross-document contradiction and several cross-repo validation gaps must be resolved before Phase 36 can start.

---

## Blocking Issues

### B1: Phase 35 Not Started (Blocks Phase 36 Entry)
Both Phase 35 and Phase 36 are `status: planned`. The `verification/performance/` directory doesn't exist. The workspace `Cargo.toml` has no `sifr_syntax`, `sifr_frontend`, `sifr_analysis`, or `sifr_lsp` crates. Phase 35 must fully exit before Phase 36 entry criteria are satisfied.

**Fix:** Complete Phase 35 through all 8 exit criteria before beginning Phase 36.

---

### B2: VS Code Extension Repo Does Not Exist
The issue doc requires `sifr-lang/sifr-vscode` but no such repository has been created. Milestone 36_7 cannot be started, and the `milestone_36_1` repo-boundary decision is moot until the repo exists.

**Fix:** Create `sifr-lang/sifr-vscode` as an empty repo before `milestone_36_1` ships.

---

### B3: Phase 36 Exit Gate Missing Extension-Repo Validation Coordination
Phase 36's exit gate says: "`scripts/run_all_tests.sh --profile quick` passes in the main repo" and "Extension repo CI passes" and "Reviewer approves." But Phase 36's own quality contract says all tooling checks must be in `scripts/run_all_tests.sh --profile pr`.

The main repo's quick/PR validation has no hook to validate the extension repo unless `check_vscode_extension.py` is run from the main repo against a sibling/pinned checkout. The issue doc says main-repo validation "can locate" a sibling checkout, but doesn't specify how or what happens when it can't find it (error vs skip).

**Fix:** In `verification/tooling/check_vscode_extension.py` and `vscode_extension_rules.json`, add:
```python
# Detect extension repo: 
# 1. Try sibling `../sifr-vscode` relative to repo root
# 2. Try SIFR_VSCODE_REPO env var (absolute path)
# 3. If not found: fail with "extension repo not found; set SIFR_VSCODE_REPO or clone sibling"
#    (do NOT skip silently)
```

---

### B4: Cross-Document Contradiction - Verification Script Names
| Document | Script name |
|---|---|
| Phase 36 `verification/tooling/` list | `check_vscode_extension.py` |
| Issue doc (Cross-Repository Validation) | `check_vscode_extension.py` |
| Issue doc (PR Sequence) | "validation evidence" - undefined script |

Phase 36 also lists `check_vscode_extension.py` in its exit criteria, but the issue doc's PR sequence item 7 says "main repo PR: cross-repo contract check, documentation, and validation evidence" without naming the script. This mismatch means the exit gate references a script the PR sequence doesn't define, and Phase 36's verification infrastructure list never includes a cross-repo contract script for the extension (it only includes `check_vscode_extension.py` which runs in the extension repo).

**Fix:** Phase 36 should add `verification/tooling/check_vscode_extension_rules.py` to its `verification/tooling/` list, consuming `vscode_extension_rules.json` from the main repo and validating against the extension repo at `SIFR_VSCODE_REPO`. The issue doc's PR sequence item 7 should name this script explicitly.

---

### B5: Phase 36 `verification/tooling/` List Missing Cross-Repo Contract Script
Phase 36's required verification files include `check_vscode_extension.py` but not the main-repo-side cross-repo contract validator. The `check_vscode_extension.py` runs in the extension repo; the main repo needs its own validator to enforce the contract as part of `scripts/run_all_tests.sh --profile pr`.

**Fix:** Add `verification/tooling/check_vscode_extension_rules.py` to Phase 36's required files list, with scope:
- Read `vscode_extension_rules.json`
- Locate extension repo via `SIFR_VSCODE_REPO` or sibling path
- Validate: extension id, language id, launch command, required settings, required commands, no forbidden semantic dependencies in `package.json`
- Fail if extension declares a type-checker, parser, formatter, or linter setting

---

### B6: Phase 36 `verification/tooling/` List Missing `check_lsp_split_brain.py` Description
Phase 36's list of required verification files mentions `check_lsp_split_brain.py` but the description only says "verifies LSP handlers do not import or traverse forbidden semantic internals directly." It should clarify that this includes forbidding `ty_python_semantic`, `ty_project` Python semantics, and any direct HIR traversal for semantic answers.

**Fix:** In the Phase 36 verification infrastructure section, update the `check_lsp_split_brain.py` description:
> `check_lsp_split_brain.py` - verifies LSP handlers do not import or traverse forbidden semantic internals directly, including `ty_python_semantic`, `ty_project` Python semantics, `ruff_server` diagnostics as Sifr behavior, Python module-resolution paths, and direct HIR traversal for semantic answers.

---

## Non-Blocking Nits

### N1: PR Sequence Items Don't Map Cleanly to Phase 36 Milestones
The issue doc's 8 PR sequence items don't align 1:1 with Phase 36's 8 milestones. For example:
- Issue doc PR 1 (main repo lock) corresponds to `milestone_36_1`
- Issue doc PRs 2-6 (extension PRs) are all part of `milestone_36_7`
- Issue doc PR 7 (cross-repo contract) is between `milestone_36_7` and `milestone_36_8`
- Issue doc PR 8 (phase closeout) is `milestone_36_8`

This isn't wrong but it could confuse future implementers. Consider a footnote in the issue doc mapping each PR to its milestone.

### N2: Phase 35 `lsp-query` Budget IDs Are Reserved But Not Documented
Phase 35 reserves `lsp-query` budget IDs for Phase 36 (Phase 35 exit criteria and manifest sections). However, Phase 35 doesn't document the exact reserved budget ID names. Phase 36's budgets section defines defaults but says "final budgets must be derived from checked-in baselines." There's no gap here, but consider adding a `verification/performance/lsp_query_budget_ids.txt` or `verification/performance/budget_ids_reserved.md` in Phase 35 that documents the reserved IDs so Phase 36 doesn't have to invent names.

### N3: Phase 36 Exit Gate Should Clarify That Extension Validation Is Part of Main Repo Quick/PR
The current exit gate says "`scripts/run_all_tests.sh --profile quick` passes in the main repo." An implementer could reasonably conclude this doesn't cover the extension. Clarify:
> `scripts/run_all_tests.sh --profile quick` passes in the main repo, including `verification/tooling/check_vscode_extension_rules.py` when `SIFR_VSCODE_REPO` is set.

### N4: Phase 36 LSP Budget Defaults Are Not in `verification/performance/budgets.json`
Phase 36 lists 15 `lsp-query` budget defaults (cold start <=1000ms, diagnostics <=500ms, etc.) but these are "phase-start defaults" that must be recorded in `verification/performance/budgets.json` after baseline capture. This is by design but worth noting: the budgets section of Phase 36 is not actionable until Phase 35's `budgets.json` exists and Phase 36 adds its IDs.

### N5: Missing "VS Code Engine Version" Policy Decision
Phase 36 says "minimum VS Code engine version" but doesn't specify which engine version. The issue doc lists it as a checkbox item but leaves the value TBD. This is fine for planning but should be resolved in `milestone_36_1` before the extension is scaffolded.

---

## Summary of Required Fixes Before Implementation

| Priority | Fix | Location |
|---|---|---|
| **Blocking** | B1: Complete Phase 35 fully before Phase 36 | Phase 35 exit gate |
| **Blocking** | B2: Create `sifr-lang/sifr-vscode` empty repo | External (GitHub) |
| **Blocking** | B3: Add `SIFR_VSCODE_REPO` detection to contract check (fail, don't skip) | Phase 36 verification infra + issue doc |
| **Blocking** | B4: Add `check_vscode_extension_rules.py` to Phase 36's required files | Phase 36 verification infra |
| **Blocking** | B5: Add main-repo cross-repo contract validator to Phase 36 | Phase 36 verification infra |
| **Blocking** | B6: Expand `check_lsp_split_brain.py` description to include all forbidden paths | Phase 36 verification infra |
| Nit | N1: Add footnote mapping issue PR sequence to Phase 36 milestones | Issue doc |
| Nit | N2: Document reserved `lsp-query` budget ID names in Phase 35 | Phase 35 verification infra |
| Nit | N3: Clarify quick/PR gate covers extension contract check | Phase 36 exit gate |
| Nit | N4: No fix needed; confirm by design | - |
| Nit | N5: Resolve minimum engine version in `milestone_36_1` | Phase 36 milestone 36_1 |
