# Review: Ownership/Mutability Boundary Root-Cause Analysis

**Reviewer:** agent (pass 1)
**Date:** 2026-04-02
**Report under review:** `issues/ownership-mutability-boundary-root-cause-2026-04-02.md`

---

## 1. Factual Correctness of Ownership Category Count and Sub-Buckets

**Verdict: CORRECT**

All counts are internally consistent across all three source artifacts:

| Sub-bucket | Report | Taxonomy JSON | Breakdown JSON | Fixture list length |
|---|---|---|---|---|
| `immutable_parameter_mutation` | 30 | 30 | 30 | 30 |
| `immutable_parameter_reassignment` | 11 | 11 | 11 | 11 |
| `borrowed_parameter_escape_store` | 4 | 4 | 4 | 4 |
| `borrowed_parameter_escape_return` | 2 | 2 | 2 | 2 |
| **Total** | **47** | **47** | **47** | **47** |

Parameter frequency table sums to 47 (18 singletons + 7 doubles + `node`(4) + `nums`(11) = 47). All frequency values match the breakdown JSON exactly.

The report correctly notes the rerun count of 47 (not the previously discussed 48). No discrepancy found.

## 2. Compiler-vs-Fixture Adaptation Split

**Verdict: CORRECT**

The report's classification of all 47 failures as fixture/source adaptation (not compiler defects) is verified against `internal_docs/architecture.md`:

- **Line 149**: Confirms `mut`/`own mut` is explicitly required for parameter reassignment/mutation. Report's `root_cause_a` (41 fixtures) is a direct consequence.
- **Lines 308-324**: Confirms borrow-by-default semantics with the four valid surface forms (`x`, `mut x`, `own x`, `own mut x`). The two-axis model (ownership x mutability) matches the report's decomposition exactly.
- **Line 338**: Confirms borrowed parameters (including `mut` borrows) cannot escape by return or store. Report's `root_cause_b` (6 fixtures) is a direct consequence.

All three architecture references in the report cite the correct line numbers and accurately characterize the language rules.

The "do not loosen language semantics" decision is architecturally sound. These are Sifr's core invariants; weakening them would undermine the borrow-by-default contract.

## 3. Architectural Consistency with Explicit Mutability/Ownership Semantics

**Verdict: CORRECT, with one refinement needed**

The report's two root causes map cleanly to the two axes of the Sifr parameter model:

- `root_cause_a` (mutability axis): parameter needs `mut` or `own mut`
- `root_cause_b` (ownership axis): parameter needs `own` or `own mut`

**Refinement needed on overlap analysis:**

The report claims `0075_sort_colors`, `0669_trim_a_binary_search_tree`, and `0701_insert_into_a_binary_search_tree` "likely need compound boundary annotations (`own mut`) spanning both mutability and ownership dimensions." This was verified against raw compiler stderr:

- **0669**: Emits BOTH `cannot mutate through immutable parameter 'root'` (x2) AND `cannot return borrowed parameter 'root'`. **Confirmed compound** -- needs `own mut`.
- **0701**: Identical diagnostic shape to 0669. **Confirmed compound** -- needs `own mut`.
- **0075**: Emits ONLY `cannot return borrowed parameter 'nums'` (escape). No mutation diagnostic in stderr. The claim that it needs `own mut` is **reasonable from remediation logic** (sort-colors mutates in-place then returns), but the compiler does NOT currently emit a mutation diagnostic for this fixture. The mutation error is likely cascade-suppressed after the escape error.

**Correction:** The report says "estimated 3-5" compound fixtures. The verifiable count from diagnostics is **2** (0669, 0701). 0075 is plausible but not diagnostic-confirmed. The report should distinguish diagnostic-confirmed compound fixtures from inferred-compound fixtures.

## 4. Missing Root-Cause Dimensions or Remediation Gaps

### 4a. Secondary errors masked by primary ownership classification

**Gap found.** Several fixtures in this bucket emit multiple errors spanning different categories. The taxonomy assigns each fixture to a single category based on the first diagnostic, but the raw results show:

| Fixture | Primary (ownership) | Secondary errors |
|---|---|---|
| `0002_add_two_numbers` | `borrowed_parameter_escape_store` | return type mismatch, unsupported operand, use of moved value |
| `0075_sort_colors` | `borrowed_parameter_escape_return` | return type mismatch, tuple unpacking (x2) |
| `0669_trim_a_binary_search_tree` | `immutable_parameter_mutation` | borrow-escape-return (same fixture, cross-axis) |
| `0701_insert_into_a_binary_search_tree` | `immutable_parameter_mutation` | borrow-escape-return (same fixture, cross-axis) |

The report mentions this at the remediation level ("at least 6 node/root-style fixtures may unmask secondary categories after ownership/mutability adaptation") but does not enumerate which fixtures have secondary errors or classify the secondary error types. This creates a risk of underestimating the residual work after the ownership adaptation pass.

