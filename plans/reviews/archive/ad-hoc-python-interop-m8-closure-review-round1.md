All checks complete. Here's the review summary.

## M8 tracker/roadmap closure — verification

**Tracker (`plans/issues/active/ad-hoc-declaration-first-python-interop.md`)**
- Status paragraph (L5-12): now "M0 through M8 are implemented, locally validated, reviewed, and linked below … M9 and later milestones are not yet implemented." Overall phase still "In progress." ✓ (The added word "reviewed" is fair — a milestone-level review exists in the archive.)
- Milestone list (L143): `[x] M8 async context managers — [PR #2970](…), [PR #2972](…)`. Both PRs cited, comma-separated, correct URLs. ✓
- M9-M17 (L144-152): all remain `[ ]`, no PR links. ✓
- Wave 2 checkbox (L645-651): flipped `[ ]` → `[x] Atomically activate async contexts and close M8 evidence — [PR #2972](…):` while sub-bullet body is preserved. Wave 1 (L632) was already `[x]` from PR #2971. No later wave/milestone checkboxes exist for M8, none moved. ✓

**Roadmap (`plans/roadmap.md`, L129)**
- Advances "M0-M7 implementation is active through the owned asyncio runtime, typed coroutine declarations, structured cancellation, and consuming async close" → "M0-M8 implementation is active through typed synchronous/asynchronous declarations and context managers on the owned asyncio runtime, with callback and zero-copy protocols sequenced next"; phase entry still marked "in progress". ✓ Mirrors the M7 closure convention (PR #2969 / commit b7a978a4d).

**Review moves (13 tracked files, deletion in `active/` + creation in `archive/`)**
- design.md, design-review-round1/2, wave1-implementation-review-round1/2/3/4, wave1-pr-review-round1/2, wave1-tracker-review, wave2-review-round1/2, wave2-pr2972-review-round1: all 13 byte-identical between `HEAD:plans/reviews/active/<f>` and working-tree `plans/reviews/archive/<f>` (verified via diff). ✓
- Additionally, two untracked archive-only files present: `ad-hoc-python-interop-m8-milestone-review-round1.md` (74 lines, SATISFIED verdict) and `ad-hoc-python-interop-m8-wave2-pr2972-review-round2.md` (34 lines, SATISFIED verdict) — the final #2972 verification and milestone review. ✓
- `plans/reviews/active/ad-hoc-python-interop-m8-closure-review-round1.md` is an empty 0-byte placeholder for the pending closure-PR review, not part of the closure content. Not a defect.

**Scope**
- `git diff --stat`: only `plans/issues/active/ad-hoc-declaration-first-python-interop.md` (+9/-8), `plans/roadmap.md` (+1/-1), and the review moves. No source, runtime, verification, or unrelated plan files touched. ✓

**Validation**
- `git diff --check` → clean (no output). ✓
- `python3 scripts/check_file_size_guardrails.py` → `file-size guardrails: PASS (2556 files, limit 900 lines)`. (Guardrail only checks `.rs`/`.py`/`.sifr`; the 1039-line tracker `.md` is not in scope.) ✓
- `python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py` → `verification taxonomy ok`. ✓

No mismatches, no bookkeeping drift, no unrelated edits. Convention matches PR #2969 (M7 closure) with the natural extension for two M8 PRs.

VERDICT: SATISFIED
