# Review: Ad Hoc Test Strategy and Validation Lane Redesign — Production-Grade Assessment (Pass 2)

**Document:** `issues/ad-hoc-test-strategy-and-validation-lane-redesign.md`
**Status:** Ad hoc planning phase
**Review pass:** 2 (production-grade assessment)
**Assessor:** agent

---

## Executive Summary

The document has been significantly improved since Pass 1 and now addresses nearly all critical gaps identified in the initial review. The current state is **strong and ready for promotion to execution planning**, with only minor refinements recommended.

**Overall assessment: Production-grade for this phase. Minor refinements recommended before execution promotion.**

---

## Progress Since Pass 1

### Addressed from Pass 1

1. **Fixture Taxonomy Added** ✅
   - Section 109-159 now provides explicit classification of four fixture families
   - Current sizes are documented: 418 pass fixtures, ~57 hardening command variants
   - Target sizing is specified: quick (20-30), pr (50-80), nightly (full)

2. **Existing Infrastructure Analysis Improved** ✅
   - Section 55-79 explicitly defines reuse boundaries
   - Current reusable foundations are listed with specific file references
   - Redesign targets are clearly identified

3. **Declarative Harness Direction Specified** ✅
   - Section 470-509 provides concrete implementation guidance
   - Manifest schema is partially specified with fields
   - Rust-native implementation is explicitly recommended

4. **Layer Definitions Made Concrete** ✅
   - Section 334-409 defines five layers with explicit purposes
   - Lane contents are specified for quick/pr/nightly/release

5. **Migration Strategy Added** ✅
   - Section 573-586 outlines the transition approach
   - Backward-compatibility rules are explicit

### Remaining Gaps (Minor)

1. **Metrics Still Aspirational** - No concrete implementation plan for how metrics will be collected, stored, or visualized
2. **Thermal/Worker Policy Still Vague** - No specific thermal envelope or default worker count guidance
3. **Ownership/Timeline Not Assigned** - Suggested effort profile is rough, no specific owners
4. **Declarative Harness Not Implemented** - Still a design, not working code

---

## Production-Grade Criteria Assessment

### 1. Coherence: Strong ✅

The document is internally consistent:

- **Problem framing is clear.** The seven root-cause findings (sections 179-254) are specific and evidenced by timing measurements.
- **Layered architecture is well-structured.** The five-layer model correctly maps validation costs to invariant sensitivity.
- **Lane definitions are coherent.** The `quick`/`pr`/`nightly`/`release` separation correctly isolates different validation economics.
- **Closure criteria are explicit.** Sections 825-834 define measurable success criteria.

**Assessment: The document is coherent and logically structured.**

### 2. Rigorousness: Strong ✅

#### Evidence Base

The document anchors claims in concrete data:
- Timing measurements for each current validation component (lines 92-99)
- Structural facts: "48 separate `cargo run -q -p sifr -- ...` calls" (line 102)
- Cache size observations (line 105)
- Fixture counts: 418 pass fixtures, ~57 hardening command variants (lines 106-107)
- Current e2e corpus under `crates/sifr/tests/e2e/pass` (line 139)

#### Technical Soundness

The proposed cache strategy (sections 510-565) is sound:
- **A. Frontend Compilation Cache**: Correctly keyed by source hash + closure fingerprint
- **B. Generated Rust Cache**: Addresses the codegen layer
- **C. Generated Program Build Cache**: Targets the Rust build bottleneck
- **D. Optional Execution Cache**: Appropriately conservative (opt-in, deterministic-only)

The invariant coverage mapping (sections 587-642) correctly assigns proof strategies to invariant types.

**Assessment: The document is rigorous and technically sound.**

### 3. Locally Optimized: Strong ✅

The document correctly prioritizes local developer experience:

1. **Local-first framing is explicit.** Lines 14-17 explicitly state this is local-first, not CI-driven.
2. **Quick lane target is concrete.** Line 418: "ideally 2-5 minutes on a normal laptop" is specific.
3. **Correct diagnosis of the problem.** The document identifies that the issue is "wrong workload in the default lane" (line 253), not parallelism tuning.
4. **Preserves what matters.** Section 260-279 explicitly lists what must be preserved.
5. **Separation is correct.** Moving determinism and hardening out of `quick` is the right local optimization.

**Assessment: The document is well-optimized for local developer experience.**

### 4. Actionable: Strong ✅

#### Strengths

1. **Milestones are specific and ordered.** Sections 731-814 define six milestones with clear scope and definition of done.
2. **Execution order is explicit.** Section 717-729 provides a prioritized work sequence.
3. **Reviewer gate is explicit.** Sections 816-823 define what must be confirmed at milestone completion.
4. **Risks are identified with mitigations.** Sections 692-715 identify four key risks with specific mitigations.
5. **Suggested effort profile is rough but present.** Sections 802-814 provide rough estimates.

