---
name: Language Hardening Phase
overview: "Insert a new \"Phase: Language Hardening\" between Phase 3 (Standard Library) and Phase 4 (Ecosystem) consisting of 10 sequential milestones that fix every compiler issue identified across 9 audits (type system, type inference, python basics, lexical/syntax, object model, iteration protocol, modules/imports, stdlib, and 396 LeetCode problems). This phase takes the compiler from 2% LeetCode pass rate to approximately 80%+."
todos:
  - id: m1-codegen-fixes
    content: "milestone_codegen_fixes: Fix all codegen bugs (tuple indexing, union return wrapping, int/int, print(None), escaped quotes, narrowed reassignment, etc.)"
    status: pending
  - id: m2-narrowing-v2
    content: "milestone_narrowing_v2: Fix elif equality, elif isinstance 3+ unions, early-return narrowing, and-narrowing, sequential narrowing, len() on nested optionals"
    status: pending
  - id: m3-ownership-v2
    content: "milestone_ownership_v2: Fix move semantics — auto-borrow for print/comparison/function params, stop consuming values after use"
    status: pending
  - id: m4-subscript-mutation
    content: "milestone_subscript_mutation: list[i]=val, dict[key]=val, self.field+=1, augmented subscript assignment"
    status: pending
  - id: m5-iteration-v2
    content: "milestone_iteration_v2: String/dict iteration, tuple unpacking in for, comprehension over range(), dict comprehension"
    status: pending
  - id: m6-builtins-v2
    content: "milestone_builtins_v2: max(a,b), min(a,b), range(a,b,c), sorted(key=), mixed int/float, module-level vars, pow(), optional auto-wrapping"
    status: pending
  - id: m7-syntax-expansion
    content: "milestone_syntax_expansion: Nested functions/closures, bitwise operators, multiple assignment, chained assignment, @classmethod cls, higher-order fn types"
    status: pending
  - id: m8-recursive-types
    content: "milestone_recursive_types: Self-referential types (ListNode, TreeNode) using Box<T> in codegen"
    status: pending
  - id: m9-inference-v2
    content: "milestone_inference_v2: Return type inference, parameter type inference for helpers, Result unwrapping in try blocks"
    status: pending
  - id: m10-stdlib-hardening
    content: "milestone_stdlib_hardening: set() type, import aliases, fill sifr.math/json/io/env/random/hash gaps, defaultdict/Counter access"
    status: pending
isProject: false
---

# Phase: Language Hardening

This phase sits between `milestone_codegen_quality_v3` (end of Phase 3) and `milestone_async` (start of Phase 4). It addresses every issue found across 9 audit reports (type system, type inference, python basics, lexical/syntax, object model, iteration protocol, modules/imports, stdlib, and 396 LeetCode problems).

Current state: **2% of LeetCode problems compile**. Target: **80%+**.

## Issue Inventory (Deduplicated Across All 9 Audits)

Every issue from every audit maps to exactly one milestone below. The issues cluster into these groups:

- **Narrowing & type flow** — elif chains, early-return narrowing, and-narrowing, 3+ union isinstance codegen, optional auto-wrapping
- **Ownership / move semantics** — print consuming values, collections moved after use, dunder operators consuming operands
- **Subscript & mutation** — `list[i] = val`, `dict[key] = val`, `self.field += 1`
- **Iteration gaps** — string iteration, dict iteration, tuple unpacking in for, comprehension over range, dict comprehension
- **Missing syntax** — nested functions/closures, multiple assignment, bitwise operators, chained assignment
- **Builtins & stdlib** — `max(a,b)`, `min(a,b)`, `range(start,stop,step)`, `sorted(key=)`, mixed int/float arithmetic, module-level variables, stdlib API gaps
- **Type system** — generics, recursive types, return type inference, parameter type inference
- **Codegen bugs** — tuple indexing, union return wrapping, `int/int` codegen, escaped quotes, `print(None)`, narrowed variable reassignment

