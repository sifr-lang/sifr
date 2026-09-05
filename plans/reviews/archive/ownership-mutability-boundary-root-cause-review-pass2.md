# Review: Ownership/Mutability Boundary Root-Cause Analysis (Pass 2)

**Reviewer:** agent (pass 2)
**Date:** 2026-04-02
**Report under review:** `issues/ownership-mutability-boundary-root-cause-2026-04-02.md`
**Prior review:** `reviews/ownership-mutability-boundary-root-cause-review-pass1.md`

---

## Pass 1 Amendment Verification

### Change #1 (Minor): Confirmed vs inferred compound-own-mut distinction

**Status: ADDRESSED**

Report lines 27-31 now explicitly split compound fixtures:

- "diagnostic-confirmed (2): 0669, 0701" — both emit mutability + borrow-escape diagnostics
- "inferred (1+): 0075" — semantically `own mut` but only borrow-escape diagnostic emitted

The original "estimated 3-5" language is removed. The distinction is clean and verifiable.

### Change #2 (Medium): mut-only vs own-mut split inside mutation/reassignment workstream

**Status: ADDRESSED**

Report lines 157-159 split the 41-fixture workstream:

- `mut`-only adaptation (39)
- compound `own mut` adaptation (2 confirmed: 0669, 0701; plus 1+ inferred: 0075)

This matches the pass1 recommendation exactly. The two sub-streams have different annotation strategies, preventing rework on compound fixtures.

### Change #3 (Medium): Explicit secondary-error inventory coverage

**Status: ADDRESSED**

Report lines 93-131 add a full "Secondary Error Inventory" section:

- Enumerates 33/47 fixtures with their first non-ownership secondary diagnostic
- Provides family-level totals: operator/truthiness (20), undefined variable (19), destructuring/assignment target (11), return contract (8)
- The remaining 14/47 fixtures implicitly have no non-ownership secondary diagnostics in this run

The inventory covers all four fixtures flagged in pass1 (0002, 0075, 0669, 0701). Note: 0669 and 0701 are absent from the secondary inventory because their cross-axis borrow-escape diagnostic is absorbed by the compound classification (change #1), which is the correct treatment — the secondary inventory is reserved for non-ownership errors.

### Change #4 (Minor): Per-fixture copy-type vs move-type reassignment guidance

**Status: ADDRESSED**

Report lines 166-168 now enumerate:

- Copy-type scalar params: `left`, `columnNumber`, `k`, `n` (x2), `speed` — prefer `let mut local = param`
- Move-type rebinding params: `a`, `s`, `head`, `nums`, `nums1` — prefer explicit parameter `mut` / `own mut`
- Collection/object in-place edits — prefer `mut` (or `own mut` when escaping)

This gives per-fixture actionable guidance for the 11 reassignment fixtures, distinguishing the two strategies as pass1 requested.

### Change #5 (Minor): Variant-fixture note for 0605 v1/v2

**Status: ADDRESSED**

Report line 173 adds: "Apply identical adaptation policy to fixture variants when diagnostics are identical (for example `0605_can_place_flowers` and `0605_can_place_flowers_v2`)."

This is correctly scoped — identical diagnostics warrant identical adaptation. No separate analysis needed.

---

## Remaining Observations (Non-Blocking)

### Observation A: Secondary inventory presentation

The per-fixture listing (lines 97-129) shows "representative first secondary diagnostics," while the family totals (line 131) sum to ~58, exceeding the 33-fixture count. This indicates the family totals count ALL secondary diagnostics per fixture (not just the first), while the listing is one-per-fixture. The dual perspective is useful, but a one-line clarification that the family totals are aggregate (not first-only) would prevent misreading. **Not blocking.**

### Observation B: Implicit clean-14 statement

The inventory covers 33/47 fixtures. The remaining 14 are implied to have no non-ownership secondaries, but this is never stated explicitly. A brief note like "remaining 14/47 fixtures emit only ownership-category diagnostics in this run" would close the gap. **Not blocking.**

### Observation C: "At least 6" unmask estimate vs 33 baseline

Remediation item 4 (line 170) retains the "at least 6 node/root-style fixtures may unmask secondary categories" estimate, now contextualized against the 33-fixture secondary baseline (line 171). The "6" refers specifically to NEW secondaries surfacing after ownership fixes — distinct from the 33 already visible. This is internally consistent but could be clearer about what "unmask" means in context. **Not blocking.**

---

## Final Implementation-Readiness Verdict

**READY.**

All five pass1-required changes are addressed:

| # | Pass1 Severity | Change | Pass2 Status |
|---|---|---|---|
| 1 | Minor | Confirmed vs inferred compound split | Addressed (lines 27-31) |
| 2 | Medium | mut-only vs own-mut workstream split | Addressed (lines 157-159) |
| 3 | Medium | Secondary-error inventory | Addressed (lines 93-131) |
| 4 | Minor | Copy-type vs move-type per-fixture guidance | Addressed (lines 166-168) |
| 5 | Minor | 0605 variant note | Addressed (line 173) |

The two medium-severity blocking conditions from pass1 — secondary-error inventory (change #3) and workstream sub-split (change #2) — are both resolved. The three minor documentation improvements (changes #1, #4, #5) are also incorporated.

The report is implementation-ready for driving fixture adaptation across both the mutation/reassignment (41) and escape (6) workstreams. The observations above (A, B, C) are non-blocking presentation refinements that can be addressed opportunistically.