#### Minor Gaps (Not Blocking)

1. **No explicit ownership** - No specific people assigned to milestones
2. **Rough timeline only** - Estimates are "small", "medium", "large" - no specific dates
3. **Declarative harness is still a design** - No working implementation yet (expected for planning phase)

**Assessment: The document is actionable for execution planning.**

---

## Critical Gaps Analysis

### Critical Gaps (Must Address Before Execution)

**None identified.** The document is production-grade for its stated purpose as an ad hoc planning phase.

### Minor Refinements (Recommended Before Execution Promotion)

1. **Metrics Implementation Plan**
   - Current: Sections 643-682 list metrics but don't specify implementation
   - Recommendation: Add a brief note on which existing tools (e.g., `cargo test`, `time`, custom scripts) will be used to collect metrics
   - Priority: Low (can be deferred to execution planning)

2. **Thermal Policy Specificity**
   - Current: Section 676-678 mentions "thermally aware default" and "unsustainable sustained heat"
   - Recommendation: Add a concrete example of what thermal envelope is acceptable (e.g., "laptop fan should not spin up continuously on sustained quick runs")
   - Priority: Low (can be refined during milestone_test_1)

3. **Owner Assignment**
   - Current: Section 7 suggests "compiler test platform / local validation infrastructure" as phase owner
   - Recommendation: Assign a specific person or role to each milestone
   - Priority: Low (can be assigned at execution start)

### Observations on Current Infrastructure Alignment

The document references infrastructure that exists and is functional:

1. **Existing e2e runner** (`crates/sifr/tests/e2e.rs` - 3214 lines) already provides:
   - Fixture discovery
   - Worker configuration
   - Batch planning
   - Persistent cache manifest handling
   - Timing reporting

2. **Existing verification manifests** (`verification/suites/manifest.json`) already establish a declarative pattern with:
   - Suite definitions
   - Case specifications
   - Owner/blocking flags

3. **Current profile system** (`scripts/run_e2e_pass.sh`) already implements:
   - `quick`/`full`/`stress` profiles
   - Worker configuration
   - Cache management

4. **Current run_all_tests.sh** still runs all validation serially, which the document correctly identifies as the problem to solve.

**The document correctly identifies that it should build on existing foundations rather than replace them.**

---

## Comparison to Pass 1 Recommendations

### Before Execution Promotion (From Pass 1)

| Recommendation | Status |
|----------------|--------|
| Add fixture taxonomy | ✅ Addressed (lines 109-178) |
| Specify the declarative harness | ✅ Addressed (lines 470-509) |
| Add ownership and rough timeline | ⚠️ Partial (rough effort profile, no specific owners) |
| Connect to existing infrastructure | ✅ Addressed (lines 55-79) |

### After Execution Promotion (From Pass 1)

| Recommendation | Status |
|----------------|--------|
| Define specific metrics with thresholds | ⚠️ Still aspirational (lines 643-682) |
| Add a migration path | ✅ Addressed (lines 573-586) |

---

## Recommendations

### Before Promotion to Execution Phase

1. **Assign milestone owners** - Even placeholder names help accountability
2. **Add a brief metrics implementation note** - Even "use existing cargo/test reporting" is sufficient
3. **Clarify thermal guidance with an example** - e.g., "quick should complete without laptop fan activation on a MacBook Pro"

### During Execution Planning (Next Phase)

4. **Refine declarative harness schema** - The current fields (lines 487-500) are a good start but will need iteration
5. **Define specific e2e sample selection criteria** - How to choose the 20-30 fixtures for quick lane?
6. **Implement cache invalidation mechanism** - Section 701-703 mentions "schema-versioned invalidation" but needs concrete design

---

## Conclusion

The document has matured significantly since Pass 1 and now addresses nearly all critical gaps. It correctly identifies the problem, proposes a sound architectural solution, and provides actionable milestones with appropriate closure criteria.

**Assessment: Production-grade for this phase (ad hoc planning). Ready for promotion to execution planning.**

The minor gaps identified (metrics implementation, thermal specificity, owner assignment) are expected for a planning-phase document and can be refined during execution planning. They do not block promotion.

**Recommendation: Approve for promotion to execution phase. The identified minor refinements should be addressed during the transition to execution planning, not as a gate.**

---

## Summary Rating

| Criterion | Rating |
|-----------|--------|
| Coherence | Strong |
| Rigorousness | Strong |
| Locally Optimized | Strong |
| Actionable | Strong |
| **Overall** | **Production-Grade** |