## Milestone Sequence

The order is designed so each milestone builds on the previous one. No parallelism.

```
milestone_codegen_quality_v3 (end of Phase 3)
    |
    v
milestone_codegen_fixes          -- Fix codegen bugs that produce wrong Rust
    |
    v
milestone_narrowing_v2           -- Fix type narrowing gaps
    |
    v
milestone_ownership_v2           -- Fix move semantics to stop consuming values
    |
    v
milestone_subscript_mutation     -- Subscript assignment + augmented assignment on fields
    |
    v
milestone_iteration_v2           -- String/dict iteration, tuple unpacking in for, comprehensions
    |
    v
milestone_builtins_v2            -- Multi-arg builtins, mixed arithmetic, module-level vars
    |
    v
milestone_syntax_expansion       -- Nested functions, closures, bitwise ops, multiple assignment
    |
    v
milestone_recursive_types        -- Self-referential types (ListNode, TreeNode)
    |
    v
milestone_inference_v2           -- Return type inference, parameter inference
    |
    v
milestone_stdlib_hardening       -- Fill stdlib API gaps, set() type, import aliases
    |
    v
milestone_async (start of Phase 4)
```

---

## Milestone 1: `milestone_codegen_fixes`

**Goal:** Fix all codegen bugs that produce syntactically or semantically wrong Rust code. These are "the compiler accepts the program but the generated Rust doesn't compile or produces wrong output."

**Issues resolved:**

- Tuple index codegen: `pair[0]` emits `pair.0_i64` instead of `pair.0` (type inference audit, Issue 5)
- Union return value wrapping: `return 42` in `-> int | str` emits bare `42_i64` instead of `IntOrStr::Int(42_i64)` (type inference audit, Issue 3)
- `int / int` codegen: type system says `float` but codegen emits `i64 / i64` (type inference audit, Issue 6; python basics, Issue 1 partial)
- `print(None)` codegen: `()` has no `Display` (type inference audit, Issue 2)
- Escaped quotes in strings: `\"` generates invalid Rust (lexical/syntax audit, Issue 1)
- Narrowed variable reassignment: `name = name.upper()` inside `if name is not None:` emits immutable binding (type system audit, Issue 10)
- `float * int` operator precedence in `as` casts (type system audit, Issue 11)
- `**=` augmented power assignment returns `f64` for `i64` variable (python basics, Issue 11)
- `bool()` on collections codegen (python basics, Issue 18)
- 3-way union `is None` codegen: generates full enum but calls `.is_none()` (type system audit, Issue 13)

**Where to fix:** Primarily `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)`, with some fixes in `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)`.

**Rationale for going first:** These are bugs in already-implemented features. Fixing them first means all subsequent milestones build on correct codegen. Many of these also unblock audit test cases that are "almost passing."

---

## Milestone 2: `milestone_narrowing_v2`

**Goal:** Make type narrowing comprehensive so that all standard Python patterns work.

**Issues resolved:**

- `elif` equality narrowing: `if x == "GET": ... elif x == "POST":` narrows to `Never` (type system audit, Issue 2)
- `elif isinstance` codegen for 3+ unions: middle branches silently dropped (type system audit, Issue 3)
- Early-return narrowing: `if x is None: return` doesn't narrow `x` after (LeetCode audit — 36 problems)
- `and`-based compound narrowing: `if a is not None and b is not None:` doesn't narrow (LeetCode audit)
- Sequential narrowing: `if x is None: ... elif isinstance(x, int): ... else:` doesn't narrow to remaining type (type system audit, Issue 9)
- `len()` on nested optional types: `len(matrix[0])` fails because `matrix[0]` is `list[int] | None` (LeetCode audit — 21 problems)

