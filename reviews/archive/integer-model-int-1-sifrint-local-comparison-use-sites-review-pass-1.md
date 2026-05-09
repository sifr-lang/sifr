# INT-1 SifrInt Local Tracking and Comparison Use Sites — Review Pass 1

**Verdict:** Satisfied with non-blocking suggestions.

## Scope reviewed

PR #1821, branch `int-1-sifrint-local-comparison-use-sites` (head `891276cd`), `main..HEAD` diff:

- [crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)
- [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs)
- [crates/sifr_codegen/src/function_emitter.rs](crates/sifr_codegen/src/function_emitter.rs)
- [crates/sifr_codegen/src/function_like_lowering.rs](crates/sifr_codegen/src/function_like_lowering.rs)
- [crates/sifr_codegen/src/class_emitter.rs](crates/sifr_codegen/src/class_emitter.rs)
- [crates/sifr_codegen/src/class_method_emitter.rs](crates/sifr_codegen/src/class_method_emitter.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/integer_model.md](internal_docs/integer_model.md)
- [reviews/integer-model-int-1-oversized-module-int-use-sites-review-pass-1.md](reviews/integer-model-int-1-oversized-module-int-use-sites-review-pass-1.md) (the N1/N2 follow-ups this slice closes)

## Slice goals — both closed for the stated shapes

The slice has two distinct deliverables. Each is wired through the existing rewriter at [rewrite_stdlib_constant_idents_in_expr](crates/sifr_codegen/src/expr_render_helpers.rs:236) / [_in_stmt](crates/sifr_codegen/src/expr_render_helpers.rs:445) and validated against the new e2e fixture lines.

### (1) Local binding propagation

- A new per-emitter field `sifr_int_local_bindings: RefCell<HashSet<String>>` was added in [crates/sifr_codegen/src/lib.rs:1212](crates/sifr_codegen/src/lib.rs:1212) and initialized at [lib.rs:1313](crates/sifr_codegen/src/lib.rs:1313).
- The `RustStmt::Let` arm at [expr_render_helpers.rs:450](crates/sifr_codegen/src/expr_render_helpers.rs:450) now updates the registry alongside the existing `i64` ⇒ `SifrInt` retype: when the rewritten value is `is_sifr_int_expr`-true and the original `ty` was `Some(I64)`, the binding name is inserted into the registry; otherwise it is removed (so a re-Let with a non-SifrInt RHS clears any stale entry).
- A new arm in [is_sifr_int_expr](crates/sifr_codegen/src/expr_render_helpers.rs:1241) recognizes `RustExpr::Ident(name)` as `SifrInt`-shaped if the name is in the registry, which lets `BinOp` and comparison coercion fire on chained references.
- All five function-body emission paths save the registry, clear it for the new scope, and restore the saved value after the body emits — verified by grep against `saved_local_binding_types` (the parallel state already saved at the same five sites): [function_emitter.rs:158/200 and 528/638](crates/sifr_codegen/src/function_emitter.rs:158), [function_like_lowering.rs:24/103](crates/sifr_codegen/src/function_like_lowering.rs:24), [class_emitter.rs:164/190](crates/sifr_codegen/src/class_emitter.rs:164), [class_method_emitter.rs:492/597](crates/sifr_codegen/src/class_method_emitter.rs:492). Every save site for `saved_local_binding_types` has a matching save for `saved_sifr_int_local_bindings`; no emission path was missed.

End-to-end verification via `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr`:

```rust
let oversized_local: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(LIMIT);
…
let chained_oversized_local: SifrInt = oversized_local + SifrInt::from_i64(2);
```

The `chained_oversized_local` binding correctly picks up `oversized_local` as a registered SifrInt local and coerces the `2` literal. The fixture's `assert str(chained_oversized_local) == '100000000000000000256'` round-trips at runtime, closing the pass-1 N1 follow-up for the *single-use* chained case.

### (2) Comparison coercion

