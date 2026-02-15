---
name: LeetCode Failure Analysis
overview: Comprehensive classification of all 367 LeetCode audit failures into three categories -- truly by-design (will never change), planned improvements (not yet implemented but on the roadmap), and actual bugs (unexpected compiler deficiencies).
todos: []
isProject: false
---

# LeetCode Failure Analysis: By-Design vs. Planned vs. Bugs

## Summary

Out of 411 LeetCode problems: **44 pass**, 330 fail at Sifr compile, 36 fail at Rust compile, 1 fails at runtime.

After careful analysis against Sifr's design principles and milestone roadmap:

- **Truly by-design (will never change):** ~3 categories affecting a small core of problems
- **Planned improvements (on the roadmap, not yet implemented):** ~12 categories, already have milestones
- **Actual bugs (unexpected deficiencies):** ~6 categories in codegen

The key insight: **most failures are NOT "by-design"** -- they are features that Sifr explicitly plans to support but hasn't implemented yet. Only a handful of failures reflect genuine, permanent design divergences from Python.

---

## CATEGORY A: TRULY BY-DESIGN (Will Never Change)

These are permanent Sifr design decisions documented in the "Python Divergences" table. Code that relies on these Python behaviors will **always** need to be rewritten for Sifr.

### A1. Mandatory Type Annotations on All Parameters

- **Error**: `parameter 'x' in function 'foo' is missing a type annotation`
- **Problems affected**: ~118
- **Design principle**: Sifr enforces static typing on every function parameter. This is a core language invariant, like TypeScript's strict mode.
- **Will it ever change?** No. This is fundamental to Sifr's compile-time safety guarantee. The plan explicitly states: "Types are strict with an opt-in `Any` escape hatch."
- **What Sifr code looks like**: `def foo(x: int, y: str) -> bool:` -- always annotated.

### A2. Move Semantics / Use After Move