**Where to fix:** `[crates/sifr_hir/src/narrow.rs](crates/sifr_hir/src/narrow.rs)`, `[crates/sifr_hir/src/scope.rs](crates/sifr_hir/src/scope.rs)`, `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (for elif isinstance match arm generation).

**Rationale for position:** Narrowing must be fixed before ownership (milestone 3) because many ownership workarounds depend on narrowing patterns. Also, narrowing fixes alone unblock 36+ LeetCode problems.

---

## Milestone 3: `milestone_ownership_v2`

**Goal:** Fix overly aggressive move semantics so that values can be used after printing, passing to functions, or using in operators.

**Issues resolved:**

- `print(obj)` consumes the object, making it unusable afterward (type system audit, Issue 4 — 5 tests)
- String method results moved on first use: `parts = s.split(","); print(parts); joined = ", ".join(parts)` (python basics, Issue 3)
- List mutation after use: `nums.append(9); print(nums); nums.insert(0, 0)` (python basics, Issue 4)
- Tuple `len()` after print triggers move (python basics, Issue 6)
- Dunder operators consume left operand: `a + b` then `a == ...` fails (object model audit, Issue 1)
- Chained operations on inferred types: `filter().map().len()` fails (type inference audit — 5 tests)

**Implementation approach:** Auto-borrow (`&`) for `print()`, comparison operators, and function parameters where the callee doesn't need ownership. For collections, emit `.clone()` or use `&` references. For classes, derive `Clone` automatically.

**Where to fix:** `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (ownership tracking), `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (emit `&` or `.clone()`), `[crates/sifr_type_system/src/types.rs](crates/sifr_type_system/src/types.rs)` (ownership rules).

**Rationale for position:** After narrowing (which changes how types flow) but before subscript mutation (which needs correct ownership for `&mut` references).

---

## Milestone 4: `milestone_subscript_mutation`

**Goal:** Support subscript assignment and augmented assignment on non-simple targets.

**Issues resolved:**

- `list[i] = val` — "assignment target must be a simple name" (LeetCode audit — 19 problems; python basics, Issue 5)
- `dict[key] = val` — same error (python basics, Issue 5)
- `self.field += 1` — "augmented assignment target must be a simple name" (python basics, Issue 19)
- `list[i] += val` — augmented subscript assignment
- `self.field = self.field + 1` workaround support

**Where to fix:** `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (recognize subscript/attribute assignment targets), `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (emit `vec[i] = val`, `map.insert(key, val)`, `self.field += val`).

**Rationale for position:** After ownership fixes (milestone 3) because subscript assignment requires `&mut` references, which depend on correct ownership tracking.

---

## Milestone 5: `milestone_iteration_v2`

**Goal:** Complete the iteration protocol so all standard Python iteration patterns work.

**Issues resolved:**

- String iteration: `for ch in "abc":` — "cannot iterate over type 'str'" (iteration audit, Issue 1; LeetCode — 12 problems)
- Dict iteration: `for key in d:` — "cannot iterate over type 'dict'" (iteration audit, Issue 2)
- Tuple unpacking in `for`: `for i, v in enumerate(...)` — "for loop target must be a simple name" (iteration audit, Issue 3; LeetCode — 17 problems)
- Comprehension over `range()`: `[x*x for x in range(5)]` — "cannot iterate over type 'range'" (python basics, Issue 7; LeetCode — 10 problems)
- Dict comprehension: `{k: v for k, v in ...}` — "unsupported expression type" (python basics, Issue 8)
- `for k, v in dict.items()` — combines dict iteration + tuple unpacking

**Where to fix:** `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (for loop target destructuring, comprehension range support), `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (emit `.chars()` for string, `.keys()` / `.iter()` for dict, destructuring in for).

**Rationale for position:** After subscript mutation (milestone 4) because dict comprehension and `for k, v in d.items()` patterns often combine with `d[k] = v`. Also, iteration patterns are needed by the builtin expansions in milestone 6.

---

## Milestone 6: `milestone_builtins_v2`

**Goal:** Expand built-in functions and arithmetic to match Python's standard behavior.

**Issues resolved:**

- `max(a, b)` / `min(a, b)` — 2-arg overloads (LeetCode — 15 problems)
- `range(start, stop, step)` — 3-arg range (LeetCode — 5+ problems; python basics, Issue 7 partial)
- `sorted(list, key=lambda x: ...)` — key parameter
- Mixed `int`/`float` arithmetic: `10 + 3.5` — auto-promotion (python basics, Issue 1)
- Module-level variables / global constants: `PI = 3.14` accessible from functions (python basics, Issue 12)
- `pow()` built-in (python basics, Issue 10)
- `list.pop(index)` — pop with argument (LeetCode — 2 problems)
- `abs()` as standalone function for all numeric types
- `with` statement variable binding: `with X as name:` — `name` accessible in block (python basics, Issue 15)
- Optional auto-wrapping at call sites: passing `T` where `T | None` expected (type system audit, Issue 6)

**Where to fix:** `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (builtin resolution, module-level scope), `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (emit multi-arg builtins, int-to-float coercion), `[crates/sifr_type_system/src/types.rs](crates/sifr_type_system/src/types.rs)` (numeric promotion rules).

**Rationale for position:** After iteration (milestone 5) because `sorted(key=...)` depends on lambda iteration patterns. Module-level variables are also a prerequisite for many real-world programs.

---

## Milestone 7: `milestone_syntax_expansion`

**Goal:** Add missing syntax constructs that block real-world Python programs.

**Issues resolved:**

- Nested functions / closures: `def helper():` inside another function (LeetCode — 38 problems; python basics, Issue 13)
- Bitwise operators: `&`, `|`, `^`, `~`, `<<`, `>>` (lexical/syntax audit, Issue 2; LeetCode — 4 problems)
- Multiple assignment: `a, b = 1, 2` and `a, b = b, a` (LeetCode — 9 problems; python basics, Issue 17)
- Chained assignment: `x = y = z = 0` (python basics, Issue 17)
- `@classmethod` with `cls` constructor (python basics, Issue 14)
- Higher-order function type syntax: `Callable[[int], int]` as parameter type (type system audit, Issue 12)
- Unary `+` operator (python basics, Issue 2)

**Where to fix:** Parser (`[crates/sifr_parser/](crates/sifr_parser/)`), HIR lowering (`[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)`), codegen (`[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)`).

**Rationale for position:** Nested functions/closures are the 3rd most impactful LeetCode fix (38 problems). They require correct ownership (milestone 3) for captured variables, and benefit from iteration (milestone 5) since closures are often used with `map`/`filter`. Bitwise operators are straightforward parser+codegen additions.

---

## Milestone 8: `milestone_recursive_types`

**Goal:** Support self-referential class types for linked lists, trees, and other recursive data structures.

**Issues resolved:**

- `class ListNode: val: int; next: ListNode | None` — "unknown type: 'ListNode'" (LeetCode — 49 problems)
- `class TreeNode: val: int; left: TreeNode | None; right: TreeNode | None` — same
- Custom recursive types for graphs, tries, etc.

**Implementation approach:** Use `Box<T>` in generated Rust for recursive fields. The type system must detect self-referential types during class registration and mark recursive fields for boxing.

**Where to fix:** `[crates/sifr_type_system/src/types.rs](crates/sifr_type_system/src/types.rs)` (recursive type detection), `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (class field analysis), `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (emit `Box<T>` for recursive fields).

**Rationale for position:** After syntax expansion (milestone 7) because tree/graph algorithms heavily use nested functions (DFS/BFS helpers) and closures. Also, recursive types are a complex type system change that benefits from all prior fixes being stable.

---

## Milestone 9: `milestone_inference_v2`

**Goal:** Improve type inference to reduce annotation burden.

**Issues resolved:**

- Return type inference: omitting `-> ReturnType` defaults to `None` instead of inferring (type inference audit, Issue 1; LeetCode — 44 problems)
- Parameter type inference for nested/helper functions: `def helper(arr, target):` — "missing a type annotation" (LeetCode — 44 problems)
- `Result` unwrapping in `try` blocks: variable inferred as `Result[T, E]` instead of `T` (type inference audit, Issue 4)

**Where to fix:** `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (return type inference from return statements), `[crates/sifr_type_system/src/types.rs](crates/sifr_type_system/src/types.rs)` (inference rules).

