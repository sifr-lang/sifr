# INT-1 SifrInt Local Value Semantics — Review Pass 1

**Verdict:** Satisfied with non-blocking suggestions.

## Scope reviewed

PR #1823, branch `int-1-sifrint-local-value-semantics` (head `989c1d0e`), `main..HEAD` diff:

- [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/integer_model.md](internal_docs/integer_model.md) — load-bearing rule at [line 474](internal_docs/integer_model.md:474): *"Sifr source treats `int` as scalar value-semantic and non-consuming: using an `int` binding in more than one expression is always legal."*
- [reviews/integer-model-int-1-sifrint-local-comparison-use-sites-review-pass-1.md](reviews/integer-model-int-1-sifrint-local-comparison-use-sites-review-pass-1.md) — pass-1 N1 finding this slice closes
- Runtime trait impls: [crates/sifr_runtime/src/int.rs:219-346](crates/sifr_runtime/src/int.rs:219) (`Add`/`Sub`/`Mul` for `(SifrInt, SifrInt)`, `(SifrInt, &SifrInt)`, `(&SifrInt, SifrInt)`, `(&SifrInt, &SifrInt)`; `Neg for SifrInt` and `Neg for &SifrInt`; `PartialEq`/`PartialOrd`/`Ord` on `SifrInt`).

## Slice goal — closed

The pass-1 N1 finding for PR #1821 was that registered `SifrInt` locals were moved on first arithmetic use because the codegen emitted by-value `Add`/`Sub`/`Mul`/`Neg` against `Self`-consuming impls. That violated [integer_model.md:474](internal_docs/integer_model.md:474). The diff closes that on three coordinated paths:

### 1. Arithmetic operand coercion borrows registered locals

