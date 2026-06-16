## Review — Docs-only phase closure for Phase 31.8

### Blocking findings
None.

### Diff verification
- `plans/issues/active/...md → plans/issues/archive/...md` — pure rename, similarity 100%.
- Archived issue file changes are exactly two lines:
  - Header status: `in final closeout; …opened in PR #2650` → `complete; archived after Wave 10.2 …merged through PR #2650`.
  - Wave 10.2 entry: `implemented locally on codex/wave-10-final-closeout-evidence and opened in PR #2650` → `implemented, reviewed, locally validated, and merged through PR #2650`.
- `plans/phases/index.md:40` — row 31.8 `in_progress → complete`, link `active/ → archive/`.
- `plans/roadmap.md:58` — row 31.8 `in_progress → complete`, link `active/ → archive/`, trailing prose now ends `…completed through PR #2650.` (drops the residual "Wave 10 is closing…" forward-looking clause).

### Internal consistency
- Status wording and the PR #2650 anchor match across the three updated docs.
- No remaining "in_progress" or "in final closeout" references to Phase 31.8 in roadmap/phases/issue header.
- `codegen-test-triage.md` is still at `plans/issues/active/…`, so the archived file's existing references to it remain valid (it just wasn't archived together with this issue; that's the existing pattern, not introduced here).

### Non-blocking observations (out of this closure's scope)
Three frozen artifacts outside `plans/` still contain the literal `plans/issues/active/ad-hoc-world-class-verification-standard-and-gate-closure.md` string. None are introduced by this diff and none break at runtime:
- `verification/areas/coverage_matrix/checks/coverage_matrix_self_test.py:151` — opaque test-fixture string; `coverage_matrix.py:237` only validates it's a non-empty string, never opens the path.
- `verification/areas/generated_code_quality/codegen_red_blocker_inventory.json:19` — frozen inventory snapshot dated 2026-06-14, `status: closed`.
- `verification/areas/coverage_matrix/reports/world_class_verification_closeout_2026-06-16.md:10` — the PR #2650 closeout evidence report, dated today; treating it as frozen evidence is consistent with how prior phase closeouts were handled.
- ~165 references in `plans/reviews/active/*` historical review artifacts naming the original path — also frozen.

Leaving these as-is is consistent with the stated closure scope ("only aligns tracking state after merge"); broadening to rewrite frozen evidence/reports would mutate historical artifacts. Worth a follow-up only if your convention requires retargeting inventory `issue:` pointers on archive.

### Verdict
Reviewer satisfied for the docs-only phase closure. No blockers.
