# INT-1 SifrInt Assignment Targets — Review Pass 1

**Verdict:** Changes requested.

A small, narrow-shape regression on a natural and previously-working code pattern (`b: int = a` where `a` is a registered SifrInt local) gates merge. The slice's stated goal — making `total = total + big` compile — is achieved, and most other shapes I probed work. The regression is mechanically simple to fix and the rest of the implementation is sound, so I expect a quick turnaround. Details below.

## Scope reviewed

PR #1825, branch `int-1-sifrint-assignment-targets` (head `32c5c818`), `main..HEAD` diff:

- [crates/sifr_codegen/src/lib.rs](crates/sifr_codegen/src/lib.rs)
- [crates/sifr_codegen/src/function_emitter.rs](crates/sifr_codegen/src/function_emitter.rs)
- [crates/sifr_codegen/src/function_like_lowering.rs](crates/sifr_codegen/src/function_like_lowering.rs)
- [crates/sifr_codegen/src/class_emitter.rs](crates/sifr_codegen/src/class_emitter.rs)
- [crates/sifr_codegen/src/class_method_emitter.rs](crates/sifr_codegen/src/class_method_emitter.rs)
- [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/integer_model.md](internal_docs/integer_model.md) — value-semantic rule at [line 474](internal_docs/integer_model.md:474).
- [reviews/integer-model-int-1-sifrint-local-value-semantics-review-pass-1.md](reviews/integer-model-int-1-sifrint-local-value-semantics-review-pass-1.md) — the prior review whose N2 ("`total = total + big` still emits invalid Rust") this slice closes.
- Pre-PR baseline behavior verified by checking out commit `7da872e0` (PR #1823's tracker, equivalent to PR #1823's implementation state) into a worktree and re-emitting probes.

## Slice goal — partially closed

The pass-1 N2 follow-up was that `total: int = 0; total = total + big` (with `big` a registered SifrInt local) emitted `total = SifrInt::from_i64(total) + &big`, failing rustc because `total: i64`. This slice closes that case.

### Pre-scan: `register_sifr_int_forced_local_bindings`

A new pass at [function_emitter.rs:97-148](crates/sifr_codegen/src/function_emitter.rs:97) runs during `register_local_body_binding_types`. It walks the function body once with `TraversalConfig::LOCAL_SCOPE_ONLY` (so nested function bodies are not visited — their own emit calls handle them) and identifies `int`-typed local bindings whose initializers or assignment values transitively require SifrInt storage. The fixed-point loop adds new names until convergence; transitive cases like `a = BIG_LIMIT + 1; b = a; c = b` all converge to forced.

The visitor only matches `HirStmt::Let` and `HirStmt::Assign` (with `name` directly on the stmt) — so function parameters, AugAssign targets, tuple-unpack targets, and class-attr assignments are not pre-scanned. This is consistent with the slice description ("avoid changing function parameter/signature boundaries") and with the prior tracker's split between local-codegen migration (this slice) and signature migration (deferred).

`hir_expr_needs_sifr_int_storage` at [function_emitter.rs:707-737](crates/sifr_codegen/src/function_emitter.rs:707) recognizes:

- `LargeIntLiteral` — true.
- `Name` — true if the name is in `forced` or in `module_sifr_int_bindings` (Type::Int helpers with a `()` rust_name).
- Type::Int `BinOp { +, -, * }` — true if either operand needs SifrInt.
- Type::Int `UnaryOp { +, - }` — true if operand needs SifrInt.
- Anything else — false.

The set of operators is appropriately restricted to the operators the codegen rewriter actually coerces. That keeps the pre-scan from forcing locals on operator shapes the BinOp coercion path doesn't handle (e.g., `<<`, `>>`, `//`, `%`).

### Per-function scope save/restore

All five emitter entry points save/clear/restore `sifr_int_forced_local_bindings`:

- [function_emitter.rs:213/256](crates/sifr_codegen/src/function_emitter.rs:213) and [function_emitter.rs:587/700](crates/sifr_codegen/src/function_emitter.rs:587) (regular and alt function emitter)
- [function_like_lowering.rs:25/107](crates/sifr_codegen/src/function_like_lowering.rs:25) (operator/protocol lowering)
- [class_emitter.rs:165/194](crates/sifr_codegen/src/class_emitter.rs:165) (Display impl)
- [class_method_emitter.rs:493/603](crates/sifr_codegen/src/class_method_emitter.rs:493) (class methods)

Each site mirrors the existing `saved_sifr_int_local_bindings` plumbing one-for-one. `RefCell::borrow().clone()` save, `borrow_mut().clear()` post-save, body emit, then `*borrow_mut() = saved` restore. Verified by grep: every save site for `saved_sifr_int_local_bindings` has a matching save for `saved_sifr_int_forced_local_bindings`. ✓

### Let/Assign rewrite

The `RustStmt::Let` arm at [expr_render_helpers.rs:473-499](crates/sifr_codegen/src/expr_render_helpers.rs:473) is amended:

```rust
let value = self.rewrite_stdlib_constant_idents_in_expr(value);
let force_sifr_int = self.is_forced_sifr_int_local(&name);
let value_is_sifr_int = self.is_sifr_int_expr(&value);
let (ty, value) = if matches!(ty, Some(crate::RustType::I64))
    && (value_is_sifr_int || force_sifr_int)
{
    let value = self.coerce_expr_to_sifr_int(value);   // <-- new
    self.sifr_int_local_bindings.borrow_mut().insert(name.clone());
    (Some(crate::RustType::Named("SifrInt".to_string())), value)
} else {
    if !force_sifr_int {
        self.sifr_int_local_bindings.borrow_mut().remove(&name);
    }
    (ty, value)
};
```

Three changes:
- Forcing-aware retype gate (`value_is_sifr_int || force_sifr_int`).
- The value gets passed through `coerce_expr_to_sifr_int` so a forced local with a small-literal initializer (`assigned_total: int = 0`) gets `SifrInt::from_i64(0)` instead of leaving the literal as `i64`.
- The else branch only removes from `sifr_int_local_bindings` when not force-flagged, preserving the registry for forced locals across re-Lets with non-SifrInt-valued sources.

The `RustStmt::Assign` arm at [expr_render_helpers.rs:514-530](crates/sifr_codegen/src/expr_render_helpers.rs:514) gains a target-aware coerce: when the rewritten target is `Ident(name)` and the name is registered or forced, the rewritten value is run through `coerce_expr_to_sifr_int`, then the name is re-inserted into the registry (idempotent for already-registered names; load-bearing for transitive forcing where the registry might not have been populated yet).

Together, these make `assigned_total: int = 0; assigned_total = assigned_total + reusable_oversized_local; assigned_total = assigned_total + 2` emit:

```rust
let mut assigned_total: SifrInt = SifrInt::from_i64(0);
assigned_total = &assigned_total + &reusable_oversized_local;
assigned_total = &assigned_total + SifrInt::from_i64(2);
```

…which compiles and round-trips at runtime. The fixture's final assert `str(assigned_total) == '100000000000000000003'` passes.

### `coerce_expr_to_sifr_int`'s new BinOp arm

A new arm in [coerce_expr_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1278-1287) handles `BinOp { +/-/* }` whose operands include a SifrInt-shaped side, recursively re-coercing each side. This is needed because the Assign target-aware coerce runs over an already-rewritten BinOp from `rewrite_stdlib_constant_idents_in_expr`, and without this arm the outer coerce would fall to the `other if is_sifr_int_expr(&other)` pass-through and miss the recursive structure. The recursion is bounded by the AST and the operands quickly hit the `Ref { Ident }` or pass-through terminators. ✓

I verified the working cases via `cargo run -q -p sifr -- emit` against several probes:

| Probe                                            | Emitted Rust                                           | Result |
|--------------------------------------------------|--------------------------------------------------------|--------|
| `total: int = 0; total = total + big; …`         | `let mut total: SifrInt = …; total = &total + &big;`   | ✓      |
| `total: int = 0; total = a + 0` (BinOp source)   | `total = &a + SifrInt::from_i64(0)`                    | ✓      |
| `total: int = 0; total = -a` (UnaryOp source)    | `total = -&a`                                          | ✓      |
| `b: int = BIG_LIMIT` (helper)                    | `let b: SifrInt = __const_BIG_LIMIT()`                 | ✓      |
| `total: int = 0; total = total + 1; …` (small-only) | stays `i64`                                          | ✓      |
| Transitive force `a → b → c` through chained arithmetic | each retyped to SifrInt where SifrInt source flows | ✓ |

Quick validation reproduces `report_signature=e1bf653aaa770517` and the e2e fixture round-trips.

## Blocker

### B1 — Bare `Ident` (registered SifrInt local) in Let or Assign value position emits invalid Rust

The new `coerce_expr_to_sifr_int(value)` calls in the Let arm and the Assign arm are unconditional whenever the target is registered/forced. But `coerce_expr_to_sifr_int`'s first match arm — added in PR #1823 — wraps a registered Ident in `Ref { mutable: false, expr: Ident }` ([expr_render_helpers.rs:1271-1276](crates/sifr_codegen/src/expr_render_helpers.rs:1271)). That borrow shape was designed for **operand position** (BinOp/UnaryOp), where `Add<&SifrInt> for SifrInt` and friends accept it. In **value position** (the RHS of a `let _: SifrInt = …` or a `total = …` assign), `&local` produces `&SifrInt`, which does not unify with the target's `SifrInt` type.

#### Reproduction (post-this-PR)

```sifr
BIG_LIMIT: int = 10 ** 20

def main():
    a: int = BIG_LIMIT + 1
    b: int = a            # bare-Name alias
    print(str(b))
```

`cargo run -q -p sifr -- emit` produces:

```rust
let a: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let b: SifrInt = &a;          // E0308: expected `SifrInt`, found `&SifrInt`
println!("{}", format!("{}", b));
```

`cargo run -q -p sifr -- run` fails compilation with rustc error E0308. Same shape via Assign:

```sifr
def main():
    a: int = BIG_LIMIT + 1
    total: int = 0
    total = a              # bare-Name alias on assign
```

emits:

```rust
let mut total: SifrInt = SifrInt::from_i64(0);
total = &a;                   // E0308
```

#### Pre-PR baseline (PR #1823, commit `7da872e0`)

I checked out the previous file states for the six changed `sifr_codegen` files and re-ran emit on the same `b: int = a` probe:

```rust
let a: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let b: SifrInt = a;           // compiles, prints "100000000000000000001"
```

So pre-PR the bare-Name alias compiled (with move semantics for `a`). Post-PR it fails outright. **This is a real regression on a natural, common Sifr pattern** (renaming/aliasing an `int` local).

#### Why it happens

The pre-PR Let arm (introduced in PR #1819, refined in #1821 / #1823) only **retyped** the binding — `value` was passed through unchanged whenever `value_is_sifr_int` was true. Pre-PR-#1825 emission for `Let { name: "b", value: Ident("a") }` was `let b: SifrInt = a;`. Compiles via direct assignment (move).

This PR adds `let value = self.coerce_expr_to_sifr_int(value);` ([expr_render_helpers.rs:480](crates/sifr_codegen/src/expr_render_helpers.rs:480)). For the Ident-registered case, that walks the first match arm of `coerce_expr_to_sifr_int` and wraps in `Ref`. The wrap was correct in PR #1823's operand-position contexts but is incorrect in value-position contexts.

The new test `rewrites_forced_sifr_int_assignment_target_storage` covers the literal-source forcing case (`Cast(Literal(0), I64)` → `SifrInt::from_i64(0)`), which works correctly. There is no test for the bare-Name registered-source case, which is exactly why the regression slipped through. The e2e fixture also uses a literal source (`assigned_total: int = 0`) and never aliases a SifrInt local through a bare Name in Let/Assign position, so it likewise misses the regression.

#### Suggested fix

The minimum change that closes B1 is to skip the value-position coerce when `value_is_sifr_int` is already true. That preserves PR #1823's pre-existing pass-through for already-SifrInt-shaped values, while still applying the wrap for the forced-but-not-yet-SifrInt-shaped case (small literal initializers, which is the new path the slice needs to enable):

```rust
let (ty, value) = if matches!(ty, Some(crate::RustType::I64))
    && (value_is_sifr_int || force_sifr_int)
{
    let value = if value_is_sifr_int {
        value                                    // already SifrInt; preserve PR #1823 behavior
    } else {
        self.coerce_expr_to_sifr_int(value)      // forced + non-SifrInt: wrap small literal / cast
    };
    self.sifr_int_local_bindings.borrow_mut().insert(name.clone());
    (Some(crate::RustType::Named("SifrInt".to_string())), value)
} else {
    …
};
```

The same gate applies to the Assign arm: only run the target-aware coerce when the rewritten value isn't already SifrInt-shaped (i.e., when it's a literal/cast). For the BinOp/UnaryOp shapes the slice claims (`total = total + big`, `total = -big`), the value rewrite already produces a SifrInt-shaped expression via the BinOp/UnaryOp coerce path; the post-rewrite coerce is then a no-op pass-through.

If the team wants `b: int = a` to also preserve source-level value semantics (so `a` stays usable after `b: int = a`, per [integer_model.md:474](internal_docs/integer_model.md:474)), a value-position-specific coerce that emits `a.clone()` for a registered Ident — instead of `a` (move) or `&a` (borrow) — would be the durable fix. That is strictly an improvement over the pre-PR move behavior. Either fix unblocks merge; the minimum fix above just restores the pre-PR working shape.

A regression test pinning the bare-Name case (e.g., a unit test asserting `Let { ty: Some(I64), value: Ident("registered") }` rewrites to `Let { ty: Some(SifrInt), value: Ident or Clone, … }` rather than `Ref`) would prevent re-introduction.

## Determinism / regression check (other paths)

I probed several other shapes; all produce correct Rust:

- **i64-only paths untouched**: `total: int = 0; total += 1; total = total + 1` — pre-scan correctly does not force `total` because no SifrInt source flows in. Emits `let mut total: i64 = 0 as i64; total += (1 as i64); total = total + (1 as i64);`. ✓
- **Helper-only Let**: `b: int = BIG_LIMIT` — emits `let b: SifrInt = __const_BIG_LIMIT();`. The helper FnCall passes through the new coerce arms (it's `is_sifr_int_expr`-true and not Ident, not Paren, not BinOp). ✓
- **BinOp source Let**: `b: int = a + 0` — emits `let b: SifrInt = &a + SifrInt::from_i64(0);`. ✓
- **UnaryOp source Let**: `b: int = -a` — emits `let b: SifrInt = -&a;`. ✓
- **Transitive forcing**: `a = BIG_LIMIT + 1; b = a + 1; c = b + 1` (no bare-Name aliases) — all retyped, all use `&a`/`&b` operand borrows. ✓
- **Existing operand cases from #1819/#1821/#1823** (`reuse_a`, `reuse_b`, `negated_reuse`, `reusable_oversized_local < reuse_b`, `BIG_LIMIT > 100`) — all unchanged in shape and round-trip the e2e fixture asserts.
- **`scripts/run_all_tests.sh --profile quick`** reports `report_signature=e1bf653aaa770517` (same as #1817–#1823), confirming no snapshot/test deltas across the rest of the suite.

## Tests

- [rewrites_large_int_module_const_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1486) (carried) — still passes.
- [rewrites_large_int_module_const_let_type_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1516) (carried) — still passes.
- [rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1526) (carried) — still passes.
- [rewrites_large_int_module_const_comparison_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1554) (carried) — still passes.
- [rewrites_registered_sifr_int_local_comparison_to_borrowed_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1598) (carried) — still passes.
- [rewrites_forced_sifr_int_assignment_target_storage](crates/sifr_codegen/src/expr_render_helpers.rs:1620) (new) — covers the literal-source forcing case (Let with `0_i64` value retyping to `SifrInt::from_i64(0)`, then Assign with `2_i64` value rewriting to `total = SifrInt::from_i64(2)`). The structural assertions are correct for the case they cover.

E2E coverage in [module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr) adds:

- `assigned_total: int = 0` (literal-source forced Let)
- `assigned_total = assigned_total + reusable_oversized_local` (BinOp-source forced Assign)
- `assigned_total = assigned_total + 2` (BinOp-source forced Assign)
- `assert str(assigned_total) == '100000000000000000003'` (round-trip)

These pin the load-bearing shapes for the slice's stated scope. Coverage gaps that contributed to B1 slipping through:

- **No test for `let b: int = a` where `a` is a registered SifrInt local.** The bare-Name value-position case.
- **No test for `total = a` (Assign with bare-Name value).** Same shape on the Assign side.
- **No test asserting the i64-only shape stays untouched** when `register_sifr_int_forced_local_bindings` runs and finds nothing forceable. Useful as a guard against future widening of the pre-scan rules.

Adding any of these as unit tests would have caught B1.

## Scope drift

- Stays inside `sifr_codegen`. No HIR, type system, runtime, or driver changes. No public API growth.
- The new helpers (`register_sifr_int_forced_local_bindings`, `is_forced_sifr_int_local`, `hir_expr_needs_sifr_int_storage`) are private. The new `RustEmitter` field `sifr_int_forced_local_bindings: RefCell<HashSet<String>>` is private.
- The fixed-point pre-scan loop terminates because each iteration either adds a name (monotonic) or breaks. Bounded by the local int binding count. ✓
- Function parameters are correctly excluded from the pre-scan because the visitor only walks Let/Assign targets — not parameters or pattern-bound names. The slice description's claim "avoid changing function parameter/signature boundaries" is honored. ✓
- AugAssign (`total += a`) is not pre-scanned and not rewrite-coerced. Same shape was broken pre-PR (`total: i64 += a: SifrInt` failed rustc) and stays broken. Worth an explicit follow-up note (see N1 below).

## Non-blocking findings

(Once B1 is fixed, the items below remain.)

### N1 — `AugAssign` is unhandled

`total += a` (with `a` a SifrInt local) emits `let mut total: i64 = …; total += a;` regardless of forcing, because the pre-scan visitor at [function_emitter.rs:122](crates/sifr_codegen/src/function_emitter.rs:122) only matches `HirStmt::Let` and `HirStmt::Assign`. The rewrite path for `RustStmt::AugAssign` ([expr_render_helpers.rs:530-534](crates/sifr_codegen/src/expr_render_helpers.rs:530)) also doesn't apply target-aware coerce.

This is not a regression — pre-PR the same code emitted the same broken Rust. But the slice description ("assignment targets such as `total = total + big`") implies broad coverage of mutating statements, and the most common shorthand for it is `total += big`. Worth either:

- Extending the pre-scan visitor to include `HirStmt::AugAssign { name, value }` with the same predicate.
- Or explicitly carving it out in the open follow-up bullet so a future reader doesn't think it's covered.

### N2 — Pre-scan walker uses `LOCAL_SCOPE_ONLY`, which is correct

`TraversalConfig::LOCAL_SCOPE_ONLY` skips nested function bodies — which is what we want, because nested-fn emission has its own per-function pre-scan. Other constructs (loops, if branches, match arms, blocks) are descended. I confirmed no nested-fn leakage by reading [hir_analysis/traversal.rs:1-19](crates/sifr_codegen/src/hir_analysis/traversal.rs:1). ✓

### N3 — `coerce_expr_to_sifr_int`'s arm order is now even more load-bearing

After this PR, `coerce_expr_to_sifr_int` has five match arms in a specific order:

1. `Ident` registered → `Ref` (operand-position borrow).
2. `Paren` → recurse.
3. `BinOp { +/-/* }` with SifrInt operands → recurse and re-coerce both sides.
4. `other if is_sifr_int_expr(&other)` → pass through.
5. `Cast { ty: I64 }` → `from_i64`.
6. `other` → `from_i64`.

The arm-1 placement before arm-4 is load-bearing: arm-4 would also match a registered Ident (since `is_sifr_int_expr` was extended in PR #1821 to recognize them) and would pass it through, losing the borrow needed for operand position. PR #1823's review already noted this; the new BinOp arm (arm-3) has analogous ordering sensitivity. A short comment block explaining "arms 1 and 3 must precede arm 4 to ensure operand-position semantics" would protect future contributors from refactoring.

This is a documentation suggestion, not a correctness concern. Optional.

### N4 — Carry-over follow-ups still apply

Lexical shadowing, legacy-emission-path coverage, fallible `//` and `%`, and function argument/return boundaries remain open from prior reviews. Not introduced or affected by this slice. Already tracked in [issues/…/checklist:434](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md).

## Validation

I re-traced rather than re-ran all of the listed validation. The cited results are consistent with the code:

- `cargo test -p sifr_codegen rewrites_forced_sifr_int_assignment_target_storage -- --nocapture` — I ran this; passes.
- `cargo test -p sifr_codegen rewrites_registered_sifr_int_local -- --nocapture` — I ran this; both passes.
- `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr | rg "assigned_total|reusable_oversized_local"` — I reproduced; the new fixture lines emit cleanly.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — round-trips.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=68.55s`. Same signature as #1817–#1823.

The validation matrix passes because **none of the listed validations exercise the bare-Name aliasing shape that B1 is about**. The regression is silent against the current test suite.

## Verdict

**Changes requested.** The slice's stated goal — making `total: int = 0; total = total + big` compile — is met cleanly: a pre-scan walks Let/Assign targets and forces SifrInt storage on locals reachable from a SifrInt source, scope save/restore is correctly applied at all five emission entry points, the Let arm retypes both forced-with-literal and SifrInt-valued initializers, and the Assign arm coerces the value when the target is registered or forced. Most shapes I probed work, including the slice's e2e fixture.

The blocker (B1) is that the new `coerce_expr_to_sifr_int(value)` calls in the Let arm and Assign arm produce `let _: SifrInt = &a;` for a registered-Ident value, where pre-PR they emitted `let _: SifrInt = a;`. This is a strict regression on a natural and previously-working code shape (`b: int = a` aliasing). The fix is small — skip the value-position coerce when `value_is_sifr_int` is already true (so already-SifrInt-shaped values pass through unchanged, matching PR #1823's behavior), or introduce a value-position coerce that clones registered Idents instead of borrowing them. A regression test for the bare-Name case should be added at the same time.

Once B1 is closed, the remaining non-blocking findings (N1 AugAssign coverage gap, N3 documentation polish) can land separately.
