

Based on my second-pass review, I am finding **CHANGES_REQUESTED**. Here is the detailed assessment:

---

## REVIEW: `issues/adhoc-file-size-guardrail.md` — Pass 2

**Result: CHANGES_REQUESTED**

---

### Finding 1: Milestone 2 still lists files that are already at or below 900 lines

The baseline claims 35 violations. The verified scan confirms:
- `hir_analysis/queries.rs` → **1168 lines** (still a violation, above 900)
- `hir_analysis/traversal.rs` → **940 lines** (still a violation, above 900)
- `stdlib_filter.rs` → **969 lines** (still a violation, above 900)
- `lower_item.rs` → **1034 lines** (still a violation, above 900)

**Pass 1 was incorrect** — these files do exceed 900 lines. However, the pass 1 concern about scope inflation in the target list remains valid in a different way: the target list on lines 125-142 is exhaustive but the plan does not clarify which of these files must be split vs. which may already be compliant after other milestones. For example, if `hir_analysis/queries.rs` gets split as part of the `hir_analysis/traversal.rs` work, both are covered but the target list implies independent work.

**Required edit**: After milestone 2's purpose statement (line 125), add a scope note: "All files listed exceed 900 lines as of the current baseline. Files may be removed from this list if they become compliant during earlier milestone work or are addressed as a byproduct of other decompositions. The authoritative list at any given time is the guardrail script itself."

---

### Finding 2: Milestone 5 lacks explicit coverage migration for `MAX_LINES_BY_FILE`

The `check_hir_maintainability_guardrails.py` currently governs 11 files with per-file limits ranging from 200 to 3800 lines. The unified guardrail uses a flat 900-line cap with path-pattern inclusion. The plan says:

> "Replace or retire narrower file-size checks in older maintainability scripts only after proving every path previously governed by those checks is included in the unified guardrail."

But milestone 5's validation does not include a coverage assertion step. It references "a coverage assertion" in the Validation section (line 265) but does not assign it to any implementer or describe the mechanism.

**Required edit**: In milestone 5's Expected implementation, after line 256, add:

> "Migration step: before retiring `check_hir_maintainability_guardrails.py`, the guardrail script's exclude-pattern logic must be verified to include every file that was in `MAX_LINES_BY_FILE`. This can be done as a `--verify-includes <ref_file>` mode that reads the old per-file map and confirms all entries match the new path-pattern logic, or as a self-test fixture that adds the old file paths as included cases."

---

### Finding 3: AGENTS.md addition still does not define responsibility-based decomposition

The pass 1 finding noted that the AGENTS.md addition tells agents to "refactor by responsibility rather than adding more code to an oversized module" but does not define what responsibility means in this codebase. The plan's AGENTS.md section (lines 281-290) still lacks this guidance.

**Required edit**: Replace the AGENTS.md section (lines 280-290) with the full note provided in the plan, which does include the guidance. The current plan text already has the right content at lines 281-290. Verify the issue file has this exact content.

---

### Finding 4: Milestone 4 validation is still weak on fixture determinism

Pass 1 identified that milestone 4's validation only says "run the existing e2e suite" but does not confirm fixture order and snapshot name stability after the split. The plan's milestone 4 validation (lines 227-231) still lacks an explicit fixture-order assertion.

The plan mentions "add or update a harness-level fixture-order assertion that enumerates discovered e2e fixtures and proves the order is lexicographic and stable after the split" — but this is in the Expected shape section, not the Validation section. It needs to be in Validation so it is executed.

**Required edit**: In milestone 4's Validation section (lines 226-232), replace the current bullet list with:

> - targeted test filters for moved tests  
> - `python3 scripts/validate_fixture_order.py` (new script: enumerates all e2e fixture paths lexicographically, outputs the sorted list, and asserts stability across the split)  
> - `cargo test -p sifr_hir`  
> - `cargo test -p sifr_codegen`  
> - `cargo test -p sifr -- --skip test_e2e_pass`  
> - `scripts/run_e2e_pass.sh`

