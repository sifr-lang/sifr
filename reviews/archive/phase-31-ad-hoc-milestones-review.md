# Phase 31 Ad-Hoc Follow-up Milestones Review

**Date:** 2026-03-11
**Source:** `issues/phase31-ad-hoc-followup-milestones.md`
**Status:** Ready for implementation

---

## Executive Summary

The ad-hoc follow-up milestones document is well-constructed, correctly derived from source artifacts, and ready for implementation. The milestones cover all 45 remaining failing cases (44 supportable + 1 intentional divergence), follow a sound dependency-aware sequencing, and include appropriate validation gates.

**Recommendation:** Approve for execution. The planning is complete and the approach is sound.

---

## Data Verification

All data points in the milestones document were verified against source artifacts:

| Data Point | Document | Verified | Source |
|------------|----------|----------|--------|
| Optional narrowing cases | 16 | ✓ | `phase31_scorecard.json` line 87 |
| Destructuring cases | 7 | ✓ | `phase31_scorecard.json` line 111 |
| Stdlib module cases | 6 | ✓ | `phase31_scorecard.json` line 126 |
| Nested function cases | 6 | ✓ | `phase31_scorecard.json` line 140 |
| Generic check cases | 3 | ✓ | `phase31_scorecard.json` line 166 |
| Recursive node cases | 4 | ✓ | `phase31_scorecard.json` line 154 |
| Attribute expression cases | 1 | ✓ | `phase31_scorecard.json` line 177 |
| Unsupported AST cases | 1 | ✓ | `phase31_scorecard.json` line 186 |
| Ownership divergence (1299) | 1 | ✓ | `phase31_scorecard.json` line 195 |
| Total remaining | 45 | ✓ | Sum of wave_1_remaining |
| Supportable | 44 | ✓ | 45 - 1 divergence |
| Current passes | 5 | ✓ | Phase 31 scorecard |

---

## Milestone Readiness Assessment

### m31_a: Optional Narrowing Core

- **Status:** Ready
- **Blocked cases:** 16 (largest remaining bucket)
- **Dependencies:** None (correctly identified as independent)
- **Validation:** Type-system regression tests, corpus rerun, full suite
- **Rationale:** Correctly placed first due to highest impact potential and independence

### m31_b: Destructuring Target Lowering

- **Status:** Ready
- **Blocked cases:** 7
- **Dependencies:** None
- **Validation:** E2E coverage for destructuring assignments/loops, corpus rerun
- **Rationale:** Independent lowering work; builds on wave 1 tuple unpack fix

### m31_c: Stdlib Module Parity

- **Status:** Ready
- **Blocked cases:** 6
- **Dependencies:** None (correctly identifies as runtime/API surface)
- **Validation:** Stdlib parity tests, demo showing graph/heap case
- **Rationale:** Can proceed in parallel with compiler fixes

### m31_d: Nested Function Pipeline

- **Status:** Ready (with dependency note)
- **Blocked cases:** 10 (6 + 3 + 1)
- **Dependencies:** Correctly identified - `nested_function_annotation_support` depends on `unsupported_ast_shape`
- **Validation:** Lowering tests, frontend tests, corpus rerun
- **Rationale:** Correctly combines three related buckets; dependency chain is sound

### m31_e: Tree Node Surface

- **Status:** Ready (with dependency note)
- **Blocked cases:** 5 (4 + 1)
- **Dependencies:** Correctly identified - `attribute_expression_support` depends on `recursive_node_forward_reference`
- **Validation:** Type-system tests, lowering tests, corpus rerun
- **Rationale:** Correctly combines dependent buckets; closes tree-domain gap

### m31_f: Ownership Divergence Resolution

- **Status:** Ready (decision milestone, not implementation)
- **Blocked cases:** 1 (case 1299)
- **Scope:** Product decision required
- **Validation:** Documentation update
- **Rationale:** Correctly identifies this as a policy decision, not a bug fix

---

## Sequencing Rationale Assessment

The proposed order is justified:

1. **m31_a first:** Largest independent bucket, highest potential impact
2. **m31_b second:** Another independent lowering fix
3. **m31_c parallel-ready:** Stdlib work can proceed independently
4. **m31_d after m31_b:** Nested function pipeline requires AST shape support (dependency: `unsupported_ast_shape` must be fixed first)
5. **m31_e after m31_a:** Tree node surface requires recursive forward reference support (dependency: `recursive_node_forward_reference`)
6. **m31_f last:** Policy decision, not implementation

**Dependency verification from backlog:**
- `frontend.nested_function_annotation_support` → `lowering.unsupported_ast_shape` ✓
- `lowering.attribute_expression_support` → `type_system.recursive_node_forward_reference` ✓

---

## Planning Quality

### Strengths

1. **Complete coverage:** All 45 remaining cases are assigned to milestones
2. **Clear exit conditions:** Each milestone has measurable definition of done
3. **Validation gates:** Every milestone specifies required tests, corpus reruns, and demo evidence
4. **No fallback semantics:** Correctly avoids weakening ownership/type guarantees
5. **Root-cause focus:** Fixes buckets, not individual LeetCode programs
6. **PR-ready structure:** Each milestone can execute as its own plan→implement→validate→PR loop

### Minor Observations

1. **Effort sizing:** The backlog indicates `large` effort for optional narrowing and `medium` for most others - this seems reasonable
2. **m31_d scope:** Combines 3 buckets (10 cases) - may be large but the dependency rationale is sound
3. **m31_c independence:** Can likely run in parallel with other milestones if resources permit

---

## Risk Assessment

- **Technical risk:** Low - all milestones target specific, well-categorized failure buckets
- **Dependency risk:** Low - dependencies are correctly identified and sequenced
- **Scope creep risk:** Low - clear boundaries per milestone with explicit case counts
- **Validation coverage:** Adequate - each milestone requires regression tests, corpus reruns, and demo evidence

---

## Conclusion

The ad-hoc follow-up milestones are ready for implementation. The planning is thorough, the data is verified, the sequencing is justified by dependencies, and each milestone has clear validation gates.

**Action:** Approve for execution. Begin with m31_a (optional narrowing core) as planned.
