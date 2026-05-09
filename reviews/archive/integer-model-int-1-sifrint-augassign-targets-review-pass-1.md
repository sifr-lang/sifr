# Review: INT-1 SifrInt AugAssign Targets Pass 1

## Verdict

Satisfied.

## Findings

None.

The slice cleanly closes the prior tracker's "augmented assignment targets such as `total += big`" follow-up for the supported-ops scope, with three coordinated changes:

1. **Pre-scan extension** at [function_emitter.rs:137-148](crates/sifr_codegen/src/function_emitter.rs:137). The forced-local visitor now matches `HirStmt::AugAssign { name, op, value }` when:
   - `local_int_bindings.contains(name)` — only int locals
   - `is_sifr_int_augassign_op(op)` — only `+=`, `-=`, `*=` (HIR-level form)
   - `forced.contains(name) || hir_expr_needs_sifr_int_storage(value, …)` — either name was already forced by an earlier Let/Assign, or the value transitively brings a SifrInt source

   The two-condition disjunction is load-bearing and correct: a Sifr-source pattern like
   ```sifr
   total: int = 0
   total += big           # forces total via value side
   total += 2             # already forced → keeps forcing through small literal
   ```
   converges in one walk pass. The fixed-point loop pre-existed; the new arm participates correctly.

2. **Op-set consistency** between pre-scan and rewrite. The HIR pre-scan checks `is_sifr_int_augassign_op` (`+=`/`-=`/`*=`); the Rust IR rewrite checks `is_sifr_int_arithmetic_op` (`+`/`-`/`*`). I verified at [crates/sifr_hir/src/lower/aug_assign_lowering.rs:34-46](crates/sifr_hir/src/lower/aug_assign_lowering.rs:34) that HIR stores the augmented form (`+=`, `-=`, `*=`) and at [crates/sifr_codegen/src/lower_stmt.rs:4114](crates/sifr_codegen/src/lower_stmt.rs:4114) that Rust IR strips the trailing `=`. The two op-set predicates are therefore consistent — both express the same `{+, -, *}` semantic set in their respective IRs. Unsupported HIR ops like `//=`, `%=`, `<<=`, `>>=`, `**=` correctly fall through both gates.

3. **AugAssign rewrite** at [expr_render_helpers.rs:532-557](crates/sifr_codegen/src/expr_render_helpers.rs:532). When target is `Ident(name)`, op is in `{+, -, *}`, and name is registered or forced:
   - Insert into `sifr_int_local_bindings` (idempotent).
   - Convert `target op= value` to `target = &target op coerce(value)`.
   
   The rewrite output is a plain `RustStmt::Assign` whose RHS is `BinOp { Ref{target}, op, coerce_expr_to_sifr_int(value) }`. The `&target` borrow preserves Rust ownership of the local across iterations, and `coerce_expr_to_sifr_int` (operand-position) borrows registered locals on the value side or wraps small literals in `from_i64`. Rust's `Add<&SifrInt> for &SifrInt` and friends accept the resulting `&SifrInt op &SifrInt`/`&SifrInt op SifrInt` shape, returning owned `SifrInt`, which assigns back to the SifrInt-typed target.

### End-to-end verification

`cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr` shows the new fixture lines lower to:

```rust
let mut augmented_total: SifrInt = SifrInt::from_i64(0);
augmented_total = &augmented_total + &reusable_oversized_local;
augmented_total = &augmented_total + SifrInt::from_i64(2);
assert!((format!("{}", augmented_total) == "100000000000000000003".to_string()));
```

…and the trailing `assert str(reusable_oversized_local) == ...` (later in the fixture) compiles, pinning that `reusable_oversized_local` was *not* moved by the AugAssign — value semantics for the source local are preserved. The fixture round-trips at runtime.

### Probe matrix (verified against the post-fix tip)

