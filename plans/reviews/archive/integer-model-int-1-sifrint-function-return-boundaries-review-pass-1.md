# Review: INT-1 SifrInt Function Return Boundaries Pass 1

## Verdict

Satisfied with non-blocking suggestions.

The slice's stated narrow scope — promote module-level `-> int` functions whose returns transitively depend on SifrInt sources, coerce their return statements, and recognize zero-arg calls to them as SifrInt at call sites — is met cleanly. The pre-scan, scope save/restore for the primary `emit_function` path, and call-site detection are all sound. End-to-end and probe matrix verification confirms the load-bearing cases work and pre-existing i64 paths stay untouched.

I do flag two non-blocking quality concerns (N1, N2 below) that surface in code shapes that were already broken pre-PR. Neither is a strict regression, and both are out of the slice's stated scope, but they're worth tracking explicitly in the next tracker so they don't get lost behind the function-return tick.

## Findings

None blocking.

The implementation matches the design and phase-issue framing:

### Pre-scan correctness

[register_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:131-156) walks `module.functions` in a fixed-point loop. Within each iteration, every function with `Type::Int` return is checked via `hir_function_returns_sifr_int`, which in turn computes a per-function forced-locals set and looks for any Return whose value `hir_expr_needs_sifr_int_storage` (now extended with the `function_sifr_int_returns` parameter). The fixed-point loop correctly handles transitive function dependencies — verified by probing:

```sifr
def inner() -> int:
    return BIG_LIMIT + 1
def outer() -> int:
    return inner()
```

emits both `inner` and `outer` with `-> SifrInt`, the call site `let result: SifrInt = outer()` correctly retypes, runtime round-trips. ✓

Mutual recursion (`f -> g -> f` with one side bringing a SifrInt source) also converges in two iterations because the fixed-point reads `function_returns` as a snapshot per iteration, then extends — verified by tracing.

The pre-scan order is correct: [lib.rs:1379-1383](crates/sifr_codegen/src/lib.rs:1379) calls `prescan_module_metadata` → `emit_module_constants` (populates `module_constants`) → `register_sifr_int_function_returns` (reads `module_constants`) → `emit_module_body`. The data dependency on `module_constants` is honored. ✓

The `register_local_body_binding_types` path now also extends `sifr_int_forced_local_bindings` via `collect_sifr_int_forced_locals`, threading the same `function_sifr_int_returns` set through the predicate. This means a local `let x: int = make_big()` gets pre-promoted to SifrInt because `Call{make_big, []}` with empty args and a promoted name now satisfies `hir_expr_needs_sifr_int_storage`. The walker descends through loop/if/match bodies via `TraversalConfig::LOCAL_SCOPE_ONLY` (excluding nested function bodies), which is the right scope for the per-function forced set.

### Scope save/restore for primary emit_function

`current_sifr_int_return` is saved at [function_emitter.rs:606](crates/sifr_codegen/src/function_emitter.rs:606), set via `self.current_sifr_int_return = self.function_returns_sifr_int(&func.name)` at [function_emitter.rs:617](crates/sifr_codegen/src/function_emitter.rs:617), and restored at [function_emitter.rs:720](crates/sifr_codegen/src/function_emitter.rs:720). This is the primary entry point called from `emit_module_body` for each module-level function. The save/restore correctly resets the flag for sibling functions in a module — verified by emitting the e2e fixture: `circle_area`, `returned_big_limit`, and `main` each have the flag set independently based on their own promotion status.

### Call-site recognition

[is_sifr_int_returning_function_call](crates/sifr_codegen/src/expr_render_helpers.rs:1406-1408) added to `is_sifr_int_expr`'s zero-arg `FnCall` arm. Verified end-to-end:

| Probe                             | Emitted Rust                                                | Result |
|-----------------------------------|-------------------------------------------------------------|--------|
| `let v: int = make_big()`         | `let v: SifrInt = make_big();`                              | ✓ |
| `let v: int = make_big() + 1`     | `let v: SifrInt = make_big() + SifrInt::from_i64(1);`       | ✓ |
| `let v: int = -make_big()`        | `let v: SifrInt = -(make_big());`                           | ✓ |
| `let v: int = make_big() * 2`     | `let v: SifrInt = make_big() * SifrInt::from_i64(2);`       | ✓ |
| `let d: bool = make_big() > 100`  | `let d: bool = (&make_big() > &SifrInt::from_i64(100));`    | ✓ |
| `let v: int = inner()` (transitive) | `let v: SifrInt = inner();` after `inner` itself promoted | ✓ |