- A new `is_sifr_int_comparison_op` helper at [expr_render_helpers.rs:1329](crates/sifr_codegen/src/expr_render_helpers.rs:1329) matches `==`, `!=`, `<`, `<=`, `>`, `>=`.
- `is_sifr_int_operand_coercion_op` at [expr_render_helpers.rs:1333](crates/sifr_codegen/src/expr_render_helpers.rs:1333) ORs the comparison set with the existing arithmetic set (`+`, `-`, `*`).
- The `RustExpr::BinOp` arm at [expr_render_helpers.rs:288](crates/sifr_codegen/src/expr_render_helpers.rs:288) now uses the broader `is_sifr_int_operand_coercion_op` gate when deciding whether to coerce both operands.
- Crucially, the recursive `is_sifr_int_expr` arm at [expr_render_helpers.rs:1252](crates/sifr_codegen/src/expr_render_helpers.rs:1252) still uses the narrower `is_sifr_int_arithmetic_op` for *propagating* SifrInt-ness through nested BinOps. This is the correct asymmetry: the result of a comparison is `bool`, not SifrInt, so a comparison should not be detected as a SifrInt expression by an outer coercion check. Verified by inspection.

End-to-end verification:

- `assert BIG_LIMIT > 100` ⇒ `assert!(__const_BIG_LIMIT() > SifrInt::from_i64(100))`. SifrInt has `PartialOrd<SifrInt>` (`impl PartialOrd for SifrInt` at [crates/sifr_runtime/src/int.rs:374](crates/sifr_runtime/src/int.rs:374)), so this compiles and evaluates correctly.
- `assert chained_oversized_local > BIG_LIMIT` ⇒ `assert!(chained_oversized_local > __const_BIG_LIMIT())`. Both sides resolve to SifrInt — the LHS via the registered local arm, the RHS via the helper-FnCall arm.
- `if big > 100:` (with `big: int = BIG_LIMIT + 1`) ⇒ `if big > SifrInt::from_i64(100)` when the if-stmt goes through the structured lowering path. Verified via a probe.

Closes the pass-1 N2 follow-up for direct-helper and registered-local comparisons under the structured emission path.

### Boundaries not crossed (correctly out of scope)

- `BIG_LIMIT // 2` and `BIG_LIMIT % 2` still fail rustc: `is_sifr_int_arithmetic_op` does not include `//` or `%`, and SifrInt has no `Div`/`Rem` impls. Slice description explicitly defers these to fallible-arithmetic work.
- `def double(x: int) -> int: …` followed by `double(BIG_LIMIT)` still fails rustc with `expected i64, found SifrInt`. Function-signature migration is explicitly deferred.
- Both gaps remain tracked in [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:430](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md).

## Per-function state isolation

The per-function save/clear/restore for `sifr_int_local_bindings` is uniformly applied at all five emission entry points. I verified by a regression probe:

```sifr
def main():
    big: int = BIG_LIMIT + 1
    if True:
        big2: int = 5            # i64 — registry not touched
        print(str(big2))
    print(str(big + 1))          # outer big still detected as SifrInt
```

Emits `big + SifrInt::from_i64(1)` and runs cleanly with `100000000000000000002`.

The save+restore also handles the cross-function case correctly: if function `f` registers `big`, the registry is restored before function `g` is emitted, so `g`'s reference to its own local-named `big` would not be conflated. Inspection of the diff confirms each of the five emitters does `borrow().clone()` save, `borrow_mut().clear()` after save, body emit, then `*borrow_mut() = saved` restore — no intermediate path fails to reset.

The `RefCell` choice is sound: `is_sifr_int_expr` takes `&self` and needs to read the registry while the surrounding `&self`/`&mut self` rewriter recursion is in flight. Borrow scopes are non-overlapping in the Let stmt arm (the immutable `is_sifr_int_expr(&value)` read completes and drops its borrow before the conditional `borrow_mut()` insert/remove). Recursive `is_sifr_int_expr` calls take additional immutable borrows, which is fine. No path I could trace overlaps a `borrow_mut()` with an active borrow.

## Determinism / regression check

