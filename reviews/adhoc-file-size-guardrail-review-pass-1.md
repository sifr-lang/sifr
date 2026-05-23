

Based on my review, I need to compile my findings. The review file appears to be empty so I'll write the output directly.

---

## REVIEW: `issues/adhoc-file-size-guardrail.md`

**Result: CHANGES_REQUESTED**

---

### Finding 1: Milestone 2 target list includes compliant files

Milestone 2 lists 16 files as targets. But several are already at or below 900 lines:

| File | Current lines |
|------|--------------|
| `hir_analysis/queries.rs` | 940 |
| `hir_analysis/traversal.rs` | 940 |
| `stdlib_filter.rs` | 969 |
| `lower_item.rs` | 1034 |
| `intrinsics/mod.rs` | 1312 |
| `intrinsics/test.rs` | 160 |

These don't belong in the target list. Including them inflates PR scope without purpose.

**Required edit**: Remove files already at or below 900 lines from the milestone 2 target list. Add a note that only files exceeding the cap are targeted.

---

### Finding 2: `nested_function_inference.rs` is not in the target list or the baseline

The current HIR files are: `mod.rs`, `imports.rs`, `diagnostics.rs`, `classes.rs`, `typing_and_functions.rs`, `statements.rs`, `expressions.rs`, `builtin_calls.rs`, `stdlib/*.rs`. 

Milestone 3 targets `crates/sifr_hir/src/lower/nested_function_inference.rs` but this file doesn't exist in the codebase. Meanwhile `builtin_calls.rs` (1169 lines) exceeds 900 but is not listed as a milestone 3 target.

**Required edit**: Remove the non-existent `nested_function_inference.rs` from the milestone 3 target list. Add `builtin_calls.rs` if it exceeds 900 and needs work.

---

### Finding 3: Scope mismatch on Python tooling files

The scope covers `scripts/**/*.py` and `verification/**/*.py`, but the plan only targets `scripts/run_verification_hardening.py` (1962 lines). 

The baseline states "1 Python tooling file under scripts/". The plan should clarify whether Python files are in scope and, if so, which ones are violations. Currently there's no explicit Python decomposition plan beyond the verification hardening script.

**Required edit**: Either explicitly scope Python files out of this phase, or add a specific milestone addressing Python tooling file violations. The current language leaves ambiguity.

---

### Finding 4: Demos/Sifr fixture files are in scope but unaddressed

The scope explicitly includes `demos/**/*.sifr` and `crates/sifr/tests/**/*.sifr`. Zero milestones target `.sifr` files. If any `.sifr` fixture exceeds 900 lines, the plan provides no path to compliance.

**Required edit**: Either confirm no `.sifr` files exceed 900 lines (with a scan to verify), or add a milestone addressing Sifr fixture violations.

---

### Finding 5: Transition from per-file override scripts is underspecified

The existing guardrails use per-file override budgets (e.g., `expressions.rs` has a 3800-line cap in `check_hir_maintainability_guardrails.py`). The unified guardrail uses path-pattern-based exclusion with a uniform 900-line cap.

Milestone 3 says to retire older HIR file-size logic but doesn't explain the mechanism. Specifically:
- After milestone 3, the per-file budgets in `MAX_LINES_BY_FILE` will be obsolete, but the script remains in `run_all_tests.sh`.
- The plan says "Replace or retire narrower file-size checks in older maintainability scripts only when the unified guardrail covers their behavior" — but no validation step confirms the unified guardrail actually covers them.

**Required edit**: Add a milestone 3 validation step that confirms the unified guardrail's path-pattern logic (when implemented in milestone 5) covers all files previously governed by per-file overrides. Alternatively, clarify in milestone 5's description that it must include a migration plan for `MAX_LINES_BY_FILE` entries.

---

### Finding 6: Checklist docs need transition plan

`internal_docs/hir_maintainability_guardrails.md` and `internal_docs/sifr_driver_maintainability_guardrails.md` both contain review checklists with file-size items. When the unified guardrail supersedes these, the checklists need updating.

The plan mentions retiring "narrower file-size checks in older maintainability scripts" but doesn't address the checklist documents. An agent following the Phase 20 checklist after milestone 5 completes would be checking against obsolete per-file limits.

