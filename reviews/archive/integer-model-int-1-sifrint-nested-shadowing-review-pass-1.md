# Review: INT-1 SifrInt Nested Shadowing Pass 1

## Verdict

Satisfied.

The slice closes the single-level nested-helper shadowing gap from PR #1843's pass-1b N1. Three coordinated mechanisms cooperate to propagate outer shadows into nested helper analysis and emission: (1) shadow-aware variants of the pre-scan (`hir_function_returns_sifr_int_with_extra_forced_and_shadowed`, `collect_function_sifr_int_forced_locals_with_extra_and_shadowed`, and the threaded `outer_shadowed_module_bindings` parameter through `collect_nested_sifr_int_function_returns`), (2) `recursive_capture_lowers_to_sifr_int` excludes module-helper captures that are shadowed by an outer local, and (3) `try_lower_structured_nested_function_stmt` inserts captured shadow names into the inner scope's `local_binding_types` so the rewriter's PR #1843 early-return suppresses `__const_*()` rewrites in the nested body.

I flag one notable non-blocking observation (N1): multi-level nesting (helper inside helper) doesn't propagate the shadow because `collect_referenced_vars_with_types` uses `LOCAL_SCOPE_ONLY` traversal which doesn't descend into nested function bodies. Pre-PR same; out of stated slice scope ("nested helper" singular, e2e fixture only tests single-level).

## Findings

No blocking findings.

### 1. Shadow propagation through pre-scan layers

The slice introduces `_with_extra_forced_and_shadowed` variants of the pre-scan helpers ([function_emitter.rs:949](crates/sifr_codegen/src/function_emitter.rs:949), [function_emitter.rs:1019](crates/sifr_codegen/src/function_emitter.rs:1019)) that take both `extra_forced_locals` and `extra_shadowed_module_bindings` parameters. The original `_with_extra_forced` variants delegate to the new `_and_shadowed` variants with an empty `extra_shadowed` set, preserving backward compatibility.

[collect_nested_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:1146) now takes `outer_shadowed_module_bindings` and threads it to nested helper analysis:

```rust
let captured_shadowed = collect_sifr_int_captured_shadowed_module_bindings(
    func,
    outer_shadowed_module_bindings,
);
... && hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
    func,
    module_sifr_int_bindings,
    &function_returns,
    &captured_forced,
    &captured_shadowed,
)
```

This ensures that when outer's pre-scan walks its body for nested-helper returns, each nested helper's own analysis receives the captured-shadow set. Inside the nested helper's analysis, the shadow set extends `collect_function_local_shadow_names(func)`:

```rust
let mut shadowed_module_bindings = collect_function_local_shadow_names(func);
shadowed_module_bindings.extend(extra_shadowed_module_bindings.iter().cloned());
```

So the helper's body analysis correctly recognizes captured outer shadows when checking `hir_expr_needs_sifr_int_storage` for `Name(BIG_LIMIT)`. ✓

### 2. `recursive_capture_lowers_to_sifr_int` shadow check

[function_emitter.rs:234-241](crates/sifr_codegen/src/function_emitter.rs:234) adds a `&& !self.local_binding_types.contains_key(&capture.name)` gate to the module-source branch:

```rust
matches!(...Type::Int)
    && (self.module_sifr_int_bindings().contains(&capture.name)
        && !self.local_binding_types.contains_key(&capture.name)
        || self.is_registered_sifr_int_local(&capture.name)
        || self.is_forced_sifr_int_local(&capture.name))
```

Operator precedence reads as `(A && !B) || C || D`, which is the intended semantic: a recursive nested helper's hidden capture parameter for a module-source name is promoted to `SifrInt` only when the name is NOT shadowed by an outer local. ✓

I verified end-to-end with the recursive shadow case:
```rust
fn shadow_exact_module_constant_with_recursive_nested_local() -> i64 {
    let BIG_LIMIT: i64 = 5 as i64;
    fn helper(remaining: i64, BIG_LIMIT: i64) -> i64 { ... }    // <-- i64, not SifrInt
    return helper(2 as i64, BIG_LIMIT);                          // <-- passes outer's i64 5
}
```

The capture parameter `BIG_LIMIT: i64` correctly stays i64. ✓ Round-trips with `'8'` (5 + 1 + 2 = 8).

