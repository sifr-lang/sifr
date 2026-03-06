# Phase 26 Review: Type-System Soundness

## Overview

Phase 26 (merged March 7, 2026) consists of four milestones addressing type-system soundness:

| Milestone | PR | Focus |
|-----------|-----|-------|
| 26.1 | #891 | TypeVar constraint enforcement |
| 26.2 | #892 | Inheritance and mutable variance corrections |
| 26.3 | #893 | Optional arithmetic soundness |
| 26.4 | #894 | Protocol-bound strictness closure |

All tests pass and demos execute correctly.

---

## 26.1 TypeVar Constraint Enforcement

### Implementation Summary
- **Location**: `crates/sifr_hir/src/lower/mod.rs`, `crates/sifr_hir/src/lower/type_bounds.rs`
- **Mechanism**: TypeVar bounds/constraints captured in `type_param_bounds` HashMap during lowering, validated at call sites via `type_satisfies_bound()` and `type_satisfies_constraint()`

### Findings

**Correctly Implemented:**
- PEP 695 syntax (`def f[T](x: T)`) properly registers TypeVars
- `TypeVar("T", int, str)` constraints work correctly
- `TypeVar("T", bound=Comparable)` bounds work correctly
- Mixed declaration styles (module-level + inline) are combined

**Potential Edge Cases / Concerns:**

1. **TypeVar forwarding across scopes** (`type_bounds.rs:39-42`)
   - The `current_owner_typevar_specs` function relies on `ctx.current_owner` to identify which function owns the TypeVar
   - If `current_owner` is not set correctly in all call paths, constraints may not be validated
   - *Risk*: Low - tested in demo and e2e tests

2. **Constraint vs bound encoding** (`type_bounds.rs:45-50`)
   - Constraints use `encode_typevar_constraint()` prefix `__constraint__:` to differentiate from bounds
   - This is a string-based hack that works but is fragile
   - *Risk*: Low - internal implementation detail

3. **Missing: Multiple bounds on single TypeVar**
   - PEP 694 allows `T: A & B` (intersection bounds), but this isn't supported
   - Current syntax only handles single bounds
   - *Risk*: Medium - feature gap, not a regression

4. **Unconstrained TypeVars silently accept anything**
   - If a TypeVar has no bounds or constraints, `typevar_satisfies_spec` returns `false` at line 40-42
   - But concrete types bypass TypeVar checking entirely in `type_satisfies_bound` (line 94-97)
   - This means unconstrained TypeVars correctly accept any type
   - *Risk*: None - correct behavior

---

## 26.2 Inheritance and Mutable Variance Corrections

### Implementation Summary
- **Location**: `crates/sifr_hir/src/lower/classes.rs`, `crates/sifr_type_system/src/types.rs`
- **Mechanism**: Inheritance chain stored as pipe-separated string (`"Parent|Grandparent"`), mutable collections (list, set, dict) are invariant via `contains_any` escape hatch in `is_assignable_to`

### Findings

**Correctly Implemented:**
- Transitive inheritance works: `Leaf → Mid → Base` chain is correctly validated
- Invariance enforced: `list[int]` not assignable to `list[int | str]`
- `contains_any` escape hatch allows explicit `Any` in collection types

**Potential Edge Cases / Concerns:**

1. **Inheritance chain format** (`classes.rs:175-180`)
   ```rust
   parent_class_chain = Some(if let Some(chain) = parent_parent_chain {
       format!("{}|{}", parent_name, chain)
   } else {
       parent_name.to_string()
   });
   ```
   - Pipe-separated format could break with unusual class names
   - *Risk*: Low - class names are validated elsewhere

2. **Missing: Covariant read-only containers**
   - No distinction between `list[T]` (mutable) and read-only views
   - This is a design limitation, not a regression
   - *Risk*: Low - known limitation

3. **Variance in nested collections** (`types.rs:628-632`)
   - `list[list[int]]` vs `list[list[int | str]]` - each level independently checked
   - Correctly enforces invariance at each level
   - *Risk*: None

4. **Protocol vs class variance**
   - Classes have nominal typing with inheritance
   - Protocols have structural typing
   - No variance annotations exist for either
   - *Risk*: Low - consistent with current design

---

## 26.3 Optional Arithmetic Soundness

### Implementation Summary
- **Location**: `crates/sifr_type_system/src/check.rs`
- **Mechanism**: Binary operations reject union types containing `None` without explicit narrowing

### Findings

**Correctly Implemented:**
- `int | None + int` correctly rejected
- Explicit narrowing via `if x is None: return ...` works correctly
- Demo shows both positive (safe_add_one with narrowing) and negative (unsafe_add_one without narrowing) paths

