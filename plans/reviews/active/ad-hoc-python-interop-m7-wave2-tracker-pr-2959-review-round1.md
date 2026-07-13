I've reviewed PR #2959 against the criteria. Verification results:

**Content verification:**
- **Phase status**: Updated text at lines 8–11 reads "M0 through M6 are implemented, locally validated, and linked below; M7 is in progress with its frontend-contract and owned-loop waves merged, while later milestones are not yet implemented." ✓ Accurate.
- **M7 Wave 2 checkbox and PR link**: Line 521 flips Wave 2 ("Add the application-owned asyncio runtime and raw submission path") to `[x]` linked to PR #2958. Confirmed via `gh` that PR #2958 is merged (state=MERGED, mergedAt=2026-07-13T04:29:44Z). ✓
- **Wave 3 and later**: Lines 533, 546, 556, 567, 574 remain `[ ]` (cooperative-cancellation carrier, supervisor/shutdown substrate, typed async wrappers, async-close lifecycle, activation-and-close). ✓
- **M7 top-level milestone**: Line 141 keeps `[ ] M7 owned asyncio runtime and async declarations` unchecked, correctly reflecting that M7 overall is still in progress. ✓
- **Contract wording preserved**: Complete-design language ("Opus High pass 5 approved the complete design"), Non-Goals, Delivery Rule (lines 116–128), and Milestones section headings are untouched. The "In progress" preamble was refined without weakening any contract. ✓
- **Prior Wave 1 entry**: Line 507 already carried `[x]` with PR #2956 from the earlier tracker commit — this PR does not re-touch it, avoiding churn. ✓

**PR shape:**
- Diff limited to a single file (`plans/issues/active/ad-hoc-declaration-first-python-interop.md`), +5/-3. Docs-only, focused. ✓
- PR title and summary accurately describe the change (record Wave 2 merge, correct phase status). ✓

**Minor observation (non-blocking):** The PR is currently in `DRAFT` state on GitHub. Content is ready for merge; only workflow state distinguishes it from mergeable. Not a defect in the change itself — flagging so the author can flip it to "Ready for review" when they intend to land it.

No actionable findings against the content or contract preservation.

VERDICT: SATISFIED
