# Review: INT-1 SifrInt Function Parameter Boundaries Pass 1

**Verdict: Blockers remain. Changes requested.**

The slice's pre-scan correctly discovers which module-level function parameters need promotion, the `lower_module_function_param_type` correctly emits `SifrInt` for promoted positions, and `register_function_scope_params` correctly registers promoted parameter names as inner SifrInt locals so the body coerces uses of them. The e2e fixture's module-helper-and-literal call shapes round-trip cleanly.

But the slice has one reachable correctness blocker: when a call site passes a **registered SifrInt local** (e.g., `big: int = BIG_LIMIT + 1` then `echo(big)`) to a promoted parameter, the codegen emits `echo(SifrInt::from_i64(big.clone()))` — a `from_i64` wrap around an already-`SifrInt` value, which fails rustc with `expected i64, found SifrInt`. This is the slice's stated primary case ("function argument expressions that are already `SifrInt` need uniform parameter lowering"), and it's broken for the load-bearing local-source shape.

The cause is double-application of `coerce_expr_to_sifr_int_value` across two code paths, with the intermediate `Clone(Ident)` shape unrecognized by `is_sifr_int_expr`. Fix is small (one match arm).

## Findings

### B1 — Blocker: `echo(big)` for registered SifrInt local emits invalid Rust

**Reproduction:**

```sifr
BIG_LIMIT: int = 10 ** 20

def echo(value: int) -> int:
    return value

def main():
    big: int = BIG_LIMIT + 1
    a: int = echo(big)
    print(str(a))
```

Post-PR emits:

```rust
fn echo(value: SifrInt) -> SifrInt {
    return value.clone();
}

fn main() {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let a: SifrInt = echo(SifrInt::from_i64(big.clone()));   // E0308: expected i64, found SifrInt
    …
}
```

`rustc` rejects the call site:

```
error[E0308]: mismatched types
  --> src/main.rs:20:45
   |
20 |     let a: SifrInt = echo(SifrInt::from_i64(big.clone()));
   |                           ----------------- ^^^^^^^^^^^ expected `i64`, found `SifrInt`
```

Same shape repros with chained locals (`big1: int = BIG_LIMIT + 1; big2: int = big1 + 1; echo(big2)` → `echo(SifrInt::from_i64(big2.clone()))`).

**Pre-PR-#1841 baseline:** `fn echo(value: i64) -> i64 { return value; }` and `echo(big)` failed with `expected i64, found SifrInt` at the call site. Both versions fail to compile, so this is **not a strict regression** — but the slice's stated primary scope is "function argument expressions that are already `SifrInt` need uniform parameter lowering instead of legacy `i64`", and the fix doesn't deliver for the registered-local case.

**Cause — double-application of `coerce_expr_to_sifr_int_value`:**

For a stmt like `let a: int = echo(big)`, the call goes through two coerce passes:

1. **First pass** at [adapt_plain_call_args_with_signature_for_ir](crates/sifr_codegen/src/stmt_support_emitter.rs:5216-5219):

   ```rust
   if self.function_param_lowers_to_sifr_int(func, idx) {
       let lowered_arg = self.rewrite_stdlib_constant_idents_in_expr(lowered_arg);
       adapted.push(self.coerce_expr_to_sifr_int_value(lowered_arg));
       continue;
   }
   ```

   For `Ident("big")` (registered SifrInt local), this produces `Clone(Ident("big"))` via the value-position coerce's first match arm.