| Source                                                  | Emitted Rust                                                | Result |
|---------------------------------------------------------|-------------------------------------------------------------|--------|
| `total: int = 0; total += a` (SifrInt local source)     | `total = &total + &a;`                                      | ✓ borrows both |
| `total -= a`                                            | `total = &total - &a;`                                      | ✓ |
| `total *= 2`                                            | `total = &total * SifrInt::from_i64(2);`                    | ✓ from_i64 |
| `total += a + 1` (BinOp value, mixed local + literal)   | `total = &total + (&a + SifrInt::from_i64(1));`             | ✓ inner BinOp borrows operands |
| `total += -a` (UnaryOp value)                           | `total = &total + -&a;`                                     | ✓ |
| `for i in [1,2,3]: total += a` (loop body)              | `for … { total = &total + &a; }`                            | ✓ source survives loop |
| Repeated `total += a` (multiple uses)                   | `&a` each time — never moves                                | ✓ source still usable after |
| Pure i64: `total: int = 0; total += 1; total += 2`      | `let mut total: i64 = …; total += 1 as i64; total += 2;`    | ✓ unchanged |
| Pure i64: `total: int = 10; total //= 3; total %= 2`    | `total /= 3 as i64; total %= 2 as i64;`                     | ✓ unchanged |

Quick validation reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1825).

## Notes

(Non-blocking observations only.)

- **N1 — Mixed unsupported AugAssign ops on a forced SifrInt local emit invalid Rust.** A program like
  ```sifr
  a: int = BIG_LIMIT + 1
  total: int = 0
  total //= 2
  total += a
  ```
  pre-PR failed at line 4 (`i64 += SifrInt`); post-PR it fails at line 3 because `total` is forced to `SifrInt` by the supported `+= a` and `total /= 2 as i64` (HIR `//=` lowers to Rust `/`, which `is_sifr_int_arithmetic_op` doesn't match) emits `SifrInt /= i64`. **Not a regression** — both versions fail to compile and the user couldn't run this code under either version. The slice's open follow-up at [issues/…/checklist:437](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) explicitly defers fallible `//` and `%` to a separate sub-slice; once those route through Result-returning runtime helpers, the failure-shape shift will resolve naturally. Worth keeping in mind that promoting a local to SifrInt because of a supported AugAssign extends to the local's *other* AugAssign uses on unsupported ops, but this is a sharp edge of the broader migration, not of this slice.

- **N2 — Subscript AugAssign (`arr[0] += big`) is out of scope.** [build_subscript_augassign_elem_stmt](crates/sifr_codegen/src/lower_stmt.rs:4087) is a separate path that emits a `RustStmt::AugAssign` with a `Deref` target, which the new rewrite arm correctly skips (it only matches `Ident` targets). Pre-PR was also broken for this case; same shape stays broken. Not a slice concern.

- **N3 — Unit-test coverage is correct but minimal.** [rewrites_forced_sifr_int_augassign_to_assignment](crates/sifr_codegen/src/expr_render_helpers.rs:1738) pins the literal-source case (`+= Cast(2, I64)` → `&total + SifrInt::from_i64(2)`). The bare-Name registered-local source case (`+= a` → `&total + &a`) is exercised end-to-end by the e2e fixture but not by a focused unit test. The pass-1 review of PR #1825 noted that adding focused unit tests for the bare-Name shape would have caught a regression earlier; the same hardening would apply here. A unit-test sibling that pre-registers a `source` local and asserts `+= source` rewrites to `BinOp { Ref{target}, +, Ref{source} }` would close the matrix. Optional polish, not a correctness concern.

- **N4 — Op-set test coverage is asymmetric.** The unit test covers `+`. The e2e fixture covers `+=`. There's no test exercising `-=` or `*=` directly. Functionally `is_sifr_int_arithmetic_op` and `is_sifr_int_augassign_op` cover all three uniformly, but a parameterized test or sibling cases for `-=` and `*=` would harden against accidental future divergence between the two predicates.

- **N5 — Carry-forwards from prior reviews still apply.** Lexical shadowing, legacy-emission-path coverage, fallible `//`/`%` (especially relevant per N1 above), and function argument/return boundaries — all carried-forward open items at [issues/…/checklist:437](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). Not affected by this slice.

- **N6 — The empty `on_expr` closure** in the pre-scan is a no-op (`let mut on_expr = |_expr: &HirExpr| {};`). It's required by the `walk_stmts` traversal API. No concern, just noting that the visitor descends through expression trees without inspecting them — appropriate because the slice's logic operates at stmt granularity, and the new AugAssign matching is on the stmt itself.