### 3. Inner-scope `local_binding_types` insert for captured shadows

[function_emitter.rs:381-385](crates/sifr_codegen/src/function_emitter.rs:381) inserts the captured shadow names into the inner scope's `local_binding_types` after the clear:

```rust
for name in &captured_shadowed_module_bindings {
    self.local_binding_types.insert(name.clone(), Type::Int);
}
```

This is the load-bearing change for the rewriter behavior. When the inner closure body's `BIG_LIMIT` Ident goes through `rewrite_special_ident`, PR #1843's early-return `if self.local_binding_types.contains_key(&name)` returns true → the Ident stays as-is → Rust's lexical scoping resolves it to the outer's local 5.

I verified with the e2e fixture's three new entries:

```rust
fn shadow_exact_module_constant_with_nested_local() -> i64 {
    let BIG_LIMIT: i64 = 5 as i64;
    let helper = || {
        return BIG_LIMIT + (1 as i64);     // <-- outer's local, no __const_BIG_LIMIT()
    };
    return helper();
}

fn shadow_exact_module_constant_param_with_nested(BIG_LIMIT: i64) -> i64 {
    let helper = || {
        return BIG_LIMIT + (1 as i64);     // <-- outer's parameter, no __const_BIG_LIMIT()
    };
    return helper();
}
```

Both round-trip with `'6'`. ✓

### 4. Order change in `emit_function`

The diff swaps the order at [function_emitter.rs:777-781](crates/sifr_codegen/src/function_emitter.rs:777):

```rust
self.register_function_scope_params(&func.name, &func.params);
let active_function_returns = self.function_sifr_int_returns_for_body(&func.body);
*self.sifr_int_function_returns.borrow_mut() = active_function_returns;
self.register_local_body_binding_types(&func.body);
```

`register_function_scope_params` now runs BEFORE `function_sifr_int_returns_for_body`. This is necessary so that `local_binding_types` includes parameters when `function_sifr_int_returns_for_body` builds its shadow set:

```rust
let mut shadowed_module_bindings = self.local_binding_types.keys().cloned().collect::<HashSet<_>>();
shadowed_module_bindings.extend(collect_locally_defined_vars(body));
```

Without this order change, parameter shadows wouldn't propagate to the nested-helper analysis. The reordering correctly makes the parameter-shadow case work, as verified by `shadow_exact_module_constant_param_with_nested`. ✓

### 5. `collect_captured_outer_names` refactor

The previous `collect_sifr_int_captured_forced_locals` was generalized into `collect_captured_outer_names(func, outer_names)` ([function_emitter.rs:1207-1228](crates/sifr_codegen/src/function_emitter.rs:1207)) so both `collect_sifr_int_captured_forced_locals` and the new `collect_sifr_int_captured_shadowed_module_bindings` delegate to it. Cleaner refactor; no semantic change for the existing forced-locals callers.

### 6. Probe matrix verified

| Probe                                                               | Result |
|---------------------------------------------------------------------|--------|
| Non-recursive nested helper capturing outer-local shadow — fixture  | ✓ i64 throughout, no `__const_BIG_LIMIT()` |
| Recursive nested helper capturing outer-local shadow — fixture      | ✓ i64 capture param, i64 return, i64 throughout |
| Non-recursive nested helper inside outer with parameter shadow — fixture | ✓ uses param i64, no `__const_BIG_LIMIT()` |
| Unshadowed nested helper still uses module SifrInt (probe)          | ✓ unchanged from PR #1843 |
| Sibling functions (one shadow, one not) state isolation             | ✓ each compiles per its own shadow status |
| Mixed shadowed BIG_LIMIT + unshadowed small const                   | ✓ both i64, no spurious promotion |
| Mixed recursive: shadow vs unshadowed siblings                      | ✓ correct per-function promotion |
| Pre-existing milestone shapes (BinOp, AugAssign, captures, returns) | ✓ all still pass; e2e fixture round-trips |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1843), confirming no test deltas.

### 7. Test coverage

The slice retains and extends focused unit tests:

