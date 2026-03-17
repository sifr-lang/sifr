# Wave PSP-A2 Review: Gap Analysis and CPython Test Parity Quality (R2)

**Reviewer:** Claude Code
**Date:** 2026-03-17
**Wave:** `wave_psp_a2` (Core Object Models: list/dict/set/tuple/str)
**Branch:** `main` (worktree: 0761)
**Status:** Second-Pass Production-Grade Review

---

## Executive Summary

Wave PSP-A2 implements core object model method argument normalization and builtin semantics for `list`, `dict`, `set`, `tuple`, and `str`. The implementation is functional for the tests that exist, but there is a significant bug related to local variable mutability for dict and set mutating methods.

---

## 1. Implementation Gaps (Actionable)

### 1.1 Critical Bug: Local Variable Mutability for Dict/Set Methods

| Method | Type | Status | Notes |
|--------|------|--------|-------|
| `dict.setdefault(key, default)` | dict | **BUG** | Works only if another mutating method (like `update`) is called first. Fails with "cannot borrow as mutable" when used as the first mutating operation on a local variable. |
| `set.intersection_update(iterable)` | set | **BUG** | Same issue as setdefault - fails when used as first mutating operation. |
| `set.difference_update(iterable)` | set | **BUG** | Same issue. |
| `set.symmetric_difference_update(iterable)` | set | **BUG** | Same issue. |

**Root Cause Analysis:**

The HIR correctly identifies these as mutating methods (see `mutating_methods.rs:52-63`), but the check only applies to **parameter bindings**, not **local variable bindings**. When a local variable is used with these methods, the generated Rust code doesn't include `mut` on the variable declaration, causing a compile error.

**Proof of Bug:**

```sifr
# This fails - setdefault as first mutating operation
def main():
    data: dict[str, int] = {"x": 1}
    assert data.setdefault("d", 8) == 8  # Compile error: cannot borrow as mutable

# This works - update called first
def main():
    data: dict[str, int] = {"x": 1}
    data.update(a=2)  # Sets variable to mutable
    data.setdefault("d", 8)  # Works because data is already mutable
```

The test file `phase_psp_a2_core_object_model_surface.sifr` passes only because it calls `data.update(a=2)` before `data.setdefault("d", 8)`, inadvertently making the variable mutable.

### 1.2 List Methods Work Correctly

| Method | Type | Status | Notes |
|--------|------|--------|-------|
| `list.pop(index)` | list | ✅ Works | Correctly generates mutable variable |
| `list.index(value, start, stop)` | list | ✅ Works | Returns `T \| None` instead of raising |
| `list.extend(iterable)` | list | ✅ Works | |
| Other list methods | list | ✅ Works | |

---

## 2. CPython Parity Assessment

### 2.1 Keyword Argument Normalization

| Method | Keywords | Sifr Status | CPython Status |
|--------|----------|-------------|----------------|
| `list.index` | `start=`, `stop=` | ✅ Complete | CPython uses positional-only |
| `list.pop` | N/A | ✅ Acceptable | Positional-only in CPython |
| `dict.get` | `default=` | ✅ Complete | CPython uses positional-only |
| `dict.pop` | `default=` | ✅ Complete | CPython uses positional-only |
| `dict.update` | `**kwargs` | ✅ Complete | CPython also supports kwargs |
| `dict.setdefault` | N/A | ⚠️ Bug | Works but has mutability bug |
| `tuple.index` | `start=`, `stop=` | ✅ Complete | CPython uses positional-only |
| `str.split` | `sep=`, `maxsplit=` | ✅ Complete | Same as CPython |
| `str.replace` | `count=` | ✅ Complete | Same as CPython |
| `set.update` | (variadic) | ✅ Complete | Works but has mutability bug |
| `set.intersection` | (variadic) | ✅ Complete | |
| `set.intersection_update` | (variadic) | ⚠️ Bug | Works but has mutability bug |
| `set.difference_update` | (variadic) | ⚠️ Bug | Works but has mutability bug |
| `set.symmetric_difference_update` | (variadic) | ⚠️ Bug | Works but has mutability bug |