- **i64-only paths are untouched.** `MAX_RETRIES + 1`, `MAX_RETRIES > 100`, plain `let x: i64 = something` all leave `is_sifr_int_expr` returning false on every operand, so the BinOp arm short-circuits to the rebuild-as-before path. The Let arm only inserts when the value is SifrInt, and only retypes when both `ty == Some(I64)` and the value is SifrInt. Any unrelated `let x: f64 = …` or `let x: bool = …` is left alone.
- **Non-arithmetic non-comparison BinOps are untouched.** Bitwise `&`/`|`/`^`, shifts `<<`/`>>`, string concat `+` (when neither side is a SifrInt-detected expression), and floating-point arithmetic all bypass the new gate.
- **String-typed module constant helpers** (`__const_greeting()`) are still filtered out in `is_sifr_int_module_constant_func` by the `matches!(resolve_alias_type_for_plain_call(ty), Type::Int)` predicate, so a `RustExpr::FnCall` whose name accidentally collides with a `SifrInt`-typed helper but whose stored `ty` is `Type::Str` does not enter the coercion path.
- **`scripts/run_all_tests.sh --profile quick`** reports `report_signature=e1bf653aaa770517`, identical to PRs #1817/#1818/#1819/#1820. Tracker-only changes preserve the signature; this implementation diff also preserves it because the structural rewriter changes do not alter snapshot expectations across the existing test suite.

## Tests