2. **Second pass** at [the FnCall arm in rewrite_stdlib_constant_idents_in_expr](crates/sifr_codegen/src/expr_render_helpers.rs:259-279):

   ```rust
   crate::RustExpr::FnCall { func, args } => {
       let func = self.rewrite_stdlib_constant_idents_in_expr(*func);
       let args = if let Some(func_name) = rust_expr_identifier_path(&func) {
           args.into_iter()
               .enumerate()
               .map(|(idx, arg)| {
                   let arg = self.rewrite_stdlib_constant_idents_in_expr(arg);
                   if self.function_param_lowers_to_sifr_int(&func_name, idx) {
                       self.coerce_expr_to_sifr_int_value(arg)
                   } else {
                       arg
                   }
               })
               .collect()
       …
   }
   ```

   This runs on the already-coerced `Clone(Ident("big"))`. `coerce_expr_to_sifr_int_value(Clone(Ident("big")))`:

   - Doesn't match `Ident` arm (it's `Clone`, not `Ident`).
   - Doesn't match `Paren`, `BinOp` arms.
   - Falls to `other if is_sifr_int_expr(&other)` — but [is_sifr_int_expr](crates/sifr_codegen/src/expr_render_helpers.rs:1392-1410) has no `Clone(...)` arm, so it returns false (wildcard).
   - Falls to `Cast { ty: I64, expr } => from_i64(...)` — doesn't match (it's Clone).
   - Falls to `other => sifr_int_from_i64_expr(other)` — wraps `Clone(Ident("big"))` in `SifrInt::from_i64(...)`.

   Result: `SifrInt::from_i64(big.clone())` — invalid because `from_i64` expects `i64`.

**Why module-helper and literal call shapes work** (and the e2e fixture passes):

- `echo(BIG_LIMIT)`: arg is `__const_BIG_LIMIT()` (FnCall to module helper). First coerce: `is_sifr_int_expr(FnCall)` → true via `is_sifr_int_module_constant_func` → pass through. Second coerce: same → pass through. ✓
- `echo(5)`: arg is `Cast(Literal(5), I64)`. First coerce: matches `Cast { ty: I64 }` → `from_i64(5)`. Second coerce: `is_sifr_int_expr(FnCall(SifrInt::from_i64))` → true via the from_i64 path arm → pass through. ✓
- `echo(echo(BIG_LIMIT))`: nested SifrInt-returning call. First coerce passes through (FnCall to promoted echo). Second coerce same. ✓

Only the **registered local** (Ident) case hits the Clone shape, which the wildcard path mishandles.

**Suggested fix** (smallest, most local):

Add a `Clone(expr)` arm to `is_sifr_int_expr` that recurses on the inner expression:

```rust
// In is_sifr_int_expr, add:
crate::RustExpr::Clone(expr) => self.is_sifr_int_expr(expr),
```

With this arm, `Clone(Ident(registered_local))` recurses into `Ident("big")` → registered → true. Then the second coerce's `other if is_sifr_int_expr(&other)` arm fires and passes the `Clone(Ident)` through unchanged. The final emit would be `echo(big.clone())`, which is well-typed because `big.clone()` is `SifrInt` and `echo` takes `SifrInt`.

Alternative fix (slightly more involved): unify the call-arg coercion to fire in only one path. Currently the rewriter's FnCall arm and `adapt_plain_call_args_with_signature_for_ir` both apply `coerce_expr_to_sifr_int_value`. Choose one.

A **regression test** for the load-bearing shape would also be needed — e.g., add to `module_constants.sifr`:

```sifr
echoed_local_parameter: int = echo_int_parameter(big_local_already_in_fixture)
assert str(echoed_local_parameter) == '...'
```

This would have caught the bug.

## Notes

(Non-blocking observations only — these can be addressed alongside or after the B1 fix.)

### N1 — Pre-scan structure is sound

[register_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:135-200) now does a fixed-point loop that interleaves return-type discovery and parameter-promotion discovery. The structure is correct:

1. Discover SifrInt-returning functions (using the current `function_params` map to seed forced-locals analysis via `extra_forced_params`).
2. For each function, discover which call-site arguments are SifrInt-shaped, mark the corresponding callee's parameter index as promoted.
3. Loop until both `function_returns` and `function_params` converge.

The combined termination check `if function_returns.len() == before && after_params == before_params` correctly waits for both maps to stabilize.