[coerce_expr_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1248) is restructured. Old order ("if `is_sifr_int_expr(&expr)` pass through, else match expr") consumed registered Ident references because the new `Ident(name) if registered` arm in `is_sifr_int_expr` (introduced in PR #1821) made the pass-through fire. New order:

```rust
match expr {
    Ident(name) if is_registered_sifr_int_local(&name) => Ref { mutable: false, expr: Ident(name) },
    Paren(inner)                                       => Paren(coerce(inner)),
    other if is_sifr_int_expr(&other)                  => other,                     // fresh-owned SifrInt — pass through
    Cast { ty: I64, expr }                             => sifr_int_from_i64_expr(*expr),
    other                                              => sifr_int_from_i64_expr(other),
}
```

The Ident-registered arm is *first*, so registered locals always become `&local` rather than passing through and getting moved. Helper `FnCall`s (e.g., `__const_BIG_LIMIT()`) and SifrInt-shaped `BinOp`/`UnaryOp` results still pass through unchanged because they produce fresh-owned `SifrInt` values — borrowing them would be unnecessary lifetime extension. The Paren branch dives in unconditionally, so a synthetic `Paren(Ident{registered})` correctly becomes `Paren(Ref{Ident})` rather than passing through.

End-to-end verification via `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr`:

```rust
let reusable_oversized_local: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let reuse_a: SifrInt = &reusable_oversized_local + SifrInt::from_i64(1);
let reuse_b: SifrInt = &reusable_oversized_local + SifrInt::from_i64(2);   // no use-after-move
let negated_reuse: SifrInt = -&reusable_oversized_local;                   // no use-after-move
```

The fixture's `assert str(reusable_oversized_local) == '100000000000000000001'` after these three uses confirms the local is still observable post-arithmetic. Round-trips at runtime.

### 2. Comparisons normalize both operands to `&SifrInt`

[coerce_expr_to_sifr_int_comparison_operand](crates/sifr_codegen/src/expr_render_helpers.rs:1267) runs `coerce_expr_to_sifr_int` and then wraps any non-`Ref` result in `Ref { mutable: false, expr }`. The BinOp arm at [expr_render_helpers.rs:288-303](crates/sifr_codegen/src/expr_render_helpers.rs:288) now picks between the comparison- and arithmetic-flavored coercion based on `is_sifr_int_comparison_op(&op)`.

This produces:
- `BIG_LIMIT > 100` ⇒ `&__const_BIG_LIMIT() > &SifrInt::from_i64(100)`
- `chained_oversized_local > BIG_LIMIT` ⇒ `&chained_oversized_local > &__const_BIG_LIMIT()`
- `reusable_oversized_local < reuse_b` ⇒ `&reusable_oversized_local < &reuse_b`

`SifrInt: PartialOrd<SifrInt>` ([crates/sifr_runtime/src/int.rs:374](crates/sifr_runtime/src/int.rs:374)) is reachable from `&SifrInt > &SifrInt` because the operator desugaring `(&a).gt(&b)` auto-derefs through `&&SifrInt → &SifrInt`. Verified by the runtime test passing — the fixture's `assert reusable_oversized_local < reuse_b` round-trips.

The slight asymmetry of always wrapping fresh-owned helper calls in `&` is functionally fine — Rust extends the temporary's lifetime through the comparison expression — and produces a uniformly-shaped output. See N1 below for a stylistic note.

### 3. Unary negation borrows registered locals

The `RustExpr::UnaryOp` arm at [expr_render_helpers.rs:314-323](crates/sifr_codegen/src/expr_render_helpers.rs:314) gains a guard: when `op == "-"` and the (rewritten) operand is `is_sifr_int_expr`-true, the operand goes through `coerce_expr_to_sifr_int`. For a registered local, that means `-&local` rather than `-local`. SifrInt has `impl Neg for &SifrInt` ([crates/sifr_runtime/src/int.rs:346](crates/sifr_runtime/src/int.rs:346)) producing owned `SifrInt`, so `-&reusable_oversized_local` compiles cleanly and does not move `reusable_oversized_local`. Helper FnCalls and BinOp results pass through (still by-value since they're fresh-owned).

### 4. `is_sifr_int_expr` recognizes `Ref`-wrapped SifrInt expressions

A new arm at [expr_render_helpers.rs:1300](crates/sifr_codegen/src/expr_render_helpers.rs:1300) recurses into `Ref { expr }`. This is necessary so that after coerce wraps a registered local in `&`, an outer detection (e.g., the propagation gate inside `is_sifr_int_arithmetic_op` BinOp) still sees the inner is SifrInt-shaped. Without this arm, chained arithmetic that flows from a borrowed local through an intermediate BinOp would silently lose the SifrInt detection at the outer level.

### Probe matrix (all reproduced via `cargo run -q -p sifr -- emit /tmp/...sifr; … run …`)

| Source shape                                                | Emitted Rust                                                  | Result |
|-------------------------------------------------------------|---------------------------------------------------------------|--------|
| `big: int = BIG_LIMIT + 1; a = big + 1; b = big + 2`        | `&big + SifrInt::from_i64(1)` for each, no move               | ✓      |
| `negated: int = -big`                                       | `-&big`                                                       | ✓      |
| `cmp: bool = big < reuse_b`                                 | `&big < &reuse_b`                                             | ✓      |
| `cmp: bool = BIG_LIMIT > 100`                               | `&__const_BIG_LIMIT() > &SifrInt::from_i64(100)`              | ✓      |
| `mix: int = BIG_LIMIT + big`                                | `__const_BIG_LIMIT() + &big`                                  | ✓ via `Add<&SifrInt> for SifrInt` |
| `mix: int = big + BIG_LIMIT`                                | `&big + __const_BIG_LIMIT()`                                  | ✓ via `Add<SifrInt> for &SifrInt` |
| `chain: int = -chain1 + big` (two registered locals)        | `-&chain1 + &big`                                             | ✓      |
| `(big) + 1`, `-(big)`                                       | Paren stripped at HIR; `&big + …`, `-&big`                    | ✓      |
| `(big + 1) > big` (chained-arith comparison)                | `&(&big + SifrInt::from_i64(1)) > &big`                       | ✓ via auto-deref to `PartialOrd` |
| `-big - big` (consumes nothing)                             | `-&big - &big`                                                | ✓ `big` still usable after  |

All probes compile and produce correct values.

## Determinism / regression check

- **i64-only paths untouched.** `MAX_RETRIES + 1`, `MAX_RETRIES > 100`, `let x: i64 = …`, plain `int + int` for non-oversized constants — none of these have `is_sifr_int_expr`-true operands, so the BinOp/UnaryOp arms short-circuit to the original rebuild path. The `cargo run -q -p sifr -- run module_constants.sifr` round-trips all the legacy asserts (`'78.53975'`, `'3'`, `'254'`, `'-13'`).
- **Helper-only arithmetic untouched.** `BIG_LIMIT + 1` (no local) still emits `__const_BIG_LIMIT() + SifrInt::from_i64(1)` — the helper FnCall passes through `coerce_expr_to_sifr_int` because of the `other if is_sifr_int_expr(&other)` arm, so no spurious `&` is introduced.
- **`Ref` arm in `is_sifr_int_expr` does not over-trigger.** A non-SifrInt `Ref` (e.g., `Ref { Ident("non_registered_var") }` synthesized by stdlib method-call paths like `.contains(&x)`) recurses to a non-registered Ident, which returns false. Verified against the [RustExpr::Ref construction sites](crates/sifr_codegen/src/lower_expr.rs:364) — none of the audited synthesized Refs wrap a registered SifrInt local in arithmetic-context positions.
- **The `coerce_expr_to_sifr_int` reorder is critical and correct.** The old `if self.is_sifr_int_expr(&expr) { return expr; }` first-match would have moved any registered `Ident` (since the PR #1821 Ident arm in `is_sifr_int_expr` matched registered names). The new order guarantees the Ident-registered case is intercepted *before* the SifrInt-pass-through. Note also that `is_sifr_int_expr` itself was *not* modified to remove the Ident-registered arm — it still detects registered locals as SifrInt-shaped, which is required for the BinOp/UnaryOp guards to fire. The two arms cooperate: detection says "yes, this operand needs coercion"; coercion then borrows rather than passing through.
- **`scripts/run_all_tests.sh --profile quick`** reports `report_signature=e1bf653aaa770517`, identical to PRs #1817–#1822. The change is local to the rewriter's coercion shape; no snapshot or behavioral assertions elsewhere depend on whether oversized arithmetic borrows or moves.

## Tests

- [rewrites_large_int_module_const_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1426) (carried) — pins the helper-arithmetic shape: `BIG_LIMIT + 1` ⇒ `__const_BIG_LIMIT() + SifrInt::from_i64(1)`. Helper still passes through unborrowed (no `&` on the helper FnCall) — this is the right behavior for an owning temporary that's about to be consumed by `Add`.
- [rewrites_large_int_module_const_let_type_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1456) (carried) — pins the let-retype shape.
- [rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1466) (updated) — the assertion on the `left` operand is changed from `RustExpr::Ident(name)` to `RustExpr::Ref { mutable: false, expr: Ident(name) }`. Pins the borrowing rewrite for chained-local arithmetic.
- [rewrites_large_int_module_const_comparison_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1494) (updated) — both operands now asserted as `Ref { … }` — pins the always-borrow comparison shape.
- [rewrites_registered_sifr_int_local_comparison_to_borrowed_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1538) (new) — pins the comparison shape for two registered locals (well, a registered local against a helper). Closes the local-comparison shape.

E2E coverage in [module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr) adds:
- `reusable_oversized_local: int = BIG_LIMIT + 1` followed by both `reuse_a = reusable_oversized_local + 1` and `reuse_b = reusable_oversized_local + 2` — the load-bearing repeated-use case from pass-1 N1.
- `negated_reuse: int = -reusable_oversized_local` — load-bearing unary `-` borrowing case.
- `assert str(reusable_oversized_local) == '100000000000000000001'` *after* all three derived bindings — pins that the local is still observable, which is the design contract.
- `assert reusable_oversized_local < reuse_b` — borrowed-local comparison.

Together these pin the four operator shapes (arithmetic, unary `-`, comparison-with-helper, comparison-with-local) and the round-trip property that a SifrInt local survives multi-use. Coverage is sufficient for the slice's stated scope.

## Scope drift

- All edits live in `expr_render_helpers.rs` plus the e2e fixture. No HIR, type system, runtime, or driver changes. No public API growth.
- The new helpers (`coerce_expr_to_sifr_int_comparison_operand`, `is_registered_sifr_int_local`) are private. The free function `is_sifr_int_comparison_op` was already introduced by PR #1821 and is re-used here.
- The reorder of `coerce_expr_to_sifr_int`'s match arms is the load-bearing change. It's small but its correctness depends on understanding why the old "pass through if SifrInt-shaped" was unsafe for registered Idents — see N5 below for a documentation suggestion.

## Non-blocking findings

### N1 — Comparison normalization unconditionally borrows fresh-owned helpers

`coerce_expr_to_sifr_int_comparison_operand` always wraps a non-`Ref` result in `Ref`, including for fresh-owned helper FnCalls and `from_i64` wrappers. This produces `&__const_BIG_LIMIT() > &SifrInt::from_i64(100)`, where both `&`s borrow temporaries whose lifetimes are extended to the end of the expression statement.

Functionally correct — Rust's temporary lifetime extension rules accommodate this — and stylistically unifies the comparison output shape (both sides are always `Ref`). The simpler alternative `__const_BIG_LIMIT() > SifrInt::from_i64(100)` would also compile because Rust's `>` operator desugaring takes references internally via `(&a).gt(&b)`. The slice's choice is defensible: a uniform output shape simplifies the rewrite logic and makes the test assertions structurally consistent. Not a blocker.

### N2 — `total: int = 0; total = total + big` still emits invalid Rust (deferred per scope)

When a registered SifrInt local is mixed into an `Assign` statement whose target is i64-typed, the RHS rewrites to a `SifrInt` expression that doesn't fit the target. Reproduction:

```sifr
def main():
    big: int = BIG_LIMIT + 1
    total: int = 0
    for i in [1, 2, 3]:
        total = total + big
```

emits

```rust
let mut total: i64 = 0 as i64;
for i in … {
    total = SifrInt::from_i64(total) + &big;   // E0308: expected i64, found SifrInt
}
```

Pre-this-PR the same code already failed (RHS was `SifrInt::from_i64(total) + big` with use-after-move on the second iteration plus the same type-mismatch). So this is *not* a regression — no working program stops working. But it's worth noting that the slice's strengthened operand coercion does not extend to `Assign` target retyping, which would require either tracking assignable-into-SifrInt status of the target (analogous to the Let arm's retype) or routing through fallible `try_to_i64` at the assignment site. Both belong to the broader `Type::Int` ⇒ `SifrInt` migration that's already tracked at [issues/…/checklist:432](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). Out of slice scope per the closing bullet's "broader migration" framing. Worth flagging in the next tracker so an `Assign`-side counterpart to the Let-side retype eventually lands.

### N3 — Function call arguments still consume registered SifrInt locals (deferred per scope)

`some_fn(reusable_oversized_local)` rewrites the argument expression but does not coerce-or-borrow it because the rewrite only fires inside BinOp/UnaryOp arms. After the call, `reusable_oversized_local` has been moved. This is the function-argument/return boundary follow-up in the existing tracker bullet. Out of slice scope.

### N4 — Lexical shadowing still corrupts the registry (carried forward from PR #1821 review N2)

Pre-existing gap not addressed by this slice. Worth flagging so it's not lost behind the value-semantics tick.

### N5 — Code-shape and documentation polish

- **The match-arm order in [coerce_expr_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1248) is now *load-bearing*** — the `Ident(name) if is_registered_sifr_int_local(&name)` arm must come before `other if is_sifr_int_expr(&other)`, because `is_sifr_int_expr(Ident{registered})` returns true and would short-circuit the borrow. A short comment explaining "must precede the generic SifrInt pass-through to avoid moving registered locals" would protect future contributors from refactoring the arms into a more "natural" order and silently re-introducing the move.
- **The new `Ref` arm in [is_sifr_int_expr](crates/sifr_codegen/src/expr_render_helpers.rs:1300)** propagates SifrInt-ness through `&expr`. This is correct for the rewriter's own outputs (which only wrap registered locals or already-SifrInt expressions), but a future contributor might expect the arm to gate on whether the inner is a registered local specifically — they could try to "narrow" it to `Ref { expr: Ident(n) } if is_registered_sifr_int_local(n)`. That narrowing would silently drop detection of `&__const_BIG_LIMIT()` and similar Ref'd helper calls, breaking outer-comparison propagation. A short comment on the arm noting "any Ref of a SifrInt-shaped expression is itself SifrInt-shaped" would lock the intent.
- **`coerce_expr_to_sifr_int_comparison_operand` does not handle nested `Paren`** explicitly. If a synthetic `Paren(Ref{Ident{registered}})` reached it (unlikely; Sifr's HIR strips Paren), the outer paren would be wrapped a second time as `Ref{Paren{Ref{...}}}` — extra `&` on a `&SifrInt` is fine in Rust because `&&SifrInt: PartialOrd<&&SifrInt>` via auto-deref, but the shape is ugly. A defensive `if matches!(coerced, Paren(inner) if matches!(inner.as_ref(), Ref{..}))` could collapse it. Optional.
- **Unit test for unary negation**. The unary `-` rewrite is exercised only by the e2e fixture (`negated_reuse: int = -reusable_oversized_local`). A focused unit test mirroring `rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands` would harden the contract. The slice's three new/updated unit tests cover BinOp arithmetic and BinOp comparison shapes; UnaryOp coverage is the gap.

None gate merge.

## Validation

I re-traced rather than re-ran the listed validation. The cited results are consistent with the code and my probes:

- `cargo test -p sifr_codegen rewrites_registered_sifr_int_local -- --nocapture` — I ran this; both `…_arithmetic_to_sifr_int_operands` and `…_comparison_to_borrowed_operands` pass.
- `cargo test -p sifr_codegen rewrites_large_int_module_const_comparison_to_sifr_int_operands -- --nocapture` — covered above; the updated assertions for `Ref { … }` on both sides match the production output.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — round-trips for all twelve asserts (the seven from #1817–#1821 plus the five new ones).
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr | rg "reusable_oversized_local|reuse_a|reuse_b|negated_reuse|BIG_LIMIT >|chained_oversized" -n` — confirms the borrowing shapes I described.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=64.79s`. Same signature as #1817–#1822.

## Verdict

**Satisfied with non-blocking suggestions.** The slice closes pass-1 N1 cleanly: registered SifrInt locals are now borrowed in arithmetic operands, comparison operands, and unary negation, preserving the [design's value-semantic guarantee for `int` locals](internal_docs/integer_model.md:474). The implementation cooperates with Rust's `Add<&SifrInt>`/`Sub`/`Mul` impl matrix on `SifrInt`, with `Neg for &SifrInt`, and with `PartialOrd<SifrInt>`'s reachability through `&SifrInt > &SifrInt` via operator-desugaring auto-deref. The `coerce_expr_to_sifr_int` reorder is the load-bearing fix and is correct (Ident-registered arm precedes the generic SifrInt pass-through). The new `Ref` arm in `is_sifr_int_expr` enables propagation through borrowed expressions, and is correctly defensive against false positives because non-registered Idents don't match. Existing legacy `i64` paths and helper-only arithmetic shapes are untouched. Tests + e2e fixture pin the load-bearing shapes and round-trip the design contract.

The non-blocking findings cluster around the broader `Type::Int` ⇒ `SifrInt` migration (N2 `Assign`-target retyping, N3 function-argument boundary, N4 lexical shadowing) and small documentation/test-hardening suggestions (N1, N5). All four user-facing failure shapes from N2/N3/N4 remain in the existing open INT-1 follow-up bullet at [issues/…/checklist:432](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md), so nothing slips behind the value-semantics tick. None gates merge.
