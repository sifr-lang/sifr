# Language Hardening

This phase addresses systemic gaps exposed by auditing 396 LeetCode problems and 8 feature audits against the compiler after Phases 1–3. It fixes codegen bugs, narrowing edge cases, ownership tracking, iteration, builtins, syntax gaps, recursive types, inference, stdlib holes, and then a second round of fixes for nested functions, forward references, generics, comprehensions, union operations, and remaining Phase 2/3 bugs. By the end of this phase, the core language compiles ~60% of LeetCode problems and is stable enough to change the default parameter passing convention.

---

## milestone_codegen_fixes: Codegen Fixes

status: completed

**Goal:** Fix codegen bugs in already-implemented features — tuple indexing, union returns, int/int division, print None, string escapes.

---

## milestone_narrowing_v2: Narrowing v2

status: completed

**Goal:** Fix elif chains, early-return narrowing, and-narrowing, and narrowing for 3+ union members.

---

## milestone_ownership_v2: Ownership v2

status: completed

**Goal:** Auto-borrow for print and stop consuming values unnecessarily.

---

## milestone_subscript_mutation: Subscript Mutation

status: completed

**Goal:** Support `list[i]=val`, `dict[key]=val`, and `self.field += 1` subscript mutation patterns.

---

## milestone_iteration_v2: Iteration v2

status: completed

**Goal:** Support string/dict iteration, tuple unpacking in for loops, and dict comprehensions.

---

## milestone_builtins_v2: Builtins v2

status: completed

**Goal:** Support max/min 2-arg forms, range 3-arg form, mixed arithmetic, and module-level variables.

---

## milestone_syntax_expansion: Syntax Expansion

status: completed

**Goal:** Support nested functions, closures, bitwise operators, and multi-target assignment.

---

## milestone_recursive_types: Recursive Types

status: completed

**Goal:** Support self-referential types (ListNode, TreeNode) using Box for heap allocation.

---

## milestone_inference_v2: Inference v2

status: completed

**Goal:** Support return type inference, parameter type inference, and Result unwrap patterns.

---

## milestone_stdlib_hardening: Stdlib Hardening

status: completed

**Goal:** Add set type, fix import aliases, and close math/json/io/env stdlib gaps.

---

## milestone_nested_functions: Nested Functions and Closures

status: completed

**Goal:** Lower `def` inside `def` to Rust closures or inner functions. This is the single biggest blocker -- 68 "unsupported statement type" + 128 "undefined function" errors in LeetCode. Blocks ~200 problems including DFS/BFS helpers, backtracking, and recursive algorithms.

Lambda closures work (from `milestone_generics`), but `def` inside `def` is not lowered. This is the most impactful fix possible.

### Implementation