### Return statement coercion

The Return arm at [expr_render_helpers.rs:565-575](crates/sifr_codegen/src/expr_render_helpers.rs:565) reads `current_sifr_int_return` and routes the expression through `coerce_expr_to_sifr_int_value` when promoted. Verified by emit:

```rust
fn returned_big_limit() -> SifrInt {
    return __const_BIG_LIMIT() + SifrInt::from_i64(1);
}
```

The value-position coerce is correct here: a registered local source would be cloned (not borrowed), helper FnCalls and BinOp results pass through, and small literals get `from_i64`-wrapped. Same semantics PRs #1825/#1827 established.

### Both `RustType::I64` and `Named("i64")` shapes

The new `is_legacy_i64_type` predicate at [expr_render_helpers.rs:1474-1477](crates/sifr_codegen/src/expr_render_helpers.rs:1474) handles both encodings. Reasonable: production codegen produces `RustType::Named("i64")` in some paths (e.g., `preamble.rs:502` for `Vec<i64>`-typed initializers and various `intrinsic_method_emitters.rs` paths). The `rewrites_sifr_int_returning_function_call_named_i64_let_type` test pins this defensive coverage. ✓

### Tests

- [rewrites_sifr_int_returning_function_call_let_type](crates/sifr_codegen/src/expr_render_helpers.rs:1858) — pins `Let { ty: Some(I64), value: FnCall{Ident("make_big"), []} }` retypes to `Some(SifrInt)`.
- [rewrites_sifr_int_returning_function_call_named_i64_let_type](crates/sifr_codegen/src/expr_render_helpers.rs:1885) — same but for the `Named("i64")` encoding.
- E2E fixture adds `returned_big_limit() -> int` plus four call-site uses (`returned_big`, `returned_plus_one` and their asserts), exercising both let-retype and operand-position recognition.

Coverage gaps that didn't surface as bugs but would harden the slice (see N3 below).

## Notes

(Non-blocking observations only.)

### N1 — Closure body Returns inherit `current_sifr_int_return` from a promoted outer function

The flag is saved/restored at function-emit boundaries, but **not at closure-body boundaries**. The Closure and ClosureBlock arms in [rewrite_stdlib_constant_idents_in_expr](crates/sifr_codegen/src/expr_render_helpers.rs:380-400) descend into the body without touching `current_sifr_int_return`. Inside a promoted outer function, any `RustExpr::Closure { body: …Return(...) }` gets its return value coerced as if it returns SifrInt.

Reproduction:

```sifr
BIG_LIMIT: int = 10 ** 20

def outer() -> int:
    def inner() -> int:
        return 42
    x: int = inner()
    return BIG_LIMIT + x
```

Post-PR emits:

```rust
fn outer() -> SifrInt {
    let inner = || {
        return SifrInt::from_i64(42);   // <-- leaked coercion
    };
    let x: i64 = inner();              // E0308: expected i64, found SifrInt
    return __const_BIG_LIMIT() + SifrInt::from_i64(x);
}
```

`inner`'s body's `return 42` gets coerced to `SifrInt::from_i64(42)` because the rewrite arm reads the outer's `current_sifr_int_return = true`. This makes `inner`'s closure type `Fn() -> SifrInt`, mismatching the `let x: i64 = inner()` site.

**Not a strict regression.** Pre-PR (commit `74bb52c5`) the same fixture failed at the outer's return statement (`return SifrInt + i64` against `-> i64`); post-PR it fails at the `let x: i64 = inner()`. Both versions fail to compile. I checked: shapes that don't trigger outer's promotion (no SifrInt source in the body) compile and run identically pre- and post-PR.

But the leak is a real codegen bug, and the fix is small: save+restore (or save+clear+restore) `current_sifr_int_return` at the Closure and ClosureBlock arms, since closures have their own return semantics independent of the surrounding function. Worth tracking in the next open follow-up bullet alongside the broader function-boundary work, since both will likely be unblocked when full parameter+return migration lands.

### N2 — Functions with parameters get promoted but call sites with arguments don't compile

[register_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:131) promotes any function whose return needs SifrInt storage, **regardless of parameter count**. But call-site recognition in [hir_expr_needs_sifr_int_storage](crates/sifr_codegen/src/function_emitter.rs:836) and [is_sifr_int_returning_function_call](crates/sifr_codegen/src/expr_render_helpers.rs:1406) requires `args.is_empty()`. So:

