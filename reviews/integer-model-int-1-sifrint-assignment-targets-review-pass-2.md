# INT-1 SifrInt Assignment Targets — Review Pass 2

**Verdict:** Satisfied. No blockers. The pass-1 B1 regression is fully fixed.

## Scope reviewed

PR #1825 pass-2, head `3a708c0b` ("Fix SifrInt assignment value aliases"), `32c5c818..HEAD` diff (the increment over pass-1):

- [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [reviews/integer-model-int-1-sifrint-assignment-targets-review-pass-1.md](reviews/integer-model-int-1-sifrint-assignment-targets-review-pass-1.md) — the pass-1 review whose B1 this revision closes.
- [internal_docs/integer_model.md](internal_docs/integer_model.md) — value-semantic rule at [line 474](internal_docs/integer_model.md:474).
- Pre-fix baseline (commit `32c5c818`, pass-1 state) confirmed broken on the bare-Name aliasing shape; post-fix (commit `3a708c0b`) confirmed working via direct probes.

## B1 — fully closed

The pass-1 finding was that the new `coerce_expr_to_sifr_int(value)` calls in the Let arm and Assign arm wrapped registered `Ident` locals in `Ref { mutable: false, expr: Ident(name) }` — correct for operand position (where `Add<&SifrInt>` etc. accept the borrow) but invalid for value position (where the LHS expects an owned `SifrInt`, not `&SifrInt`).

The fix introduces a sibling helper [coerce_expr_to_sifr_int_value](crates/sifr_codegen/src/expr_render_helpers.rs:1297-1322):

```rust
fn coerce_expr_to_sifr_int_value(&self, expr: crate::RustExpr) -> crate::RustExpr {
    match expr {
        crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
            crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(name)))
        }
        crate::RustExpr::Paren(inner) => {
            crate::RustExpr::Paren(Box::new(self.coerce_expr_to_sifr_int_value(*inner)))
        }
        crate::RustExpr::BinOp { left, op, right }
            if is_sifr_int_arithmetic_op(&op)
                && (self.is_sifr_int_expr(&left) || self.is_sifr_int_expr(&right)) =>
        {
            crate::RustExpr::BinOp {
                left: Box::new(self.coerce_expr_to_sifr_int(*left)),    // operand-position
                op,
                right: Box::new(self.coerce_expr_to_sifr_int(*right)),  // operand-position
            }
        }
        other if self.is_sifr_int_expr(&other) => other,
        crate::RustExpr::Cast {
            expr,
            ty: crate::RustType::I64,
        } => sifr_int_from_i64_expr(*expr),
        other => sifr_int_from_i64_expr(other),
    }
}
```

The structural design is sound:

- **Arm 1** (the load-bearing change): registered `Ident` → `Clone(Ident)`, producing `local.clone()` instead of `&local`. This is the durable fix the pass-1 review suggested as the value-semantic-preserving option, not the simpler "leave the move alone" option. Cloning matches the [design rule](internal_docs/integer_model.md:474) that source-level `int` is value-semantic and non-consuming, so `b: int = a` now leaves `a` usable afterwards.
- **Arm 2**: `Paren` recurses into the value-position coerce — keeps the value-position semantics inside parentheses.
- **Arm 3**: A `BinOp` whose result *is* a value-position SifrInt expression. The result of an addition is itself a fresh-owned SifrInt that doesn't need cloning. But the operands of that `BinOp` are still in **operand position**, so the recursion delegates to the existing `coerce_expr_to_sifr_int` (operand-position coerce). This is the correct asymmetry: a Let-RHS that happens to be `a + b` should clone neither side and instead borrow them as `&a + &b`. Verified by my probe — `let s1: SifrInt = &a + SifrInt::from_i64(1)` (no clone on the BinOp itself).
- **Arms 4–6**: identical to operand-position — pass through already-SifrInt-shaped helper calls and BinOp results, wrap `Cast(I64)` and other non-SifrInt expressions in `from_i64`.

The two call sites at [expr_render_helpers.rs:481](crates/sifr_codegen/src/expr_render_helpers.rs:481) (Let arm) and [expr_render_helpers.rs:526](crates/sifr_codegen/src/expr_render_helpers.rs:526) (Assign arm) are the only changes needed; the existing pre-scan and scope save/restore plumbing is unchanged.

### Pass-1 reproducers — fixed

`b: int = a` (the original B1 case):

```sifr
BIG_LIMIT: int = 10 ** 20

def main():
    a: int = BIG_LIMIT + 1
    b: int = a
    print(str(b))
    print(str(a))   # <-- now legal, value semantics preserved
```

emits

```rust
let a: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let b: SifrInt = a.clone();
println!("{}", format!("{}", b));
println!("{}", format!("{}", a));
```

Compiles and prints `100000000000000000001` twice. Both `b` and `a` are usable independently after the let, satisfying the design rule. ✓

`total = a` (the Assign-side B1 case):

```sifr
def main():
    a: int = BIG_LIMIT + 1
    total: int = 0
    total = a
    print(str(total))
    print(str(a))
```

emits

```rust
let a: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let mut total: SifrInt = SifrInt::from_i64(0);
total = a.clone();
…
```

Compiles and runs. ✓

`b = a` with no annotation (a parallel inference path):

```rust
let b: SifrInt = a.clone();
```

…also works because the codegen-level inference still gives `Some(RustType::I64)` from `Type::Int`, so the Let arm's gate fires identically. ✓

### Operand-position cases — preserved

I re-probed the operand shapes from the prior PRs to confirm no regression from extending `coerce_expr_to_sifr_int_value`:

| Probe                                                | Emitted Rust                                    | Result |
|------------------------------------------------------|-------------------------------------------------|--------|
| `s1: int = a + 1`                                    | `let s1: SifrInt = &a + SifrInt::from_i64(1);`  | ✓ borrowed |
| `s2: int = -a`                                       | `let s2: SifrInt = -&a;`                        | ✓ borrowed |
| `s4: bool = a < a + 1`                               | `let s4: bool = &a < &(&a + SifrInt::from_i64(1));` | ✓ borrowed |
| `total: int = 0` (forced)                            | `let mut total: SifrInt = SifrInt::from_i64(0);` | ✓ from_i64 |
| `total = total + a` (forced + BinOp source)          | `total = &total + &a;`                          | ✓ borrowed |
| `total = total + 5` (forced + small literal in BinOp) | `total = &total + SifrInt::from_i64(5);`       | ✓ from_i64 |
| `b: int = BIG_LIMIT` (helper alias)                  | `let b: SifrInt = __const_BIG_LIMIT();`         | ✓ pass-through |

The asymmetry of arms 1 and 3 in `coerce_expr_to_sifr_int_value` keeps these working: BinOp value-position routes operands back through the *operand* coerce, which is what produces the `&a` shapes for sibling/UnaryOp operand cases.

### Source value semantics

The new e2e fixture lines specifically pin the value-semantic guarantee:

```sifr
alias_reuse: int = reusable_oversized_local
alias_assign: int = 0
alias_assign = reusable_oversized_local
assert str(alias_reuse) == '100000000000000000001'
assert str(alias_assign) == '100000000000000000001'
assert str(reusable_oversized_local) == '100000000000000000001'   # <-- still usable
```

Emitted as:

```rust
let alias_reuse: SifrInt = reusable_oversized_local.clone();
let mut alias_assign: SifrInt = SifrInt::from_i64(0);
alias_assign = reusable_oversized_local.clone();
…
assert!((format!("{}", reusable_oversized_local) == "…".to_string()));   // <-- compiles, runs
```

The trailing `assert str(reusable_oversized_local)` is the load-bearing assertion that catches any future regression to move-by-default — if the codegen reverted to bare `Ident` (move) or stayed at `&Ident` (borrow), this assert would either fail to compile (use-after-move on a previous SifrInt local that was supposed to be cloned) or fail to compile from `&SifrInt` mismatch. ✓

## Tests

The new unit test [rewrites_sifr_int_value_position_aliases_to_clone](crates/sifr_codegen/src/expr_render_helpers.rs:1674-1714) is the missing pass-1 N4 coverage:

- Stages `source` in `sifr_int_local_bindings` and `target` in `sifr_int_forced_local_bindings`.
- Asserts `Let { ty: Some(I64), value: Ident("source") }` rewrites to `Let { ty: Some(SifrInt), value: Clone(Ident("source")) }`.
- Asserts `Assign { target: Ident("target"), value: Ident("source") }` rewrites to `Assign { target: Ident("target"), value: Clone(Ident("source")) }`.

Both assertions match the expected post-fix shape. The structural test would have caught B1 if it had existed in pass-1.

The e2e fixture additions (`alias_reuse`, `alias_assign`, three asserts) close the same gap at the integration level.

I ran:

- `cargo test -p sifr_codegen rewrites_sifr_int_value_position_aliases_to_clone` — passes.
- `cargo test -p sifr_codegen rewrites_forced_sifr_int_assignment_target_storage` — passes (pass-1's literal-source forcing test, unchanged).
- `cargo test -p sifr_codegen rewrites_registered_sifr_int_local` — both arithmetic and comparison tests pass (the operand-position tests from PRs #1821/#1823 are unaffected).

`cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` round-trips all 18 asserts (the 9 from #1817–#1823, the 4 from pass-1 of #1825, and the 3 new value-position assertions added in pass-2).

`scripts/run_all_tests.sh --profile quick` reports `report_signature=e1bf653aaa770517`, identical to the signature recorded across #1817–#1823.

(Note: `cargo test -p sifr_codegen` shows 22 unrelated failures in `lib_codegen_tests::*` and `lower_*::tests::*` modules. I verified by checking out `main` and re-running — these failures pre-exist on `main` (parse errors on test class definitions) and are *not* introduced by this PR. The canonical validation gate per AGENTS.md is `scripts/run_all_tests.sh --profile quick`, which passes.)

## Determinism / scope drift

- Diff is +56/-2 lines: one new helper function (~26 lines), two call-site swaps (1 line each), one new unit test (~26 lines), and 6 lines of e2e fixture growth. Tightly scoped to the pass-1 fix.
- All edits live in `expr_render_helpers.rs` and the e2e fixture. No HIR, type system, runtime, or driver changes. No public API growth.
- The pass-1 plumbing (pre-scan, per-function scope save/restore for `sifr_int_forced_local_bindings`, retype gate, post-rewrite Assign coerce site) is unchanged. The only behavioral change is *which* coerce function the Let and Assign arms call.
- The previous `coerce_expr_to_sifr_int` (operand-position) is unchanged and continues to be called from BinOp arms (in `rewrite_stdlib_constant_idents_in_expr`'s arm and recursively from the new `coerce_expr_to_sifr_int_value`'s BinOp arm) and from comparison-position via `coerce_expr_to_sifr_int_comparison_operand`.

## Non-blocking observations

(All carry forward from pass-1; none gate merge.)

### N-pass2-1 — AugAssign still unhandled (pass-1 N1)

`total += a` (with `a` a SifrInt local) emits `let mut total: i64 = …; total += a;` regardless of forcing. The pre-scan visitor only matches `HirStmt::Let | HirStmt::Assign`. The rewriter's `RustStmt::AugAssign` arm doesn't apply target-aware coerce. Pre-existing gap; same shape was broken pre-this-PR. The slice description doesn't claim AugAssign coverage, but it's the obvious next sub-slice.

### N-pass2-2 — Coerce arm-ordering documentation (pass-1 N3)

`coerce_expr_to_sifr_int_value` and `coerce_expr_to_sifr_int` now share the same arm-ordering invariant: arms 1 and 3 (Ident-registered, BinOp-with-SifrInt-operand) must precede arm 4 (`other if is_sifr_int_expr(&other)`). A short comment block explaining "arms 1 and 3 must precede arm 4 to ensure the value-position clone / operand-position borrow happens before the generic SifrInt pass-through" would protect future contributors who try to refactor either function. Optional polish, not a correctness concern.

### N-pass2-3 — `Clone` chains for repeated bare-Name aliases

Each `b: int = a; c: int = a; d: int = a;` produces three `a.clone()` calls. Each clone is allocator-bound for the `Big` variant of `SifrInt`. For the typical small-int case the `Small(i64)` variant clones cheaply, but for arbitrary-precision values the cost adds up. This is consistent with the design's "value-semantic" guarantee but worth noting as an INT-8 perf gate concern. Not in slice scope.

### N-pass2-4 — Open follow-ups still apply

Lexical shadowing, legacy-emission-path coverage, fallible `//` and `%`, function argument/return boundaries — all carried-forward open items at [issues/…/checklist:434](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). Not affected by this slice.

## Verdict

**Satisfied. No blockers.** The pass-1 B1 regression is closed via a tightly-scoped fix: a sibling `coerce_expr_to_sifr_int_value` helper that clones registered Idents in value position while delegating to the operand-position helper for BinOp sub-operands. Both Let-RHS and Assign-RHS shapes for bare-Name aliases now compile and preserve the [design's value-semantic guarantee for `int` locals](internal_docs/integer_model.md:474) — verified by both the new unit test (`rewrites_sifr_int_value_position_aliases_to_clone`) and the new e2e assertions (`alias_reuse`, `alias_assign`, with the trailing `str(reusable_oversized_local)` assert pinning that the source local stays usable).

Operand-position semantics from #1819/#1821/#1823 are preserved: arithmetic still emits `&a + …`, comparisons still emit `&a < &b`, unary negation still emits `-&a`, helper FnCalls still pass through unborrowed, small-literal forced storage still uses `from_i64`. Quick validation reproduces `report_signature=e1bf653aaa770517`. Non-blocking carry-forwards (N-pass2-1 AugAssign, N-pass2-2 arm-ordering doc comment, N-pass2-3 clone chain perf, N-pass2-4 broader migration) stay in the open INT-1 follow-up.
