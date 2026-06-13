# INT-1 Oversized Module `int` Use Sites — Review Pass 1

**Verdict:** Satisfied with non-blocking suggestions.

## Scope reviewed

PR #1819, branch `int-1-oversized-module-int-use-sites` (head `2f0f4e32`), `main..HEAD` diff:

- [crates/sifr_codegen/src/expr_render_helpers.rs](crates/sifr_codegen/src/expr_render_helpers.rs)
- [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)

Reference docs:
- [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
- [internal_docs/integer_model.md](internal_docs/integer_model.md)
- [reviews/integer-model-int-1-large-module-int-codegen-review-pass-1.md](reviews/integer-model-int-1-large-module-int-codegen-review-pass-1.md) (the N1 follow-up this slice closes)

## Slice goal — closed for the stated shape

The pass-1 N1 follow-up was that once an oversized `int` module constant lowered to `__const_BIG_LIMIT() -> SifrInt`, expressions like `BIG_LIMIT + 1` emitted `__const_BIG_LIMIT() + (1 as i64)`, which `rustc` rejected with `cannot add i64 to SifrInt`. The diff closes that on two paths:

1. **BinOp coercion in [rewrite_stdlib_constant_idents_in_expr](crates/sifr_codegen/src/expr_render_helpers.rs:285).** After both operands of a `RustExpr::BinOp` are recursively rewritten, a new arm checks `is_sifr_int_arithmetic_op(&op) && (is_sifr_int_expr(&left) || is_sifr_int_expr(&right))`. When at least one side is recognized as `SifrInt`-typed, both operands are passed through `coerce_expr_to_sifr_int`, which strips an outer `Cast { ty: I64 }` and wraps the inner expression in `SifrInt::from_i64(...)`. `is_sifr_int_arithmetic_op` is restricted to `+`, `-`, `*` ([expr_render_helpers.rs:1320](crates/sifr_codegen/src/expr_render_helpers.rs:1320)), which matches the operators with `Add`/`Sub`/`Mul` impls on `SifrInt` in [crates/sifr_runtime/src/int.rs:219-326](crates/sifr_runtime/src/int.rs:219).

2. **Let-type rewrite in [rewrite_stdlib_constant_idents_in_stmt](crates/sifr_codegen/src/expr_render_helpers.rs:450).** After rewriting the RHS, if the annotated `ty` was `Some(I64)` and the rewritten value is now SifrInt-shaped, the binding is retyped to `Some(Named("SifrInt"))`. This is what lets `oversized_local: int = BIG_LIMIT + LIMIT` end up as `let oversized_local: SifrInt = …` instead of `let oversized_local: i64 = …`.

`is_sifr_int_expr` recognizes:
- `FnCall { func, args: [] }` whose func name (after `rust_expr_identifier_path`) matches a `module_constants` entry whose `ty.resolve_alias()` is exactly `Type::Int` and whose `rust_name` ends in `()` ([expr_render_helpers.rs:1261](crates/sifr_codegen/src/expr_render_helpers.rs:1261)).
- `FnCall` whose func is `SifrInt::from_i64` or `sifr_runtime::SifrInt::from_i64`.
- `BinOp` with `+`/`-`/`*` whose either operand is recursively `SifrInt`.
- `UnaryOp { op: "-", … }` whose operand is `SifrInt`.
- `Paren` of a `SifrInt` expression.

I traced the production path end-to-end via `cargo run -q -p sifr -- emit` against the new fixture and several probes:

- `BIG_LIMIT + 1` ⇒ `__const_BIG_LIMIT() + SifrInt::from_i64(1)` (matches the e2e fixture's `assert str(BIG_LIMIT + 1) == '100000000000000000001'`).
- `oversized_local: int = BIG_LIMIT + LIMIT` (where `LIMIT: int = BASE_LIMIT + 4` is i64-backed) ⇒ `let oversized_local: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(LIMIT)`. The fixture's `assert str(oversized_local) == '100000000000000000254'` round-trips.
- `1 + BIG_LIMIT` ⇒ `SifrInt::from_i64(1) + __const_BIG_LIMIT()` — the `||` in the arithmetic guard catches helper-on-right, and `coerce_expr_to_sifr_int` strips the i64 cast before wrapping.
- `-BIG_LIMIT` ⇒ `-__const_BIG_LIMIT()`, picking up `Neg` for `SifrInt` from [crates/sifr_runtime/src/int.rs:333](crates/sifr_runtime/src/int.rs:333).
- `BIG_LIMIT * 2 - 5` ⇒ `(__const_BIG_LIMIT() * SifrInt::from_i64(2)) - SifrInt::from_i64(5)`. The inner BinOp returns SifrInt-shaped, so the outer guard fires recursively; `is_sifr_int_expr(BinOp)` correctly recurses through the arithmetic-op arm.
- `big = BIG_LIMIT + 1` (no annotation) ⇒ the original `Some(I64)` annotation produced by inference is rewritten to `Some(SifrInt)`, so the statement becomes `let big: SifrInt = …`. Runs cleanly and prints `100000000000000000001`.

The runtime parse-decimal panic shape from the wave-3 slice is unchanged, the auto-import collector still picks up `SifrInt` and `sifr_runtime` symbols transitively (per the existing `mark_symbol` in `ir_imports.rs`), and small-int constants like `MAX_RETRIES + 1` continue to lower as plain `MAX_RETRIES + (1 as i64)` because `is_sifr_int_expr` returns false for any `Ident` (see N1 below).

## Determinism / regression check

- **i64 + i64 paths are untouched.** `MAX_RETRIES + 1` ⇒ both operands go through the recursive rewrite, but `Ident("MAX_RETRIES")` and `Cast(1, I64)` both fall into `is_sifr_int_expr`'s wildcard and return false. The new arithmetic guard short-circuits and the original `BinOp { left, op, right }` is rebuilt as before. No diff in the existing fixture's `assert str(LIMIT) == '254'` or `assert str(NEGATIVE_LIMIT) == '-13'` lines.
- **Non-arithmetic operators are untouched.** Comparison `>`/`<`/`==`, bitwise `&`/`|`/`^`, shifts, string concat, list/dict ops, and float arithmetic all leave `is_sifr_int_arithmetic_op` returning false, so they fall through to the unchanged plain-BinOp path.
- **Non-`Type::Int` module constants don't leak into the rewrite.** [is_sifr_int_module_constant_func](crates/sifr_codegen/src/expr_render_helpers.rs:1261) filters by `matches!(resolve_alias_type_for_plain_call(ty), Type::Int)`, which rejects `Type::FixedInt(_)`, `Type::BigInt`, `Type::Float`, etc. Aliased `int` (e.g. `type Meters = int`) still resolves through `resolve_alias_type_for_plain_call` and is correctly recognized.
- **String-typed module constant helpers (`__const_greeting()`)** also do not match because their stored `ty` is `Type::Str`, not `Type::Int`. So `__const_greeting() + "..."` still goes through the legacy path. Verified by the existing `lowers_simple_module_string_const_item` tests staying green and by inspecting `try_lower_simple_module_string_const_item` registering `Type::Str`.
- **The let-type rewrite only fires when the original annotation was `Some(I64)` and the value is `SifrInt`.** A `let x: f64 = …` is left alone; a `let x = …` (no annotation) is also left alone since `Some(I64)` doesn't match `None`. The narrowness of the gate avoids accidental retyping of unrelated annotated lets.
- **`scripts/run_all_tests.sh --profile quick`** is reported as `report_signature=e1bf653aaa770517`, identical to the prior implementation review's signature for PR #1817 and matching the tracker review's. So no test or build deltas across the suite.

## Scope drift

- The diff only touches `expr_render_helpers.rs` (the rewrite plumbing) and the e2e fixture. No edits to HIR, type system, runtime, or driver. No churn in unrelated files.
- The new helpers (`coerce_expr_to_sifr_int`, `is_sifr_int_expr`, `is_sifr_int_module_constant_func`, plus the free helpers `sifr_int_from_i64_expr`, `is_sifr_int_arithmetic_op`, `rust_expr_identifier_path`, `string_path_matches`) are private to the module. No public API growth.
- The `BinOp` arm in `rewrite_stdlib_constant_idents_in_expr` uses an early `return` rather than match-arm-return; minor stylistic inconsistency with the surrounding match, but the conditional shape doesn't fit cleanly into a single match arm. Acceptable.

## Tests

- [rewrites_large_int_module_const_arithmetic_to_sifr_int_operands](crates/sifr_codegen/src/expr_render_helpers.rs:1355) pins the BinOp rewrite for `BIG_LIMIT + (1 as i64)` ⇒ `__const_BIG_LIMIT() + SifrInt::from_i64(1)`. Sufficient as a structural anchor.
- [rewrites_large_int_module_const_let_type_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1385) pins the let-type retype for `let x: i64 = BIG_LIMIT` ⇒ `let x: SifrInt = __const_BIG_LIMIT()`.
- [module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr) adds:
  - `assert str(BIG_LIMIT + 1) == '100000000000000000001'` — exercises BinOp coercion end-to-end.
  - `oversized_local: int = BIG_LIMIT + LIMIT` followed by `assert str(oversized_local) == '100000000000000000254'` — exercises let-type retype end-to-end and pins SifrInt + i64-const-name (`LIMIT`) coercion.

Coverage gaps worth closing in this slice or the next (see N2 below): no focused tests for `-`/`*` (covered functionally by my probes but not pinned), no test for helper-on-right (`1 + BIG_LIMIT`), no test for unary `-BIG_LIMIT`, and no test asserting that non-arithmetic ops (`>`, `<<`, etc.) still fall through unchanged.

## Non-blocking findings

### N1 — Chained use of a `SifrInt`-typed local with small i64 literals still emits invalid Rust

The let-type retype makes `oversized_local: SifrInt`. Subsequent arithmetic that mixes that local with a small integer literal still falls through to the legacy `i64` path, because `is_sifr_int_expr` only recognizes helper `FnCall`, `from_i64` `FnCall`, SifrInt-shaped `BinOp`/`UnaryOp`, and `Paren` — *not* `Ident`. Reproduction:

```sifr
BIG_LIMIT: int = 10 ** 20

def main():
    oversized_local: int = BIG_LIMIT + 1   # OK — let retyped to SifrInt
    chained: int = oversized_local + 2     # rustc error: cannot add i64 to SifrInt
    print(str(chained))
```

`cargo run -q -p sifr -- emit` produces:

```rust
let oversized_local: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
let chained: i64 = oversized_local + (2 as i64);
```

`rustc` rejects the second line with `cannot add i64 to SifrInt`. The slice's stated goal — "expressions like `BIG_LIMIT + 1` no longer fall through to invalid legacy `i64` Rust" — is met for the *direct* helper-touching expression, but the failure shape moves one hop downstream. From the user's perspective this is the same `int + 2` shape, so the slice trades one brittle line for another.

This is *not* a regression — the chained expression was already broken before this PR (because the first line was also broken), so no working program is now broken. It's a remaining gap in the broader `Type::Int` codegen migration tracked at [issues/…/checklist:428](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). Two reasonable closure paths:

- Track a per-binding `SifrInt`-shape map in the rewrite pass so `Ident` resolution can return true from `is_sifr_int_expr` for retyped locals.
- Or wait for the broader `Type::Int` ⇒ `SifrInt` migration, which will retype all `Type::Int` locals/params/returns uniformly and obviate the per-Ident check.

Worth flagging in the open INT-1 follow-up bullet so it doesn't get lost behind the "BIG_LIMIT + 1 works now" tick.

### N2 — Comparisons (`>`, `<`, `==`, `!=`, `<=`, `>=`) with helper still emit invalid Rust

`is_sifr_int_arithmetic_op` is restricted to `+`, `-`, `*`. A natural pattern like `if BIG_LIMIT > 100:` lowers to `if __const_BIG_LIMIT() > (100 as i64) { … }`, which `rustc` rejects with `expected SifrInt, found i64`. SifrInt has `PartialOrd`/`PartialEq` against itself but not against `i64`, so the same coercion shape used for arithmetic would apply.

This was already broken before this PR — the slice does not introduce or fix this. Per the design doc ([internal_docs/integer_model.md:178](internal_docs/integer_model.md:178)), comparisons between `int` and fixed-width integers are allowed and exact, so the user-visible expectation is that `BIG_LIMIT > 100` should compile. Worth tracking under the same broader-migration follow-up; arguably the more important user-facing footgun than N1 because comparisons are extremely common in `int` constants (range checks, guards, etc.).

### N3 — `BIG_LIMIT // 2` and `BIG_LIMIT % 2` still emit invalid Rust

Floor-division and modulo aren't in `is_sifr_int_arithmetic_op`. SifrInt does not currently impl `Div` or `Rem` (verified at [crates/sifr_runtime/src/int.rs:219-326](crates/sifr_runtime/src/int.rs:219)), so even if the coercion fired, the generated code would still fail. Per the design ([integer_model.md:135-140](internal_docs/integer_model.md:135)) `int // int` and `int % int` are `Result[int, DivisionError]` — fallible — so this belongs to the INT-3 scalar-arithmetic milestone, not this slice. Just noting it so it's explicit that the generated `BIG_LIMIT // 2` shape doesn't compile today.

### N4 — Function calls receiving an oversized helper as an `int` parameter still emit invalid Rust

```sifr
def double(x: int) -> int:
    return x + x

def main():
    print(str(double(BIG_LIMIT)))   # rustc: expected i64, found SifrInt
```

`rust` produces `fn double(x: i64) -> i64 { return x + x; }` and the call site is `double(__const_BIG_LIMIT())`, which fails with `expected i64, found SifrInt`. This is the broader `Type::Int` ⇒ `SifrInt` migration boundary at function signatures. Not in slice scope (the slice only touches expression rewriting and let bindings), but worth bundling into the same broader-migration follow-up.

### N5 — Unit-test coverage is minimal for the rewrite surface

The two new tests pin the load-bearing shapes (BinOp coercion, let retype) but don't pin operator coverage or refactor-resistance. Adding focused unit tests for:
- `BIG_LIMIT - 1` and `BIG_LIMIT * 2` (sibling operators).
- `1 + BIG_LIMIT` (helper on right).
- `-BIG_LIMIT` (unary `-`).
- `BIG_LIMIT > 100` (assert *not* rewritten — pins the deliberate gap so a future widening of `is_sifr_int_arithmetic_op` doesn't silently break i64 comparisons).
- `MAX_RETRIES + 1` (assert i64 path stays untouched — guards against accidental over-coercion if someone fattens the helper-detection rule).

…would harden the slice without much cost.

### N6 — Minor code-shape nits

- [is_sifr_int_module_constant_func](crates/sifr_codegen/src/expr_render_helpers.rs:1261) does an O(N) scan of `module_constants.values()` per check, where N is the number of module constants. For deeply-nested expressions in files with many constants, this is a small per-AST-node cost. A reverse index (helper-fn-name ⇒ has-int-type) computed once per emit pass would make this O(1). Not a hot path today; nice-to-have.
- [coerce_expr_to_sifr_int](crates/sifr_codegen/src/expr_render_helpers.rs:1225)'s `other => sifr_int_from_i64_expr(other)` fallback wraps any non-`I64`-cast expression in `SifrInt::from_i64(...)`. For valid Sifr the operand will always be `i64`-typed (the BinOp guard fires only when one side is SifrInt and the other is the i64-typed `int`), so this is safe. A `debug_assert!` documenting the invariant — or a more explicit fallthrough that asserts the operand carries an `i64`-shape — would make the contract self-describing.
- The early `return` inside the `BinOp` match arm is mildly inconsistent with the surrounding match-arm-return style. Refactoring to a small `match (...) { (true, ...) => ..., _ => ... }` would land all paths inside arms; trivial and not worth blocking on.

## Validation

I re-traced rather than re-ran the listed validation. The cited results are consistent with the code:

- `cargo test -p sifr_codegen rewrites_large_int_module_const -- --nocapture` — both new tests sit on the `rewrites_large_int_module_const_*` prefix and pin the structural shapes my probes confirmed.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` — I reproduced this directly via `emit` + `run`. The two new asserts (`BIG_LIMIT + 1 == '100000000000000000001'` and `oversized_local == '100000000000000000254'`) round-trip.
- `cargo run -q -p sifr -- emit /tmp/sifr_big_const_use_XXXX.sifr` — confirmed visually for `+`/`-`/`*`, helper-on-right, unary `-`, and chained nested arithmetic. All produce well-typed Rust.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=60.52s`. Same signature as PRs #1817 and #1818, consistent with no test deltas elsewhere.

## Verdict

**Satisfied with non-blocking suggestions.** The slice closes the pass-1 N1 follow-up cleanly for its stated shape: direct arithmetic between an oversized `int` module-constant helper and a small-`int` operand now lowers to `SifrInt`-typed Rust on both sides, and `int`-annotated locals receiving such expressions are retyped to `SifrInt` so the binding compiles. The rewrite plumbing is narrowly gated (operators, type tag, type tag of the let), preserves all i64-only paths byte-for-byte, and is anchored by two structural unit tests plus two new e2e asserts. Quick validation reproduces the same `report_signature` as upstream.

The non-blocking findings (N1–N6) cluster around the broader `Type::Int` ⇒ `SifrInt` codegen migration: chained use of a retyped local (N1), comparisons (N2), floor-division/modulo (N3), and function call boundaries (N4) all remain reachable rustc-error shapes that the open INT-1 follow-up bullet at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:428](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) should keep tracking. None gates merge for this slice. N5 and N6 are test-hardening and code-shape suggestions — easy to take in this PR or the next.