- **Error**: `use of moved value: 'nums'`
- **Problems affected**: ~9
- **Design principle**: Sifr follows Rust's ownership model. Values are moved by default; using a value after it has been moved is a compile error.
- **Will it ever change?** No. Ownership is a core Sifr guarantee. However, `milestone_ownership_v2` will add **auto-borrow** for common patterns (e.g., `print(x)` won't consume `x`), which will reduce false positives. The fundamental move semantics remain.

### A3. Safe Indexing Returns `Option[T]` / Division Returns `Result`

- **Errors**: `type mismatch: expected 'int', got 'int | None'`, `Result[float, str]` in arithmetic
- **Problems affected**: ~12 (pure division issues), many more cascading
- **Design principle**: `x[i]` returns `Option[T]`, `a / b` returns `Result[float, str]`. This is the "if it compiles, it works" guarantee -- no runtime panics from bad indices or division by zero.
- **Will it ever change?** No. This is documented in the Safety Philosophy and Python Divergences table. Code must explicitly handle `None`/`Err` cases.
- **Note**: `milestone_union_ops` will make it **easier** to work with `T | None` values (arithmetic on optionals, better narrowing), but the fundamental requirement to handle the `None` case remains.

---

## CATEGORY B: PLANNED IMPROVEMENTS (On the Roadmap, Not Yet Implemented)

These are features Sifr **intends to support** but hasn't built yet. They have explicit milestones in the roadmap. These are NOT bugs and NOT by-design rejections -- they're the "not yet" category.

### B1. Nested Functions / Closures (~100+ problems)

- **Error pattern**: Inner functions (`dfs`, `backtrack`, `helper`) can't access outer variables; show as `undefined variable`
- **Planned milestone**: `milestone_nested_functions` (M11) -- "Lower def-inside-def to Rust closures/inner fns, capture outer variables, recursive inner fns"
- **Roadmap note**: This is explicitly called out as "the single biggest blocker (~200 LeetCode problems)" in the milestone ordering rationale.
- **Status**: Pending. First milestone in Language Hardening Phase 2.

### B2. Forward References for `ListNode` / `TreeNode` (~51 problems)

- **Errors**: `unknown type: 'ListNode'` (49), `unknown type: 'TreeNode'` (43)
- **Planned milestone**: `milestone_forward_refs` (M12) -- "Two-pass class registration for forward type references (ListNode, TreeNode, Node)"
- **Also**: `milestone_recursive_types` -- "ListNode, TreeNode, Box for self-referential"
- **Note**: The audit files DO include class definitions for these types, but the compiler can't handle self-referential types yet (e.g., `class ListNode: next: ListNode | None`). This is NOT just a test artifact -- it's a missing compiler feature.

### B3. `set()` Constructor (~36 problems)

- **Error**: `undefined function: 'set'`
- **Planned milestone**: `milestone_stdlib_hardening` -- "set() type, import aliases, math/json/io/env gaps"
- **Also**: `milestone_ext_collections` plans full `Set[T]` with operations
- **Status**: `Set[T]` exists as a type but `set()` constructor isn't wired up yet.

### B4. `collections` Module: `defaultdict`, `deque`, `Counter` (~37 problems)

- **Errors**: `undefined function: 'defaultdict'`, `undefined function: 'deque'`, `undefined function: 'Counter'`, `undefined variable: 'collections'`
- **Planned milestone**: `milestone_ext_collections` -- explicitly plans `Counter[T]`, `defaultdict[K, V]`, and `Deque` with full APIs.
- **Note**: Sifr won't use Python's `import collections` syntax. These will be built-in types or part of `sifr.collections`. But the **functionality** is planned.

### B5. `heapq` Module (~19 problems)

- **Error**: `undefined variable: 'heapq'`
- **Planned**: Not explicitly a named milestone, but falls under stdlib expansion. Sifr would likely provide a `Heap[T]` built-in type rather than Python's function-based `heapq` module.
- **Status**: Not yet planned with a specific milestone. This is a gap in the roadmap.

### B6. Attribute Access on Class Instances (`.next`, `.val`, `.left`) (~34 problems)

- **Error**: `attribute access '.next' is not supported as an expression`
- **Planned milestone**: `milestone_classes` -- "struct + impl, __init__, methods, auto-derive"
- **Also**: `milestone_subscript_mutation` covers `self.field` access patterns
- **Note**: Field access is fundamental OOP and IS planned. The current compiler only supports method calls, not field reads.

### B7. Class Field Initialization in `__init__` (~24 problems)

- **Error**: `type 'MyClass' has no field 'stack'`
- **Planned milestone**: `milestone_classes` -- fields assigned via `self.field = value` in `__init__` should be registered.
- **Status**: The class system exists but field tracking from `__init__` is incomplete.

### B8. Subscript Assignment (`a[i] = x`, `a[i] += 1`) (~6 problems)

- **Error**: `augmented subscript assignment target must be a simple name`
- **Planned milestone**: `milestone_subscript_v2` (M15) -- "Nested subscript assign, &mut self, mutability"
- **Also**: `milestone_subscript_mutation` -- "list[i]=val, dict[key]=val, self.field += 1"

### B9. Tuple Unpacking in Complex Targets (`a, b = b, a`) (~11 problems)

- **Error**: `tuple unpacking target must be a simple name`
- **Planned milestone**: `milestone_comprehension_v2` (M16) -- "tuple unpacking in for/comprehension"
- **Also**: `milestone_iteration_v2` -- "tuple unpack in for"

### B10. Missing Built-in Functions: `ord()`, `chr()`, `min(a,b,c)` (~8+ problems)

- **Errors**: `undefined function: 'ord'`, `undefined function: 'chr'`, `min() takes 1 or 2 arguments`
- **Planned milestone**: `milestone_builtins_v2` -- "max/min 2-arg, range 3-arg, mixed arithmetic"
- **Note**: `ord()` and `chr()` aren't explicitly mentioned in any milestone. This is a **gap** -- they should be added to `milestone_builtins_v2` or `milestone_ext_stdlib`.

### B11. String Iteration and Wider Iterable Support (~12 problems)

- **Errors**: `cannot iterate over type 'str'`, `enumerate() argument must be a list, got 'str'`, `reversed() argument must be a list, got 'range'`
- **Planned milestone**: `milestone_iteration_v2` -- "String/dict iteration, tuple unpack in for"
- **Status**: Strings should be iterable (yielding characters). This is explicitly planned.

### B12. `list.pop(index)` and `.copy()` (~4+ problems)

- **Errors**: `list.pop() takes no arguments`, `type 'Any' has no method 'copy'`
- **Planned**: Falls under `milestone_ergonomics` or `milestone_union_ops` -- "list.remove, list+list concat"
- **Note**: `list.pop(i)` is a standard Python operation. `.copy()` should be supported for all collection types.

---

## CATEGORY C: ACTUAL BUGS (Codegen Deficiencies)

These 36 problems **pass Sifr compilation** but generate invalid Rust code. These are unambiguously compiler bugs -- the type system accepted the code but codegen produced something Rust rejects.

### C1. Missing `mut` on Reassigned Variables (~11 problems)

- **Rust error**: `error[E0384]: cannot assign twice to immutable variable`
- **Root cause**: Codegen emits `let x = ...` but the variable is later reassigned in a loop body (e.g., binary search `l = mid + 1`). Should emit `let mut x`.
- **Severity**: High. Affects all binary search and two-pointer problems.
- **Fix**: Improve mutability analysis to detect reassignment in loop bodies and conditional branches.

### C2. `list * int` Emitted as `Vec * i64` (~5 problems)

- **Rust error**: `error[E0369]: cannot multiply Vec<i64> by i64`
- **Root cause**: Python's `[0] * n` (list repetition) is emitted as Rust multiplication. Should emit `vec![0_i64; n as usize]`.
- **Severity**: Medium. Common pattern for initializing arrays.
- **Fix**: Detect `List * Int` pattern in codegen and emit `vec![elem; count]`.

### C3. `Option<i64>` Used in Arithmetic Without Unwrap (~5 problems)

- **Rust error**: `cannot add-assign Option<i64> to i64`
- **Root cause**: Safe indexing returns `Option<i64>`, but codegen doesn't unwrap before using in `+=` operations.
- **Severity**: Medium. The type system should either reject this at Sifr level or codegen should handle it.
- **Fix**: Either propagate `Option` type through the Sifr type system (so it's caught earlier) or add unwrap in codegen.

### C4. `i64` vs `usize` Type Mismatches (~15 problems)

- **Rust error**: `error[E0308]: mismatched types` (various)
- **Root cause**: Sifr uses `i64` for all integers, but Rust requires `usize` for array indexing. Codegen doesn't always insert `as usize` casts.
- **Severity**: High. Affects any problem that indexes arrays with computed values.
- **Fix**: Insert `as usize` casts at all array index positions in codegen.

### C5. Duplicate Function Names Not Rejected (~3 problems)

- **Rust error**: `error[E0428]: the name 'foo' is defined multiple times`
- **Root cause**: Some LeetCode files have multiple implementations of the same function. Sifr accepts all of them but Rust rejects duplicates.
- **Severity**: Low. Edge case from test file structure.
- **Fix**: Either reject duplicate function names at Sifr level or suffix them.

### C6. `Box<dyn Any>` Fallback Breaks Operations (~2 problems)

- **Rust error**: `the method 'join' exists for Vec<Box<(dyn Any + 'static)>> but its trait bounds were not satisfied`
- **Root cause**: When type inference falls back to `Any`, codegen emits `Box<dyn Any>` which doesn't support string operations.
- **Severity**: Low. Indicates type inference should have resolved a concrete type.
- **Fix**: Improve type inference to avoid `Any` fallback in these cases.

---

## Summary Table

| Category | ID | Description | Problems | Verdict | Milestone |
|----------|-----|-----------------------------------|----------|--------------------------|-------------------------------|
| **Truly by-design** | A1 | Mandatory type annotations | ~118 | Will never change | -- |
| | A2 | Move semantics | ~9 | Will never change | (improved by ownership_v2) |
| | A3 | Safe indexing / division | ~12+ | Will never change | (easier with union_ops) |
| **Planned improvements** | B1 | Nested functions / closures | ~100+ | Planned | `milestone_nested_functions` |
| | B2 | Forward refs (ListNode/TreeNode) | ~51 | Planned | `milestone_forward_refs` |
| | B3 | `set()` constructor | ~36 | Planned | `milestone_stdlib_hardening` |
| | B4 | defaultdict / deque / Counter | ~37 | Planned | `milestone_ext_collections` |
| | B5 | heapq / Heap type | ~19 | **GAP** -- no milestone | needs planning |
| | B6 | Attribute access on instances | ~34 | Planned | `milestone_classes` |
| | B7 | `__init__` field tracking | ~24 | Planned | `milestone_classes` |
| | B8 | Subscript assignment | ~6 | Planned | `milestone_subscript_v2` |
| | B9 | Tuple unpacking targets | ~11 | Planned | `milestone_comprehension_v2` |
| | B10 | Missing builtins (ord, chr) | ~8 | Partially planned | `milestone_builtins_v2` |
| | B11 | String iteration | ~12 | Planned | `milestone_iteration_v2` |
| | B12 | list.pop(index), .copy() | ~4 | Planned | `milestone_ergonomics` |
| **Codegen bugs** | C1 | Missing `mut` on reassignment | ~11 | **BUG** | needs fix |
| | C2 | List repetition (`[0]*n`) | ~5 | **BUG** | needs fix |
| | C3 | Option in arithmetic | ~5 | **BUG** | needs fix |
| | C4 | i64 vs usize mismatches | ~15 | **BUG** | needs fix |
| | C5 | Duplicate function names | ~3 | **BUG** | needs fix |
| | C6 | Box dyn Any fallback | ~2 | **BUG** | needs fix |

---

## Gaps in the Roadmap

Two items surfaced that don't have explicit milestones:

1. **Heap / Priority Queue type** (~19 problems): Python's `heapq` module is used heavily in LeetCode. Sifr should plan a `Heap[T]` built-in type or include it in `milestone_ext_collections`.

2. **`ord()` and `chr()` built-ins** (~5 problems): These basic character/integer conversion functions aren't mentioned in any milestone. Should be added to `milestone_builtins_v2`.

---

## Estimated Impact

| Scenario | Pass Rate |
|----------|-----------|
| Current | 44 / 411 (10.7%) |
| Fix codegen bugs only (C1-C6) | ~80 / 411 (19.5%) |
| + Implement nested functions (B1) | ~150 / 411 (36.5%) |
| + Forward refs + classes (B2, B6, B7) | ~190 / 411 (46.2%) |
| + All planned improvements (B1-B12) | ~250 / 411 (60.8%) |
| Theoretical max (by-design failures remain) | ~280 / 411 (68.1%) |

The remaining ~130 problems would still fail due to mandatory type annotations (A1), move semantics (A2), and safe indexing (A3) -- all permanent Sifr design choices.