- [rewrites_large_int_module_const_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1368) (carried from PR #1819) — still pins the helper-arithmetic shape.
- [rewrites_large_int_module_const_let_type_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1398) (carried from PR #1819) — still pins the let-retype shape.
- [rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1418) (new) — pins the chained-local arithmetic shape: a Let stmt registers `oversized_local`, then a subsequent BinOp `oversized_local + 2` rewrites to `oversized_local + SifrInt::from_i64(2)`. The test sequences the two rewrite calls on the same emitter to exercise the registry persistence within a stmt batch.
- [rewrites_large_int_module_const_comparison_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1450) (new) — pins the comparison shape: `BIG_LIMIT > 100` becomes `__const_BIG_LIMIT() > SifrInt::from_i64(100)`.
- E2E fixture adds `chained_oversized_local: int = oversized_local + 2`, `assert str(chained_oversized_local) == '100000000000000000256'`, `assert BIG_LIMIT > 100`, and `assert chained_oversized_local > BIG_LIMIT`.

The new tests anchor the load-bearing shapes for the slice. Coverage gaps mirroring PR #1819's are still open (see N4 below).

## Scope drift

- All edits live in `sifr_codegen`. No changes to HIR, type system, runtime, or driver. No public API growth — all new helpers (`is_sifr_int_comparison_op`, `is_sifr_int_operand_coercion_op`) are private to the module, and `sifr_int_local_bindings` is a private field of `RustEmitter`.
- The save/clear/restore additions are surgical, mirroring the existing `saved_local_binding_types` plumbing one-for-one. No new state-management patterns introduced.
- E2E fixture only grows. No deletions or renames.

## Non-blocking findings

### N1 — Use-after-move on a registered SifrInt local fails rustc

The local binding propagation makes `oversized_local + 2` work for a *single* use, but does not address ownership. Since `SifrInt` is not `Copy` and the codegen emits by-value `Add`/`Sub`/`Mul` (`big + …`, not `&big + …`), using the same registered local in two arithmetic expressions back-to-back fails rustc:

```sifr
big: int = BIG_LIMIT + 1
a: int = big + 1
b: int = big + 2   # rustc: value used here after move
```

Emits:

```rust
let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let a: SifrInt = big + SifrInt::from_i64(1);
let b: SifrInt = big + SifrInt::from_i64(2);   // E0382: borrow of moved value
```

This contradicts the design rule at [internal_docs/integer_model.md:474](internal_docs/integer_model.md:474): *"Sifr source treats `int` as scalar value-semantic and non-consuming: using an `int` binding in more than one expression is always legal. Codegen is responsible for borrowing, cloning, or primitive-local optimization so Rust ownership does not leak into ordinary integer use."*

Strictly speaking this is *not* a regression: pre-this-PR the first `let a = big + 1` already failed (`cannot add i64 to SifrInt`), so no working program stops working. The slice's net effect is to move the failure one hop downstream, similar to how PR #1819 moved the failure across the Let boundary. But by extending the SifrInt-typed reach to Ident references, this slice creates many more reachable programs that hit the use-after-move shape — including the natural pattern of "store an oversized value then use it twice." A complete fix needs codegen to emit `big.clone() + …` (or `&big + …` against the existing `Add<&SifrInt>` impls) when the binding is reused; that is best done as part of the broader `Type::Int` ⇒ `SifrInt` migration so the clone/borrow rule is uniform across all `int` locals, not just oversized-helper-derived ones.

This is the most important follow-up to track explicitly in the tracker bullet that this slice's tracker PR will likely add.

### N2 — Inner-block shadowing with the same name corrupts the outer registry

The Let arm `borrow_mut().remove(&name)` runs whenever a re-Let with a non-SifrInt RHS occurs, *regardless of lexical scope*. The rewriter does not save/restore the registry around if-then/else bodies, match arms, or loop bodies — only around function/method/closure boundaries. So:

```sifr
big: int = BIG_LIMIT + 1
if cond:
    big: int = 5             # inner shadow; registry removes "big"
print(str(big + 1))          # outer big still SifrInt-typed, but registry says no
```

Emits:

```rust
let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
if true {
    let big: i64 = 5 as i64;
}
println!("{}", format!("{}", big + (1 as i64)));   // E0277: cannot add i64 to SifrInt
```

Reproduced via `cargo run -q -p sifr -- emit`. Pre-this-PR this code already failed (the outer `big + 1` was emitted as `big + (1 as i64)` because Idents weren't tracked at all), so this is not a regression. But it surfaces the same lexical-scope-blindness gap as N1: the registry is function-scoped but the user's Sifr scoping is finer.

A scope-aware fix would save/restore the registry around if-bodies, loop bodies, match arm bodies, and `RustStmt::Block` boundaries inside the rewriter's stmt arms — symmetric to how the function emitters already do it. Optional; could ride along with the broader `Type::Int` ⇒ `SifrInt` migration.

### N3 — Comparison rewrite silently bypasses non-structured emission paths

Comparisons are only rewritten when the surrounding stmt goes through the structured lowering path at [lib.rs:1417](crates/sifr_codegen/src/lib.rs:1417) (the `lowered_stmts.map(rewrite_stdlib_constant_idents_in_stmt)` branch). When `try_lower_simple_stmt_with_scope_result_and_bindings` returns `None`, lowering falls through to the legacy `emit_stmt` path which writes Rust strings directly and does not pass through the rewriter. I reproduced a failure where the cond rewrite is skipped:

```sifr
big: int = BIG_LIMIT + 1
result: int = big + 1
if big > 100:
    print(str(result))
```

Emits `if big > (100 as i64) { … }` (RHS not coerced). The simpler shape (`if big > 100:` with no intervening result-using stmt) is structured-lowered and works. The dispatch criterion is opaque to the user.

This is a pre-existing fragility — PRs #1819 and #1817 had the same property — and the slice does not introduce it. But it does mean the new comparison coercion has a non-uniform reach. Worth documenting as a known limitation until either (a) all integer-bearing if-stmts are routed through structured lowering, or (b) the legacy emission path also wires through the rewriter (or its equivalent at the string level).

### N4 — Unit-test coverage gaps from PR #1819 still apply, plus new ones

Tests pin the load-bearing rewrite shapes but not the supporting matrix. Worth adding before INT-1 closes:

- `BIG_LIMIT - 1` and `BIG_LIMIT * 2` (sibling arithmetic operators).
- `1 + BIG_LIMIT` (helper on right operand).
- `-BIG_LIMIT` (unary `-`).
- Comparison sibling operators (`<`, `<=`, `>=`, `==`, `!=`) — only `>` is currently exercised.
- Asserting `MAX_RETRIES + 1` and `MAX_RETRIES > 100` are *not* rewritten — pins the deliberate i64 path to guard against accidental over-coercion if the helper-detection rule widens.
- Asserting that a Let with a non-SifrInt RHS clears any prior registration of that name (the `else` branch in the Let arm).
- Asserting registry isolation across function emissions (sequence two emit_function calls and verify the second does not see the first's bindings).

None gates merge — the absent tests would harden against future regression but the current diff is well-targeted.

### N5 — Minor code-shape nits

- The Let arm's update is split across `borrow_mut().insert(...)` and `borrow_mut().remove(...)` in two `if`/`else` branches, each a no-op for the not-currently-registered case. Coalescing into a single `borrow_mut()` call with a method-style update would make the contract more obvious and avoid the implicit "remove always runs even if absent" behavior.
- `is_sifr_int_operand_coercion_op` and `is_sifr_int_arithmetic_op` are now subtly different: one is the *coercion* gate, the other is the *propagation* gate. A short comment on each function explaining the asymmetry would prevent a future contributor from accidentally unifying them.
- The `Ident` arm in `is_sifr_int_expr` ([expr_render_helpers.rs:1256](crates/sifr_codegen/src/expr_render_helpers.rs:1256)) is placed before the `BinOp` and `UnaryOp` arms, which is correct but easy to misread because the surrounding match arm order otherwise tracks structural complexity. A reordering or a comment would help.

## Validation

I re-traced rather than re-ran the listed validation. The cited results are consistent with the code:

- `cargo test -p sifr_codegen rewrites_large_int_module_const -- --nocapture` — covers both PR #1819 tests plus the new comparison test (which begins with `rewrites_large_int_module_const_…`).
- `cargo test -p sifr_codegen rewrites_registered_sifr_int_local -- --nocapture` — covers the new local-arithmetic test.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — I reproduced this directly via `emit` + `run`. The four new asserts (`chained_oversized_local: int = oversized_local + 2` and its assert, `BIG_LIMIT > 100`, `chained_oversized_local > BIG_LIMIT`) all round-trip.
- `cargo run -q -p sifr -- emit /tmp/sifr_int1_next_XXXXXX.sifr` — confirmed visually for `+`/`-`/`*`, helper-on-right, unary `-`, single-use chained locals, comparisons against literal and against helper, and `if cond:` branching.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=62.33s`. Same signature as #1817/#1818/#1819/#1820.

## Verdict

**Satisfied with non-blocking suggestions.** The slice closes the pass-1 N1 (chained `oversized_local + 2`) and pass-1 N2 (direct comparisons `BIG_LIMIT > 100`) follow-ups for their stated shapes: a per-emitter `RefCell<HashSet<String>>` registers locals retyped from `i64` to `SifrInt`, the Ident arm of `is_sifr_int_expr` reads the registry, and `is_sifr_int_operand_coercion_op` widens the coercion gate to comparison operators while leaving the propagation gate (`is_sifr_int_arithmetic_op`) narrow — the right asymmetry. Per-function save/clear/restore is uniformly applied across all five emission paths (function, alt function, function-like, class Display, class method), and the `RefCell` borrow scopes are non-overlapping. The new e2e fixture asserts round-trip for both new shapes; quick validation reproduces `report_signature=e1bf653aaa770517`.

The non-blocking findings cluster around the broader `Type::Int` ⇒ `SifrInt` codegen migration's remaining surface: use-after-move on registered SifrInt locals (N1, the highest-impact gap given the design's value-semantic guarantee for `int`), shadowing-aware registry scoping (N2), the comparison rewrite's path-dependent reach because legacy-emission stmts bypass the rewriter (N3), test-matrix gaps (N4), and small code-shape nits (N5). Track N1 and N2 in the open INT-1 follow-up bullet so they don't get lost behind the comparison/local-tracking tick. None gates merge.