```sifr
def make_big_with_arg(x: int) -> int:
    return x + BIG_LIMIT

def main():
    result: int = make_big_with_arg(5)
    print(str(result))
```

emits

```rust
fn make_big_with_arg(x: i64) -> SifrInt {
    return SifrInt::from_i64(x) + __const_BIG_LIMIT();
}

fn main() {
    let result: i64 = make_big_with_arg(5 as i64);   // E0308
    …
}
```

The function definition compiles cleanly (the body's return coerces correctly). The call site fails because the let isn't pre-promoted to SifrInt — `args.is_empty()` is false.

**Not a strict regression.** Pre-PR, the same fixture failed at the function body's return (`expected i64, found SifrInt`) — the function definition itself didn't compile. Post-PR, the function compiles but call sites with args fail. Strict improvement at the function-definition level.

The slice description acknowledges the zero-arg call-site restriction. The asymmetry (broad promotion, narrow call-site recognition) is acceptable per scope, but it does mean a Sifr file that defines a parametrized SifrInt-returning helper can't be called via `int = f(arg)`. Two fix shapes for the next slice:

1. Restrict the promotion criterion to `func.params.is_empty()` or to functions whose call sites are *all* zero-arg, mirroring the call-site predicate. This would leave parametrized functions broken at the body return (same as pre-PR), but consistent.
2. Extend call-site recognition to non-zero-arg calls when the function name is in `sifr_int_function_returns`. This is what the broader function-boundary follow-up should do.

Option 2 is the durable fix. Worth flagging in the open INT-1 follow-up so the asymmetry is explicit.

### N3 — `current_sifr_int_return` save/restore not added to `function_like_lowering.rs`, `class_emitter.rs`, `class_method_emitter.rs`

The slice adds save/restore at the two `function_emitter.rs` sites (the regular `emit_function` and `try_lower_structured_nested_function_stmt`) but not at the other three function-body emission paths. Currently benign because:

- The other three paths emit class methods, Display impls, and operator-protocol bodies — none of which are in `module.functions` and so `current_sifr_int_return` is `false` when entering them.
- These paths aren't typically invoked recursively from inside `emit_function`'s body emission, so the outer `true` value doesn't leak in.

Verified by reading the call graph: [function_like_lowering.rs:21](crates/sifr_codegen/src/function_like_lowering.rs:21) is invoked from `type_emitters.rs:333` and `operator_protocol_emitters.rs:291`, both class/enum emission paths that run at module-emit-body time, with `current_sifr_int_return = false`.

But this is a fragile invariant. A future feature (nested classes, inline operator overloads, etc.) could enter one of these paths while the flag is `true`, silently coercing class method or Display body returns. A defensive save+clear+restore at all three sites — symmetric to how PR #1825 added `sifr_int_forced_local_bindings` save/restore uniformly — would future-proof the code without behavioral change today. Optional polish.

### N4 — Test coverage for pre-scan and Return coercion is e2e-only

The two new unit tests cover let-retype shapes (`let result: I64/Named("i64") = make_big()` → SifrInt). What's not unit-tested:

- **Pre-scan transitive dependency**: a unit test that constructs a small `HirModule` with two `Type::Int`-returning functions where one calls the other, and asserts both end up in `sifr_int_function_returns` after `register_sifr_int_function_returns`.
- **Return coercion**: a unit test that stages `current_sifr_int_return = true` and rewrites a `RustStmt::Return(Some(Cast(Literal(0), I64)))` into `RustStmt::Return(Some(FnCall{from_i64, [Literal(0)]}))`.
- **Operand-position zero-arg call recognition** in BinOp/UnaryOp/comparison shapes — currently only e2e-covered.
- **"Should not promote"** sibling test for a Type::Int-returning function whose body doesn't need SifrInt — pins the deliberate gap.

E2E coverage is sufficient for merge confidence, but the unit-level matrix mirrors prior slices' patterns (see PR #1825 pass-1 review's similar gap-flag).

### N5 — Carry-forward open items

Lexical shadowing, legacy-emission paths, fallible `//` and `%`, and the *broader* function argument/return boundary work all remain open from prior reviews. Not addressed by this slice. Specifically: this slice's scope explicitly preserves the broader function-boundary migration as still open — N2 above is one concrete shape that the broader work should close, and the open INT-1 follow-up bullet should be updated by the next tracker PR to reflect both that "module-level zero-arg `-> int` returns" is now closed and that "function arguments and call sites with arguments" remains open.
