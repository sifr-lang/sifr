# Review: INT-1 SifrInt Lexical Shadowing Pass 1b

## Verdict

Satisfied.

The slice cleanly closes the immediate-scope shadowing gap for exact-int module constants. Both pre-scan analysis and expression rewriting now recognize when a function-local or parameter binding shadows a module constant, and correctly avoid promoting the function's return type to `SifrInt` and avoid rewriting `Ident(name)` to `__const_*()` for shadowed names. The unshadowed path (calls to oversized module constants without a same-named local) continues to lower through `SifrInt` helpers.

## Findings

No blocking findings.

### 1. Pre-scan analysis correctly threads `shadowed_module_bindings`

The new [collect_function_local_shadow_names](crates/sifr_codegen/src/function_emitter.rs:1031-1035) returns the union of locally-defined names (from `collect_locally_defined_vars(&func.body)`) and parameter names. This set is computed once per function and threaded through the relevant pre-scan helpers:

- [hir_function_returns_sifr_int](crates/sifr_codegen/src/function_emitter.rs:885) computes `shadowed_module_bindings` and threads it into both `collect_sifr_int_forced_locals` and `hir_expr_needs_sifr_int_storage`.
- [hir_function_returns_sifr_int_with_extra_forced](crates/sifr_codegen/src/function_emitter.rs:936) does the same.
- [collect_function_sifr_int_forced_locals_with_extra](crates/sifr_codegen/src/function_emitter.rs:992) does the same.
- [collect_sifr_int_call_arg_function_params](crates/sifr_codegen/src/function_emitter.rs:1037) takes the shadow set as a parameter and threads it.
- [register_sifr_int_forced_local_bindings](crates/sifr_codegen/src/function_emitter.rs:108) computes the shadow set from `self.local_binding_types.keys()` (which has been populated by params + body locals at the time of this call).

The new `shadowed_module_bindings` parameter is correctly added to [hir_expr_needs_sifr_int_storage](crates/sifr_codegen/src/function_emitter.rs:1232-1240) where the Name arm now reads:

```rust
HirExpr::Name { name, .. } => {
    forced_locals.contains(name)
        || (module_sifr_int_bindings.contains(name)
            && !shadowed_module_bindings.contains(name))
}
```

This is the load-bearing change: a name in `module_sifr_int_bindings` is treated as SifrInt-shaped only if it's *not* shadowed. The disjunction with `forced_locals.contains(name)` correctly preserves the case where a forced local has the same name (the local takes precedence).

### 2. Rewriter early-returns for local-shadowed names

[rewrite_special_ident](crates/sifr_codegen/src/expr_render_helpers.rs:1289-1294) now starts with:

```rust
fn rewrite_special_ident(&self, name: String) -> crate::RustExpr {
    if self.local_binding_types.contains_key(&name) {
        return crate::RustExpr::Ident(name);
    }
    ...
}
```

This shadows ALL special-ident treatment (stdlib constants, module constants, math constants) when the name is a local binding. For the load-bearing `BIG_LIMIT` case, this means a function with `BIG_LIMIT: int = 5` (local) emits `Ident("BIG_LIMIT")` for body references rather than `__const_BIG_LIMIT()`. ✓

Note that the early-return triggers on *any* local binding, not just `Type::Int`. This is correct because Rust's lexical scoping means the local always shadows the module-level item, regardless of type.

### 3. End-to-end verification

I emitted the new e2e fixture entries:

```
fn shadow_exact_module_constant_with_local() -> i64 {
    let BIG_LIMIT: i64 = 5 as i64;
    return BIG_LIMIT + (1 as i64);
}

fn shadow_exact_module_constant_with_param(BIG_LIMIT: i64) -> i64 {
    return BIG_LIMIT + (1 as i64);
}
```

Both shadow functions correctly:
- Return type: `i64` (not `SifrInt`).
- Local/parameter `BIG_LIMIT: i64`.
- Body: `return BIG_LIMIT + (1 as i64);` — references the local/param directly, no `__const_BIG_LIMIT()` call.

Calls in `main`: `let local_shadow: i64 = ...` and `let param_shadow: i64 = ...` — both stay i64. Round-trip asserts `'6'` and `'6'`. ✓

The existing fixture entries (`returned_big_limit()`, `BIG_LIMIT + 1`, `oversized_local: int = BIG_LIMIT + LIMIT`, etc.) all continue to round-trip with the unshadowed module-helper-rewrite path. ✓

### 4. Probe matrix

| Probe                                                              | Result |
|--------------------------------------------------------------------|--------|
| Local `BIG_LIMIT: int = 5` shadow — fixture entry                  | ✓ i64, no `__const_BIG_LIMIT()` |
| Parameter `BIG_LIMIT: int` shadow — fixture entry                  | ✓ i64, no `__const_BIG_LIMIT()` |
| Unshadowed `BIG_LIMIT` reference (e.g., `BIG_LIMIT + 1` in main)   | ✓ still rewrites to `__const_BIG_LIMIT()` |
| Same-function shadow + arithmetic on it (`BIG_LIMIT + 1` after let) | ✓ uses local i64 throughout |
| Sibling functions, one shadowed and one not                        | ✓ state-isolated (pre-scan threads per-function shadow set) |
| Module helpers (`BIG_LIMIT`) used in non-shadow functions          | ✓ unchanged |
| Pre-existing milestone shapes (BinOp, AugAssign, captures)         | ✓ all still pass; 14 expr_render_helpers tests pass |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1841), confirming no test deltas elsewhere.

