# milestone_codegen_quality_v2 — Phase 2 Codegen Polish

## 1. Product Requirements

### Objective

Phase 2 milestones (protocols, inheritance, generics, generators, decorators) introduced new codegen patterns — lambdas, iterator chains, inheritance field access, protocol impls, generators, and variadics — that produce **correct but non-idiomatic** Rust output. This milestone cleans up 7 systematic quality issues to produce cleaner, more idiomatic Rust before Phase 3 begins.

### Scope

**In Scope:**

1. Remove redundant `.clone()` on Copy types (`i64`, `f64`, `bool`) in `min`/`max` built-ins
2. Remove unnecessary `.clone()` inside `format!` arguments on `&self` fields
3. Inline lambda body in `filter()` instead of closure-within-closure pattern
4. Clean up filtered list comprehension double-deref pattern using `.copied()`/`.cloned()`
5. Fold string literal parts into `format!` format string instead of separate `{}` placeholders
6. Prefix unused `with` statement variables with `_` to suppress Rust warnings
7. Deduplicate protocol trait impl methods by delegating to inherent methods
8. Inline string literals directly in `println!` instead of `println!("{}", "literal")`

**Out of Scope:**

| Feature | Reason |
| --- | --- |
| New language features | This is a polish-only milestone |
| Refactoring HIR or type system | Changes are codegen-only |
| Performance optimizations | Focus is on code quality/idiom, not speed |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | **Given** `min(nums)` where nums is `list[int]`, **When** Rust is emitted, **Then** output uses `.unwrap()` without `.clone()` (i64 is Copy) |
| AC-2 | **Given** `filter(lambda x: x > 1, nums)`, **When** Rust is emitted, **Then** output uses a single `.filter(|x| *x > 1)` closure, no nested closure invocation |
| AC-3 | **Given** `[x for x in nums if x > 2]`, **When** Rust is emitted, **Then** output uses `.iter().copied().filter().map().collect()` pattern |
| AC-4 | **Given** `"Hello, " + name + "!"`, **When** Rust is emitted, **Then** output is `format!("Hello, {}!", name)` with literals folded into format string |
| AC-5 | **Given** `with Timer("work") as t:` where `t` is unused, **When** Rust is emitted, **Then** variable is `let _t` |
| AC-6 | **Given** a class implementing a protocol, **When** trait impl is emitted, **Then** trait methods delegate to inherent impl (no body duplication) |
| AC-7 | **Given** `print("hello")`, **When** Rust is emitted, **Then** output is `println!("hello")` not `println!("{}", "hello")` |
| AC-8 | **Given** all 94 existing E2E pass tests, **When** `cargo test` is run, **Then** all tests pass with no regressions |

---

## 2. Solution Design

### 2.1 Architecture

All changes are confined to `crates/sifr_codegen/src/lib.rs`. No HIR, type system, or parser changes are needed. Each fix is a targeted modification to an existing codegen handler.

```
sifr_codegen/src/lib.rs
    ├── needs_clone_for_type()          → Task 1
    ├── "min"/"max" handler             → Task 1
    ├── "filter" handler                → Task 2
    ├── HirExpr::ListComp handler       → Task 3
    ├── collect_string_concat_parts()   → Task 4
    ├── BinOp string concat handler     → Task 4
    ├── HirStmt::With handler           → Task 5
    ├── emit_protocol_impls()           → Task 6
    └── "print" handler                 → Task 7
```

### 2.2 Detailed Changes

**Task 1 — Redundant `.clone()` on Copy types:**
- In `min`/`max` handlers: replace `.iter().min().unwrap().clone()` with `.iter().min().unwrap()` and dereference with `*` since the element type is `Copy` (i64).
- `needs_clone_for_type()` already returns `false` for Copy types — ensure callers respect this.

**Task 2 — Inline lambda in `filter()`:**
- When `args[0]` is `HirExpr::Lambda`, emit the lambda body directly inside `.filter(|x| body)` instead of `.filter(|x| { let x = *x; (|x| body)(x) })`.

**Task 3 — List comprehension deref cleanup:**
- For filtered comprehensions with Copy element types: use `.iter().copied().filter(|x| cond).map(|x| expr).collect()`.
- For non-Copy types: use `.iter().cloned().filter(|x| cond).map(|x| expr).collect()`.
- Eliminates the `let x = **x` / `let x = *x` rebinding pattern.

**Task 4 — String literal folding in `format!`:**
- In `collect_string_concat_parts` / BinOp handler: when a part is `HirExpr::StringLiteral`, embed its value directly in the format string instead of using `{}` placeholder + `.to_string()`.

**Task 5 — Unused `with` variable prefix:**
- In `HirStmt::With` handler: scan body statements for references to the variable. If not found, prefix with `_`.

**Task 6 — Protocol impl delegation:**
- In `emit_protocol_impls`: instead of re-emitting the full method body, emit a call to the inherent method: `self.method_name(args)`.

**Task 7 — Inline string literal in `println!`:**
- In `"print"` handler: when single argument is `HirExpr::StringLiteral`, emit `println!("literal")` directly.

### 2.3 Testing Strategy

| AC-ID | Test Layer | Happy-Path Check | Non-Happy / Edge Check |
| --- | --- | --- | --- |
| AC-1 | E2E | `min(nums)` emits `*nums.iter().min().unwrap()` | Float still uses `f64::min` path |
| AC-2 | E2E | `filter(lambda, list)` emits single closure | Non-lambda filter argument unchanged |
| AC-3 | E2E | Filtered comprehension uses `.copied()` | Non-Copy types use `.cloned()` |
| AC-4 | E2E | `"a" + x + "b"` → `format!("a{}b", x)` | Pure variable concat unchanged |
| AC-5 | E2E | Unused `with` var gets `_` prefix | Used `with` var keeps name |
| AC-6 | E2E | Protocol impl delegates to inherent method | Multi-method protocols work |
| AC-7 | E2E | `print("hello")` → `println!("hello")` | Non-literal print unchanged |
| AC-8 | Full suite | All 94 E2E tests pass | `cargo test` green |

**Demo:** `./demos/milestone_codegen_quality_v2_demo.sifr`
