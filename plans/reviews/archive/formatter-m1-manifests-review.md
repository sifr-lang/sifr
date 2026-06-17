

Based on my thorough review of the Milestone 1 working tree, here are my findings ordered by severity:

---

## CRITICAL - Blocks Milestone 1 Close

**1. Phase 36 closeout guardrail fails on current branch**

File: `verification/tooling/check_phase36_closeout.py:94-96`

The closeout script checks for `issues/phase36-developer-tooling-execution.md`, but commit `c9de784f8` ("archive issues and reviews") moved this file to `issues/archive/phase36-developer-tooling-execution.md`. The file path reference was never updated.

```
File does not exist: issues/phase36-developer-tooling-execution.md
→ check_phase36_closeout.py:validate_required_files() → FAIL
→ scripts/run_all_tests.sh fails at Phase 36 closeout guardrail
```

This is a pre-existing bug on `main` that predates Milestone 1. Every validation run since `c9de784f8` landed has hit this guardrail, including all Milestone 1 validation attempts. The execution tracker correctly identifies this ("stops at the existing archived Phase 36 closeout guardrail") but does not record it as a required fix.

**Required action**: Update `check_phase36_closeout.py:16` to reference the archived file:
```python
ISSUE_DOC = REPO_ROOT / "issues" / "archive" / "phase36-developer-tooling-execution.md"
```
This is a one-line fix. Without it, no PR from this branch can pass `scripts/run_all_tests.sh --profile quick`.

---

## APPROVED - No Remaining Blockers

All other aspects are correct. Documenting for completeness:

**2. Manifest fidelity (capability_matrix.json)**  
35 rows, classifications match phase contract exactly, no deferred/unclassified rows. `implementation_requirement` strings are verbatim from the reviewed planning lock. Only `not-exposed` and `not-applicable` rows use explicit phase-deferral language ("future language-edition phase", "later product phase") which is the reviewed decision, not an implementation gap.

**3. Manifest fidelity (cli_parity.json)**  
22 rows, `required_fixture` names populated for all rows, `not-applicable` and `not-exposed` rows have reviewed rationales embedded in the contract table. The two special-case exemptions at line 121-126 (`stdin without files` and `global logging flags`) match the documented non-flag behavioral requirements.

**4. Manifest fidelity (ast_coverage.json)**  
11 rows exactly matching `REQUIRED_AST_ROWS` in `check_formatter_phase_manifests.py`. All `pending:m2`/`pending:m3` markers are correct milestones per the phase plan. No unclassified AST coverage gaps introduced.

**5. Manifest fidelity (ruff_baseline.json)**  
Correctly encodes the merged seed PR contract: `sifr-lang/ruff#1` as `b251656613629e054308951a4df1928b3f749b1b` on `sifr/0.15.12-maintenance`. `forbidden_dependency_modes` array correctly forbids the 4 prohibited patterns (ruff CLI subprocess, deleted feature branch, local patch, sifr wrapper post-processing).

**6. Ruff CLI marker checks**  
`check_formatter_phase_manifests.py:34-51` defines 16 markers. I verified all 16 exist in `third_party/ruff/crates/ruff/src/args.rs`:
- `pub check: bool` ✓ (lines 508, 1120)
- `pub diff: bool` ✓ (lines 256, 512, 1102, 1122)
- `pub no_cache: bool` ✓ (lines 417, 1107, 1121)
- `pub cache_dir: Option<PathBuf>` ✓ (lines 420, 519)
- `respect_gitignore: bool` / `no_respect_gitignore: bool` ✓ (lines 396-398, 528-530)
- `force_exclude: bool` / `no_force_exclude: bool` ✓ (lines 406-408, 547-549)
- `pub exclude: Option<Vec<FilePattern>>` ✓ (lines 340, 538)
- `pub line_length: Option<LineLength>` ✓ (lines 411, 552)
- `pub stdin_filename: Option<PathBuf>` ✓ (lines 423, 555, 1112, 1124)
- `pub extension: Option<Vec<ExtensionPair>>` ✓ (lines 427, 559)
- `pub target_version: Option<PythonVersion>` ✓ (lines 280, 562)
- `preview: bool` / `no_preview: bool` ✓ (lines 211-213, 284-286, 566-568, 605-607)
- `pub range: Option<FormatRange>` ✓ (lines 583, 1125)

**7. Ruff fork baseline check**  
`check_formatter_phase_manifests.py:167-203` uses 5 distinct git checks. Verified `third_party/ruff` HEAD is `b251656613629e054308951a4df1928b3f749b1b` (confirmed via `git submodule status` and `git -C third_party/ruff rev-parse HEAD`). Seed commit subject verified: "Format Sifr parameter conventions (#1)". Required paths verified present in seed commit via manifest checks.

**8. Fixture revision metadata**  
`ruff_fork_revalidation.json` and all 5 syntax token fixtures (`basic_module`, `class_and_methods`, `async_and_error_handling`, `collections_and_generics`, `control_flow_match`) record `b251656613629e054308951a4df1928b3f749b1b`. Rationale in `ruff_fork_revalidation.json` correctly states the seed commit changes Ruff formatter integration points only, so token fixture expectations remain valid.

**9. Test wiring**  
`scripts/run_all_tests.sh:142-143` wires `check_formatter_phase_manifests.py` and its self-test after `check_formatter_rules.py` — correct position after Phase 36 formatter contract check. No existing checks weakened.

**10. Execution tracker accuracy**  
Lines 390-391 accurately describe: new formatter manifest gate passes, lane reaches Phase 36 closeout guardrail, Phase 36 doc was archived. No overstatement of quick-lane success. `target/validation_lane_reports/quick.latest.json` exists as stated.

**11. File sizes**  
- `check_formatter_phase_manifests.py`: 268 lines (well under 900-line cap) ✓
- `ruff_fork_revalidation.json`: 6 lines (small JSON config, appropriate) ✓

---

## Summary

Milestone 1 deliverables are correct:
- 4 manifests faithfully encode the reviewed phase tables with no deferred/unclassified decisions
- manifest check script is appropriately strict for Milestone 1 scope
- Ruff fork baseline is correctly pinned and verified
- syntax token fixture revision metadata is correct for the merged seed
- test wiring is in the right lane

**One required fix**: `check_phase36_closeout.py:16` must reference `issues/archive/phase36-developer-tooling-execution.md` instead of the non-existent `issues/phase36-developer-tooling-execution.md`. This is a pre-existing bug introduced by the archive commit, not by Milestone 1 work.

**Once that one-line fix is applied, Milestone 1 is approved to close and Milestone 2 may begin.**
