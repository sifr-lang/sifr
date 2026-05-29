Reviewing the doc against each of the six prior blockers.

---

**Blocker 1: missing reproducible analyzer/bootstrap step**

**Resolved.** M0 (lines 537-554) specifies `analyze_slowness.py` inputs (raw hyperfine/memory output), required outputs (counts, inventories, ratios, excerpts), and the metadata seeding obligation for all 75 + 53 problems. The "Benchmark/Report Contract Updates" section (lines 400-440) gives the exact field schema (`parity_status`, `primary_slowness_owner`, `slowness_tags`, `benchmark_status`), seeding rules per classification, and the deterministic diff requirement.

**Blocker 2: missing registry metadata seed plan**

**Resolved.** Concrete and prescriptive. Lines 421-439 give field names, allowed values, and per-row seeding rules: compiler rows get `parity_status: "equivalent"` only when defensible; LeetCode-code rows get `parity_status: "known_divergent"`; mixed rows start as `unknown` unless divergence is already named; noise rows still get metadata. 0234 gets explicit `benchmark_status: "partial"`. Lines 549-553 in M0 codify it as an implementation obligation.

**Blocker 3: M1/M4 circular dependency**

**Resolved.** The dependency chain is explicitly named in the milestone header (line 535): **M0 → M1 → M2/M3 → M4**, with a note that M2/M3 can parallelize after M1, but M4 waits on seeded metadata from M0/M1. No ambiguity remains.

**Blocker 4: ambiguous partial handling for 0234_palindrome_linked_list**

**Resolved.** The Classification Rule (lines 99-100) gives an explicit operational definition: partial benchmarks allowed only when at least one fixture has complete Python/Sifr timing rows, and they must be marked and excluded from apples-to-apples summaries until all fixtures build, pass correctness, and produce comparable rows. 0234's table entry (line 158) explicitly shows the partial tag and the exclusion rationale. The count reconciliation (line 34) counts it as incomplete separately from the 52 no-pair failures. Registry seeding (line 439) requires `benchmark_status: "partial"` until all fixtures pass.

**Blocker 5: missing post-fix re-benchmark protocol**

**Resolved.** A dedicated "Post-Fix Re-Benchmark Protocol" section (lines 441-460) gives the required flow (subset first, then category, then analyzer refresh), the reclassification thresholds by ratio band, correctness override behavior, and the memory regression discipline.

**Blocker 6: missing failed-to-benchmarkable conversion path**

**Resolved.** A dedicated "Failed-To-Benchmarkable Conversion" section (lines 462-471) explains how the incomplete appendix is a live pipeline, gives concrete steps for marking newly fixed problems complete, and explicitly treats newly slower cases as follow-up scope for this phase rather than unrelated regressions.

---

**Blocker 7 (from pass 1): missing string-heavy Sifr parity note under C1**

**Resolved.** C1 (lines 265-269) now has an explicit "Important distinction" paragraph: some problems choose a more string-heavy Sifr approach than the Python implementation and need a LeetCode-code parity check before attributing slowness to compiler fixes, with specific problem names named (`0402`, `0567`, `0179`, `0187`, `0929`, `1888`).

---

**Remaining issues — polish only, no blockers:**

1. **Line 413-416**: The raw-analysis command example uses `report-html` and `--output` flags that may not exist yet. M0 owns adding `analyze_slowness.py`, so this is already framed as an implementation prerequisite.

2. **Failure appendix**: Representative excerpts are short and non-actionable (e.g., `use of moved value: 'result'`). This is fine for a diagnostic doc; actionable triage would happen during M0/M1.

3. **No newline before Rust code block** (line 208): The first Rust block starts immediately after the sentence ending at line 207. Markdown rendering may be correct anyway, but a blank line is conventional.

4. **Acceptance criteria line 597**: "Registry metadata is seeded" could be strengthened to require M0 validation that all required fields are present, not just that seeding was attempted. Minor — the milestone language in M0 already covers this.

---

**Verdict: implementation-ready.** All six blockers are resolved with concrete, actionable language. The doc has a clear taxonomy, locked dependency chain, complete milestone structure, and post-fix protocol. Polish items above are minor and do not affect implementation tractability.
