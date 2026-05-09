# Review: INT-1 SifrInt Recursive Capture Params Pass 1

## Verdict

Satisfied with non-blocking suggestions.

The slice cleanly closes the stated narrow scope: recursive nested helpers that capture module exact-int sources (especially `BIG_LIMIT`) now lower correctly. The parameter type, the call-site argument, and the body's references all align as SifrInt-typed Rust. State isolation across sibling functions is preserved. Non-recursive nested helpers and pure-i64 paths are unaffected.

I flag one notable observation (N1) that the slice's predicate is broader than what actually delivers — only the module-source branch fires at the load-bearing line 281 insert site, while the other branches (registered/forced locals) are effectively dead. Pre-PR was also broken for those cases, so it's not a regression, but the predicate's appearance might mislead future readers about supported scope. None of the findings gate merge.

## Findings

None blocking.

### 1. Module-source recursive capture is correctly fixed

The pre-PR failure mode was: `BIG_LIMIT` captured into a recursive nested helper got typed as `i64` parameter, but the call-site arg was `__const_BIG_LIMIT()` (SifrInt-returning helper) → mismatch, fails rustc. I verified pre-PR (commit `91ea75a7`) with [/tmp/recursive_basic.sifr]:

```
fn helper(remaining: i64, BIG_LIMIT: i64) -> SifrInt { … }
return helper(remaining - 1, __const_BIG_LIMIT()) + …;   // E0308
```

Post-PR-#1835 emits:

```rust
fn helper(remaining: i64, BIG_LIMIT: SifrInt) -> SifrInt {
    if remaining <= (0 as i64) {
        return __const_BIG_LIMIT() + SifrInt::from_i64(1);
    }
    return helper(remaining - (1 as i64), __const_BIG_LIMIT()) + SifrInt::from_i64(1);
}
return helper(2 as i64, __const_BIG_LIMIT());
```

Compiles and runs cleanly, e2e fixture round-trips with `'100000000000000000003'`. ✓

The three coordinated changes:

1. [lower_recursive_capture_param_type](crates/sifr_codegen/src/function_emitter.rs:184-189) checks `recursive_capture_lowers_to_sifr_int(capture)` and emits `RustType::Named("SifrInt")` when true. Called *after* state restoration in `try_lower_structured_nested_function_stmt`, so the predicate sees the outer's restored state — module sources, registered locals, and forced locals all visible. ✓

2. [lower_recursive_capture_arg_for_ir](crates/sifr_codegen/src/stmt_support_emitter.rs:5289-5319) also checks the predicate and routes the capture-ident through `rewrite_stdlib_constant_idents_in_expr` + `coerce_expr_to_sifr_int_value`. Called from three sites:
   - `intrinsic_method_emitters.rs:3552` — generic plain-call emission with captures.
   - `stmt_support_emitter.rs:2121, 4781` — body-stmt emission.
   - State at call time depends on emission context. For the *outer's* call to the helper (`return helper(2)`), state is the outer's body-emit state where forced/registered locals are visible. For the *inner recursive call* (`helper(remaining - 1)`), state is the inner cleared+inserted state.

3. [Body-emit register insert](crates/sifr_codegen/src/function_emitter.rs:281-287) inserts the captured name into `sifr_int_local_bindings` if the predicate is true at that point. This happens *after* the inner state clear at line 247, so the inserted bindings are visible during the helper's body emit (so references inside the body can be coerced via the BinOp arm).

### 2. Probe matrix

| Probe                                                                  | Result |
|------------------------------------------------------------------------|--------|
| Module-source recursive capture (`BIG_LIMIT`) — fixture                | ✓ fixed (pre-PR was broken) |
| Sibling recursive helpers with different promotion needs               | ✓ state isolated |
| Non-recursive nested helper (closure path, unchanged)                  | ✓ unaffected |
| Pure i64 recursive function (no SifrInt source)                        | ✓ unaffected |
| Outer with unrelated forced local + recursive helper using `BIG_LIMIT` | ✓ both work, outer's `&big` operand-position fires |
| Earlier milestone shapes (e2e fixture full asserts)                    | ✓ all still round-trip |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1833), confirming no test deltas elsewhere.

### 3. Visibility widening is appropriate

Three previously-private items become `pub(super)`:
- `coerce_expr_to_sifr_int_value`
- `is_registered_sifr_int_local`
- `is_forced_sifr_int_local`

These are needed by `function_emitter.rs` and `stmt_support_emitter.rs` which now use them in the new recursive-capture predicates. Reasonable scope-widening within the crate, no new public surface.

### 4. State save/restore preserved

The existing save/restore plumbing in `try_lower_structured_nested_function_stmt` (sifr_int_local_bindings, sifr_int_forced_local_bindings, current_sifr_int_return) is unchanged. The new line 281 insert happens between the clear (line 247) and the body emit, and is naturally restored by the existing restore (line 270-274) after body emit. ✓

## Notes

(Non-blocking observations only.)

### N1 — Predicate at line 281 only catches module-source captures, not registered/forced locals

[recursive_capture_lowers_to_sifr_int](crates/sifr_codegen/src/function_emitter.rs:177-183) checks three sources:

