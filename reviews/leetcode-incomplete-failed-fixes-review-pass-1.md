## Review: ad-hoc-leetcode-incomplete-failed-benchmark-fixes.md

### Overall Assessment

The phase doc is structurally sound with correct root-cause analysis (confirmed against raw logs), proper 3-track separation, and a logical milestone dependency order. However, there are **4 material blockers** and several non-blocking gaps before engineers can implement from it.

---

### Material Blockers

**1. M0 is ambiguous (line 369)**

"Add or extend the slowness analyzer" does not specify whether this means:
- Modifying existing `benchmarks/analyze_slowness.py`
- Creating new inventory tooling
- Locking down the existing 53 entries that the slowness phase M0 already seeds

The slowness phase M0 explicitly states it "seeds registry metadata for all 53 incomplete/failed problems." This phase doc M0 appears to duplicate or build on that work, but the relationship is unclear.

**Requested edit**: Clarify M0 scope: "Extend `analyze_slowness.py` to emit the 53 rows deterministically as JSON with `primary_track` and `benchmark_status` fields. Validate against the slowness phase M0's seeded registry metadata."

**2. L1 helper surface is undefined (line 198)**

"A small LeetCode-audit helper surface for guaranteed-safe integer math" is never specified:
- Where does it live? (`audits/leetcode/src/helpers/`? Compiler prelude? Inline in each solution?)
- What's the API? `checked_div(a: int, b: int) -> int`? `ceil_div(a: int, b: int) -> int`?
- Error behavior? "Return early" vs "default" are different (line 198 says "return early OR default")

Without this, engineers implementing L1 fixes don't know what to call or where to put it.

**Requested edit**: Add a concrete L1 helper proposal with API, location, and error-handling semantics. E.g.: "Add `checked_div(a: int, b: int) -> int` and `checked_mod(a: int, b: int) -> int` to `audits/leetcode/src/helpers/math.sifr`. Both return early with a sentinel (-1) when the divisor is proven zero by the solver, otherwise unwrap the Result. These are used only in benchmark audit code, not the compiler."

**3. H1 implementation strategy is ambiguous (line 107)**

"render list/tree rendering does not consume the result" has three plausible interpretations:
- Borrow-based helpers: change `listNodeToString(own result: ...)` to borrow
- One-shot rendering: generated runner renders once, stores string, reuses it
- Clone-based: add `Clone` to TreeNode/ListNode, clone before rendering

The right answer is probably option 2 (one-shot rendering in the runner generator), but this needs to be explicit.

**Requested edit**: Specify the approach: "Fix H1 by changing `single_sifr_runner_body` in `generic.py` to render structured results into a local string before comparison, then use the same string in wrong-result printing. Do not modify helper signatures."

**4. "no-complete-pair failures" category is never enumerated (lines 23-26, 40)**

The doc mentions "52 no-complete-pair failures" but:
- Doesn't list which 52 problems are in this category
- Doesn't explain whether these are the same 52 as the slowness doc's failure appendix
- Doesn't clarify whether ALL 53 table entries are "no-complete-pair" failures, or if some are partial/complete

This matters for correctness validation: a "correctness fix" for a problem without a passing Python baseline is meaningless unless you first establish the Python baseline is valid.

**Requested edit**: Either (a) enumerate the 52 "no-complete-pair" problems explicitly, or (b) clarify that all 53 table entries appear in the slowness doc's failure appendix and reference that as the authoritative list.

---

### Non-Blocking Issues

**5. H3 is generated-Sifr-code work, not harness work (lines 146-164)**

The H3 fix changes the wrong-result path in `single_sifr_runner_body` from `str(result)` to `treeToString(result)`. This is correctly in `generic.py`, but it's a generated-runner concern, not a runtime-harness concern. This is fine — just note it in the track description to avoid confusion.

**6. 0269_alien_dictionary "Best fix" changes fixture semantics (line 264)**

"Use topological-order validity expected shape" means changing the expected fixture, which is a benchmark infrastructure change. This should be tracked as either a harness/registry change (for fixture shape) or a code change (if making Sifr match Python DFS order). The doc says "LeetCode Sifr code" but the fix is at the fixture layer.

**7. M4 is underspecified (lines 398-402)**

"Reintegrate with Performance Analysis" needs:
- Who runs the post-fix re-benchmark protocol (this phase's engineers? slowness phase engineers?)
- How newly complete problems are added to slowness phase metadata
- Whether M4 requires a separate review/approval step

**8. M0/M1/M2/M3 don't reference the slowness phase's post-fix protocol (lines 364-397)**

The slowness doc's "Post-Fix Re-Benchmark Protocol" (lines 446-451) is the authoritative procedure for re-running benchmarks after fixes. This phase should explicitly reference it, or state that it owns that protocol for these 53 problems.

---

### What's Solid

- **Table count**: 53 rows matches the slowness doc's 53 incomplete problems. ✓
- **Root-cause analysis**: Confirmed correct against raw logs (H1: "use of moved value: 'result'" in 0206, H3: `Display` error in 0226, L2: "expected 'ListNode', got 'None | ListNode'" in 0141). ✓
- **Track separation**: Harness fixes (H1/H2/H3) don't require solution rewrites. LeetCode code fixes (L1-L6) are scoped to specific ports. Compiler work (Track C) is deferred. ✓
- **Dependency order**: M0 → M1 → M2 → M3 → M4 is correct. ✓
- **Acceptance criteria**: Tests are concrete enough (all 53 entries listed, harness fixes don't require rewrites, correctness fixes validated against fixtures). ✓

---

### Summary

The phase doc is close to implementation-ready. The material blockers (M0 ambiguity, undefined L1 helper surface, ambiguous H1 strategy, unenumerated failure category) are all specification gaps, not fundamental design problems. Fix these 4 items and it's ready for implementation.
