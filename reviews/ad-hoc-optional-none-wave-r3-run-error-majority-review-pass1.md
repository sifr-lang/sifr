# Review: Wave-R3 Run-Error Majority Plan (Pass 1)

Date: 2026-03-30
Reviewer channel: Claude via `talk-to-claude` handoff flow
Status: **not ready** (corrections required before implementation)

## Reviewer Workflow Note

The reviewer session returned findings in the handoff terminal/log, but direct file write from reviewer process was blocked by reviewer-side write permissions. Findings below are transcribed from that returned review summary so the gate is still recorded.

## Critical Findings (Severity Ordered)

1. The residual `RUN_ERROR` set must remain **compiler-first closure**; reviewer explicitly rejects fixture-rewrite-first treatment for this batch.
2. The `E0308` residuals are not one cluster; they must be split into distinct root-cause buckets:
   - Optional/index contamination,
   - control-flow-as-expression (`()` vs return type),
   - `Vec<Box<dyn Any>>` container typing leakage,
   - string/borrow mismatches.
3. `0463_island_perimeter` has an explicit `+==` emission defect that must be called out as a dedicated codegen bug; it was missing from prior decomposition.
4. Some fixtures are mixed-cause (example called out: `0463`, `0763`) and require both narrowing/typing and codegen fixes, so wave ownership boundaries must reflect mixed ownership.

## Reviewer Recommendation on Wave Ordering

- Keep compiler-first direction.
- Use split waves by root-cause ownership (not by fixture batch shape).
- Ensure panic/statement-emission class is already closed before continuing residual waves.
- Prioritize strict typing/correctness invariants before cosmetic/demo normalization.

## Required Corrections Before “Ready”

- Expand Wave-R3 plan to explicitly isolate:
  - `+==` augassign render/emission defect lane,
  - empty-collection `Any` leakage lane,
  - bool-condition typing contract lane,
  - optional index/key contamination lane.
- Add fixture-to-bucket mapping table to avoid cross-wave ownership drift.
- Add regression guardrails for each bucket (targeted test names + fixture probes per lane).

## Verdict

**Not ready** until the above decomposition and guardrail updates are applied to the plan.