```rust
matches!(crate::resolve_alias_type_for_plain_call(&capture.ty), Type::Int)
    && (self.module_sifr_int_bindings().contains(&capture.name)
        || self.is_registered_sifr_int_local(&capture.name)
        || self.is_forced_sifr_int_local(&capture.name))
```

When called at [function_emitter.rs:281](crates/sifr_codegen/src/function_emitter.rs:281) (inside `try_lower_structured_nested_function_stmt`, body-emit-time insert), the state is:

- `module_sifr_int_bindings()` — computed from `self.module_constants` (never cleared). Always available.
- `is_registered_sifr_int_local(name)` — checks `sifr_int_local_bindings` (just cleared at line 247).
- `is_forced_sifr_int_local(name)` — checks `sifr_int_forced_local_bindings` (just cleared at line 248).

So at line 281, only the module-source branch can fire. The other branches are effectively dead at that call site. The predicate at the *other* call sites — `lower_recursive_capture_param_type` at line 321 (after state restore) and `lower_recursive_capture_arg_for_ir` from outer's body-emit context — *do* see the registered/forced state because state isn't cleared there.

Concrete impact: a recursive helper that captures a *forced or registered* outer local (rather than a module source) gets:
- ✓ Correct SifrInt parameter type (via line 321, predicate works).
- ✓ Correct `big.clone()` outer's-call arg (via `lower_recursive_capture_arg_for_ir` from outer's body-emit, predicate works).
- ✗ Body's expressions on the captured name (e.g., `big + 0` inside helper.body) **not** coerced because line 281's insert didn't fire — the predicate saw cleared state and skipped. Body emits `big + (0 as i64)` against `big: SifrInt` parameter. rustc rejects with `cannot add i64 to SifrInt`.

I verified with [/tmp/recursive_capture_local.sifr]:

```sifr
def outer() -> int:
    big: int = BIG_LIMIT + 1
    def helper(remaining: int) -> int:
        if remaining <= 0:
            return big + 0
        return helper(remaining - 1) + 1
    return helper(2)
```

Post-PR emits `fn helper(remaining: i64, big: SifrInt) -> i64 { ... return big + (0 as i64); }` — fails rustc.

Pre-PR-#1835 emits `fn helper(remaining: i64, big: i64) -> i64 { ... }` and `return helper(2, big);` — fails rustc at the call site (i64 param ← SifrInt arg). **Pre-PR was also broken**, so this is not a regression. The slice's primary target (module sources, especially `BIG_LIMIT`) is met cleanly.

The fix would be to check the predicate against the *outer's* saved state at line 281 — for example, by passing `saved_sifr_int_local_bindings` and `saved_sifr_int_forced_local_bindings` to the predicate, or by reordering so the predicate runs before the clears at lines 247–248. Worth tracking as a follow-up alongside the broader function-boundary migration.

The slice description's emphasis on "especially module exact-int constants such as `BIG_LIMIT`" correctly signals that the module-source case is the primary target. The predicate's broader appearance (checking three sources) might mislead future readers into thinking the local-source case is fully handled. A short comment at line 281 noting "only module-source captures fire here because local-bindings state has just been cleared; the other predicate branches still fire correctly at param-type lowering and outer's call-arg coercion" would make the timing explicit.

### N2 — Module-source captured parameter is dead code in the helper body

The emit shows `fn helper(remaining: i64, BIG_LIMIT: SifrInt) -> SifrInt { ... }` with `BIG_LIMIT` as a parameter. But the body's references to `BIG_LIMIT` get rewritten by `rewrite_special_ident` to `__const_BIG_LIMIT()` (the module helper) before the parameter is consulted — `rewrite_special_ident` always rewrites Idents whose names match `module_constants`, regardless of whether there's a same-named parameter in scope. So the parameter is unused in the rewritten body.

Rust may emit `unused_variables` warnings on this, though the code still compiles and runs correctly. Two cleanup options for a future slice:
- Suppress the recursive-capture parameter when the captured name is rewriter-targeted (e.g., a module helper). The captured value would be reconstructed from the module rather than passed through.
- Use the parameter consistently — rewrite `BIG_LIMIT` references inside the recursive helper body to the parameter `BIG_LIMIT` (rather than `__const_BIG_LIMIT()`), so the parameter is actually consumed.

Both are out of scope for this slice and not user-visible (warnings, not errors).

### N3 — No focused unit tests for the new slice

The e2e fixture's `returned_big_from_recursive_nested_helper` covers the load-bearing case at runtime. No focused unit tests for:

- The `recursive_capture_lowers_to_sifr_int` predicate against staged state.
- The line 281 insert's interaction with cleared state (would document N1).
- `lower_recursive_capture_arg_for_ir` against staged module / registered / forced sources.

These would harden against future regressions without requiring a full pipeline run. Optional polish.

### N4 — Carry-forward open items unchanged

Lexical shadowing, legacy-emission paths, fallible `//` and `%`, function arguments / arg expressions that are already SifrInt, captured-local-only nested helper return analysis, and (now) the local-source recursive capture body-coercion gap (N1 above) — all stay tracked under the open INT-1 follow-up at [issues/…/checklist:446](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). The next tracker PR should:

- Mark this slice's narrow target (module-source recursive captures) complete.
- Refine the residual open items to reflect that "recursive nested helper capture parameters" is now closed for module sources but the inner-body coercion gap remains for local-source captures.