**Rationale for position:** After recursive types (milestone 8) because inference for recursive types (e.g., `def build_tree(nums)` returning `TreeNode | None`) requires the type system to already support those types. Also, inference is a "polish" feature — the language works without it (just more verbose), so it comes late.

---

## Milestone 10: `milestone_stdlib_hardening`

**Goal:** Fill gaps in the standard library and import system.

**Issues resolved:**

- `set()` type with `add()`, `in`, `discard()`, `len()`, set comprehension (LeetCode — 18 problems)
- `import X as alias` — import aliasing (modules/imports audit, Issue 1)
- `sifr.math` missing functions: `log`, `sin`, `cos`, `tan`, `pow` (stdlib audit, Issue 1)
- `sifr.json` only accepts strings, not arbitrary types (stdlib audit, Issue 2)
- `sifr.io` missing `read_text`, `write_text`, `exists` (stdlib audit, Issue 4)
- `sifr.env` API name mismatches (stdlib audit, Issue 5)
- `sifr.random` only works with `list[int]`, not generic (stdlib audit, Issue 6)
- `sifr.hash` missing `md5` (stdlib audit, Issue 7)
- `defaultdict` and `Counter` accessibility (LeetCode — 6 problems)

**Where to fix:** `[crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)` (set type codegen), `[crates/sifr_hir/src/lower.rs](crates/sifr_hir/src/lower.rs)` (import aliasing, set constructor), stdlib modules in codegen.