1. **Lower `def` inside `def`** to Rust inner functions or closures in `lower_stmt` (currently has no case for `Stmt::FunctionDef` inside function bodies)
2. **Capture variables from outer scope** (read-only via clone, mutable via move or `RefCell`)
3. **Recursive inner functions** (e.g., `backtrack`, `dfs`, `helper` -- extremely common in LeetCode)
4. **Nested function parameters**: require type annotations (consistent with Sifr's design)

**Key files:** `crates/sifr_lowering/src/lower/` (add nested function lowering in `lower_stmt`), `crates/sifr_codegen/src/lib.rs` (emit closures/inner fns), `crates/sifr_lowering/src/scope.rs` (nested scope chains)

### Definition of Done (milestone_nested_functions)

- `def` inside `def` compiles and runs correctly
- Outer variable capture works (read-only and mutable)
- Recursive inner functions work (e.g., `dfs`, `backtrack`, `helper`)
- E2E pass tests for nested functions, closures, and recursive inner functions
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing nested functions with variable capture

---

## milestone_forward_refs: Forward Type References

status: completed

**Goal:** Two-pass class registration for forward type references. 87 "unknown type" errors in LeetCode from `ListNode`, `TreeNode`, `Node` used as parameter/return types before the class is defined.

### Implementation

1. **Two-pass class registration**: first pass collects all class names as placeholder types, second pass resolves field/param types
2. **Forward references in function parameter and return type annotations**
3. **`__init__` parameter type resolution** when class is defined later in the file

**Key files:** `crates/sifr_lowering/src/lower/` (two-pass `lower_module` -- currently single-pass)

### Definition of Done (milestone_forward_refs)

- Classes can reference types defined later in the same file
- Function parameters and return types can reference forward-declared classes
- `__init__` parameters resolve correctly for forward-declared types
- E2E pass tests for forward references (ListNode, TreeNode patterns)
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing forward type references

---

## milestone_narrowing_v3: Narrowing Fixes

status: completed

**Goal:** Fix equality narrowing over-narrowing to `Never` (23+6 errors in LeetCode), field access on narrowed union types (17 errors), and collection truthiness (15 errors).

### Implementation

1. **Equality narrowing**: After `if x == "GET":`, the `elif` branch should narrow to the remaining type, NOT `Never`. Fix `narrow_type` in `crates/sifr_type_system/src/narrow.rs`.
2. **Field access on narrowed types**: After `if isinstance(shape, Circle):`, allow `shape.radius`. The narrowed type should expose the class's fields.
3. **Comparison on union/optional types**: Allow `==`, `!=`, `<`, `>` between `T | None` and `T`.
4. **`not collection` truthiness**: `not list_var` should emit `list_var.is_empty()` (15 LeetCode errors).

**Key files:** `crates/sifr_type_system/src/narrow.rs`, `crates/sifr_lowering/src/lower/`, `crates/sifr_type_system/src/check.rs`

### Definition of Done (milestone_narrowing_v3)

- Equality narrowing does not over-narrow to `Never` in elif chains
- Field access works on narrowed union types after isinstance checks
- Comparison operators work on union/optional types
- Collection truthiness (`not list_var`) emits `.is_empty()`
- E2E pass tests for all narrowing fixes
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing narrowing improvements

---

## milestone_union_ops: Union Type Operations

status: completed

**Goal:** Support operations on union/optional types. 90+ errors in LeetCode from arithmetic, indexing, and `len()` on `T | None`.

### Implementation

1. **Arithmetic on `T | None`**: Auto-unwrap or require narrowing for `+`, `-`, `*` on optional types
2. **Indexing `list[T] | None`**: Allow indexing when the non-None member is indexable
3. **`len()` on union types**: Accept `list[T] | None` and similar (37 LeetCode errors)
4. **`dict.get(key, default)`**: Support the 2-arg form (13 errors). Emit `.get(&key).cloned().unwrap_or(default)`
5. **`list.remove(val)`**: Add the missing method
6. **`list + list` concatenation**: Support `+` operator for list types (29 LeetCode errors)
7. **`abs()`, `sum()`, `min()`, `max()` on union types**: Extend builtins to handle optional arguments

**Key files:** `crates/sifr_type_system/src/check.rs`, `crates/sifr_lowering/src/lower/`, `crates/sifr_codegen/src/lib.rs`

### Definition of Done (milestone_union_ops)

- Arithmetic, indexing, and `len()` work on union/optional types
- `dict.get(key, default)` 2-arg form works
- `list.remove(val)` works
- `list + list` concatenation works
- Builtins (`abs`, `sum`, `min`, `max`) handle optional arguments
- E2E pass tests for all union operations
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing union type operations

---

## milestone_subscript_v2: Subscript and Mutability Fixes

status: completed

**Goal:** Fix nested subscript assignment (14 errors), augmented subscript assignment (13 errors), `&mut self` for methods, variable mutability, and codegen type mismatches.

### Implementation

1. **`matrix[i][j] = val`**: Nested subscript assignment
2. **`result[i] += val`**: Subscript augmented assignment (some patterns still fail from M4)
3. **`&mut self` for methods**: Methods that mutate `self.field` need `&mut self` in Rust
4. **Variable mutability**: Reassigned variables not emitted as `mut` (3 Rust failures in LeetCode)
5. **Codegen type mismatches (i64 vs usize)**: 6 Rust failures from wrong integer types in indexing

**Key files:** `crates/sifr_codegen/src/lib.rs`, `crates/sifr_lowering/src/lower/`

### Definition of Done (milestone_subscript_v2)

- Nested subscript assignment (`matrix[i][j] = val`) works
- Augmented subscript assignment (`result[i] += val`) works
- Methods that mutate fields emit `&mut self`
- Reassigned variables correctly emitted as `mut`
- Integer type casts correct for indexing (i64 vs usize)
- E2E pass tests for all subscript and mutability fixes
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing subscript and mutability improvements

---

## milestone_comprehension_v2: Comprehension and Iteration Fixes

status: completed

**Goal:** Fix range in comprehension (19 errors), dict/set comprehension (11 errors), and tuple unpacking in for loops (34 errors).

### Implementation

1. **Comprehension over `range`**: `[x*x for x in range(10)]` -- fix comprehension lowering to accept `Type::Range`
2. **Dict comprehension**: `{k: v for k, v in items}` -- implement `lower_dict_comp` and codegen
3. **Set comprehension**: `{x*x for x in range(10)}` -- implement `lower_set_comp` and codegen
4. **Tuple unpacking in for loops**: `for i, v in enumerate(lst)` -- extend for-loop target to support all tuple destructuring patterns
5. **Tuple unpacking in comprehensions**: `[v for i, v in enumerate(lst)]`

**Key files:** `crates/sifr_lowering/src/lower/`, `crates/sifr_codegen/src/lib.rs`

### Definition of Done (milestone_comprehension_v2)

- Comprehension over `range` works
- Dict comprehension works
- Set comprehension works
- Tuple unpacking in for loops works
- Tuple unpacking in comprehensions works
- E2E pass tests for all comprehension and iteration fixes
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing comprehension improvements

---

## milestone_generics_impl: Generics Implementation

status: completed

**Goal:** Implement generics. Generics were scoped in Phase 2 `milestone_generics` but the type system has NO `TypeVar` or generic type parameter support. This is a significant gap.

### Implementation

1. **`Type::TypeVar`**: Add a type variable variant to the type system
2. **Generic function syntax**: `def first[T](items: list[T]) -> T` -- parse, lower, and monomorphize
3. **Generic class syntax**: `class Stack[T]:` -- parse, lower, and monomorphize
4. **`Callable[[int], int]]`**: Add callable type syntax for higher-order function parameters
5. **Protocol as generic bound**: `def sort[T: Comparable](items: list[T])` -- use existing protocol infrastructure

**Key files:** `crates/sifr_type_system/src/types.rs` (add `TypeVar`), `crates/sifr_lowering/src/lower/` (generic resolution), `crates/sifr_codegen/src/lib.rs` (monomorphization or trait-based codegen)

### Definition of Done (milestone_generics_impl)

- `TypeVar` variant exists in the type system
- Generic functions (`def first[T](items: list[T]) -> T`) compile and run
- Generic classes (`class Stack[T]:`) compile and run
- `Callable` type syntax works for higher-order function parameters
- Protocol bounds on type parameters work
- E2E pass tests for generics, callable types, and protocol bounds
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing generics

---

## milestone_phase_fixes: Phase Bug Fixes and Polish

status: completed

**Goal:** Catch-all for remaining bugs in already-shipped Phase 2/3 features, plus stdlib gaps and codegen polish.

### Implementation

1. **Protocol method dispatch**: Fix calling methods through protocol-typed params
2. **Context manager scope**: Fix `with ... as conn` variable scope in codegen
3. **@classmethod `cls(...)` calls**: Fix `cls` as constructor call
4. **Import alias codegen for stdlib**: Fix generated Rust referencing original name instead of alias
5. **`print(None)` / unit type Display**: Emit `println!("None")`
6. **Union return wrapping**: `return 42` in `int | str` function must wrap in enum variant
7. **Optional field/init codegen**: Fix wrong Rust types for optional class fields
8. **`f64 * i64` mixed arithmetic**: Emit explicit cast
9. **Empty collection inference**: `[]` and `{}` need annotation or first-usage inference
10. **Stdlib gaps**: Add `sifr.math` trig (log, sin, cos, tan), fix `sifr.json` to accept non-str types, fix audit test files to use correct API names
11. **Module-level constants**: Top-level `PI = 3.14` accessible from functions

**Key files:** `crates/sifr_codegen/src/lib.rs`, `crates/sifr_lowering/src/lower/`, `crates/sifr_stdlib`, audit test files

### Definition of Done (milestone_phase_fixes)

- Protocol method dispatch works through protocol-typed params
- Context manager `with ... as conn` scope works
- `@classmethod` `cls(...)` constructor calls work
- Import alias codegen correct for stdlib modules
- `print(None)` outputs "None"
- Union return wrapping correct for non-Option unions
- Optional class fields generate correct Rust types
- Mixed `f64 * i64` arithmetic emits correct casts
- Empty collection inference works
- Stdlib trig functions and API name fixes in place
- Module-level constants accessible from functions
- E2E pass tests for all fixes
- All existing E2E tests pass (no regressions)
- Milestone demo showcasing phase fixes

---

## milestone_audit_fixup: Audit Fix-Up

status: completed

**Goal:** Close out remaining fixable audit failures identified after Language Hardening Phase 2. Targets 13 audit tests across type_system, python_basics, and stdlib categories.

### 1. PEP 695 Inline Generics

Support `def f[T](x: T) -> T` and `class C[T]` syntax. The AST already parses `type_params` on `StmtFunctionDef` and `StmtClassDef` (`third_party/ruff/crates/ruff_python_ast/src/nodes.rs`); the lowering layer just needs to wire them through.

- In `lower_function` / `extract_function_type`: check `func.type_params` from the AST; if present, register each `TypeParam` name in `ctx.type_vars` and store in `HirFunction.type_params`
- In `collect_class_type`: check `class_def.type_params`; register type params for the class scope
- `resolve_annotation_expr` already handles `Type::TypeVar` lookup -- no change needed

**Fixes:** `verification/areas/core_language/fixtures/audits/type_system/21_generic_functions_syntax.sifr`, `verification/areas/core_language/fixtures/audits/type_system/22_generic_class_syntax.sifr`

### 2. Protocol Method Dispatch

Enable method calls on Protocol-typed function parameters. Currently Protocol types generate `Box<dyn ProtocolName>` in Rust, but method calls on Protocol-typed variables fail during lowering because method lookup doesn't check Protocol definitions.

- In the method call resolution path in `lower.rs`: when the receiver type is `Type::Protocol(name)`, look up the protocol's method signatures and resolve the call
- Codegen should emit correct trait method calls on `Box<dyn Protocol>` automatically once HIR is correct

**Fixes:** `verification/areas/core_language/fixtures/audits/type_system/23_interface_as_param.sifr`, `verification/areas/core_language/fixtures/audits/type_system/34_protocol_param_dispatch.sifr`

### 3. Multi-Generator Comprehensions

Support `[x for row in matrix for x in row]` (multiple `for` clauses in a single comprehension).

- Extend `HirExpr::ListComp` (and `SetComp`, `DictComp`) in `hir_nodes.rs` to support a `Vec` of generators instead of a single `var`/`iter`
- Remove the `generators.len() != 1` guard in `lower.rs`; process generators in order, nesting scopes for each
- Update codegen to emit nested `for` loops for multi-generator comprehensions

**Fixes:** `verification/areas/core_language/fixtures/audits/python_basics/15_list_comprehension.sifr`

### 4. Stdlib Fixes

All stdlib signature changes primarily live in `crates/sifr_stdlib` (and codegen mappings where needed):

- **Math**: Add the original intrinsic implementations for `log`, `sin`, `cos`, `tan`, power, min/max, and rounding. The pre-v1 public contract later standardized `fabs` and `pow`; the implementation spellings are private.
- **IO**: Rename `write_file` -> `write_text`, `read_file` -> `read_text`, `file_exists` -> `exists`
- **Env**: Keep `_sifr.sys.env_get` and `_sifr.sys.env_set` private; expose `getenv_opt`, typed-default `getenv`, and `setenv` publicly.
- **Hash**: Rename `md5_hash` -> `md5`
- **JSON**: Widen `json_dumps` parameter type from `str` to accept any serializable type
- **Random**: Provide generic public `choice[T]`; low-level random intrinsics remain private implementation details.

**Fixes:** `verification/areas/stdlib_parity/fixtures/audits/stdlib/01_math.sifr`, `verification/areas/stdlib_parity/fixtures/audits/stdlib/02_json.sifr`, `verification/areas/stdlib_parity/fixtures/audits/stdlib/06_io.sifr`, `verification/areas/stdlib_parity/fixtures/audits/stdlib/08_env.sifr`, `verification/areas/stdlib_parity/fixtures/audits/stdlib/09_random.sifr`, `verification/areas/stdlib_parity/fixtures/audits/stdlib/10_hash_encoding.sifr`

### 5. Set[T] Type (Stretch Goal)

Add `Set[T]` to the type system and collections stdlib, similar to how `list` and `dict` are handled. Includes constructor, `contains`, `add`, `remove` methods, and `len()` support.

**Fixes:** `verification/areas/stdlib_parity/fixtures/audits/stdlib/05_collections.sifr`

**Key files:** `crates/sifr_lowering/src/lower/`, `crates/sifr_ir`, `crates/sifr_stdlib`, `crates/sifr_codegen/src/lib.rs`, `crates/sifr_type_system/src/types.rs`

### Definition of Done (milestone_audit_fixup)

- PEP 695 `def f[T]` and `class C[T]` syntax works end-to-end
- Protocol method dispatch works through protocol-typed params
- Multi-generator comprehensions compile and run correctly
- Stdlib naming and type signature fixes in place
- E2E pass tests for all new features
- All existing E2E tests pass (no regressions)
- Audit pass rates improve for type_system, python_basics, and stdlib categories
- Milestone demo showcasing fixes

---

## milestone_ownership_v3: Ownership Hardening

status: completed

**Goal:** Close all ownership/borrowing detection gaps in Sifr's HIR checker so that use-after-move errors are caught by Sifr's own diagnostics (not deferred to rustc). This milestone is the foundation for fearless concurrency -- without complete ownership tracking at the Sifr level, future Send/Sync/async inference cannot work.

### 1. Assignment-Based Move Detection

Track moves through variable assignment (`s2 = s1`). When the RHS of an assignment is a `Name` expression referencing a Move-type variable, mark the source variable as moved. Applies to both `lower_assign()` (untyped assignment) and `lower_ann_assign()` (annotated assignment).

**Key files:** `crates/sifr_lowering/src/lower/` (lower_assign, lower_ann_assign)

### 2. Move-in-Loop Detection

Detect outer-scope variables consumed inside loop bodies. Before lowering a loop body, snapshot the moved state. After lowering, check which outer-scope variables were newly moved -- these would be unavailable on subsequent iterations.

**Key files:** `crates/sifr_lowering/src/scope.rs` (save_moved_state, moved_since), `crates/sifr_lowering/src/lower/` (lower_for, lower_while)

### 3. Conditional Move Tracking

Save/restore/merge moved state across if/elif/else branches, matching the existing narrowing snapshot pattern. If a variable is moved in any branch, it is conservatively marked as moved after the if/else block.

**Key files:** `crates/sifr_lowering/src/lower/` (lower_if)

### 4. Set Display Codegen Fix

Add `Type::Set(_)` to the Debug-format pattern in print codegen so `print(set)` emits `println!("{:?}", ...)` instead of `println!("{}", ...)`.

**Key files:** `crates/sifr_codegen/src/lib.rs`

### Concurrency Enablement

This milestone is the prerequisite for fearless concurrency:

- **Closure capture inference** (needed for `tokio::spawn`) requires knowing which variables are moved vs borrowed at every point
- **Send + Sync checking** requires tracking that no `&mut` aliases exist across `.await` points
- **Channel ownership** (`tx.send(value)`) requires the compiler to mark `value` as moved

### Definition of Done (milestone_ownership_v3)

- All assignment-based move errors caught by `sifr check` with Sifr-level error messages
- Loop move errors caught by `sifr check`
- Conditional move tracking works correctly across branches
- `print(set)` works
- All existing E2E tests pass (no regressions)
- `verification/areas/core_language/fixtures/audits/ownership/` shows 0 "Fail (Rust compile)" results
- Core-language ownership audit fixtures: 38 pass, 12 correct Sifr rejections, 0 Rust failures

---

## Milestone ordering

The milestones within this phase follow a deliberate dependency chain:

- **milestone_codegen_fixes first:** Bugs in already-implemented features must be fixed before adding new ones — all subsequent milestones build on correct codegen.
- **milestone_narrowing_v2 before milestone_ownership_v2:** Many ownership workarounds depend on narrowing patterns (`if x is not None:`). Fixing narrowing first unblocks 36+ LeetCode problems.
- **milestone_ownership_v2 before milestone_subscript_mutation:** Subscript assignment requires `&mut` references, which depend on correct ownership tracking.
- **milestone_subscript_mutation before milestone_iteration_v2:** Dict comprehension and `for k, v in d.items()` patterns often combine with `d[k] = v`.
- **milestone_iteration_v2 before milestone_builtins_v2:** `sorted(key=...)` depends on lambda iteration patterns; builtins benefit from working iteration.
- **milestone_builtins_v2 before milestone_syntax_expansion:** Module-level variables and mixed arithmetic are prerequisites for many real-world programs that also use nested functions.
- **milestone_syntax_expansion before milestone_recursive_types:** Tree/graph algorithms heavily use nested functions (DFS/BFS helpers) and closures, which must work before recursive types are useful.
- **milestone_recursive_types before milestone_inference_v2:** Inference for recursive types (e.g., `def build_tree(nums)` returning `TreeNode | None`) requires the type system to already support those types.
- **milestone_inference_v2 before milestone_stdlib_hardening:** Stdlib hardening is the least blocking — programs can work around missing stdlib functions but not missing syntax or broken codegen.
- **milestone_stdlib_hardening before milestone_nested_functions:** Stdlib hardening completes the first 10 hardening milestones. The post-hardening audit revealed systematic remaining failures requiring a second round of fixes.
- **milestone_nested_functions first in Hardening Phase 2:** Nested functions is the single biggest blocker (~200 LeetCode problems). Blocks DFS/BFS helpers, backtracking, and recursive algorithms.
- **milestone_forward_refs after milestone_nested_functions:** Forward refs are the second biggest blocker (~60 LeetCode problems). Together with nested functions, unblocks the majority of class-based algorithms.
- **milestone_narrowing_v3 after milestone_forward_refs:** Narrowing fixes depend on types being resolvable (forward refs). Equality chains and field access on narrowed types are pervasive.
- **milestone_union_ops after milestone_narrowing_v3:** Union operations depend on narrowing being correct first. 90+ LeetCode errors from arithmetic, indexing, and `len()` on `T | None`.
- **milestone_subscript_v2 after milestone_union_ops:** Subscript/codegen fixes build on correct union handling. Nested subscript assignment and `&mut self` for methods.
- **milestone_comprehension_v2 after milestone_subscript_v2:** Comprehensions are syntactic sugar that benefit from all prior fixes. Range in comprehension, dict/set comprehension, tuple unpacking.
- **milestone_generics_impl after milestone_comprehension_v2:** Generics is the largest new feature. Everything else should be stable before adding type parameters.
- **milestone_phase_fixes last in Hardening Phase 2:** Catch-all for remaining bugs -- protocol dispatch, context managers, stdlib gaps, and codegen polish.
- **milestone_audit_fixup after milestone_phase_fixes:** Closes out remaining fixable audit failures identified after the main hardening work — PEP 695 generics, protocol dispatch, multi-generator comprehensions, and stdlib fixes.
- **milestone_ownership_v3 after milestone_audit_fixup:** Ownership hardening is the capstone — it requires all language features to be stable before tracking moves, borrows, and conditional ownership across the full language surface. It is the foundation for fearless concurrency.
