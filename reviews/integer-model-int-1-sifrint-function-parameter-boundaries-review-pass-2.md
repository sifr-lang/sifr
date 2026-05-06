# Review: INT-1 SifrInt Function Parameter Boundaries Pass 2

**Verdict: Satisfied. No blockers.**

The pass-1 B1 blocker is closed by the minimum fix I suggested in pass-1's "What I'd accept" section. The pass-2 delta is exactly two changes: one match arm added to `is_sifr_int_expr` to recognize `Clone(expr)` as SifrInt-shape when the inner is SifrInt, and one regression assertion added to the e2e fixture exercising the registered-local-argument shape. Both are minimal, focused, and preserve all earlier milestone shapes.

## Findings

None blocking.

### B1 — Closed

The pass-1 B1 reproducer (`echo(big)` for a registered SifrInt local `big`) was emitting `echo(SifrInt::from_i64(big.clone()))` because `coerce_expr_to_sifr_int_value` was applied twice — first by `adapt_plain_call_args_with_signature_for_ir` (producing `Clone(Ident("big"))`) and then by the FnCall arm in `rewrite_stdlib_constant_idents_in_expr` (where the wildcard fallback wrapped `Clone(Ident)` in `from_i64` because `is_sifr_int_expr` didn't recognize `Clone`).

Pass-2 closes B1 with a one-line addition at [expr_render_helpers.rs:1422](crates/sifr_codegen/src/expr_render_helpers.rs:1422):

```rust
crate::RustExpr::Clone(expr) => self.is_sifr_int_expr(expr),
```

This makes `is_sifr_int_expr(Clone(Ident registered))` recurse into the inner Ident, recognize it as registered → true. Then the second coerce's `other if is_sifr_int_expr(&other)` arm fires and passes the `Clone(Ident)` through unchanged. The final emit is `echo(big.clone())`, which is well-typed because `big.clone()` is `SifrInt` and `echo`'s parameter is `SifrInt` (promoted by the slice's pre-scan).

I reproduced the pass-1 B1 case post-fix:

```sifr
def echo(value: int) -> int:
    return value

def main():
    big: int = BIG_LIMIT + 1
    a: int = echo(big)
    print(str(a))
    print(str(big))
```

Post-PR-#1841-pass-2 emits:

```rust
fn echo(value: SifrInt) -> SifrInt {
    return value.clone();
}

fn main() {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let a: SifrInt = echo(big.clone());      // <-- single-coerced, well-typed
    println!("{}", format!("{}", a));
    println!("{}", format!("{}", big));      // <-- big still usable
}
```

Compiles and runs, prints `100000000000000000001` twice. ✓ `big` retains source-level value semantics (the clone produces an independent SifrInt; `big` is still observable).

The chained-local case (`big1 = BIG_LIMIT + 1; big2 = big1 + 1; echo(big2)`) also works post-fix, emitting `echo(big2.clone())`. ✓

### Probe matrix verified

| Probe                                                                | Result |
|----------------------------------------------------------------------|--------|
| Registered SifrInt local passed to promoted parameter — fixture line | ✓ `echo_int_parameter(reusable_oversized_local.clone())` |
| Chained registered locals (`big1 → big2` → call)                     | ✓ `echo(big2.clone())` |
| Multiple promoted parameters with mixed args (`add_two(big, BIG_LIMIT)`) | ✓ `add_two(big.clone(), __const_BIG_LIMIT())` |
| Same registered local at multiple promoted positions (`add_two(big, big)`) | ✓ `add_two(big.clone(), big.clone())` |
| Nested calls with promoted params (`echo(echo(BIG_LIMIT))`)          | ✓ helper FnCall pass-through |
| Module helper as arg (`echo(BIG_LIMIT)`)                             | ✓ unchanged from pass-1 |
| Literal as arg (`echo(5)`)                                           | ✓ `echo(SifrInt::from_i64(5))` unchanged |
| Source local still usable after `echo(big.clone())` call             | ✓ value-semantic preservation |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1839), confirming no test deltas elsewhere.

### Soundness check on the new `Clone(expr)` arm

The new `is_sifr_int_expr` arm recurses into the inner expression. I verified there are no false positives:

- `Clone(Ident("not_registered"))` — recurse → Ident not in `sifr_int_local_bindings` → false. ✓
- `Clone(FnCall(non_promoted_function))` — recurse → FnCall arm checks against module helpers, promoted functions, and from_i64 paths; for an unrelated function name, all three fail → false. ✓
- `Clone(Cast(literal, I64))` — Cast doesn't match any arm → wildcard → false. ✓ Correct because `Clone(literal_cast)` doesn't represent a SifrInt value.

And no false negatives for the load-bearing case:

- `Clone(Ident registered)` — recurse → registered → true. ✓
- `Clone(FnCall to module helper)` — recurse → helper recognized → true. ✓
- `Clone(FnCall to promoted function)` — recurse → promoted-function-call recognized → true. ✓

The arm is sound and correctly minimal.

### Regression test coverage

The new e2e fixture line at [crates/sifr/tests/e2e/pass/module_constants.sifr:107](crates/sifr/tests/e2e/pass/module_constants.sifr:107):

```sifr
echoed_local_parameter: int = echo_int_parameter(reusable_oversized_local)
```

…with the matching assert at line 112:

```sifr
assert str(echoed_local_parameter) == '100000000000000000001'
```

…exercises exactly the pass-1 B1 reproducer: a registered SifrInt local (`reusable_oversized_local`, set at fixture line 102 via `BIG_LIMIT + 1`) passed to a promoted parameter (`echo_int_parameter`'s `value`). The emitted Rust shows `echo_int_parameter(reusable_oversized_local.clone())` — single-coerced, well-typed. The trailing `assert str(reusable_oversized_local) == '100000000000000000001'` (already in the fixture) pins that the source local stays usable after the call.

This regression test would have caught B1 if it had existed in pass-1.

### Prior pass-1 N1–N6 observations remain accurate

- **N1** Pre-scan structure: still sound, still correctly converges `function_returns` and `function_params` via fixed-point.
- **N2** Per-position promotion: still precise (verified by `add_to_exact_parameter` and `add_right_exact_parameter` in the fixture).
- **N3** Body coercion of promoted parameter uses: still works (verified by `alias_exact_parameter`'s body shape).
- **N4** Return-boundary, nested-helper, capture, and closure-return-state behavior preserved: confirmed by 14 expr_render_helpers tests passing and full e2e fixture round-tripping.
- **N5** Carry-forward open items unchanged.
- **N6** Test coverage now includes the registered-local case (B1 fix paired with regression assertion).

### Notable structural observation

After this slice, the open INT-1 follow-up loses its "function argument expressions that are already `SifrInt`" item. The remaining open items are:
- Lexical shadowing and legacy-emission paths.
- Unsupported augmented assignment / fallible `//` and `%`.

These are non-function-boundary concerns, signaling that the function-boundary sub-phase of INT-1 is now fully closed across return types, parameter types, captures (recursive and non-recursive, module-source and local-source), and call-site coercion.

## Notes

(Non-blocking observations only.)

- **N1 — The minimum-cost fix matches pass-1's "What I'd accept" suggestion.** Adding `Clone(expr) => self.is_sifr_int_expr(expr)` is the smallest possible change that closes B1 without touching the double-coercion path (which would have been more invasive). The diff is two lines (one in production, one in fixture plus assertion). Good targeted change.

- **N2 — The Clone arm sits adjacent to the Ref arm in `is_sifr_int_expr`** ([expr_render_helpers.rs:1421-1422](crates/sifr_codegen/src/expr_render_helpers.rs:1421)), forming a small group of "wrapper" arms that recurse into their inner. Both arms have the same shape (`recurse into inner`), so the pairing reads naturally.

- **N3 — No focused unit test added for the Clone-recognition arm.** The e2e fixture covers the load-bearing shape, but a focused unit test asserting `is_sifr_int_expr(Clone(Ident registered))` returns true (and `is_sifr_int_expr(Clone(Ident unregistered))` returns false) would harden against future regressions to the arm. Optional.

- **N4 — Carry-forward open items unchanged.** Lexical shadowing, legacy-emission, fallible `//` and `%` — all stay tracked under the open INT-1 follow-up. Once a tracker PR records this slice complete, the open follow-up bullet should remove "function argument expressions that are already `SifrInt`" since this slice closes that residual.

- **N5 — Pass-pattern observation.** PR #1841 follows the dual-pass pattern of #1825 and #1827 — an initial implementation with a B1 blocker, then a small focused fix in pass 2. The history pattern accommodates this.

- **N6 — Quick validation.** `report_signature=e1bf653aaa770517` matches all prior milestone PRs, and the cited `wall_time=65.65s` is in the normal range for tracker-only or small implementation deltas.

## What I'd accept (resolved)

Pass-1 asked for:

- **B1 fix**: add `Clone(expr) => self.is_sifr_int_expr(expr)` arm to `is_sifr_int_expr`. ✓ Done at [expr_render_helpers.rs:1422](crates/sifr_codegen/src/expr_render_helpers.rs:1422).
- **Regression e2e assertion** exercising the registered-local-arg case. ✓ Done at [crates/sifr/tests/e2e/pass/module_constants.sifr:107,112](crates/sifr/tests/e2e/pass/module_constants.sifr:107) with `echoed_local_parameter`.

Both items are addressed. Verdict flips to Satisfied.
