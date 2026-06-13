# Full Corpus Strategy Review - Pass 1

**Date:** 2026-03-14
**Source:** `verification/leetcode/full_corpus_current_results_20260314.json`
**Corpus Size:** 411 problems
**Current Pass Rate:** 53/411 (12.9%)

## Executive Summary

After analyzing the full 411-problem corpus against the current failure taxonomy, **the two existing broad ad hoc phases (recursive types and own_mut) are sufficient** to address the primary structural language gaps. The remaining failures fall into categories that can be handled by:

1. Normal compiler closure work (existing Phase 31 milestones)
2. Canonical fixture rewrites
3. Stdlib expansion

**No new broad language/compiler phases are required beyond recursive types and own_mut.**

---

## Failure Distribution Analysis

| Category | Count | Primary Root Cause | Handling Strategy |
|----------|-------|-------------------|-------------------|
| PASS | 53 | — | — |
| **Recursive Types** | 56 | Unknown types (ListNode/TreeNode/Node) + attribute access on recursive structures | `ad-hoc-full-recursive-type-feature` (BROAD PHASE) |
| **own_mut** | 2 | Cannot return borrowed parameter (0006, 1299) | `ad-hoc-own-mut-parameter-convention` (BROAD PHASE) |
| Optional Flow | 66 | `int \| None` type mismatches | `m31_a_optional_flow_completion` |
| Container Literal | 22 | `dict[Any, Any]` specialization | `m31_g_container_literal_specialization` |
| Destructuring | 28 | Tuple unpacking in loops/assignments | `m31_b_destructuring_and_composite_lvalues` |
| Class Field Access | 30 | `type 'X' has no field 'Y'` | Narrow closure or stdlib expansion |
| Stdlib Gaps | 25 | Missing `ord`, `chr`, `reversed`, `Counter`, etc. | Normal stdlib expansion |
| Any Cascading | 72 | Downstream of above issues | Will resolve when root causes fixed |
| Parse Errors | 2 | Fixture issues (0200, 0261) | Fixture cleanup |
| **Total Failing** | **358** | | |

---

## 1. Families That Justify Broad Prerequisite Phases

### 1.1 Recursive Types (56 cases)

**Cases:** 0002, 0019, 0021, 0023, 0024, 0025, 0061, 0083, 0086, 0092, 0094, 0098, 0100, 0101, 0102, 0104, 0106, 0110, 0112, 0118, 0124, 0133, 0138, 0141, 0143, 0144, 0145, 0147, 0148, 0160, 0199, 0206, 0211, 0212, 0226, 0230, 0234, 0235, 0236, 0297, 0450, 0513, 0543, 0572, 0606, 0617, 0669, 0701, 0729, 0783, 0876, 1448, 1609, 1669, 1721, 2130

**Error Pattern:**
- `unknown type: 'ListNode'`
- `unknown type: 'TreeNode'`
- `attribute access '.next' is not supported as an expression`
- `attribute access '.val' is not supported as an expression`
- `attribute access '.left' is not supported as an expression`

**Why this is a BROAD phase:** This is not a narrow LeetCode closure issue. It requires:
- Forward reference resolution for class names
- Recursive type representation in the type system
- Attribute expression lowering for recursive class instances
- Rust codegen for boxed recursive fields

This is exactly what `ad-hoc-full-recursive-type-feature` addresses.

**Validation:** The phase already has 6 parts completed. When this lands, all 56 cases will move past the recursive-type blockers.

---

### 1.2 own_mut Parameter Convention (2 cases)

**Cases:** 0006, 1299

**Error Pattern:**
- `cannot return borrowed parameter 'arr': it is borrowed by default -- use 'own arr' to take ownership`

**Why this is a BROAD phase:** This requires:
- Parser changes to accept `own mut` syntax
- HIR representation for dual ownership/mutability axis
- Codegen to emit `mut x: T` instead of `x: T`
- Escape analysis updates

