I've cross-checked the PR diff against the merged state and the tracker context. Here's the review.

## PR #2969 review — docs(plan): close Python interop M7

**Scope check:** Single file changed — `plans/issues/active/ad-hoc-declaration-first-python-interop.md` (+7/-8). Documentation-only, no code.

**Cross-checks against requirements:**

1. **PR #2968 merged?** Yes — `state: MERGED`, `mergedAt: 2026-07-13T11:40:13Z`, commit `b7a978a4d`, title "feat(python-interop): activate typed async declarations". Matches the reference used throughout.

2. **M7 milestone marked complete with link:** ✓ `plans/issues/active/ad-hoc-declaration-first-python-interop.md:141` — `[x] M7 owned asyncio runtime and async declarations — [PR #2968]`.

3. **Final activation wave marked complete with link:** ✓ line 578 — `[x] Atomically activate async declarations and close M7 evidence — [PR #2968]`. All six prior M7 waves already `[x]` with their own PR links (2956, 2958, 2960, 2962, 2964, 2966).

4. **"M0 through M7" complete claim:** ✓ line 8 — `M0 through M7 are implemented, locally validated, and linked below`. Milestone list confirms M0–M7 all `[x]` with PR links.

5. **Overall phase still in progress:** ✓ line 5 — `In progress. The phase defines one complete end-state architecture...`.

6. **M8–M17 left unchecked:** ✓ lines 142–151 — all ten remain `[ ]` with no PR links.

7. **Status wording doesn't overclaim later families:** ✓ Lines 9–11 name only "typed async declarations, owned-loop cancellation, and consuming async close" — all three within M7's declared scope (owned asyncio loop, `@python.coroutine`, bidirectional cancellation with `CancelledError` mapping, `cleanup=async_close`). No async context managers (M8), callbacks (M9), buffers/Arrow/DLPack (M10–M12), or later families are claimed active. The "one production path" phrasing matches M7's Delivery Rule for the atomic activation wave.

No mismatches found. The doc updates are internally consistent, accurately reflect merged PR #2968, and preserve the ordered-implementation invariant for M8+.

VERDICT: SATISFIED
