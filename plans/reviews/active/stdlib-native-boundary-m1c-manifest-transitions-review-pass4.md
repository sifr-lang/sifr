Verified pass3's blocker is fixed and pass2's requirements still hold:

**Pass3 blocker (closure/active-surface positive test) — fixed at `scripts/check_stdlib_manifest_schema.py:473-488`.** The `active_closed_surface` case builds a manifest where `_sifr.example` remains in `surface` AND in `closed_surface`, calls `_validate_transitions(active_closed_surface, base_manifest)`, and asserts on the exact "closed_surface records must not reference active surface rows" failure. This exercises the transition-level intersection at line 224-229, not the inline record validator. A regression that dropped the check would now be caught.

**Other criteria re-verified:**
- CI checkout `fetch-depth: 0` on all four jobs at `.github/workflows/local-first-validation.yml:20,43,58,73` ✓
- `_is_pr_reference` (`:274-278`) requires `PR #<digits>` or `/pull/<digits>` ✓
- `_base_manifest` failure surfaces `base_ref` and git stderr (`main` at `:55-61`, git stderr captured at `:268-269`) ✓
- All 5 allowed transitions swept in self-test (`:381-391`) ✓
- Invalid `retained-by-design → closing` transition rejected (`:393-402`) ✓
- New non-`retained-by-design` row rejected + new `retained-by-design` positive path (`:404-438`) ✓
- Closing-row deletion with/without closure + non-closing deletion rejected (`:440-471`) ✓
- Bad closure `previous_state`, non-numeric PR reference, duplicate closure ids, empty-surface + bad closure preserve both errors, PR URL closure positive (`:490-563`) ✓
- Closure-record validation runs before empty-surface early return (`:87-96`), so failures compound as expected ✓

**Live checks:**
- `python3 scripts/check_stdlib_manifest_schema.py --self-test` → PASS
- `python3 scripts/check_stdlib_manifest_schema.py` → PASS (surfaces=20, schema_version=2)
- HEAD == origin/main on this branch (M1c changes are still uncommitted), so the transition guard runs against a zero-delta base — an inert exercise, but the base ref is resolvable and self-tests independently exercise every rule. User-reported `run_all_tests.sh --profile create-pr` wall_time=176.77s, advisories=none.

No correctness holes, missing rejection paths, CI/local parity gaps, or user-path panics found. Base-ref fetch is durable (`fetch-depth: 0` on every job that runs the guard), and `_base_manifest`'s failure branch produces a hard failure rather than a silent skip.

READY