**Potential Edge Cases / Concerns:**

1. **Control flow narrowing scope** (`check.rs:503-509`)
   - Tests only cover simple if-branches
   - Narrowing might not persist across complex control flow (loops, multiple branches)
   - *Risk*: Medium - needs verification with complex examples
   ```python
   # Unclear if this works:
   if x is None:
       return
   x + 1  # Is x narrowed here?
   ```

2. **Pattern matching narrowing**
   - Only `x is None` checks are tested
   - Other narrowing patterns (`match/case`, `isinstance`) not verified
   - *Risk*: Medium - feature gap

3. **Short-circuit operators**
   - `x or 0 + 1` - does the `or` correctly narrow?
   - Not tested
   - *Risk*: Medium - potential edge case

4. **Union with multiple non-None types**
   - `int | str + int` - should this also be rejected?
   - Currently only checks for `None` presence
   - *Risk*: Low - current behavior is conservative

---

## 26.4 Protocol-Bound Strictness Closure

### Implementation Summary
- **Location**: `crates/sifr_hir/src/lower/type_bounds.rs`, `crates/sifr_hir/src/lower/expressions.rs`
- **Mechanism**: Protocol bounds validated strictly at call sites, forwarding TypeVars must satisfy target protocol

### Findings

**Correctly Implemented:**
- Unknown protocols rejected: `def f[T: MissingBound](x: T)` fails if `MissingBound` doesn't exist
- Protocol forwarding works: `relay_comparable(x)` where `U: Comparable` correctly validates `U` satisfies `Comparable`
- Built-in protocols (Comparable, Addable, Hashable) hardcoded at `type_bounds.rs:99-115`

**Potential Edge Cases / Concerns:**

1. **Protocol inheritance** (`type_bounds.rs:76-82`)
   - Code checks if source bound satisfies target bound via `type_satisfies_bound`
   - But this doesn't handle protocol inheritance (Protocol A that inherits Protocol B)
   - *Risk*: Medium - protocol inheritance not explicitly tested

2. **Self-referential protocol bounds**
   - What happens with `class Foo(Protocol): def bar(self: Foo) -> int`?
   - Not tested
   - *Risk*: Low - rare edge case

3. **Protocol method variance**
   - Only checks method existence, not method variance
   - Methods in protocols are structurally matched
   - *Risk*: Low - current design

4. **Multiple protocol bounds**
   - Not supported (related to 26.1 multiple bounds issue)
   - *Risk*: Medium - feature gap

---

## Summary of Potential Issues

### Regressions Found: None
All tests pass, and the demos work correctly.

### Missing Edge Cases:

| Area | Edge Case | Severity | Notes |
|------|-----------|----------|-------|
| 26.1 | Multiple bounds (`T: A & B`) | Medium | Feature gap, not regression |
| 26.3 | Complex control flow narrowing | Medium | Needs verification |
| 26.3 | Short-circuit operator narrowing | Medium | Untested |
| 26.4 | Protocol inheritance | Medium | Untested |
| 26.4 | Multiple protocol bounds | Medium | Feature gap |

### Unsound Behavior: None Detected

The implementation correctly enforces:
- TypeVar constraints at call sites
- Invariance for mutable collections
- Rejection of optional arithmetic without narrowing
- Strict protocol bound validation

### Recommendations:

1. **Add tests for control flow narrowing** (26.3)
   ```python
   def test_narrowing_in_loop(x: int | None) -> int:
       if x is None:
           return 0
       for i in range(x):  # Is x narrowed here?
           pass
   ```

2. **Add tests for protocol inheritance** (26.4)
   ```python
   protocol A: ...
   protocol B(A): ...  # Does B inherit from A?
   def f[T: A](x: T): ...
   ```

3. **Consider adding covariant read-only container types** (26.2)
   - Current design is safe but limiting

4. **Document the `Any` escape hatch** (26.2)
   - The `contains_any` mechanism is powerful but undocumented

---

## Test Coverage Assessment

**Unit Tests**: Comprehensive (`sifr_type_system` 73 tests, `sifr_hir` 42 tests)

**E2E Tests**: Positive and negative paths for each milestone

**Gaps**:
- Complex control flow scenarios
- Protocol inheritance scenarios
- Multiple bounds scenarios

---

## Conclusion

Phase 26 successfully closes several type-system soundness gaps. The implementation is sound for the tested scenarios. The main areas requiring future attention are complex control flow narrowing and protocol inheritance, which are feature gaps rather than regressions.
