# Review Pass 1: Codegen Runtime Build Gap Root-Cause Breakdown

**Reviewed**: 2026-04-05
**Source artifact**: `issues/codegen-runtime-build-gap-root-cause-breakdown-2026-04-05.md`
**Supporting data**: CSV breakdown, taxonomy JSON, results JSON

---

## 1. Internal Consistency Check

### 1.1 Family Counts vs Per-Case Listing

| Family | Declared Count | Actual Count (manual) | Status |
|--------|---------------|----------------------|--------|
| `recursive_field_access_surface_leaks_to_codegen` | 21 | 21 | PASS |
| `type_contract_mismatch_reaching_codegen` | 17 | 17 | PASS |
| `non_standard_rust_build_failure_without_error_code` | 7 | 7 | PASS |
| `ownership_move_reuse_in_generated_rust` | 6 | 6 | PASS |
| `missing_or_mis_scoped_binding_in_codegen_output` | 3 | 3 | PASS |
| `rust_build_contract_breakage_misc` | 3 | 3 | PASS |
| `truthiness_not_lowering_to_bool_contract` | 1 | 1 | PASS |
| **Total** | **58** | **58** | **PASS** |

### 1.2 Resolution Lane Counts

| Lane | Declared | Actual Count | Status |
|------|----------|-------------|--------|
| `both` | 38 | 38 | PASS |
| `compiler_fix` | 20 | 20 | PASS |
| **Total** | **58** | **58** | **PASS** |

### 1.3 Rust Error Code Counts

All 15 error codes verified against the per-case listing:

| Code | Declared | Actual | Status |
|------|----------|--------|--------|
| E0308 | 34 | 34 | PASS |
| E0609 | 21 | 21 | PASS |
| NO_RUST_CODE | 7 | 7 | PASS |
| E0382 | 6 | 6 | PASS |
| E0277 | 4 | 4 | PASS |
| E0282 | 2 | 2 | PASS |
| E0596 | 2 | 2 | PASS |
| E0369 | 1 | 1 | PASS |
| E0424 | 1 | 1 | PASS |
| E0425 | 1 | 1 | PASS |
| E0434 | 1 | 1 | PASS |
| E0502 | 1 | 1 | PASS |
| E0599 | 1 | 1 | PASS |
| E0600 | 1 | 1 | PASS |
| E0631 | 1 | 1 | PASS |

### 1.4 Cross-Reference with Taxonomy JSON

The taxonomy JSON subcategories break down as:
- `other_build_failure`: 50
- `rust_move_borrow_emission`: 6
- `rust_missing_binding_emission`: 1
- `unary_not_codegen_contract`: 1

Mapping to breakdown families:

| Taxonomy Subcategory | Count | Breakdown Family | Count | Reconciliation |
|---------------------|-------|-----------------|-------|----------------|
| `rust_move_borrow_emission` | 6 | `ownership_move_reuse_in_generated_rust` | 6 | Exact match |
| `rust_missing_binding_emission` | 1 | `missing_or_mis_scoped_binding_in_codegen_output` | 3 | Breakdown pulls 2 additional from `other_build_failure` (0304, 0417) |
| `unary_not_codegen_contract` | 1 | `truthiness_not_lowering_to_bool_contract` | 1 | Exact match |
| `other_build_failure` | 50 | (remaining 5 families) | 50 | 21 + 17 + 7 + 2 + 3 = 50 |

