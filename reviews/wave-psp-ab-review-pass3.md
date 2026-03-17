# Wave PSP-A/B Review Pass 3: Implementation Quality and CPython Parity Assessment

**Reviewer:** Claude Code
**Date:** 2026-03-16
**Waves Reviewed:** `wave_psp_a1`, `wave_psp_a2`, `wave_psp_b1`, `wave_psp_b2`
**Status:** Actionable findings identified

---

## Executive Summary

This review assesses CPython test porting/adaptation quality, positive/negative coverage sufficiency, and identifies parity gaps lacking executable evidence. Four key issues were identified across the four waves.

| Wave | Status | Critical Issues |
|------|--------|-----------------|
| `wave_psp_a1` | ⚠️ Needs Fix | range() keyword handling lacks negative test |
| `wave_psp_a2` | ✅ Acceptable | Good coverage |
| `wave_psp_b1` | ✅ Acceptable | Review files missing from main repo |
| `wave_psp_b2` | ⚠️ Needs Fix | No fail tests - missing negative coverage |

---

## Wave PSP-A1: Builtin Constructors and Callable Surface

### Issue 1: range() Keyword Arguments - Missing Negative Test (HIGH)

**Finding:** The `range()` implementation accepts keyword arguments (`start=`, `stop=`, `step=`) when CPython rejects them entirely. While documented as "adapted" in the traceability matrix, there is **no fail test** verifying the expected behavior.

**Evidence:**

```python
# CPython behavior:
>>> list(range(10, stop=20))
TypeError: range() takes no keyword arguments

# Sifr current behavior (permissive):
>>> list(range(10, stop=20))
[10, 11, 12, 13, 14, 15, 16, 17, 18, 19]  # Silent acceptance!
```

**Current Test Coverage:**
- ✅ Pass test uses keyword arguments: `list(range(start=1, stop=7, step=2))` (line 48 of `phase_psp_a1_builtin_callable_surface.sifr`)
- ✅ Demo uses keyword arguments: `list(range(start=2, stop=9, step=3))` (line 31 of `wave_psp_a1_builtin_callable_surface_demo.sifr`)
- ⚠️ Fail test only covers duplicate positional+keyword: `phase_psp_a1_range_duplicate_stop_keyword.sifr`
- ❌ **Missing:** Fail test for single keyword argument rejection (e.g., `range(stop=10)`)

**Traceability Matrix Claim:**
| Surface | State | Local Evidence |
|---------|-------|-----------------|
| `range(start=..., stop=..., step=...)` | adapted | pass test uses keywords |

The matrix labels this as "adapted" but provides no negative test confirming the adaptation behavior is intentional error rejection vs. silent acceptance.

**Recommended Action:**
Add fail test:
```sifr
# crates/sifr/tests/e2e/fail/phase_psp_a1_range_keyword_not_supported.sifr
# expect-error: range() does not accept keyword arguments

def main():
    print(list(range(stop=10)))
```

---

### Issue 2: Documentation Inconsistency (LOW)

**Finding:** The issue execution ledger (`issues/ad-hoc-python-source-parity-and-builtin-stdlib-surface-execution.md` lines 469-472) claims review pass 2 validated the range keyword bug as "non-actionable" because it "conflicts with the wave's documented adapted parity contract."

**Analysis:**
- The traceability matrix labels `range(start=..., stop=..., step=...)` as "adapted"
- However, "adapted" in the parity framework means "modified from CPython with documented rationale"
- There's no documented rationale for why Sifr should accept keywords that CPython rejects
- The pass test exercises this behavior, creating a behavioral expectation

**Recommended Action:**
Either:
1. Add explicit rationale in traceability: "Sifr accepts keywords for developer ergonomics; CPython rejected for historical reasons"
2. Or fix the implementation to reject keywords and update pass test

---

## Wave PSP-A2: Core Object Models and Builtin Semantics

### Assessment: ✅ Acceptable

**Positive Coverage:**
- Pass test: `phase_psp_a2_core_object_model_surface.sifr`
- 6 fail tests covering:
  - Unexpected keyword rejection
  - Invalid iterable for dict.update
  - Duplicate default argument
  - Non-iterable for set.update
  - Invalid count type for str.replace
  - Invalid bound type for tuple.index