**Required edit**: Add a milestone 5 step or done-criterion item: "Update or retire `internal_docs/hir_maintainability_guardrails.md` and `internal_docs/sifr_driver_maintainability_guardrails.md` to reference the unified guardrail."

---

### Finding 7: Milestone 5 validation doesn't test the exclude path

Milestone 5's guardrail script describes `--self-test` mode but doesn't specify what it tests. The plan lists:
- include/exclude behavior
- failure behavior with actionable output

But it doesn't specify:
- That `--self-test` creates temporary fixture trees with both included and excluded files
- That it validates that excluded paths (generated, `target/**`, `third_party/**`) are never flagged
- That it validates the 900-line threshold is enforced, not accidentally 1000 or 800

Without these specifics, an agent implementing the guardrail script can't know if their `--self-test` is complete.

**Required edit**: Expand the `--self-test` description to specify: creates temp fixture trees, proves excluded paths never fail, proves 900-line threshold is enforced with correct failure output (path + line count + limit + category).

---

### Finding 8: Milestone 4 validation is weak on fixture determinism

The milestone 4 validation items are:
- `cargo test -p sifr_hir` (implicit, uses `--skip test_e2e_pass`)
- `scripts/run_e2e_pass.sh`

These test that tests still run, but not that fixture discovery order and snapshot names remain deterministic after the e2e harness is split across discovery/execution/expectation/reporting modules.

**Required edit**: Add an explicit validation step for milestone 4 that confirms fixture order determinism: e.g., a script that enumerates all e2e fixture paths under two conditions (before split, after split) and asserts the sorted list is identical.

---

### Finding 9: Milestone 1-4 validation plans lack self-test parity with milestone 5

Milestone 5 explicitly describes `--self-test` mode for the guardrail script. Milestones 1-4 say things like "generated-code corpus checks for changed emission surfaces" and "existing maintainability guardrails" — which are not self-contained validations, they're references to other checks.

For a reviewer to validate milestone 4's e2e split, they need an explicit test that proves fixture order and snapshot stability, not just "run the existing e2e suite."

**Required edit**: For each milestone, replace vague validation references with explicit, self-contained validation steps. E.g., milestone 4 should have "validate_fixture_order.py" that runs as part of the milestone 4 PR.

---

### Finding 10: AGENTS.md addition doesn't guide decomposition

The AGENTS.md addition says:
> "If a touched file exceeds the cap, refactor it by responsibility rather than adding more code to an oversized module."

This tells agents to refactor by responsibility but doesn't describe what "responsibility" means in the Sifr context. An agent might naively split a 2000-line file into 500-line chunks by function type or alphabetical order, which would violate the intent.

**Required edit**: Add a brief guidance note in the AGENTS.md section explaining what responsibility-based decomposition looks like in this codebase. Reference the existing Phase 20 and Phase 31 guardrail checklists as models. E.g., "Use the HIR lowering module layout as a reference for decomposition: `expressions.rs`, `statements.rs`, `classes.rs` are split by semantic responsibility, not mechanical count."

---

### Summary of required edits

1. **Milestone 2 target list**: Remove files already ≤ 900 lines. Add note that only violations are targeted.
2. **Milestone 3 target list**: Remove non-existent `nested_function_inference.rs`. Add `builtin_calls.rs` if it's a violation.
3. **Python tooling scope**: Explicitly scope Python files in or out of this phase.
4. **Demos/Sifr fixtures scope**: Confirm no `.sifr` files exceed 900 or add a milestone to address them.
5. **Transition plan for override scripts**: Add milestone 3 or 5 step confirming the unified guardrail covers all files previously governed by per-file `MAX_LINES_BY_FILE` entries.
6. **Checklist doc transition**: Add done-criterion item to update/retire `hir_maintainability_guardrails.md` and `sifr_driver_maintainability_guardrails.md`.
7. **Milestone 5 `--self-test` description**: Specify exact behaviors (temp fixture trees, exclude-path validation, threshold enforcement).
8. **Milestone 4 fixture determinism**: Add explicit step validating fixture order stability.
9. **Validation self-containment**: Replace vague validation references in milestones 1-4 with explicit, self-contained steps.
10. **AGENTS.md guidance**: Explain what "responsibility-based decomposition" means in the Sifr context.