This is exactly what `ad-hoc-own-mut-parameter-convention` addresses.

**Validation:** Once `own mut` lands, both 0006 and 1299 can be rewritten in canonical Sifr form.

---

## 2. Families That Are Ordinary Closure Work

### 2.1 Optional Flow / Narrowing (66 cases)

**Examples:** 0004, 0053, 0057, 0062, 0063, 0064, 0072, 0097, 0120, 0121, 0122, 0135, 0152, etc.

**Error Pattern:**
- `type mismatch: expected 'int', got 'int | None'`
- `if expression branches have incompatible types: 'int | None' and 'float'`
- `return type mismatch: expected 'bool', got 'bool | None'`

**Why ordinary closure:** This is standard type narrowing after guards. The Phase 31 milestone `m31_a_optional_flow_completion` already covers this work. It does not require new language features—only extending the existing flow analysis.

---

### 2.2 Container Literal Specialization (22 cases)

**Examples:** 0010, 0017, 0039, 0047, 0076, 0078, 0084, 0090, etc.

**Error Pattern:**
- `cannot index type 'dict[Any, Any]' with 'int'`
- `cannot index type 'dict[str, int]' with 'str | None'`

**Why ordinary closure:** Empty dict/list literals start as `dict[Any, Any]` and need specialization when written to. This is covered by `m31_g_container_literal_specialization_and_state_tracking`.

---

### 2.3 Destructuring (28 cases)

**Examples:** 0012, 0027, 0056, 0075, 0146, 0189, 0280, 0283, 0295, etc.

**Error Pattern:**
- `tuple unpacking target must be a simple name`
- `for loop tuple target expects iterable elements of tuple type, got 'list[int]'`

**Why ordinary closure:** Covered by `m31_b_destructuring_and_composite_lvalues`. This is narrow compiler work, not a language feature.

---

### 2.4 Class Field Access (30 cases)

**Examples:** 0155 (MinStack), 0208 (Trie), 0225 (MyStack), 0232 (MyQueue), 0303 (NumArray), etc.

**Error Pattern:**
- `type 'MinStack' has no field 'minStack'`
- `type 'MyStack' has no field 'q'`

**Why ordinary closure:** These are typically issues with how class `__init__` fields are stored. May require:
- Narrow fix for class field initialization pattern
- Or fixture canonicalization to use proper attribute syntax

---

### 2.5 Stdlib Gaps (25 cases)

**Examples:** 0049, 0067, 0168, 0187, 0278, 0350, 0374, 0383, 0567, 0767, etc.

**Error Pattern:**
- `undefined function: 'ord'`
- `undefined function: 'chr'`
- `undefined function: 'reversed'`
- `undefined function: 'Counter'`

**Why ordinary closure:** These are missing stdlib functions. Can be addressed through normal stdlib expansion without new language phases.

---

### 2.6 Any Cascading (72 cases)

These are downstream failures that will resolve when root causes are fixed. Examples:
- `'<' not supported between instances of 'Any' and 'int'`
- `len() argument must be ... got 'Any'`

These are NOT new categories—they're consequences of the above issues.

---

## 3. Families Requiring Canonical Fixture Rewrites

### 3.1 Raw-Source Policy Mismatches

**0043 (Multiply Strings):**
- Raw Python: Uses unchecked `int(str)` conversion
- Sifr: `int(str)` returns `Result[int, ParseError]`
- Action: Canonical Sifr rewrite (already in `m31_k_canonical_sifr_fixture_normalization`)

**Parse Errors (0200, 0261):**
- These are fixture syntax issues, not compiler issues
- Action: Fixture cleanup

---

## 4. Strategy Widening/Narrowing Assessment

### What DOESN'T need to change:

1. **`ad-hoc-full-recursive-type-feature`** - The 56 cases in the full corpus confirm this is the right scope. No widening needed.