**Quality Observations:**
- Adaptations clearly documented in traceability matrix
- All fail tests correctly detect errors
- No executable evidence gaps identified

---

## Wave PSP-B1: Collections Objects and Ordered Helpers

### Issue 3: Review Files Not in Main Repository (MEDIUM)

**Finding:** The issue execution ledger references review files at:
- `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b1-review-pass1.md`
- `/Users/yaseralnajjar/.codex/worktrees/0761/codebase/reviews/wave-psp-b1-review-pass2.md`

These files do not exist in the main repository's `reviews/` directory.

**Current State in Main Repo:**
- ✅ Pass test: `phase_psp_b1_collections_ordered_helpers.sifr`
- ✅ 3 fail tests:
  - `phase_psp_b1_bisect_key_unsupported.sifr`
  - `phase_psp_b1_counter_iterable_constructor_unsupported.sifr`
  - `phase_psp_b1_deque_index_invalid_bound.sifr`
- ✅ Demo: `wave_psp_b1_collections_ordered_helpers_demo.sifr`
- ✅ Traceability: `wave_psp_b1_cpython_traceability.md`
- ❌ No review files in `/reviews/`

**Recommended Action:**
Either:
1. Copy review files to main repo's `reviews/` directory, or
2. Update issue execution ledger to remove non-existent file references

---

## Wave PSP-B2: Iterators, Functional Helpers, and Randomness

### Issue 4: No Fail Tests - Missing Negative Coverage (HIGH)

**Finding:** `wave_psp_b2` has **zero fail tests**, unlike all other waves which have at least 1.

**Current State:**
- ✅ Pass test: `phase_psp_b2_iterators_functional_randomness.sifr`
- ❌ **No fail tests**
- ✅ Demo: `wave_psp_b2_iterators_functional_randomness_demo.sifr`
- ✅ Traceability: `wave_psp_b2_cpython_traceability.md`

**Traceability Matrix Shows Several Error Cases That Should Have Tests:**

| Surface | CPython Behavior | Sifr Adaptation | Missing Test? |
|---------|------------------|-----------------|----------------|
| `chain()` with wrong arg type | TypeError | Compile-time rejection | ❌ |
| `islice()` with invalid bounds | ValueError | Compile-time rejection | ❌ |
| `product()` with negative repeat | ValueError | Should error | ❌ |
| `shuffle()` on non-list | TypeError | Compile-time rejection | ❌ |
| `randrange()` with invalid args | ValueError | Should error | ❌ |
| `choice()` on empty | IndexError | ValueError result | ❌ |

**Recommended Action:**
Add fail tests for:
1. `chain()` with non-iterable argument
2. `islice()` with negative start/stop
3. `product()` with negative repeat
4. `shuffle()` on non-list argument
5. `randrange()` with invalid step (zero)

Example:
```sifr
# crates/sifr/tests/e2e/fail/phase_psp_b2_chain_non_iterable.sifr
# expect-error: chain() arguments must be iterables

def main():
    from sifr.itertools import chain
    print(chain(42))
```

---

## Summary of Actionable Findings

| # | Wave | Severity | Finding | Action Required |
|---|------|----------|---------|-----------------|
| 1 | a1 | HIGH | range() missing fail test for keyword rejection | Add fail test |
| 2 | a1 | LOW | Documentation inconsistency on range adaptation | Clarify rationale or fix |
| 3 | b1 | MEDIUM | Review files not in main repo | Add to reviews/ or update issue |
| 4 | b2 | HIGH | No fail tests - missing negative coverage | Add 4-5 fail tests |

---

## Verification Commands

```bash
# Verify pass tests compile
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_a1_builtin_callable_surface.sifr
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_b1_collections_ordered_helpers.sifr
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/phase_psp_b2_iterators_functional_randomness.sifr

# Verify fail tests detect errors
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a1_*

# Run quick validation
scripts/run_all_tests.sh --profile quick
```

---

## Conclusion

The waves show good overall implementation quality, but two critical gaps require attention:

1. **wave_psp_a1**: Need negative test for range keyword rejection
2. **wave_psp_b2**: Need fail tests for error conditions

These are executable evidence gaps that should be addressed to ensure the claimed parity has proper test coverage.