### 2.2 Return Type Adaptations (Documented)

| Method | CPython Behavior | Sifr Behavior | Notes |
|--------|------------------|----------------|-------|
| `list.index` | Raises `ValueError` if not found | Returns `T \| None` | Safe return pattern |
| `tuple.index` | Raises `ValueError` if not found | Returns `T \| None` | Safe return pattern |
| `dict.get` | Returns `None` if missing | Returns `T \| None` | Same |
| `dict.pop` | Raises `KeyError` if missing and no default | Returns `T \| None` if default provided | Safe return |

---

## 3. Test Coverage Assessment

### 3.1 Test Files

| Test File | Type | Status |
|-----------|------|--------|
| `phase_psp_a2_core_object_model_surface.sifr` | Pass | ✅ Running (but works due to workaround) |
| `cpython_core_object_model_subset.sifr` | Pass | ✅ Running |
| `phase_psp_a2_list_unexpected_keyword.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_dict_update_invalid_pairs.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_dict_get_duplicate_default.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_dict_setdefault_invalid_default.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_set_update_non_iterable.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_str_replace_invalid_count.sifr` | Fail | ✅ Detects error |
| `phase_psp_a2_tuple_index_invalid_bound.sifr` | Fail | ✅ Detects error |

### 3.2 Demo Validation

```
$ cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr
["core", "x", "y"]
7
true
2
2
["alpha", "beta,gamma"]
bbaa
```

✅ Demo runs successfully.

---

## 4. Key Findings

### 4.1 The Bug in Detail

The `reject_immutable_parameter_method_mutation` function in `mutating_methods.rs` only checks for **parameter bindings**:

```rust
if ctx
    .scope
    .lookup(name)
    .is_some_and(|info| info.is_parameter_binding() && !info.is_mutable_binding)
{
    ctx.error(format!(
        "cannot mutate through immutable parameter `{name}`: add `mut` to the parameter declaration"
    ));
    return true;
}
```

This check does NOT apply to local variable bindings. When a local variable (like `let x: dict[str, int] = ...`) is used with a mutating method, the code compiles but generates invalid Rust (missing `mut` on the variable).

### 4.2 Why Existing Tests Pass

The test file `phase_psp_a2_core_object_model_surface.sifr` passes because it calls `data.update(a=2)` before `data.setdefault("d", 8)`. The `update` method happens to generate code that works with non-mutable variables (using `extend`), which causes the variable to be declared as `mut` in the generated Rust. By the time `setdefault` is called, the variable is already mutable.

This is a coincidence, not a fix.

---

## 5. Actionable Recommendations

### High Priority

1. **Fix local variable mutability for dict/set mutating methods**
   - Extend `reject_immutable_parameter_method_mutation` to also check local variable bindings
   - OR ensure codegen always generates `mut` for variables used with these methods
   - Affects: `dict.setdefault`, `set.intersection_update`, `set.difference_update`, `set.symmetric_difference_update`

### Test Cases to Add

```sifr
# Should compile and run correctly
def main():
    data: dict[str, int] = {"a": 1}
    result: int = data.setdefault("b", 2)  # Currently fails
    assert result == 2

def main():
    seen: set[int] = {1}
    seen.intersection_update([2, 3])  # Currently fails
    assert seen == {1}
```

---

## 6. Verification Commands

```bash
# Run pass tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_a2_core_object_model_surface.sifr
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/cpython_core_object_model_subset.sifr

# Run fail tests
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_a2_*.sifr

# Demo
cargo run -q -p sifr -- run demos/wave_psp_a2_core_object_models_demo.sifr
```

---

## 7. Conclusion

Wave PSP-A2 has made significant progress in implementing core object model parity, but there is a critical bug in how local variable mutability is handled for dict and set mutating methods. The existing tests pass only because of a fortunate ordering of method calls that happens to make the variables mutable.

**Key Issue:** Local variables used with `dict.setdefault` or set `_update` methods must either:
1. Be declared with `mut`, OR
2. Have another mutating method called before them

This is a significant usability issue and should be fixed before considering the wave complete.
