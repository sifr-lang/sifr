## Product Requirements & Solution Design

---

### 1. Product Requirements

#### **Title**

Fix Borrow-by-Default Codegen Regressions

---

#### **Objective / Problem Statement**

The borrow-by-default milestone changed function parameters from move-by-default (`T`) to borrow-by-default (`&T`). This introduced two codegen bugs and one outdated audit test that cause regressions across multiple audit suites. These regressions confuse future developers/agents reviewing audit reports and mask the true state of the compiler.

---

#### **Scope**

##### Features In

1. Fix audit test `14_reassignment_resets_move.sifr` to use `own` keyword (test was written for move-by-default)
2. Fix codegen Bug 1: `&String == String` comparisons fail in Rust (need dereference when comparing borrowed str params)
3. Fix codegen Bug 2: Inconsistent `&` prefix at union call sites (missing for non-String enum variants and None)
4. Re-run all audit suites and update reports with clean baseline

##### Features Out

| Feature | Reason for Exclusion |
| --- | --- |
| New audit tests | Out of scope — only fixing regressions |
| Other codegen improvements | Separate milestone |

---

### Acceptance Criteria

| **AC-ID** | Criterion |
| --- | --- |
| AC-1 | `audits/borrowing/14_reassignment_resets_move.sifr` compiles and runs correctly with `own` keyword |
| AC-2 | Functions with borrowed `str` params that compare with string literals compile correctly (no `&String == String` error) |
| AC-3 | Functions with union params receive consistent `&` prefix at call sites for all variant types (Int, Str, Bool, None) |
| AC-4 | All previously-passing audit tests continue to pass (no new regressions) |
| AC-5 | All audit REPORT.md files reflect the current state accurately |

---

## 2. Solution Design

### 2.1 Functional Requirements

* Audit test 14 must use `own` keyword on the `consume` function parameter
* Codegen must dereference borrowed `str` parameters in equality comparisons
* Codegen must emit `&` prefix outside union enum constructors at call sites, not just for String variants

### 2.2 High-Level Architecture

```
Fix 1: audits/borrowing/14_reassignment_resets_move.sifr
  → Add `own` to consume() parameter

Fix 2: crates/sifr_codegen/src/lib.rs (HirExpr::Compare)
  → Detect borrowed Name exprs and emit dereference (*name)

Fix 3: crates/sifr_codegen/src/lib.rs (Call argument emission)
  → Move emit_borrow_prefix OUTSIDE union enum wrapping
```

### 2.3 Detailed Component Design

**Bug 1: `&String == String` comparison**

In `emit_expr` for `HirExpr::Compare`, when the left-hand side is a `Name` expression whose type is `Str` and the variable is a function parameter with `Borrow` convention, the emitted Rust code is `name == "literal".to_string()`. Rust can't compare `&String == String`.

Fix: Before emitting the left side of a comparison, check if the expression is a borrowed reference (Name with Str type that's a function parameter). If so, emit `*` dereference prefix. This is safe because `*&String` gives `String` which implements `PartialEq<String>`.

**Bug 2: Inconsistent `&` at union call sites**

In the Call argument emission (around line 3644-3654), when wrapping an argument in a union enum variant, the code does:
```
emit_borrow_prefix(convention, arg.ty());  // emits & for Borrow
write("EnumName::Variant(");
emit_expr(arg);  // emits the value
write(")");
```

This produces `&IntOrStr::Str("hello".to_string())` for String variants (because `arg.ty()` is `Str` which is Move), but `IntOrStr::Int(42_i64)` for Int variants (because `arg.ty()` is `Int` which is Copy, so `emit_borrow_prefix` skips it).

The fix: The `&` must be applied to the WHOLE expression `&IntOrStr::Int(42_i64)`, not based on the inner arg type. The convention applies to the parameter type (the union), not the argument type (the variant's inner value). So we should check the union type's ownership, not the arg's.

### 2.4 Testing Strategy

| **AC-ID** | Test Layer | Check |
| --- | --- | --- |
| AC-1 | E2E | `cargo run -- run audits/borrowing/14_reassignment_resets_move.sifr` passes |
| AC-2 | E2E | `audits/type_inference/15_infer_from_optional_return.sifr` passes |
| AC-3 | E2E | `audits/type_system/01_basic_unions.sifr` and `29_string_methods_after_narrow.sifr` pass |
| AC-4 | Full audit | All 10 audit suites show no new regressions |
| AC-5 | Manual | REPORT.md files updated |