- `shadowed_module_const_local_does_not_promote_return_to_sifr_int` (from PR #1843).
- `shadowed_module_const_param_does_not_promote_return_to_sifr_int` (from PR #1843).
- `unshadowed_module_const_still_promotes_return_to_sifr_int` (from PR #1843, control case).
- [nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int](crates/sifr_codegen/src/function_emitter.rs:1455) — pins the new behavior: a nested helper whose body references an outer-shadow `BIG_LIMIT` doesn't promote the outer's return to SifrInt.

Plus three new e2e fixture entries (non-recursive nested, recursive nested, parameter-shadow + nested) with matching asserts.

Coverage is adequate for the stated single-level nesting scope.

## Notes

(Non-blocking observations only.)

### N1 — Multi-level nesting (helper inside helper) doesn't propagate shadow

A doubly-nested case shows the depth limit:

```sifr
def outer() -> int:
    BIG_LIMIT: int = 5
    def middle() -> int:
        def inner() -> int:
            return BIG_LIMIT + 1
        return inner()
    return middle()
```

Post-PR emits:

```rust
fn outer() -> SifrInt {                                 // <-- promoted (incorrect)
    let BIG_LIMIT: i64 = 5 as i64;
    let middle = || {
        let inner = || {
            return __const_BIG_LIMIT() + SifrInt::from_i64(1);   // <-- module helper, not outer's 5
        };
        return inner();
    };
    return middle();
}
```

Runtime returns `10^20 + 1`, not the lexical-scope answer `6`.

Cause: [collect_referenced_vars_with_types](crates/sifr_codegen/src/hir_analysis/queries.rs:394) uses `TraversalConfig::LOCAL_SCOPE_ONLY` which does NOT descend into nested function bodies. So when `collect_captured_outer_names(middle, outer_shadowed={BIG_LIMIT})` walks middle's body, it doesn't see `BIG_LIMIT` (only referenced inside inner's body). middle's `captured_shadowed_module_bindings` ends up empty. middle's analysis runs without the BIG_LIMIT shadow. inner's analysis (called from middle's analysis with `extra_shadowed=empty`) likewise doesn't see the shadow.

**Pre-PR-#1845 same code** emitted the same module-helper inner body — multi-level shadow propagation has never worked. **Not a regression.**

The slice's stated scope is "nested helper" (singular); the e2e fixture only tests single-level nesting. Multi-level is a remaining gap that would require either (a) using `INCLUDE_NESTED_FUNCTIONS` in `collect_referenced_vars_with_types` for the shadow-discovery walk, or (b) explicitly recursing into nested function bodies during shadow capture analysis. Worth tracking as a residual gap in the next tracker.

### N2 — The slice's mechanism is narrow but well-targeted

The four coordinated changes (pre-scan threading, `recursive_capture_lowers_to_sifr_int` gate, inner-scope `local_binding_types` insert, and `emit_function` order swap) work together to make single-level shadow propagation robust. Each change is small and focused. The generalization of `collect_captured_outer_names` is a nice side effect — both forced-locals and shadow-bindings now share one capture-finding helper.

### N3 — Test coverage matrix

The fixture covers three single-level shapes. A focused unit test for the recursive-capture shadow case (analogous to the existing `nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int`) would harden against future regressions, but the e2e fixture's `shadow_exact_module_constant_with_recursive_nested_local` already pins the runtime shape. Optional polish.

### N4 — Operator precedence in `recursive_capture_lowers_to_sifr_int`

The new guard reads:

```rust
self.module_sifr_int_bindings().contains(&capture.name)
    && !self.local_binding_types.contains_key(&capture.name)
    || self.is_registered_sifr_int_local(&capture.name)
    || self.is_forced_sifr_int_local(&capture.name)
```

Without parentheses, this evaluates as `(A && !B) || C || D`, which is the intended semantic. The semantic is correct, but the reading could be sharpened by adding explicit parens around the first conjunction:

```rust
(self.module_sifr_int_bindings().contains(&capture.name)
    && !self.local_binding_types.contains_key(&capture.name))
    || self.is_registered_sifr_int_local(&capture.name)
    || self.is_forced_sifr_int_local(&capture.name)
```

Functionally identical; just a readability suggestion. Optional.

### N5 — Carry-forward open items

This slice closes the nested-helper lexical shadowing case. The remaining open INT-1 follow-up shrinks to:
1. Multi-level (helper-inside-helper) shadow propagation (per N1) — could be bundled with future broader-migration work.
2. Unsupported augmented assignment / fallible `//` and `%`.

INT-1 is now very close to closure.