**Recommended addition:** A secondary-error inventory for all 47 fixtures, at minimum identifying which fixtures have non-ownership secondary diagnostics that will survive the adaptation.

### 4b. No remediation ordering between `mut` and `own mut` within root_cause_a

The 41 mutation/reassignment fixtures are grouped as a single workstream. However, 0669 and 0701 (and potentially others) need `own mut`, not just `mut`. The remediation strategy (section "Execution-Ready Remediation Strategy", item 2) does document the `mut` vs `own mut` decision rule, but the workstream split (item 1) groups all 41 under "mutation/reassignment" without separating the compound cases. This could lead to two-pass rework on compound fixtures.

**Recommended refinement:** Split the mutation/reassignment workstream into:
- `mut`-only fixtures (estimated 39)
- `own mut` compound fixtures (confirmed 2: 0669, 0701; suspected 1+: 0075)

### 4c. Copy-type scalar rebinding heuristic needs precision

The report recommends `let mut local = param` for copy-type scalar rebinding (e.g., `n`, counters). This is correct. However, the report does not enumerate which of the 11 `immutable_parameter_reassignment` fixtures involve copy-type scalars vs. move-type parameters. For instance, `0312_burst_balloons` reassigns `nums` (a list, move-type) -- this needs `mut` at the parameter level, NOT local copy. The heuristic in the report is correct in principle but the per-fixture application guidance is missing.

### 4d. No discussion of `0605_can_place_flowers` / `0605_can_place_flowers_v2` duplication

Two fixtures share the same problem ID (0605) with different variants. The report counts them as 2 distinct fixtures (correct per the data), but does not note whether the adaptation strategy differs between variants or whether one is a duplicate that should be consolidated.

## 5. Risks to Zero-Failure Objective

### Risk 1: Secondary error residuals (HIGH)

As noted in 4a, multiple fixtures in this bucket have secondary errors that are NOT ownership/mutability related. Fixing the ownership annotation will not resolve these. The report's expectation that "at least 6" will unmask secondaries is likely an undercount -- cross-referencing raw results shows at least `0002`, `0075`, `0669`, `0701` have definite secondary errors. A systematic stderr scan of all 47 fixtures is needed before claiming the bucket is fully addressable by adaptation alone.

### Risk 2: Cascade suppression hiding additional ownership errors (MEDIUM)

The compiler's cascade reduction (mentioned in the report as a quality improvement target) means some fixtures may have additional ownership errors that are not emitted because an earlier error suppressed them. After the first adaptation pass, new ownership errors may surface for the same fixture. This is acknowledged implicitly in the report ("rerun full corpus and reclassify residuals") but is not quantified.

### Risk 3: Adaptation may shift fixtures between categories (LOW)

Changing a parameter from borrowed to `own` changes the codegen for the entire function. This could expose errors that are currently latent (e.g., previously unreachable code paths now reachable after ownership change). The "rerun and reclassify" step in the remediation addresses this, but the report does not budget for the possibility that adaptations increase the total failure count temporarily.

---

## Summary of Required Changes

| # | Severity | Section | Issue | Fix |
|---|---|---|---|---|
| 1 | Minor | Overlap note | Claims "estimated 3-5" compound fixtures; only 2 (0669, 0701) are diagnostic-confirmed | Distinguish confirmed vs. inferred compound fixtures; update estimate to "2 confirmed, 1+ inferred" |
| 2 | Medium | Remediation Strategy | Mutation/reassignment workstream does not separate `mut`-only from `own mut` compound cases | Add sub-split for compound fixtures within the 41-fixture workstream |
| 3 | Medium | Missing section | No secondary-error inventory for the 47 fixtures | Add appendix enumerating non-ownership secondary diagnostics per fixture |
| 4 | Minor | Remediation Strategy item 3 | Copy-type heuristic lacks per-fixture applicability mapping | Note which reassignment fixtures involve copy-type scalars vs. move-type params |
| 5 | Minor | Scope | Dual 0605 fixtures not discussed | Add note on whether v1/v2 variants need identical or different adaptation |

## Implementation-Readiness Verdict

**CONDITIONALLY READY.**

The report's core analysis is factually correct, architecturally sound, and the compiler-vs-fixture split is well-grounded. The 47-count, sub-bucket decomposition, parameter frequencies, and architecture references all verify against source data.

The report is ready to drive the fixture adaptation work with two conditions:
1. **Before starting adaptation:** Produce the secondary-error inventory (change #3 above) to avoid surprise residuals.
2. **Before starting the mutation/reassignment workstream:** Separate `own mut` compound fixtures from `mut`-only fixtures (change #2) to avoid rework.

Changes #1, #4, and #5 are documentation quality improvements that can be addressed in a report revision without blocking execution.
