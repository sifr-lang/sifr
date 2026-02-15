# milestone_codegen_quality_v3 — Phase 3 Codegen Polish

## 1. Product Requirements

### Objective

Phase 3 milestones (core stdlib, test runner, extended collections, extended stdlib) introduced stdlib function codegen patterns that produce **correct but non-idiomatic** Rust output. This milestone cleans up 6 systematic quality issues to produce cleaner, more idiomatic Rust code with fewer unnecessary heap allocations.

### Scope

**In Scope:**

1. Remove redundant `.to_string()` on string literal arguments to stdlib functions that accept `&str`
2. Remove redundant `.clone()` on freshly-created `vec![...]` literals in set operations
3. Fix `json_dumps` to use `serde_json::to_string` instead of `.clone()`
4. Hoist second argument in `set_intersection` to avoid re-evaluation inside filter closure
5. Fix `re_replace` to avoid `.to_string().as_str()` pattern
6. Fix hash/encoding functions to avoid `.to_string().as_bytes()` pattern

**Out of Scope:**

| Feature | Reason |
| --- | --- |
| New language features | This is a polish-only milestone |
| Bytes type system changes | Issue 4 from audit (i64 intermediate in bytes roundtrip) requires a proper `bytes` type — deferred |
| Refactoring HIR or type system | Changes are codegen-only |
| Performance optimizations beyond allocation removal | Focus is on code quality/idiom |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | **Given** `read_file("/tmp/test.txt")`, **When** Rust is emitted, **Then** output uses `std::fs::read_to_string("/tmp/test.txt")` without `.to_string()` |
| AC-2 | **Given** `set_from_list([1, 2, 3])`, **When** Rust is emitted, **Then** output uses `vec![1_i64, 2_i64, 3_i64]` without `.clone()` |
| AC-3 | **Given** `json_dumps(data)`, **When** Rust is emitted, **Then** output uses `serde_json::to_string(&data).unwrap()` |
| AC-4 | **Given** `set_intersection(a, b)`, **When** Rust is emitted, **Then** second arg is hoisted to a `let` binding before the filter |
| AC-5 | **Given** `re_replace("[0-9]+", "hello123", "N")`, **When** Rust is emitted, **Then** replacement string uses `"N"` directly, not `"N".to_string().as_str()` |
| AC-6 | **Given** `sha256("sifr")`, **When** Rust is emitted, **Then** output uses `"sifr".as_bytes()` not `"sifr".to_string().as_bytes()` |
| AC-7 | **Given** all existing E2E pass tests, **When** `cargo test` is run, **Then** all tests pass with no regressions |
| AC-8 | **Given** all Phase 3 demos, **When** compiled and run, **Then** output is identical to before this milestone |

---

## 2. Solution Design

### 2.1 Architecture

All changes are confined to `crates/sifr_codegen/src/lib.rs`. No HIR, type system, or parser changes are needed. The core addition is a single helper method.

```
sifr_codegen/src/lib.rs
    ├── emit_expr_as_str_ref()            → NEW helper (core fix)
    ├── emit_stdlib_call() "read_file"    → Task 1 (use helper)
    ├── emit_stdlib_call() "write_file"   → Task 1 (use helper)
    ├── emit_stdlib_call() "env_get/set"  → Task 1 (use helper)
    ├── emit_stdlib_call() "path_exists"  → Task 1 (use helper)
    ├── emit_stdlib_call() "json_loads"   → Task 1 (use helper)
    ├── emit_stdlib_call() "set_from_list"→ Task 2 (remove .clone())
    ├── emit_stdlib_call() "set_add" etc  → Task 2 (remove .clone())
    ├── emit_stdlib_call() "json_dumps"   → Task 3 (serde_json)
    ├── emit_stdlib_call() "set_intersection" → Task 4 (hoist)
    ├── emit_stdlib_call() "re_*"         → Task 5 (use helper)
    └── emit_stdlib_call() hash/encoding  → Task 6 (use helper)
```

### 2.2 Detailed Changes

**Core — `emit_expr_as_str_ref` helper:**
- For `HirExpr::StringLiteral(val)`: emits `"val"` (bare string literal, no `.to_string()`)
- For any other expression: emits `&{expr}` (borrow the String)

**Task 1 — Remove redundant `.to_string()` on string literal args:**
- Update all stdlib call sites in `emit_stdlib_call` that pass arguments to Rust APIs accepting `&str` / `AsRef<str>` to use `emit_expr_as_str_ref` instead of `emit_expr`.

**Task 2 — Remove redundant `.clone()` on vec literals:**
- In `set_from_list`, `set_add`, `set_union`, `set_remove`: when emitting the collection argument, detect if it's a list literal and skip `.clone()`.

**Task 3 — Fix `json_dumps`:**
- Replace current `.clone()` emission with `serde_json::to_string(&expr).unwrap()`.

**Task 4 — Hoist second arg in `set_intersection`:**
- Emit the second argument into a `let __b = ...` binding before the filter, then reference `__b` inside the closure.

**Task 5 — Fix `re_replace`:**
- Use `emit_expr_as_str_ref` for pattern, input, and replacement arguments.

**Task 6 — Fix hash/encoding `.to_string().as_bytes()`:**
- For string literals: emit `"literal".as_bytes()` directly.
- For variables: emit `expr.as_bytes()` (String already has `.as_bytes()`).

### 2.3 Testing Strategy

| AC-ID | Test Layer | Happy-Path Check | Non-Happy / Edge Check |
| --- | --- | --- | --- |
| AC-1 | E2E | `read_file("/tmp/test.txt")` emits without `.to_string()` | Variable args still work with `&` |
| AC-2 | E2E | `set_from_list([1,2,3])` emits without `.clone()` | Variable args still clone correctly |
| AC-3 | E2E | `json_dumps(data)` emits `serde_json::to_string` | Nested objects serialize correctly |
| AC-4 | E2E | `set_intersection(a, b)` hoists `b` | Single-element sets work |
| AC-5 | E2E | `re_replace` uses bare string literals | Variable replacement strings work |
| AC-6 | E2E | `sha256("sifr")` uses `"sifr".as_bytes()` | Variable args use `expr.as_bytes()` |
| AC-7 | Full suite | All E2E tests pass | `cargo test` green |
| AC-8 | Demos | All 4 Phase 3 demos produce identical output | No regressions |

**Demo:** Existing Phase 3 demos serve as the verification demos for this milestone.
