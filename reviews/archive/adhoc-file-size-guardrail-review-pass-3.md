

Now let me systematically verify each of the 7 pass 2 findings against the current file.

**Finding 1** — Milestone 2 scope note for current violations/byproduct compliance (line 126-127):

> "All files listed in this milestone exceed 900 lines in the current baseline. Files may be removed from this milestone if they become compliant during earlier work or are addressed as a byproduct of another decomposition; the authoritative current violation list is always the file-size guardrail scan."

Present and concrete. ✅

**Finding 2** — Explicit migration mechanism for `MAX_LINES_BY_FILE` / per-domain coverage (lines 261-263):

> "Migration step: before retiring `check_hir_maintainability_guardrails.py` and `check_sifr_driver_maintainability_guardrails.py` file-size logic, verify that every file previously governed by `MAX_LINES_BY_FILE` or per-domain implementation limits matches the new path-pattern logic. Implement this as either a `--verify-includes <ref-file>` mode that reads the old per-file map or as self-test fixtures that add the old file paths as included cases."

Present with both mechanism options. ✅

**Finding 3** — AGENTS.md responsibility-based decomposition guidance (lines 296-298):

> "Use the existing HIR lowering and package-manager module layouts as examples of responsibility-based decomposition: split by compiler concern and ownership boundary, not by alphabetical order or line-count chunks."

Present and anchored to actual codebase examples. ✅

**Finding 4** — Milestone 4 fixture-order validation is concrete and in Validation (lines 228-235):

> - `python3 scripts/validate_fixture_order.py` (new script: enumerates all e2e fixture paths lexicographically, outputs the sorted list, and asserts stability across the split)

Script name, behavior, and assertion type are all specified. ✅

**Finding 5** — Milestone 5 self-test covers 900/901, excludes, failure shape, and `.sifr` inclusion (lines 254-259):

All required behaviors are present: 900 passes, 901 fails, excludes coverage, failure output with path/count/limit/category, and `demos/**/*.sifr` / `crates/sifr/tests/**/*.sifr` inclusion with the zero-violations baseline justification. ✅

**Finding 6** — Python verification scope clarified (lines 32-33):

> "Python tooling files under `verification/**/*.py` are in scope; the current baseline has zero `verification/**/*.py` violations above 900 lines. The unified guardrail must govern them automatically through the same include-pattern logic, with no separate Python-only rule."

Present and explicit that no separate Python milestone is needed. ✅

**Finding 7** — Old guardrail retirement in the same milestone, no re-added per-file budgets (lines 244-245 + 263):

> "Retire older per-domain maintainability checks to defer source file-size enforcement to the unified rule after the unified rule exists."
> "do not re-add per-file budgets or compatibility allowlists"

Both the retirement intent and the "do not re-add per-file budgets" constraint are present. ✅

All 7 pass 2 findings are addressed with concrete text in the current file. No remaining gaps.

**READY**