The reclassification of 0304 and 0417 out of `other_build_failure` into `missing_or_mis_scoped_binding_in_codegen_output` is **justified** (E0424 = `self` in non-method context; E0434 = fn-item can't capture environment). Both are genuinely scoping/binding issues.

**Verdict: All counts are internally consistent. No arithmetic errors found.**

---

## 2. Misclassified Cases

### 2.1 RECLASSIFY: `0211_design_add_and_search_words_data_structure`

**Current**: `ownership_move_reuse_in_generated_rust` / `compiler_fix`
**Codes**: E0277, E0382
**First error line**: `error[E0277]: can't compare 'String' with 'Option<_>'`

**Problem**: The _primary_ Rust error is E0277 (type comparison String vs Option), which is a type-contract mismatch -- the codegen is comparing a `String` to an `Option<_>`, which is a type surface error, not a move/borrow error. The E0382 is likely a cascade failure. The taxonomy labelled it `rust_move_borrow_emission` based on the E0382 presence, but the first and likely root-cause error is the type comparison.

**Recommended correction**:
- Family: `type_contract_mismatch_reaching_codegen`
- Lane: `both`

**Impact**: `ownership_move_reuse` drops from 6 to 5; `type_contract_mismatch` rises from 17 to 18. `compiler_fix` drops from 20 to 19; `both` rises from 38 to 39.

### 2.2 REVIEW: `0729_my_calendar_i`

**Current**: `rust_build_contract_breakage_misc` / `compiler_fix`
**Codes**: E0277, E0596
**First error line**: `error[E0277]: 'CalendarNode' doesn't implement 'std::fmt::Display'`

**Problem**: The missing `Display` trait on `CalendarNode` could indicate that the runtime type definition (sifr adaptation) is incomplete -- the struct needs a `#[derive(...)]` or manual `impl Display`. This isn't purely a compiler codegen issue; the runtime type definitions may also need updating.

**Recommended correction**:
- Lane: change to `both` (or investigate whether `CalendarNode` is a sifr-defined type)

### 2.3 REVIEW: `0783_minimum_distance_between_bst_nodes`

**Current**: `rust_build_contract_breakage_misc` / `compiler_fix`
**Code**: E0369
**First error line**: `error[E0369]: cannot subtract 'Option<i64>' from 'Option<i64>'`

**Problem**: This is conceptually similar to the `recursive_field_access_surface_leaks_to_codegen` family. The codegen emits arithmetic on `Option<i64>` without unwrapping, just like field access on `Option<ListNode>` without unwrapping. The underlying issue is the same: Option-wrapped values leaking into expression positions that expect bare values.

**Recommended correction**:
- Family: Consider broadening `recursive_field_access_surface_leaks_to_codegen` to `option_wrapped_value_surface_leaks_to_codegen` to encompass both field access (E0609) and operator application (E0369) on Option types. Alternatively, keep as `misc` but acknowledge the shared root cause.
- Lane: change to `both` if the Option wrapping is an adaptation-side concern.

### 2.4 MINOR: `1203_sort_items_by_groups_respecting_dependencies`

**Current**: `rust_build_contract_breakage_misc` / `compiler_fix`
**Code**: E0502
**Error**: simultaneous mutable/immutable borrow

**Observation**: E0502 (conflicting borrows) is conceptually adjacent to E0382 (use after move) -- both are Rust ownership system violations in generated code. The current `misc` classification is defensible since the `ownership_move_reuse` family name specifically says "move reuse" (E0382). However, if that family were broadened to "ownership and borrow issues in generated Rust", this case would fit.

**Recommendation**: No change required, but if the family is renamed, absorb this case.

---

## 3. Lane Assignment Pattern Analysis

The lane assignment is **deterministic by family** -- every case in a family gets the same lane:

| Family | Lane |
|--------|------|
| `recursive_field_access_surface_leaks_to_codegen` | `both` |
| `type_contract_mismatch_reaching_codegen` | `both` |
| `non_standard_rust_build_failure_without_error_code` | `compiler_fix` |
| `ownership_move_reuse_in_generated_rust` | `compiler_fix` |
| `missing_or_mis_scoped_binding_in_codegen_output` | `compiler_fix` |
| `rust_build_contract_breakage_misc` | `compiler_fix` |
| `truthiness_not_lowering_to_bool_contract` | `compiler_fix` |

**Observation**: There is no `sifr_adaptation` (adaptation-only) lane. This is defensible since all 58 cases are codegen build failures, meaning the compiler always bears at least partial responsibility. However, the blanket `both` for all 17 `type_contract_mismatch` cases may be too conservative -- some E0308 mismatches (e.g., `0189_rotate_array`, `0567_permutation_in_string`) could be pure compiler issues where the codegen emits wrong casts/coercions with no runtime type definition change needed.

**Recommendation**: Split the `type_contract_mismatch_reaching_codegen` family into:
- Cases where the mismatch involves sifr-defined types (ListNode, TreeNode, etc.) -> `both`
- Cases where the mismatch is between primitive/stdlib types -> `compiler_fix`

This requires inspecting the generated Rust code (see Section 5).

---

## 4. Recommended Execution-Ready Structure

### 4.1 Revised Family Taxonomy

```
codegen_runtime_build_gap (58 cases)
|
+-- option_wrapped_value_surface_leaks (22)
|   |   Covers: E0609 (field access on Option), E0369 (operator on Option)
|   |   Root cause: codegen accesses fields/operators on Option<T> without unwrapping
|   |   Lane: both (compiler must emit unwrap; adaptation must ensure type defs are correct)
|   |
|   +-- field_access_on_option_node (21)  [E0609 present]
|   +-- operator_on_option_value (1)      [E0369, case 0783]
|
+-- type_contract_mismatch_reaching_codegen (18)
|   |   Covers: E0308 without E0609
|   |   Root cause: codegen emits wrong types for function args, returns, assignments
|   |   Lane: both (conservative) or compiler_fix (if only primitive types involved)
|   |
|   +-- sifr_type_mismatch (TBD - needs inspection)
|   +-- primitive_type_mismatch (TBD - needs inspection)
|
+-- no_rust_code_emitted (7)
|       Covers: NO_RUST_CODE, codegen panics
|       Root cause: codegen crashes or fails to emit any Rust
|       Lane: compiler_fix
|
+-- ownership_and_borrow_violation (6)
|   |   Covers: E0382 (move reuse), E0502 (conflicting borrows)
|   |   Root cause: codegen doesn't track Rust ownership/borrow rules
|   |   Lane: compiler_fix
|   |
|   +-- use_after_move (5)     [E0382]
|   +-- conflicting_borrow (1) [E0502, case 1203]
|
+-- missing_or_mis_scoped_binding (3)
|       Covers: E0425 (undefined variable), E0424 (self misuse), E0434 (closure capture)
|       Root cause: codegen emits references to bindings outside their scope
|       Lane: compiler_fix
|
+-- truthiness_lowering (1)
|       Covers: E0600 (unary ! on non-bool)
|       Root cause: Python truthiness not lowered to Rust bool idiom
|       Lane: compiler_fix
|
+-- misc_trait_and_api (2)
        Covers: E0277 (missing Display), E0596 (mutability)
        Cases: 0729_my_calendar_i
        Root cause: generated code assumes traits/mutability not present
        Lane: both (trait impls may need adaptation)
```

### 4.2 Execution Workstreams

**Workstream 1: Option Unwrapping (22 cases, highest impact)**
- Compiler: emit `.unwrap()`, `if let Some(node) = ...`, or `match` patterns when accessing fields/operators on Option-wrapped recursive types
- Adaptation: ensure ListNode/TreeNode type definitions use correct Option wrapping and field layout
- Test gate: all 22 cases must produce valid Rust code that compiles with `cargo check`

**Workstream 2: Type Emission Correctness (18 cases)**
- Compiler: fix type coercions, function argument types, return types in codegen
- Requires per-case investigation of generated Rust to distinguish compiler-only vs adaptation-needed
- Test gate: all 18 cases must pass `cargo check`

**Workstream 3: Codegen Crash Recovery (7 cases)**
- Compiler: fix panics and empty-emission paths in sifr_codegen
- Start with 0662 (has a specific panic location: `sifr_codegen/src/lib.rs:1492:17`)
- Test gate: all 7 cases must emit non-empty Rust code

**Workstream 4: Ownership Model (6 cases)**
- Compiler: add clone/borrow-check awareness to codegen
- Lower priority -- Rust ownership is hard to get right in general codegen
- Test gate: all 6 cases must pass borrow checker

**Workstream 5: Binding Scoping (3 cases)**
- Compiler: fix variable scope tracking, self-reference emission, closure vs fn-item selection
- Test gate: all 3 cases must resolve E0424/E0425/E0434

**Workstream 6: Truthiness + Misc (3 cases)**
- Compiler: lower Python truthiness to `.is_empty()` / `!= 0` / `.is_some()`
- Adaptation: add Display trait derivation where needed
- Test gate: all 3 cases must compile

### 4.3 Priority Order (by impact)

1. **Option unwrapping** (22 cases) -- single fix pattern, highest ROI
2. **Type emission** (18 cases) -- needs per-case triage first
3. **Codegen crashes** (7 cases) -- total failures, likely separate bugs
4. **Ownership** (6 cases) -- hard problem, may need iterative approach
5. **Binding scoping** (3 cases) -- small, targeted fixes
6. **Truthiness + misc** (3 cases) -- small, targeted fixes

---

## 5. Missing Evidence

The following evidence gaps limit confidence in `compiler_fix` vs `sifr_adaptation` lane decisions:

### 5.1 Critical Missing Data

| Data Needed | Why | Impact |
|-------------|-----|--------|
| Generated Rust source code for each case | Cannot verify whether type mismatches originate in codegen logic or runtime type definitions | All 18 `type_contract_mismatch` cases have uncertain lane assignments |
| Sifr runtime type definitions (ListNode, TreeNode, CalendarNode, etc.) | Cannot determine if Option wrapping is imposed by adaptation or by codegen | All 22 option-surface cases |
| Python source for each fixture | Cannot trace type flow from Python through compiler to Rust | All 58 cases |

### 5.2 Specific Cases Needing Code Inspection

| Case | Question |
|------|----------|
| `0211_design_add_and_search_words_data_structure` | Is the E0277 (String vs Option comparison) the root cause or is E0382 (move)? Need to see the generated code. |
| `0729_my_calendar_i` | Is `CalendarNode` defined in sifr runtime or generated by compiler? Determines whether Display impl is an adaptation or compiler fix. |
| `0189_rotate_array` | E0308 on a pure array rotation -- is this a primitive type coercion issue (compiler_fix) or a type definition issue (both)? |
| `0894_all_possible_full_binary_trees` | Has 3 error codes (E0308, E0599, E0631). Which is the root cause? Are the E0599/E0631 cascades from E0308? |
| All 7 NO_RUST_CODE cases | What Python patterns trigger the codegen to emit nothing? Is there a common pattern (e.g., specific stdlib usage, specific control flow)? |

### 5.3 Recommended Next Steps for Evidence Collection

1. **Dump generated Rust code**: For each of the 58 cases, capture the `.rs` file emitted by sifr_codegen (or the empty file / panic trace for NO_RUST_CODE cases).
2. **Annotate type origins**: For every E0308 case, mark whether the mismatched type comes from a sifr runtime type definition or from a codegen-synthesized type.
3. **Validate the 0211 reclassification**: Read the generated code to confirm E0277 is the true root cause.
4. **Inspect 0662 panic**: The panic at `sifr_codegen/src/lib.rs:1492:17` is the only case with a specific crash location -- use it to understand the codegen crash pattern and check if other NO_RUST_CODE cases hit similar code paths.

---

## 6. Summary of Corrections

| Case ID | Current Family | Corrected Family | Current Lane | Corrected Lane | Confidence |
|---------|---------------|-----------------|-------------|---------------|------------|
| `0211_design_add_and_search_words_data_structure` | `ownership_move_reuse_in_generated_rust` | `type_contract_mismatch_reaching_codegen` | `compiler_fix` | `both` | High (E0277 is first/primary error) |
| `0729_my_calendar_i` | `rust_build_contract_breakage_misc` | `rust_build_contract_breakage_misc` (no change) | `compiler_fix` | `both` | Medium (needs CalendarNode origin check) |
| `0783_minimum_distance_between_bst_nodes` | `rust_build_contract_breakage_misc` | `recursive_field_access_surface_leaks_to_codegen` (or new `option_wrapped_value_surface_leaks`) | `compiler_fix` | `both` | Medium (conceptual alignment with Option-unwrap family) |
| `1203_sort_items_by_groups_respecting_dependencies` | `rust_build_contract_breakage_misc` | No change (but consider if ownership family is broadened) | `compiler_fix` | No change | Low (naming preference only) |

### Corrected Counts (if all corrections applied)

| Family | Original | Corrected |
|--------|----------|-----------|
| `recursive_field_access_surface_leaks_to_codegen` | 21 | 22 (+0783) |
| `type_contract_mismatch_reaching_codegen` | 17 | 18 (+0211) |
| `ownership_move_reuse_in_generated_rust` | 6 | 5 (-0211) |
| `rust_build_contract_breakage_misc` | 3 | 1 (-0783, -0729 stays but lane changes) |
| Others | unchanged | unchanged |

| Lane | Original | Corrected |
|------|----------|-----------|
| `both` | 38 | 41 (+0211, +0729, +0783) |
| `compiler_fix` | 20 | 17 (-0211, -0729, -0783) |

---

## 7. Overall Assessment

**Strengths of the analysis**:
- All 58 cases are accounted for with no duplicates or omissions
- The family taxonomy is well-motivated and maps cleanly to error code patterns
- Counts are arithmetically consistent across all three artifacts (markdown, CSV, taxonomy JSON)
- The deterministic family-to-lane mapping makes the analysis reproducible

**Weaknesses**:
- Lane assignments are derived from error code patterns alone, without inspecting generated Rust code
- The `type_contract_mismatch` family (17-18 cases) is too coarse -- it likely contains both pure compiler issues and mixed compiler+adaptation issues
- No `sifr_adaptation` only lane exists, which may be correct but is unverified
- The `misc` bucket (3 cases) contains at least one case (0783) that belongs in an existing family
- The 0211 misclassification is a concrete error that should be corrected before using this analysis for execution planning