I traced `echo_int_parameter(BIG_LIMIT)` and `add_to_exact_parameter(BIG_LIMIT, 3)`:
- iter 1: `echo_int_parameter` and `add_to_exact_parameter` analyzed for returns. `value` (echo's idx 0) and `value` (add's idx 0) marked promoted via call-arg detection in main's body (since `BIG_LIMIT` is module SifrInt source).
- iter 2: With promoted params seeded into forced, `echo_int_parameter`'s return `value` is now SifrInt-forced, so echo gets added to function_returns. Same for add.
- iter 3: function_returns stable, function_params stable. Break.

✓ Fixed-point converges correctly.

### N2 — Per-position parameter promotion is precise

`add_to_exact_parameter(BIG_LIMIT, 3)` correctly promotes only `value` (idx 0), leaving `offset` (idx 1) as `i64`:

```rust
fn add_to_exact_parameter(value: SifrInt, offset: i64) -> SifrInt {
    return &value + SifrInt::from_i64(offset);
}
```

Same for `add_right_exact_parameter(3, BIG_LIMIT)` which promotes only `value` (idx 1). ✓ The slice doesn't over-promote when only a subset of args at a call site are SifrInt-shaped.

### N3 — Body coercion of promoted parameter uses works

For `add_to_exact_parameter`'s body `return value + offset`, the body emit produces `&value + SifrInt::from_i64(offset)`. `value` is registered (via `register_function_scope_params`'s new SifrInt-aware branch at [function_emitter.rs:65-69](crates/sifr_codegen/src/function_emitter.rs:65)), so the BinOp arm coerces `value` to `&value` and wraps `offset` in `from_i64`. ✓

For `alias_exact_parameter`'s body `alias: int = value; return alias + 1`, the Let retypes `alias` to SifrInt (via `coerce_expr_to_sifr_int_value` of registered-local `Ident("value")` → `Clone(Ident)` → `value.clone()`), and the return BinOp coerces `alias` correctly. ✓

### N4 — Return-boundary, nested-helper, capture, and closure-return-state behavior preserved

I re-ran the e2e fixture in full. All 14 expr_render_helpers tests pass; all earlier milestone shapes (BinOp arithmetic, BinOp comparison, AugAssign, value-semantic alias, function returns, nested helpers, recursive captures, closure return-state) still emit correctly. No regression in those areas.

### N5 — Carry-forward open items unchanged

Lexical shadowing, legacy-emission paths, fallible `//` and `%` — all stay tracked. Once B1 is closed, this slice closes the "function argument expressions that are already `SifrInt` need uniform parameter lowering" residual; the next tracker should mark that complete.

### N6 — No focused unit tests added

The e2e fixture covers four parameter-promotion shapes (`echo_int_parameter`, `add_to_exact_parameter`, `add_right_exact_parameter`, `alias_exact_parameter`) but **not** the failing shape (passing a registered SifrInt local). Adding both:

- A unit test that pre-stages `function_param_lowers_to_sifr_int` and `sifr_int_local_bindings`, then asserts the call rewriter emits `func(big.clone())` instead of `func(SifrInt::from_i64(big.clone()))`.
- An e2e fixture line: `echoed_local_parameter: int = echo_int_parameter(reusable_oversized_local)` (using an already-defined registered local from the fixture) plus a matching assert.

…would have caught B1 and would harden against future regressions.

## What I'd accept

- B1 fix: add `Clone(expr) => self.is_sifr_int_expr(expr)` arm to `is_sifr_int_expr` in [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs:1392). This makes `Clone(Ident registered)` recognized as SifrInt-shape, so the second coerce pass-through arm fires instead of the from_i64 fallback.
- Add a regression e2e assertion exercising the registered-local-arg case (e.g., `echo_int_parameter(reusable_oversized_local)` with an `assert str(...) == '100000000000000000001'`).
- The N1/N2/N3 mechanisms are sound and don't need changes.

Once B1 is addressed and a regression test pins it, I'd flip the verdict to "Satisfied".
