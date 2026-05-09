

## INT-1 Field Receiver Result Promotion Review

### Verdict: SATISFIED

The slice is correct and ready for INT-1. No blockers found.

---

### Correctness

**Generated output is correct.** `Holder.divide_field` correctly emits:
```rust
let result: Result<SifrInt, DivisionError> = self.calc.clone().divide(a, b);
```

The chain `self.calc.clone().divide(...)` is recognized as a `Calculator` method returning `Result<SifrInt, DivisionError>` via:
1. `rust_expr_class_name(self.calc.clone())` → resolves `self.calc` → `Calculator` via `class_field_types`
2. `is_sifr_int_result_returning_method_call(receiver, "divide")` → checks `(Calculator, divide)` in `sifr_int_result_method_returns`

**Field receiver resolution is sound.** `RustExpr::Field { expr, field }` correctly recurses: `rust_expr_class_name(self.calc)` → `rust_expr_class_name(self)` → `"Holder"` → `class_field_types[(Holder, "calc")] = Type::Class(Calculator)`.

### Clone() Unwrapping Scope

Tight and correct. Matches only the auto-inserted clone: `MethodCall { method: "clone", args: empty }`. Any other `.clone()` usage (e.g., programmer-written on a non-Sifr type) would produce `None` from `rust_expr_class_name` and fall through to regular codegen.

### class_field_types Population

- **Module classes**: Populated in `detect_recursive_fields` (line 86-87) — runs once per module emit.
- **Imported/external classes**: Populated in `register_external_class_fields` (line 172-175) — called for each stdlib import.
- **Staleness risk**: LOW. Both sources are deterministic from source declarations. The emitter does not modify class_field_types after population — no invalidation needed.

No other code path writes to `class_field_types`, so no stale cross-module contamination risk.

### Missing Cases (Non-Blocking)

1. **Chained fields** (e.g., `self.outer.inner.method()`): Would require an additional recursive step in `rust_expr_class_name`. Not covered by this slice, not part of the original gap, can be added if needed.
2. **Nested method calls on fields** (e.g., `self.calc.get_calculator().divide(...)`): Would require `MethodCall` in `rust_expr_class_name` to resolve the intermediate method's return type — not covered, not in scope.

These are out-of-scope gaps, not regressions.

### Test Coverage

- `Holder.calc: Calculator` + `Holder.divide_field` covers the exact gap scenario.
- 584 unit tests pass.
- 24 e2e pass tests in quick lane pass.
- 22 pre-existing unit test failures are baseline (unchanged from main branch).

### CI Signal

- `cargo fmt --check`: Clean
- `cargo check -p sifr_codegen`: Clean  
- Clippy warnings in `sifr_hir` are pre-existing (unrelated to this change)

### Summary

The slice is focused, correct, and sound. It closes the field-receiver gap that was identified during INT-1 review. No blockers. **Reviewer is satisfied.**
