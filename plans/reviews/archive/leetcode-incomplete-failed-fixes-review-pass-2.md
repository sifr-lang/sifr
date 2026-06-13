

## Pass 2 Review: Ad Hoc LeetCode Incomplete/Failed Benchmark Fixes

### Blocker Status: **MATERIAL BLOCKERS REMAIN**

The document has made genuine progress — M0 relationship to slowness phase is now explicit, L1 surfaces are defined, H1 approach is constrained, and L4 (0269) correctly names both the harness and code dimensions. But three blockers block implementation readiness.

---

### Blocker 1: M0 — Slowness Phase Schema Extension Is Ambiguous

**Pass 1 finding:** M0 ambiguity with slowness phase analyzer/metadata.

**Current state:** Partially addressed. The document correctly references "Extend `audits/leetcode/benchmarks/analyze_slowness.py`" and specifies JSON output fields. But:

1. **Schema alignment is unstated.** M0 inherits from slowness phase M0 but does not define how the new output extends or overrides the existing schema. What happens to fields from slowness phase M0 that this phase doesn't need? Are `primary_track` values identical between phases?

2. **Table does not match machine-readable format.** M0 specifies `primary_track` values like `benchmark_harness`, `leetcode_sifr_code`, `mixed_harness_and_code`, `compiler_followup`. The working table (lines 320–375) uses prose like "Benchmark harness", "LeetCode Sifr code", "Benchmark harness / LeetCode Sifr code". This is editorial presentation, not the JSON that M0 should emit. The disconnect between prose table and machine-readable schema is unresolved.

3. **`failure_mode` and `failure_excerpt` are underspecified.** M0 names these as required fields but doesn't define their value set or content expectations. "moved `result` in runner validation" is prose; should it be a tagged failure type like `ownership_consumed_post_validation`?

**Required edit:** Define explicit schema relationship — either this phase's JSON is a superset of slowness phase M0, a fork with a version marker, or extends with additional fields. Clarify that the prose table is a human-readable rendering and that the machine-readable output uses snake_case identifiers. Define a small tag vocabulary for `failure_mode`.

---

### Blocker 2: L1 — Helper Surface Location and File Naming Are Undefined

**Pass 1 finding:** undefined L1 safe math helper surface.

**Current state:** Function signatures are defined (lines 202–207) and behavioral rules are specified (lines 208–214). But:

1. **Path is ambiguous.** The text references "a small LeetCode-audit helper surface for safe integer math in `audits/leetcode/src/helpers/math.sifr`." The source inputs section (lines 13–20) lists `audits/leetcode/src/helpers/list_node.sifr` and `audits/leetcode/src/helpers/tree_node.sifr` but does not mention `math.sifr`. If the intent is to place the file alongside existing helpers, it should be listed in source inputs.

2. **Naming convention is ambiguous.** The existing helpers are named after their data structure (`list_node`, `tree_node`). The proposed name `math` is generic and could conflict with standard library modules. `safe_math.sifr` or `math_helpers.sifr` would be more precise.

**Required edit:** Add `audits/leetcode/src/helpers/math.sifr` (or renamed alternative) to source inputs. Either change the L1 text to reference the renamed file or rename the file to match the existing helper naming convention.

---

### Blocker 3: L4 (0269) — Classification Changed but Not Resolved

**Pass 1 finding:** unclear 0269 classification after it was changed to benchmark harness / LeetCode Sifr code.

**Current state:** Pass 1 correctly flagged that 0269 was reclassified from correctness-only to mixed harness/code. The document now (line 367) shows "Benchmark harness / LeetCode Sifr code" as primary track. But:

1. **Both tracks remain ambiguous.** The L4 text (lines 273–277) names two options — "prefer topological-order validity expected shape" OR "intentionally port Sifr to match the Python DFS order." Neither is chosen. The "Preferred fix" language suggests the first option, but "If the benchmark framework should avoid problem-specific validators" is a conditional that doesn't resolve.

2. **Table entry for 0269 is dual-classified.** Line 367 shows "Benchmark harness / LeetCode Sifr code" — both tracks. If the preferred fix is topological-order validity in the harness, then `benchmark_harness` is the primary track and the code change is secondary. If the preferred fix is matching Python order in the Sifr code, then `leetcode_sifr_code` is primary. The table cannot have two primary tracks.

3. **The 0269 problem was changed from correctness-only.** Looking at the slowness phase, this problem's failure was classified as correctness. The reclassification to "benchmark harness" as a co-track is correct because the expected fixture itself is the constraint. But the document doesn't explain the reclassification rationale — an engineer picking up this task would not know why 0269 now has a harness track.

**Required edit:** Choose one preferred fix in L4 and make the alternative explicit as "fallback if framework design prohibits problem-specific validators." Align the table entry for 0269 to one primary track. Add a one-line reclassification note explaining why 0269 now has a harness track (fixture itself is the constraint, not just code correctness).

---

### Remaining Polish (Not Blockers)

- **H1 implementation strategy** is well-constrained (no helper signature changes, no solution rewrites) but doesn't specify which functions beyond `single_sifr_runner_body` need updating, or whether to inline the rendered-string logic or factor into a shared helper.
- **M2 "small batch"** is vague on re-run intervals. Suggest: "re-run after every 5–10 problems" or "after each track's problems are exhausted."
- **0269 Python DFS order detail** — the L4 text says "Python DFS order cba" but doesn't explain why. A one-sentence note on why Kahn vs. DFS produces different output for repeated-constraint fixtures would help implementers.

---

### Verdict

**Not implementation-ready.** Three material blockers (M0 schema alignment, L1 location/naming, L4 resolution/classification) prevent a clean handoff. Polish items are non-blocking but would improve handoff quality.

**Exact required edits:**

1. M0: State schema relationship to slowness phase M0; clarify prose table vs. machine-readable output; define `failure_mode` tag vocabulary.
2. L1: Add `audits/leetcode/src/helpers/math.sifr` (or renamed) to source inputs; rename to match existing helper convention.
3. L4: Choose preferred fix with explicit fallback; update table entry to single primary track; add reclassification note for the harness track.