**Rationale for going last:** Stdlib hardening depends on all prior language features (generics from Phase 2 for generic stdlib functions, set type needs iteration from milestone 5, etc.). It's also the least "blocking" — programs can work around missing stdlib functions, but can't work around missing syntax or broken codegen.

---

## Expected Outcome

After completing all 10 milestones:

- **LeetCode pass rate**: ~80%+ (up from 2%)
- **Type system audit**: all 41 tests pass (up from 17/41)
- **Type inference audit**: all 30 tests pass (up from 18/30)
- **Python basics audit**: all 45 tests pass (up from 22/45)
- **Iteration protocol audit**: all 5 tests pass (up from 1/5)
- **Remaining failures** will be limited to: problems needing `heapq` (Phase 4 stdlib), problems needing advanced generics (conditional types, mapped types), and problems needing features explicitly deferred to Phase 5 (metaprogramming, FFI).

## Audit Traceability

Every issue from every audit report is assigned to exactly one milestone:


| Audit                         | Total Issues | Milestone Coverage                                                                          |
| ----------------------------- | ------------ | ------------------------------------------------------------------------------------------- |
| Type System (13 issues)       | 13           | M1: 3, M2: 4, M3: 1, M6: 1, M7: 1, M8: 1, M9: 0, M10: 0, Phase 2 (generics already done): 2 |
| Type Inference (7 issues)     | 7            | M1: 4, M2: 0, M3: 0, M9: 2, correct-by-design: 1                                            |
| Python Basics (21 issues)     | 21           | M1: 3, M2: 1, M3: 3, M4: 2, M5: 2, M6: 5, M7: 4, M10: 1                                     |
| Lexical/Syntax (2 issues)     | 2            | M1: 1, M7: 1                                                                                |
| Object Model (1 issue)        | 1            | M3: 1                                                                                       |
| Iteration Protocol (5 issues) | 5            | M5: 4, M3: 1                                                                                |
| Modules/Imports (1 issue)     | 1            | M10: 1                                                                                      |
| Stdlib (7 issues)             | 7            | M10: 7                                                                                      |
| LeetCode (18 categories)      | 371 errors   | All categories mapped to M1-M10                                                             |