2. **`ad-hoc-own-mut-parameter-convention`** - Only 2 cases (0006, 1299), but these are real language gaps. No narrowing needed.

3. **Phase 31 milestones** - The category counts align with the planned milestones:
   - `m31_a_optional_flow_completion` - 66 cases
   - `m31_b_destructuring_and_composite_lvalues` - 28 cases
   - `m31_g_container_literal_specialization` - 22 cases

### What MIGHT need attention:

1. **Class field access (30 cases)** - This wasn't explicitly called out in the original Phase 31 plan. Two options:
   - Add a narrow milestone for class field patterns
   - Or these may resolve as cascading failures after other fixes

2. **Stdlib gaps (25 cases)** - May need coordination to ensure priority stdlib functions are added.

---

## 5. Detailed Case Mapping

### Recursive Types Cases (56)
```
0002, 0019, 0021, 0023, 0024, 0025, 0061, 0083, 0086, 0092,
0094, 0098, 0100, 0101, 0102, 0104, 0106, 0110, 0112, 0118,
0124, 0133, 0138, 0141, 0143, 0144, 0145, 0147, 0148, 0160,
0199, 0206, 0211, 0212, 0226, 0230, 0234, 0235, 0236, 0297,
0450, 0513, 0543, 0572, 0606, 0617, 0669, 0701, 0729, 0783,
0876, 1448, 1609, 1669, 1721, 2130
```

### own_mut Cases (2)
```
0006, 1299
```

### Optional Flow Cases (66)
```
0004, 0005, 0015, 0016, 0020, 0028, 0043, 0053, 0057, 0062,
0063, 0064, 0072, 0097, 0118, 0120, 0121, 0122, 0125, 0127,
0134, 0135, 0139, 0150, 0152, 0153, 0169, 0215, 0221, 0238,
0242, 0269, 0278, 0286, 0287, 0290, 0297, 0300, 0309, 0322,
0329, 0338, 0349, 0377, 0383, 0402, 0410, 0416, 0441, 0442,
0448, 0452, 0459, 0463, 0473, 0496, 0502, 0516, 0518, 0540,
0554, 0567, 0605, 0658, 0682, 0746, 0752, 0785, 0802, 0853,
0875, 0881, 0918, 0929, 0948, 0953, 0977, 1011, 1074, 1220,
1260, 1343, 1345, 1396, 1423, 1461, 1475, 1498, 1514, 1584,
1631, 1642, 1658, 1700, 1800, 1834, 1838, 1930, 1958, 1980,
1984, 1985, 2002, 2013, 2017, 2215, 2300, 2348, 2405, 2616,
2971
```

### Container Literal Cases (22)
```
0010, 0017, 0039, 0047, 0076, 0078, 0084, 0090, 0091, 0092,
0106, 0118, 0127, 0130, 0207, 0210, 0213, 0221, 0232, 0271,
0309, 0321
```

### Destructuring Cases (28)
```
0012, 0027, 0056, 0075, 0146, 0189, 0280, 0283, 0295, 0323,
0344, 0355, 0463, 0516, 0684, 0703, 0707, 0787, 0864, 0909,
0929, 0997, 1288, 1383, 1462, 1466, 2001, 2101
```

---

## Conclusion

The 411-problem corpus analysis confirms that:

1. **Two broad phases are sufficient:** `ad-hoc-full-recursive-type-feature` (56 cases) and `ad-hoc-own-mut-parameter-convention` (2 cases) cover the structural language gaps.

2. **The remaining ~300 failures are ordinary closure work:** optional flow, container literals, destructuring, stdlib gaps—all addressable through existing Phase 31 milestones or normal compiler development.

3. **No widening needed:** Both broad phases have the correct scope based on full corpus validation.

4. **No narrowing needed:** Both broad phases address real language gaps, not narrow LeetCode workarounds.

The Phase 31 carry-forward plan remains the correct strategy.
