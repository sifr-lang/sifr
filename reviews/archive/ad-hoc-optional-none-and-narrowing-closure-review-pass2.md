## Re-Review: `ad-hoc-optional-none-and-narrowing-closure.md`

### 1. Verdict: **Ready**

The prior review's six findings have been addressed. The document is substantially more complete than its predecessor and is execution-ready.

---

### 2. Findings (ordered by severity)

**No blocking issues.** Three minor observations that should be cleaned up but do not prevent execution:

**Minor (A) — Naming split between "workstream" and "wave":**  
The Cross-Stream Dependencies table uses `workstream_1`, `workstream_2` naming, but the Execution Order section uses `wave_1`, `wave_2`. The ledger also uses `wave_X`. These are the same items but named differently. An engineer skimming the doc could reasonably confuse them. Recommend aligning to one term throughout (preferably `workstream`, since that is the term used in the dependency table and is more descriptive).

**Minor (B) — Wave 4 parallel claim vs. sequential listing:**  
The cross-stream dependencies table states wave 4 "can run in parallel after boundary contract is confirmed," but the Execution Order section lists all five waves sequentially with no parallelization noted. If wave 4 can genuinely overlap with waves 2-3, that should be explicit in the Execution Order section. If it must wait, the dependency table should say "depends on boundary contract confirmation (may overlap with waves 2-3 once confirmed)."

**Minor (C) — Implementation hotspots have uneven granularity:**  
`crates/sifr_hir/src/cfg.rs` and `crates/sifr_type_system/src/narrow.rs` are precise file targets, while `crates/sifr/tests/e2e/` and `crates/sifr/tests/e2e/pass/` are directory-level. For a document this detailed, listing specific test fixture anchors (already present in each workstream's validation section) alongside the hotspots would close the loop.

---

### 3. What Was Fixed From Prior Review

| Prior Finding | Status in This Revision |
|---|---|
| Missing per-workstream "definition of done" | ✅ Each workstream now has a concrete before/after contract with type-level expectations |
| No technical implementation sketch | ✅ Each workstream has a "Technical approach" paragraph plus primary implementation loci |
| No cross-stream interaction mapping | ✅ Cross-Stream Dependencies table present with dependency rationale |
| No test strategy | ✅ Each workstream has a Validation section with unit test files, e2e anchors, and targeted rerun requirements |
| Execution order justification thin | ✅ "narrowing produces the concrete facts that deferred joins must consume" makes the dependency causal, not asserted |
| Workstream sizing missing | ✅ Complexity ratings (large/medium/medium-large/small) added to each workstream |

---

### 4. Remaining Minimum Changes

None required for execution readiness. The three minor observations above are polish, not gatekeepers.

If you want perfection, resolve Minor (A) by picking one naming convention (`workstream` preferred) and updating the Execution Order section to match. This is a one-pass find-and-replace operation and does not affect any logic.
