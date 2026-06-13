Now I have enough context to write the review. This is a well-structured diagnostic document, but there are material blockers and gaps that would cause real implementation trouble.

---

## Review: Ad Hoc Phase — LeetCode Benchmark Slowness Root Cause Analysis

### Overall assessment

The document is substantively correct and its taxonomy (compiler-runtime vs. LeetCode-code ownership) is the right framing. The count reconciliation in "Count Reconciliation" is methodologically sound and should remain. However, there are **two hard blockers**, several **hidden circular dependencies**, and a handful of concrete missing pieces that make the phase not yet implementation-ready.

---

### Hard blockers

**1. The reproducibility path does not exist.**

Line 414–417 explicitly calls out a non-existent tool:

```
python3 benchmarks/analyze_slowness.py --output issues/ad-hoc-leetcode-benchmark-slowness-root-cause-analysis.md
```

The document's own acceptance criteria AC1 ("Every current Sifr-slower complete benchmark is listed in this phase") requires a machine-readable inventory of those 75 problems. The analyst command that would produce that inventory does not exist. Every implementation ticket cut from this phase will need those75 problems enumerated as a starting point. Without `analyze_slowness.py`, an engineer is expected to reproduce the analysis by hand.

**Requested edit**: Add a milestone M0 — "Implement `benchmarks/analyze_slowness.py`" before or as part of M1, with a concrete description of what it reads and emits.

**2. The problem registry metadata that M4 depends on is not defined yet.**

The "Benchmark/Report Contract Updates" section (lines 400–427) specifies the exact fields each entry in `audits/leetcode/benchmarks/problems/*.json` should carry:

- `parity_status`: `equivalent`, `known_divergent`, `unknown`, or `failed_correctness`
- `primary_slowness_owner`: `compiler`, `leetcode_sifr_code`, `mixed`, or `noise`
- `slowness_tags`: list of stable tags

I verified that none of the existing JSON registry files contain these fields. The phase documents the *intent* of these fields but does not specify their *initial values* for the 75 slower problems. M4 (benchmark report semantics) cannot be implemented without this data.

**Requested edit**: Either add the fields with explicit values to every entry in the slowness table (as inline notes, or as a companion JSON sidecar), or explicitly scope M0 to "seed all 75 JSON entries with initial `parity_status`, `primary_slowness_owner`, and `slowness_tags` values." Without one of these, M4 has no anchor.

---

### Hidden circular dependencies

**3. M1 → M4 circular dependency.**

M1 ("Slowness taxonomy lock — add per-problem classification metadata for the 75 slower problems") generates the metadata that M4 ("benchmark report semantics — UI badges read these fields") consumes. But there is no prior step that creates the initial metadata. This is a bootstrap problem: M1 cannot be marked done until the metadata is in the registry, and M4 cannot be implemented until the metadata is readable by the report.

**Requested edit**: Add a dependency arrow in the milestones: M0 → M1 → M4. Make M0's scope explicit: seed all53 incomplete problems and all 75 slower problems with the three registry fields.

**4. The slowness table lists `0234_palindrome_linked_list` both as a slow case (ratio0.081x, line 156) and as a partial/failed case in the appendix (line 442).**

The slowness table row says "Compiler — linked-list helper lowering clones nodes while traversing; emit failed in one standalone pass because of optional type mismatch, so treat as compiler/helper debt." The appendix says it has "complete pairs exist for some sizes; missing size fails with `expected 'ListNode', got 'None | ListNode'`."

As written, `0234` is simultaneously a benchmark-informative slowness data point and a correct correctness failure. These are logically incompatible for the report filtering the phase wants ("runtime comparisons only for complete, correctness-passing fixtures").

**Requested edit**: Clarify the classification: either move `0234` out of the 75-slower table entirely into a "partial benchmarks" appendix, or add a `partial_benchmark: true` flag to the table row so the report can exclude it from performance comparisons while still flagging it in the failure appendix.

---

### Missing engineering considerations

**5. No mechanism to re-benchmark after fixes.**

The document specifies compiler repairs (C1–C4) and LeetCode-code repairs (L1–L3), but nowhere does it describe how to re-run the benchmark subset for affected problems after each fix, how to compare post-fix speedups to baseline, or what the "fixed" threshold is (i.e., when does Sifr go from "slower" to "equivalent" for a given problem).

This matters practically: if the trie work (L2) fixes `0211` correctness, but the shared helper is still clone-heavy, does it re-enter the benchmark as a compiler slowness case? The phase needs to document the post-fix re-benchmark protocol.

**Requested edit**: Add a section "Post-fix re-benchmark protocol" to the document. Specify: which problems to re-run after each milestone, what delta is required to reclassify (e.g., median speedup > 0.8x = equivalent, > 0.5x = close), and how to update registry fields post-fix.

**6. The C1 string indexing section does not account for code-level algorithmic changes.**

C1 (lines 250–273) describes compiler lowering for string indexing: `s.chars().nth(i)`, `s.chars().count()`, `ch.to_string()`. The required direction is compiler-side. However, several problems in the C1 list (`0402`, `0567`) use Sifr code that is intentionally more string-heavy than the Python equivalent. For those problems, fixing the compiler lowering may not be sufficient — the Sifr code itself should be revisited before attributing residual slowness to the compiler.

The document notes this for `0402` but does not propagate it to all affected problems.

**Requested edit**: Add a note under C1 listing problems where C1-to-C4 compiler fixes are necessary-but-not-sufficient because the Sifr implementation is algorithmically more string-intensive than the Python source. For those, add a dependency: a LeetCode-code parity sub-task runs before the compiler fix is evaluated.

**7. No mention of how the52 failed problems convert back into benchmark cases.**

The document identifies the failure modes (lines 432–487) — type errors, moved value, build errors, correctness — but does not address that fixing those failures might *add Problems* back into the benchmark pool. For example, if `0084_largest_rectangle_in_histogram` type errors are fixed, it enters the benchmark and could be slower on a different input shape. The phase scope should include a path for "failed → correctness-passing → benchmarkable" to close the loop.

---

### Implementation-readiness summary

| Milestone | Blocking issues | Status |
|---|---|---|
| M0 | `analyze_slowness.py` does not exist | **Hard blocker** |
| M1 | Registry seeds (75 +53 problems with parity fields) don't exist | **Hard blocker** |
| M2 | No blocking issues; L1/L2/L3 are clear | Ready to scope |
| M3 | No blocking issues; C1–C4 are clear | Ready to scope |
| M4 | Circular: depends on M1 seeding | **Dependency gap** |

### What is working well

- The compiler-runtime / LeetCode-code ownership split is clean and usable as a ticket template.
- The count reconciliation (lines 36–44) is good methodology and should be preserved as the authoritative definition.
- The "Representative Emitted Rust Evidence" section (lines 202–248) grounds the claims in concrete generated code — this is the right level of evidence and should be turned into regression tests.
- The ratio convention (line 98) is clear.
- The `0.25x` example clarifies orientation.

### RecommendationPatch the two hard blockers (add M0 with `analyze_slowness.py` scope and registry seed scope), resolve the M1/M4 circular dependency explicitly, fix the `0234` double-listing, and add the two missing engineering sections (post-fix re-benchmark protocol and failure-to-benchmarkable conversion path). After those edits, the phase is implementable.
