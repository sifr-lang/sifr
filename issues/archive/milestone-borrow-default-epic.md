# milestone_borrow_default — Borrow-by-Default Parameter Passing

## 1. Product Requirements

### Objective

Change Sifr's function parameter passing from **move-by-default** to **borrow-by-default**. Currently, Sifr has a two-tier system: built-in functions borrow their arguments (via a hardcoded `borrows_args` list of 25 names), while user-defined functions move. This creates inconsistent "use-after-move" errors and forces workarounds. Borrow-by-default unifies the model: function arguments are immutably borrowed by default (`&T`), with opt-in `mut` (mutable borrow, `&mut T`) and `own` (ownership transfer, `T`) keywords. Copy types (`int`, `float`, `bool`) always pass by value.

### Scope

**In Scope:**

1. `ParamConvention` enum (`Borrow`/`MutBorrow`/`Own`) in the type system
2. Extend `FunctionType.params` to carry conventions; extend `Callable` type variant
3. Parse `mut`/`own` as soft keywords on function parameters
4. Add `convention` field to `Parameter` AST node and `HirParam`
5. Propagate conventions through HIR lowering
6. Delete the hardcoded `borrows_args` list; replace with convention-aware move tracking
7. Update all call paths (regular, Callable, method) for convention-aware move tracking
8. Extend `func_signatures` in codegen to carry conventions; register class methods
9. Emit `&T`/`&mut T`/`T` for parameter types based on convention
10. Convention-aware call-site argument emission for `HirExpr::Call` and `HirExpr::MethodCall`
11. Borrowed parameter escape detection (return/store) with compiler errors (no silent clone)
12. Fix existing E2E tests for new semantics

**Out of Scope:**

| Feature | Reason |
| --- | --- |
| Mutable borrow exclusivity checking | Deferred to milestone_borrow_hardening |
| New E2E pass/fail tests for borrow model | Deferred to milestone_borrow_hardening |
| Parser snapshot tests | Deferred to milestone_borrow_hardening |
| Multi-module convention tests | Deferred to milestone_borrow_hardening |
| Stdlib `mut` parameter annotations | Deferred to milestone_borrow_hardening |

### Acceptance Criteria

| AC-ID | Criterion |
| --- | --- |
| AC-1 | **Given** `def process(items: list[int]) -> int`, **When** Rust is emitted, **Then** parameter type is `&Vec<i64>` (borrow by default) |
| AC-2 | **Given** `def sort_it(mut items: list[int])`, **When** Rust is emitted, **Then** parameter type is `&mut Vec<i64>` |
| AC-3 | **Given** `def consume(own items: list[int])`, **When** Rust is emitted, **Then** parameter type is `Vec<i64>` (ownership transfer) |
| AC-4 | **Given** `def add(x: int, y: int) -> int`, **When** Rust is emitted, **Then** parameter types are `i64` (Copy types pass by value) |
| AC-5 | **Given** a call `process(my_list)`, **When** Rust is emitted, **Then** call emits `process(&my_list)` |
| AC-6 | **Given** a call `sort_it(my_list)`, **When** Rust is emitted, **Then** call emits `sort_it(&mut my_list)` |
| AC-7 | **Given** a call `consume(my_list)`, **When** Rust is emitted, **Then** call emits `consume(my_list)` and `my_list` is marked moved |
| AC-8 | **Given** the hardcoded `borrows_args` list in `lower.rs`, **When** milestone is complete, **Then** the list is deleted |
| AC-9 | **Given** `FunctionType` and `Callable` types, **When** conventions are set, **Then** they survive cross-module import/export |
| AC-10 | **Given** all existing E2E pass tests, **When** `cargo test` is run, **Then** all tests pass (with adjustments for new semantics) |

## 2. Solution Design

### 2.1 Functional Requirements

- `ParamConvention` enum with `Borrow`, `MutBorrow`, `Own` variants
- Parser recognizes `mut`/`own` before parameter names using single-token lookahead
- Convention propagates: Parser → AST → HIR → Codegen
- `FunctionType.params` becomes `Vec<(String, Type, ParamConvention)>`
- `Callable` type variant carries `Vec<ParamConvention>`
- Codegen emits `&T`/`&mut T`/`T` based on convention
- Call sites emit `&arg`/`&mut arg`/`arg` based on callee conventions
- `func_signatures` carries `Vec<(Type, ParamConvention)>` per function
- Class/static method signatures registered in `func_signatures`
- Borrowed parameter escape (return/store) produces compiler error

### 2.2 High-Level Architecture

```
Parser (mut/own soft keywords)
    ↓
AST (Parameter.convention)
    ↓
HIR (HirParam.convention, convention-aware move tracking)
    ↓
Codegen (func_signatures with conventions, &T/&mut T/T emission)
```

### 2.3 Key Files

| File | Changes |
| --- | --- |
| `crates/sifr_type_system/src/types.rs` | Add `ParamConvention`, extend `FunctionType`, extend `Callable` |
| `crates/sifr_python_ast/src/nodes.rs` | Add `convention` to `Parameter` |
| `crates/sifr_python_parser/src/parser/statement.rs` | Parse `mut`/`own` in `parse_parameter` |
| `crates/sifr_hir/src/hir_nodes.rs` | Add `convention` to `HirParam` |
| `crates/sifr_hir/src/lower.rs` | Propagate conventions, delete `borrows_args`, update all call paths |
| `crates/sifr_codegen/src/lib.rs` | Extend `func_signatures`, emit `&T`/`&mut T`/`T`, convention-aware call emission |

### 2.4 Testing Strategy

| AC-ID | Test Layer | Check |
| --- | --- | --- |
| AC-1 to AC-7 | E2E | Demo programs compile and produce correct Rust output |
| AC-8 | Code review | `borrows_args` match block deleted from `lower.rs` |
| AC-9 | E2E | Multi-module test with `mut`/`own` params across imports |
| AC-10 | CI | `cargo test` passes all existing tests |
