# Review: INT-1 SifrInt Function Call Args and Closure Return State Pass 1

## Verdict

Satisfied.

The slice closes both pass-1 follow-ups from the function-return-boundaries review (#1829's pass-1 N1 closure leak and N2 parametrized-fn call-site asymmetry) cleanly, with three coordinated changes that fit the slice's stated narrow scope. End-to-end and probe matrix verification confirm the load-bearing cases work, prior cases stay correct, and the broader function-boundary migration remains explicitly open.

## Findings

None blocking.

### 1. `Cell<bool>` conversion is appropriate

Changing `current_sifr_int_return: bool` to `Cell<bool>` ([lib.rs:1219](crates/sifr_codegen/src/lib.rs:1219), [lib.rs:1322](crates/sifr_codegen/src/lib.rs:1322)) is the right interior-mutability primitive:

- `bool` is `Copy`, so `Cell::get()`/`set()` work without overhead. No `RefCell` runtime borrow tracking needed.
- The motivation is that `rewrite_stdlib_constant_idents_in_*` helpers take `&self`, so the new closure save/clear/restore in their Closure/ClosureBlock arms needs to mutate the flag without escalating to `&mut self`. `Cell` localizes the mutability without propagating up the entire rewrite call chain.
- All call sites in `function_emitter.rs` updated consistently from direct field access to `.get()`/`.set()` ([function_emitter.rs:227](crates/sifr_codegen/src/function_emitter.rs:227), [function_emitter.rs:238](crates/sifr_codegen/src/function_emitter.rs:238), [function_emitter.rs:273-274](crates/sifr_codegen/src/function_emitter.rs:273), [function_emitter.rs:608](crates/sifr_codegen/src/function_emitter.rs:608), [function_emitter.rs:619-620](crates/sifr_codegen/src/function_emitter.rs:619), [function_emitter.rs:723-724](crates/sifr_codegen/src/function_emitter.rs:723)).
- Read sites in the rewriter at [expr_render_helpers.rs:582](crates/sifr_codegen/src/expr_render_helpers.rs:582) updated to `.get()`.

### 2. Closure save/clear/restore is sound

The Closure and ClosureBlock arms ([expr_render_helpers.rs:399-431](crates/sifr_codegen/src/expr_render_helpers.rs:399)) save the outer's flag, set `false` while emitting the closure body, and restore the saved value. This addresses pass-1 N1 (closure-body Return stmts inheriting promoted outer-function state) by ensuring closure return semantics are independent of the surrounding function.

Setting to `false` unconditionally inside closures is correct because:
- Closures (lambdas, lowered nested `def`) are never in `sifr_int_function_returns` — that set tracks only `module.functions`.
- A closure body's Return stmts return *from the closure*, not from the outer function. The closure's actual return type is inferred from its body, so if the body produces a SifrInt expression naturally (via BinOp/UnaryOp coercion), the closure ends up `Fn() -> SifrInt`. The flag is only used to decide whether to additionally route Return values through `coerce_expr_to_sifr_int_value`, which is a value-position coerce that wraps registered locals in `Clone(...)` — that wrap is correct for outer-function promotion semantics, not for closure semantics.

I verified the pass-1 N1 reproducer is now fixed:

```sifr
def outer() -> int:
    def inner() -> int:
        return 42
    x: int = inner()
    return BIG_LIMIT + x
```

emits

```rust
fn outer() -> SifrInt {
    let inner = || {
        return 42 as i64;          // <-- not coerced, closure flag set to false
    };
    let x: i64 = inner();
    return __const_BIG_LIMIT() + SifrInt::from_i64(x);
}
```

Compiles, runs, prints `100000000000000000042`. ✓

Nested closures cascade the save/restore correctly. Tracing: outer (`true`) → closure A enters (save `true`, set `false`) → closure B enters within A (save `false`, set `false`) → B body emits → restore to `false` → A body emits → restore to `true` → outer continues. Each level's restore is symmetric.

The `RustExpr::Closure` (single-expr) arm save/restore is defensive — a single-expr closure has no explicit Return stmt, so the flag wouldn't be read inside its body. Including the save/restore symmetric with ClosureBlock is harmless and future-proof.

### 3. Recognizing all calls to promoted SifrInt-returning functions

Two changes drop the `args.is_empty()` guard:

- [hir_expr_needs_sifr_int_storage](crates/sifr_codegen/src/function_emitter.rs:836-839) at the HIR pre-scan: now `function_sifr_int_returns.contains(func)` regardless of args.
- [is_sifr_int_expr](crates/sifr_codegen/src/expr_render_helpers.rs:1385-1399) at the Rust IR rewrite: the merged `RustExpr::FnCall` arm now checks `is_sifr_int_returning_function_call` for any args count, while keeping `args.is_empty()` guarding `is_sifr_int_module_constant_func` (which is only correct for zero-arg helpers).

This is sound: a promoted function returns `SifrInt` at the Rust level regardless of how many arguments it takes. Recognizing the call as SifrInt-shaped at the rewriter just informs downstream coercion paths. The arguments themselves are still passed by their existing types (i64 for `int` parameters that haven't been migrated), which is the slice's explicitly-deferred boundary.

I verified end-to-end with several probes:

| Probe                                                       | Emitted Rust                                                                | Result |
|-------------------------------------------------------------|-----------------------------------------------------------------------------|--------|
| `make_big_with_arg(x: int) -> int: return BIG_LIMIT + x`    | `fn make_big_with_arg(x: i64) -> SifrInt { return __const_BIG_LIMIT() + SifrInt::from_i64(x); }` | ✓ |
| Call site `let v: int = make_big_with_arg(5)`               | `let v: SifrInt = make_big_with_arg(5 as i64);`                             | ✓ |
| Multi-arg `make_big_multi(a, b)`                            | `fn make_big_multi(a: i64, b: i64) -> SifrInt { ... }` + `let v: SifrInt = make_big_multi(...);` | ✓ |
| Recursive `promoted_recursive(n)` returning BIG_LIMIT base + recursive case | `fn promoted_recursive(n: i64) -> SifrInt { ... return promoted_recursive(n - 1) + SifrInt::from_i64(1); }` | ✓ |

All compile and round-trip at runtime.

### 4. Scope truthfulness — argument expressions still legacy

The slice description explicitly acknowledges:

> "This is still not a full function-argument migration: function parameters remain legacy-lowered where applicable, so argument expressions that themselves require SifrInt may remain future work."

I verified this is accurate:

```sifr
def make_big_with_arg(x: int) -> int:
    return BIG_LIMIT + x

def main():
    big: int = BIG_LIMIT + 1
    v: int = make_big_with_arg(big)   # passing SifrInt local to int param
```

emits `make_big_with_arg(big)` (no coercion on the arg) which fails rustc because `big: SifrInt` doesn't unify with `x: i64`. **Not a regression** — pre-PR-#1831 (with #1829 in place) the same code failed at *both* the call's arg type *and* the let's return type; post-PR fails only at the arg type. Strict improvement at the call-site recognition level. The slice does not claim to fix this, and the open follow-up bullet at [issues/…/checklist:442](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) correctly tracks "function arguments/non-zero-argument call sites".

### 5. Pre-existing scenarios preserved

I re-emitted the e2e fixture and probed the prior milestone slices' shapes:

- `BIG_LIMIT + 1` still emits `__const_BIG_LIMIT() + SifrInt::from_i64(1)`.
- `reusable_oversized_local + 1` still emits `&reusable_oversized_local + SifrInt::from_i64(1)`.
- `total += big` still emits `total = &total + &big;`.
- `b: int = a` (alias) still emits `let b: SifrInt = a.clone();`.
- Comparison `a < b` still emits `&a < &b`.
- Pure i64 paths (no SifrInt source anywhere) untouched.

All pre-existing assertions in `module_constants.sifr` round-trip.

### 6. Tests

- [rewrites_sifr_int_returning_function_call_with_args_let_type](crates/sifr_codegen/src/expr_render_helpers.rs:1925) — pins the new behavior: a `RustStmt::Let { ty: Some(Named("i64")), value: FnCall{Ident("make_big_with_arg"), [Cast(Literal(3), I64)]} }` rewrites to a SifrInt-typed Let. The choice to use `Named("i64")` (rather than `RustType::I64`) exercises the existing `is_legacy_i64_type` predicate from #1829 against the slice's new arm. ✓

- [closure_block_returns_do_not_inherit_sifr_int_return_state](crates/sifr_codegen/src/expr_render_helpers.rs:1953) — pins both that the ClosureBlock body's `Return(Cast(Literal(42), I64))` stays uncoerced (no `from_i64` wrap) when the outer flag is `true`, AND that the flag is properly restored after the closure rewrite (`assert!(emitter.current_sifr_int_return.get())`). The double assertion is the right pattern — it catches both the local-coerce and the restore-leak shapes simultaneously.

E2E coverage adds:
- `returned_big_with_offset(offset: int) -> int` + call site `returned_big_with_offset(3)` → exercises non-zero-arg call recognition end-to-end.
- `returned_big_with_nested_small() -> int` with a nested `def small_inner() -> int: return 42` → exercises the closure save/restore end-to-end (small_inner's body's `return 42` stays as `return 42 as i64`, and outer's `return BIG_LIMIT + value` coerces correctly).

Both fixture additions round-trip the assertions.

## Notes

(Non-blocking observations only.)

### N-pass2-1 — Pass-1 N3 (defensive save/restore in three other emitter paths) still applies

The slice doesn't add `current_sifr_int_return` save/restore in `function_like_lowering.rs`, `class_emitter.rs`, or `class_method_emitter.rs`. The conversion to `Cell<bool>` makes the future fix even easier (no `&mut self` propagation needed), but it remains a defensive future-proofing note. Currently benign because those paths are entered with `current_sifr_int_return = false` from module emit and don't recursively invoke `emit_function`. Worth tracking, not blocking.

### N-pass2-2 — Nested function with naturally SifrInt-shaped return doesn't propagate to non-promoted outer

A subtle remaining gap surfaced by my probe matrix:

```sifr
def outer() -> int:
    def helper() -> int:
        return BIG_LIMIT + 1
    return helper()
```

The pre-scan walks `module.functions` only, so `helper` (nested inside `outer`) is never checked for promotion. `outer`'s body has `return helper()` which the pre-scan checks via `hir_expr_needs_sifr_int_storage(Call{helper, ...})` → `function_sifr_int_returns.contains("helper")` → `false` (helper isn't in `module.functions`). Outer doesn't get promoted, stays `-> i64`. But helper's closure body has `return BIG_LIMIT + 1` which the BinOp coerces to a SifrInt expression, making helper's inferred type `Fn() -> SifrInt`. Outer's `return helper()` then mismatches `-> i64`.

Pre-PR-#1831 was also broken here (same shape). **Not a regression.** This is the inverse of pass-1 N1 — instead of the outer flag leaking into the closure, the closure's natural SifrInt-shape doesn't propagate up to promote the outer. Both directions need broader function-boundary work to fully close. Worth flagging in the next tracker as a sibling concern alongside the open follow-up's "function arguments/non-zero-argument call sites" bullet.

### N-pass2-3 — Single-expr `Closure` arm save/restore is defensive

The single-expr `Closure` body has no explicit Return stmts, so the flag wouldn't be read inside its body. Including the save/restore symmetric with ClosureBlock is harmless and protects against future shapes (e.g., if a single-expr closure body becomes a Block expression containing stmts), but currently it's dead code. Worth a one-line comment explaining "symmetric with ClosureBlock for future-proofing; single-expr bodies don't currently read the flag". Optional polish.

### N-pass2-4 — Single-expr Closure case isn't unit-tested

The new `closure_block_returns_do_not_inherit_sifr_int_return_state` test pins the ClosureBlock variant. A sibling test for the single-expr `Closure` variant (asserting it also save/restores) would harden against a future regression where someone removes the single-expr arm's save/restore thinking it's unused. Optional.

### N-pass2-5 — Carry-forward open items unchanged

Lexical shadowing, legacy-emission paths, fallible `//` and `%`, and the broader function argument / non-zero-arg call-site migration all stay tracked under the open INT-1 follow-up at [issues/…/checklist:442](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). The next tracker PR should update that bullet to reflect that the closure-leak gap and the parametrized-fn call-site recognition gap are now closed, while keeping the function-argument / arg-expression-with-SifrInt-source gap explicitly open (per N-pass2-2 above).
