The clippy warning on `is_sifr_int_checked_floor_option_expr` is pre-existing code, not from this slice. All clippy warnings are pre-existing in `sifr_hir` or unrelated code.

---

## Review Verdict: SATISFIED

The INT-1 class method result-return slice is **correct and acceptable for merge**. Here's my analysis by concern:

### 1. Fixed-point discovery — sound and terminates ✅

The loop in `register_sifr_int_function_returns` (function_emitter.rs:161-265) now tracks `result_method_returns` alongside the four existing counters (`function_returns`, `result_function_returns`, `function_params`, `result_function_params`). The termination check at lines 257-264 correctly includes `result_method_returns.len() == before_result_methods`. Each iteration can only add entries (never remove), sets are bounded by module contents, so termination is guaranteed. Method discovery is monotonic: it can trigger function param discovery, which can trigger more method discovery, but the dependency graph is acyclic by construction.

### 2. Resolving method calls by receiver class — sound enough ✅

- **`hir_expr_class_name`** (function_emitter.rs:1271): Resolves via `expr.ty()` — the HIR type annotation set by the type checker. For `self.divide()` the receiver `self` has type `Calculator`, so `hir_expr_class_name` returns `Some("Calculator")`. This is the authoritative source.
- **`rust_expr_class_name`** (expr_render_helpers.rs:1609): Limited to `Ident` (including `self` via `current_class_name`) and `Paren`. Falls through `None` for field access / subscript receivers. This is intentional and appropriate for the codegen layer — it handles the cases that actually appear in HIR-lowered Rust (simple identifiers and parenthesized variants), while complex receivers go through different lowering paths.

### 3. Class method emission saves/restores all needed state ✅

In `lower_class_method_item` (class_method_emitter.rs:501-624):
- Saves and restores `sifr_int_result_local_bindings` (lines 504-505, 621-622)
- Saves and restores `current_sifr_int_result_return` (lines 506, 623-624)
- Sets `current_sifr_int_result_return` at entry based on `sifr_int_result_method_returns` (lines 517-521)
- `lower_class_method_return_type` checks the promoted set and returns `Result<SifrInt, E>` (lines 262-269)

This mirrors the pattern used for `current_sifr_int_return` in nested functions (function_emitter.rs lines 424-425, 499-502).

### 4. Interactions with module functions, locals, method-to-method calls ✅

- `hir_expr_returns_sifr_int_result` (function_emitter.rs:1226) now handles `HirExpr::MethodCall` by looking up `result_method_returns`. This flows through `collect_nested_sifr_int_result_function_returns`, `collect_sifr_int_result_local_bindings_with_initial`, `collect_sifr_int_result_call_arg_function_params`, and `function_returns_result_sifr_int`.
- `calc.divide_again(...)` → `self.divide(...)` chain: `Calculator::divide` is promoted (direct floor division), `Calculator::divide_again` is promoted (calls promoted `self.divide`), the call site `calc.divide_again(21, 7)` correctly returns `Result<SifrInt, DivisionError>` which unwraps to `SifrInt`.
- Module functions and local aliases interact correctly — they're tracked in separate sets.

### 5. Test coverage ✅

The e2e test covers:
- `Calculator.divide()` — direct `a // b` → `Result<SifrInt, E>`
- `Calculator.divide_again()` — method-to-method propagation via `self.divide(a, b)`
- Try unwrapping `calc.divide_again(21, 7)` asserting result value

The emitted Rust confirms correct signatures for both methods and all module functions/local bindings.

### Minor note (non-blocking)

`rust_expr_class_name` returns `None` for method calls on field access (`obj.method()`) or subscript receivers. This is a limitation but not a blocker: the primary case (`self.method()` and `variable.method()` where `variable: ClassName`) is handled. Complex receivers go through the non-promoted path, which is safe (produces `Result<i64, E>` but that's already the legacy behavior). A future extension could add field/subscript support if needed.