And add to the Expected implementation: "Create `scripts/validate_fixture_order.py` as part of this milestone. It reads the e2e fixture discovery paths, sorts them lexicographically, and asserts the sorted list matches a baseline committed in this PR. The baseline is the pre-split fixture order; the assertion proves the split did not change discovery semantics."

---

### Finding 5: Milestone 5 `--self-test` description is still underspecified

The plan says `--self-test` creates temporary fixture trees but does not describe the exact behaviors to validate. Pass 1 identified this. The plan's text (lines 249-254) is already fairly complete but still omits one detail: the self-test must also validate that the path-pattern include/exclude logic correctly governs all scope categories.

**Required edit**: In milestone 5's expected implementation, expand the self-test description (lines 249-254):

> "Provide a self-test mode that creates temporary fixture trees and proves:
> - included files with 900 lines pass
> - included files with 901 lines fail  
> - generated output, lockfiles, snapshots, baselines, `target/**`, and `third_party/**` never fail even when over 900 lines  
> - each failure output includes path, current line count, limit, and category  
> - `demos/**/*.sifr` and `crates/sifr/tests/**/*.sifr` are included in the check and pass (confirmed by scan that zero `.sifr` fixtures exceed 900 lines)"

---

### Finding 6: Python tooling scope is still ambiguous

The plan's scope (lines 16-32) includes `scripts/**/*.py` but milestone 3 only targets `scripts/run_verification_hardening.py`. The baseline identifies 1 Python source violation. There may be Python files in `verification/` that exceed 900 lines.

**Required edit**: After the scope section (line 32), add a clarifying note:

> "Python tooling files under `verification/**/*.py` are in scope. A separate scan should confirm whether any `verification/` Python files exceed 900 lines before or during this phase. The guardrail script's pattern logic will govern them automatically once implemented; no separate Python milestone is required unless the scan reveals violations not addressable by the unified guardrail."

---

### Finding 7: Milestone sequencing has a potential circular dependency

Milestone 5's migration step requires that the unified guardrail covers all files previously governed by `MAX_LINES_BY_FILE`. But milestone 5 is the last milestone. This means milestone 5 must both implement the guardrail and validate coverage of the old guardrails in a single milestone. The plan does not address whether milestone 5 should be split into two sub-milestones (5a: implement guardrail, 5b: retire old scripts) or whether the coverage verification is expected to be a one-time manual step.

**Required edit**: Add a note in milestone 5's purpose section clarifying the retirement expectation:

> "The old `check_hir_maintainability_guardrails.py` and `check_sifr_driver_maintainability_guardrails.py` file-size checks must be retired in the same milestone as the unified guardrail goes live. Retire means: remove from `run_all_tests.sh`, remove the `MAX_LINES_BY_FILE` and per-file-budget entries, and update the checklist documents in `internal_docs/` to reference the unified guardrail. If the coverage assertion finds gaps in the unified guardrail's pattern logic, fix the pattern logic before retiring the old scripts — do not re-add per-file budgets."

---

## Summary of required edits

| # | Location | Required edit |
|---|---|---|
| 1 | After milestone 2 purpose (line ~125) | Add scope note clarifying files may be removed if already compliant or addressed as byproducts |
| 2 | Milestone 5 expected implementation (after line 256) | Add explicit migration step for `MAX_LINES_BY_FILE` entries with a mechanism description |
| 3 | AGENTS.md section (lines 281-290) | Verify the plan's current AGENTS.md text includes the responsibility-based decomposition guidance |
| 4 | Milestone 4 validation (lines 226-232) | Move the fixture-order assertion from Expected shape into Validation, with a new `validate_fixture_order.py` script |
| 5 | Milestone 5 self-test description (lines 249-254) | Add `.sifr` scope to self-test and confirm all scope categories are covered |
| 6 | After scope section (after line 32) | Clarify that `verification/**/*.py` is in scope and the guardrail auto-governs them |
| 7 | Milestone 5 purpose (before line 236) | Clarify that old guardrail retirement is part of the same milestone with explicit "do not re-add per-file budgets" guidance |

---

**READY** can be returned when all 7 findings are addressed with concrete edits to the issue file.