### 5. Test coverage

The slice adds focused unit tests:

- [shadowed_module_const_local_does_not_promote_return_to_sifr_int](crates/sifr_codegen/src/function_emitter.rs:1330) — pins that a local-shadowed function isn't promoted by the pre-scan.
- [shadowed_module_const_param_does_not_promote_return_to_sifr_int](crates/sifr_codegen/src/function_emitter.rs:1352) — same for parameter shadow.
- [unshadowed_module_const_still_promotes_return_to_sifr_int](crates/sifr_codegen/src/function_emitter.rs:1373) — control case pinning that without shadow the promotion still fires.
- [local_binding_shadows_large_int_module_const_rewrite](crates/sifr_codegen/src/expr_render_helpers.rs:1603) — pins the rewriter's early-return for shadowed names.

The control case is particularly valuable because it would catch a regression where the shadowing logic over-broadly disables promotion. ✓

The e2e fixture also pins both shadow shapes (`local_shadow`/`param_shadow`) AND continues to exercise unshadowed shapes throughout the rest of the fixture — full coverage of both directions.

## Notes

(Non-blocking observations only.)

### N1 — Nested-scope shadowing is not addressed (pre-PR same)

A nested helper inside an outer function with a shadow doesn't see the outer's shadow:

```sifr
def shadow_with_nested() -> int:
    BIG_LIMIT: int = 5            # outer's shadow
    def helper() -> int:
        return BIG_LIMIT + 1       # helper references BIG_LIMIT
    return helper()
```

Post-PR emits:

```rust
fn shadow_with_nested() -> SifrInt {
    let BIG_LIMIT: i64 = 5 as i64;
    let helper = || {
        return __const_BIG_LIMIT() + SifrInt::from_i64(1);   // <-- module helper, not outer's local
    };
    return helper();
}
```

At runtime this returns `10^20 + 1`, not Sifr's natural lexical-scope answer of `6`. The cause: when the nested fn is emitted via `try_lower_structured_nested_function_stmt`, `local_binding_types` is cleared and re-registered with only helper's params + body locals — outer's shadow `BIG_LIMIT` is no longer visible to the inner rewriter, so `rewrite_special_ident("BIG_LIMIT")` falls through to the module helper.

**Pre-PR-#1843 same code emitted the same module-helper closure body** (the slice's local-shadow check didn't exist, so `BIG_LIMIT` always rewrote to `__const_BIG_LIMIT()`). Not a regression.

The slice's stated scope is "a function-local or parameter binding... must shadow that module constant in both pre-scan analysis and expression rewriting" — which I read as the immediate function's scope. The nested-scope case requires propagating the outer's `local_binding_types` (or specifically the shadow names) into the inner emission context, which is broader work tied to general lexical-capture handling. Worth tracking as a residual gap, possibly under the broader "lexical shadowing and legacy-emission paths" follow-up.

### N2 — The rewriter's early-return shadows ALL special-ident handling

[rewrite_special_ident](crates/sifr_codegen/src/expr_render_helpers.rs:1289) returns immediately if `local_binding_types.contains_key(&name)`, before checking stdlib constants (e.g., `pi`, `e`, `tau`, `inf`, `nan`) or module constants. This is the correct shadowing semantic because Rust would resolve a local-named `pi` to the local, not to `std::f64::consts::PI`. But it does mean any local with a name colliding with a stdlib constant suppresses the special-ident rewrite. This is the desired behavior; just noting that the early-return is broad in scope.

### N3 — `register_sifr_int_forced_local_bindings` reads `local_binding_types.keys()` after the body insertion phase

The shadow set at [function_emitter.rs:123](crates/sifr_codegen/src/function_emitter.rs:123) is computed as `self.local_binding_types.keys().cloned().collect()`. This runs *after* lines 101-103 insert the body's bindings via `or_insert`, so by line 123 the map has both params (registered earlier in `register_function_scope_params`) and body locals. ✓

The order of operations is correct, but it depends on the existing convention that `register_function_scope_params` runs before `register_local_body_binding_types`. A future refactor that swaps these could introduce a subtle bug where `shadowed_module_bindings` doesn't see params yet. A short comment explaining the dependency might help future contributors.

### N4 — Carry-forward open items unchanged

The slice intentionally does not touch fallible `//` and `%` runtime/codegen support. With this slice closing the lexical-shadowing gap, the open INT-1 follow-up shrinks to just "unsupported augmented assignment / fallible `//` and `%`". The next tracker PR should reflect that.

### N5 — Validation notes are adequate

`cargo fmt --check`, `git diff --check`, `cargo check -p sifr_codegen`, focused `cargo test -p sifr_codegen shadowed_module_const`/`local_binding_shadows_large_int_module_const_rewrite`, e2e emit and run, and `scripts/run_all_tests.sh --profile quick` are reported. The `report_signature=e1bf653aaa770517` matches all prior milestone PRs, confirming no test deltas elsewhere.
