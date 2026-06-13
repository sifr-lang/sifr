# Review: Ad Hoc Test Strategy and Validation Lane Redesign

**Document:** `issues/ad-hoc-test-strategy-and-validation-lane-redesign.md`
**Status:** Ad hoc planning phase
**Review pass:** 1

---

## Summary

The document presents a well-reasoned, technically grounded analysis of the current test infrastructure problems and proposes a layered validation architecture. The core thesis is sound: the current test suite conflates multiple distinct validation jobs (developer feedback, merge confidence, determinism research, hardening) into a single expensive local loop. The proposed solution—layered validation lanes with explicit boundaries—is architecturally correct.

**Overall assessment: Strong design artifact with minor gaps that should be addressed before execution promotion.**

---

## Coherence: Strong

The document is internally consistent:

1. **Problem framing is clear.** The seven root-cause findings (sections 82-156) are specific, evidenced by timing measurements, and logically lead to the proposed architecture.

2. **Layered architecture is well-structured.** The five-layer model (Static → Unit → Contract → E2E → Hardening) correctly maps validation costs to invariant sensitivity. Each layer has a clear purpose, characteristics, and appropriate scope.

3. **Lane definitions are coherent.** The `quick`/`pr`/`nightly`/`release` separation correctly isolates different validation economics. The document explicitly prioritizes local developer experience over CI symmetry, which is the right trade-off for a planning document focused on local ergonomics.

4. **Closure criteria are explicit.** Sections 637-646 define measurable success criteria that would allow a reviewer to determine if the phase achieved its goals.

---

## Rigorousness: Strong with Minor Gaps

### Evidence Base

The document anchors claims in concrete data:
- Timing measurements for each current validation component (lines 65-73)
- Structural facts: "48 separate `cargo run -q -p sifr -- ...` calls" (line 76)
- Cache size observations (line 79)
- E2e execution count analysis: up to 5x repeated e2e runs (lines 100-106)

This empirical grounding strengthens the analysis considerably.

### Technical Soundness

The proposed cache strategy (sections 378-440) is sound:
- **A. Frontend Compilation Cache**: Correctly keyed by source hash + closure fingerprint
- **B. Generated Rust Cache**: Addresses the codegen layer
- **C. Generated Program Build Cache**: Targets the Rust build bottleneck
- **D. Optional Execution Cache**: Appropriately conservative (opt-in, deterministic-only)

The invariant coverage mapping (sections 441-495) correctly assigns proof strategies to invariant types.

### Gaps to Address

1. **No discussion of existing test infrastructure.** The document references source-of-truth scripts but doesn't analyze how the existing test harness in `crates/sifr/tests/e2e.rs` works or what about it specifically needs redesign. A more detailed analysis of current architecture would strengthen the foundation.

2. **Metrics section is aspirational but not actionable.** Section 497-512 lists metrics to track but doesn't specify:
   - How these metrics will be collected
   - What thresholds constitute regressions
   - What tools will store/visualize them
   - How they relate to lane definitions

3. **Missing analysis of test fixture taxonomy.** The document mentions "fixtures" repeatedly but doesn't classify them:
   - Which fixtures are in the hardening corpus?
   - What's the size of the representative e2e sample?
   - What's the total fixture count across layers?

   Without this, milestone definitions (e.g., "a very small representative Layer 3 e2e sample") lack specificity.

4. **Worker/thermal policy is vague.** Section 307 mentions "thermally aware default" but doesn't specify:
   - What thermal envelope is acceptable?
   - What worker count defaults are recommended?
   - How will thermal impact be measured?

---

## Locally Optimized: Strong

The document correctly prioritizes local developer experience:

1. **Local-first framing is explicit.** Lines 14-16: "This phase is explicitly local-first. CI/CD symmetry is not the design driver for this document."

2. **Quick lane target is concrete.** Line 306: "ideally 2-5 minutes on a normal laptop" is specific and meaningful.

3. **Correct diagnosis of the problem.** The document identifies that the issue is "wrong workload in the default lane" (line 155), not parallelism tuning. This is the right root cause.

4. **Preserves what matters.** Section 162-181 explicitly lists what must be preserved, demonstrating the document doesn't trade correctness for speed.

5. **Separation is correct.** Moving determinism and hardening out of `quick` (lines 317-321) is the right local optimization.

---

## Actionable: Moderate

The document is a strong planning artifact but needs work before execution:

### Strengths for Actionability

1. **Milestones are specific and ordered.** Sections 561-627 define six milestones with clear scope and definition of done.

2. **Execution order is explicit.** Lines 550-558 provide a prioritized work sequence.

3. **Reviewer gate is explicit.** Sections 628-635 define what must be confirmed at milestone completion.

4. **Risks are identified with mitigations.** Sections 522-545 identify four key risks with specific mitigations.

### Gaps for Execution Readiness

1. **No ownership or timeline.** No indication of:
   - Who owns each milestone
   - Estimated effort per milestone
   - Dependencies between milestones

2. **No migration plan details.** The document states "no fallback/shim architecture" (line 517) but doesn't detail how to transition from current to new without temporary duplication.

3. **The "declarative harness" is not specified.** Milestone_test_2 (lines 576-585) describes what the harness should do but not:
   - What language/tool will implement it
   - What the manifest format looks like
   - How it integrates with existing test infrastructure

4. **No discussion of backward compatibility.** How does this affect existing test commands developers might be running directly?

---

## Specific Technical Questions

### For the document author:

1. **What is the target size of the "very small representative Layer 3 e2e sample" (line 314)?** The current e2e suite has ~200+ test cases. Is the target 10? 20? 50?

2. **How do you intend to implement the declarative harness?** Is it a Rust binary? A Python script? Something else?

3. **What's the strategy for fixture classification?** Will fixtures be tagged with metadata to determine which lane they belong to?

4. **How will cache invalidation be triggered?** The document mentions "schema-versioned invalidation" (line 532) but doesn't specify the mechanism.

5. **How does the existing `insta` snapshot testing strategy integrate with this plan?** The document mentions "snapshots" but doesn't connect to the existing snapshot infrastructure.

---

## Recommendations

### Before execution promotion:

1. **Add fixture taxonomy.** Classify the current fixture set by type, size, and current lane. This provides the baseline for "representative" selection.

2. **Specify the declarative harness.** At minimum, describe the manifest format and implementation approach.

3. **Add ownership and rough timeline.** Even approximate effort estimates would help prioritization.

4. **Connect to existing infrastructure.** Analyze `crates/sifr/tests/e2e.rs` and the existing test runner to identify what specifically needs redesign vs. reuse.

### After execution promotion:

5. **Define specific metrics with thresholds.** The metrics section should become an implementation spec, not just a list.

6. **Add a migration path.** Detail how developers transition from current commands to new lanes without breaking existing workflows.

---

## Conclusion

This is a **strong design document** that correctly identifies the problem, proposes a sound architectural solution, and provides actionable milestones. The local-first framing is appropriate, the layered architecture is technically sound, and the closure criteria are measurable.

The document's main gaps are in specificity around implementation details (harness format, fixture classification, metric thresholds) and practical execution concerns (ownership, timeline, migration). These are expected gaps for an ad hoc planning phase—filling them is the work of execution planning.

**Recommendation: Approve for promotion to execution phase, with the expectation that implementation planning fills in the identified gaps.**
