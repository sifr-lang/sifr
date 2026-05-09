# Review: INT-1 SifrInt Local-Source Recursive Capture Body Pass 1

## Verdict

Satisfied with non-blocking suggestions.

The slice cleanly closes the local-source recursive capture body-coercion gap from PR #1835's pass-1 N1. When a recursive nested helper captures an outer local already forced to `SifrInt` (e.g., `big: int = BIG_LIMIT + 1`), the slice now ensures: (a) the nested return pre-scan sees the captured name as exact-int via the new `hir_function_returns_sifr_int_with_extra_forced` + `collect_sifr_int_captured_forced_locals`, (b) the helper body's BinOp arm coerces uses of the captured name (via the new line 302 sifr_int_local_bindings insert that uses precomputed `sifr_int_recursive_captures` from the outer's intact state), (c) the helper return type is promoted to `SifrInt` via the new `nested_returns_sifr_int` flag, and (d) the enclosing function's `return helper(...)` makes the outer signature also lower to `SifrInt`.

I flag one minor non-blocking observation: the pre-scan promotion logic now fires for non-recursive nested helpers too (because `collect_nested_sifr_int_function_returns` runs unconditionally), but the matching body coercion only applies to the recursive (LocalFn) path. This means non-recursive captured-local cases now get a `-> SifrInt` outer signature with a still-broken closure body. Pre-PR was also broken at the body, so it's not a regression — but the divergence between pre-scan promotion and lowering coverage is worth noting.

## Findings

None blocking.

### 1. The recursive local-source case is fully fixed

I reproduced PR #1835's pass-1 N1 reproducer and verified post-PR-#1837 emits clean Rust:

```sifr
def returned_big_from_local_recursive_nested_helper() -> int:
    big: int = BIG_LIMIT + 1
    def helper(remaining: int) -> int:
        if remaining <= 0:
            return big + 0
        return helper(remaining - 1) + 1
    return helper(2)
```

Post-PR emits:

```rust
fn returned_big_from_local_recursive_nested_helper() -> SifrInt {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    fn helper(remaining: i64, big: SifrInt) -> SifrInt {
        if remaining <= (0 as i64) {
            return &big + SifrInt::from_i64(0);
        }
        return helper(remaining - (1 as i64), big.clone()) + SifrInt::from_i64(1);
    }
    return helper(2 as i64, big.clone());
}
```

All four invariants the slice claims hold:

- **Hidden parameter** typed as `SifrInt` (was `i64` pre-PR-#1837, would have been a sub-flaw post-PR-#1835 had local-source predictably routed there). ✓
- **Hidden capture argument** uses the exact-int value path: `big.clone()` at both the outer call and the recursive call. The clone preserves source value semantics. ✓
- **Helper body** coerces uses of the captured local via the BinOp arm: `&big + SifrInt::from_i64(0)` borrows the parameter (registered in inner sifr_int_local_bindings via the line 302 insert). ✓
- **Helper return type** is `SifrInt` because `nested_returns_sifr_int` was computed against the outer's intact state via `hir_function_returns_sifr_int_with_extra_forced(helper, …, captured_forced)`. ✓
- **Enclosing function return type** is `-> SifrInt` because outer's `return helper(2)` is recognized as a SifrInt-returning call (helper now in `sifr_int_function_returns` via the module-level pre-scan + the function_sifr_int_returns_for_body enrichment).

E2E fixture round-trips with `'100000000000000000003'`. ✓

### 2. Mechanism is correctly state-aware

The key insight of the slice is that the predicates it depends on must be evaluated against the *outer's intact state*, not the inner's cleared state. The diff handles this correctly:

- [sifr_int_recursive_captures](crates/sifr_codegen/src/function_emitter.rs:231-237) is computed at function entry, **before** any state clear — so `recursive_capture_lowers_to_sifr_int(capture)` sees the outer's `sifr_int_local_bindings` (registered locals like `big`) and `sifr_int_forced_local_bindings` (forced locals).
- [nested_returns_sifr_int](crates/sifr_codegen/src/function_emitter.rs:238-246) is also computed at function entry, using `hir_function_returns_sifr_int_with_extra_forced` which threads the captured-forced names into the helper's analysis. `&self.sifr_int_function_returns.borrow()` reads the current set (which includes the outer's enriched function returns from `function_sifr_int_returns_for_body`).
- [Line 302](crates/sifr_codegen/src/function_emitter.rs:302-308) uses the precomputed `sifr_int_recursive_captures` (HashSet<String>) for the membership check, not a re-evaluation of `recursive_capture_lowers_to_sifr_int(capture)` against the cleared state. This fixes the timing bug from PR #1835.

This is the right pattern: precompute predicates against the intact state, store the result, use the stored result during the inner emission.

### 3. New helpers are sound

- [hir_function_returns_sifr_int_with_extra_forced](crates/sifr_codegen/src/function_emitter.rs:838-895) mirrors the original `hir_function_returns_sifr_int` but injects an `extra_forced_locals` set into each iteration's `forced` set. The fixed-point loop is the same shape; `forced.extend(extra_forced_locals.iter().cloned())` happens after `collect_sifr_int_forced_locals` populates the body-derived forced set. Correct.
- [collect_sifr_int_captured_forced_locals](crates/sifr_codegen/src/function_emitter.rs:965-986) computes the set of names that are *referenced* in the helper's body, *not* helper's params, *not* helper's local definitions, *and* in the outer's forced locals. Faithful translation of "captured forced outer locals". The early return when `outer_forced_locals.is_empty()` is a reasonable optimization.
- [collect_nested_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:915-961) now performs both checks (regular and with-extra-forced) for each direct nested function. The `||` between the two branches is correct — either qualifies the helper for promotion.

### 4. `hir_function_returns_sifr_int` itself now has a fixed-point loop

The original `hir_function_returns_sifr_int` did a single-pass enrichment of `function_sifr_int_returns` with nested helpers. Post-PR it loops until convergence:

```rust
loop {
    forced = collect_sifr_int_forced_locals(...);
    let before = function_sifr_int_returns.len();
    function_sifr_int_returns.extend(collect_nested_sifr_int_function_returns(
        body, ..., &forced,
    ));
    if function_sifr_int_returns.len() == before { break; }
}
```

This is necessary because `collect_sifr_int_forced_locals` consumes `function_sifr_int_returns` (e.g., a Let value calling a promoted helper forces the local), and `collect_nested_sifr_int_function_returns` consumes both `function_sifr_int_returns` and `forced` (via captured-forced detection). Mutual feedback can require multiple iterations to converge. The loop is bounded by the number of nested helpers and local int bindings (both finite per body), so termination is guaranteed.

### 5. Probe matrix verified

| Probe                                                             | Result |
|-------------------------------------------------------------------|--------|
| Recursive helper capturing forced outer SifrInt local — fixture   | ✓ fixed (was pre-PR-#1837 broken) |
| Sibling functions: one with SifrInt local, one with i64 local     | ✓ state-isolated (`-> SifrInt` vs `-> i64`) |
| Mixed captures: helper captures both SifrInt and i64 outer locals | ✓ selective per-capture promotion |
| Recursive with body arithmetic on captured local (`+= big`)       | ✓ `&big + SifrInt::from_i64(...)` correctly emitted |
| Chained forced locals (`big1 → big2`), helper captures `big2`     | ✓ fixed-point convergence works |
| Pure i64 recursive helper (no SifrInt source)                     | ✓ unaffected, signature stays `-> i64` |
| Earlier milestone shapes (e2e fixture full asserts)               | ✓ all still pass |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1835), confirming no test deltas elsewhere.

### 6. State isolation across sibling functions

The save/restore plumbing in `try_lower_structured_nested_function_stmt` (sifr_int_local_bindings, sifr_int_forced_local_bindings, sifr_int_function_returns, current_sifr_int_return) is unchanged. The slice's additions inside the function — line 302 inserts and the line 285 set — happen between the clears and the body emit, and are restored on exit.

For the module-level `register_sifr_int_function_returns` pre-scan, the new captured-forced detection happens transiently inside `hir_function_returns_sifr_int` and `hir_function_returns_sifr_int_with_extra_forced` — no persistent state mutation. The `function_sifr_int_returns_for_body` helper computes a fresh union per call without leaking.

I verified isolation: `with_local` and `with_small_local` (sibling functions with different captured locals) emit distinct signatures and don't cross-pollinate. ✓

## Notes

(Non-blocking observations only.)

### N-pass1-1 — Non-recursive captured-local case stays broken (unchanged from pre-PR)

When a *non-recursive* nested helper captures a forced outer local, the slice's helper-body coercion logic doesn't apply because:
- `collect_recursive_nested_fn_captures` returns empty for non-recursive functions (it short-circuits when `!body_calls_function(&func.body, &func.name)`).
- The new line 302 loop iterates `recursive_captures` which is empty.
- `sifr_int_local_bindings` doesn't get the captured name inserted for the inner state.
- The closure body's `Ident("big")` references aren't recognized as registered SifrInt locals during rewrite, so the BinOp arm doesn't coerce.

Reproduction:

```sifr
def outer() -> int:
    big: int = BIG_LIMIT + 1
    def helper() -> int:
        return big + 1
    return helper()
```

Post-PR emits:

```rust
fn outer() -> SifrInt {
    let big: SifrInt = …;
    let helper = || {
        return big + (1 as i64);   // <-- not coerced; SifrInt + i64 fails Add impl
    };
    return helper();
}
```

`rustc` rejects with `cannot add i64 to SifrInt` at the closure body line.

I verified pre-PR-#1837 (commit `16b87104`) emits the same closure body line (`return big + (1 as i64);`) — also broken. **Not a regression.** The slice's stated narrow scope ("recursive nested helper") explicitly excludes non-recursive cases.

However, there's a subtle divergence post-PR: the slice's pre-scan via `collect_nested_sifr_int_function_returns` calling `hir_function_returns_sifr_int_with_extra_forced` runs unconditionally for any nested function — recursive or not. So the pre-scan now correctly *predicts* that the helper's body would naturally produce SifrInt (if `big` were treated as forced), and outer gets promoted to `-> SifrInt`. But the actual non-recursive lowering doesn't carry the SifrInt-awareness through. Result: `outer -> SifrInt` signature with a broken closure body. Pre-PR was `outer -> i64` with the same broken closure body.

Two ways to address this in a future slice:
- Restrict the captured-forced detection in `collect_nested_sifr_int_function_returns` to recursive helpers only (skip the with-extra-forced check when `!body_calls_function(&func.body, &func.name)`). This would avoid the misleading pre-scan promotion for non-recursive cases.
- Or extend the non-recursive (closure) path to also propagate captured-forced into inner `sifr_int_local_bindings`, mirroring the recursive path's line 302 insert. This would be the durable fix that makes the body actually deliver SifrInt.

The current state is acceptable because it doesn't introduce a regression (both versions fail to compile for this shape) and because the slice's stated scope is explicit about "recursive". Worth tracking in the next tracker as a residual gap.

### N-pass1-2 — Module-source captured parameter is dead code in helper bodies (carry-forward from PR #1835 N2)

For module-source captures (e.g., `BIG_LIMIT`), the helper signature still has `BIG_LIMIT: SifrInt` parameter, but the body's references to `BIG_LIMIT` get rewritten to `__const_BIG_LIMIT()` before the parameter is consulted. So the parameter is unused. Rust may emit `unused_variables` warnings. Pre-existing, not introduced by this slice. Functionally fine.

### N-pass1-3 — No focused unit tests added

The e2e fixture covers the load-bearing case at runtime, and the existing 14 expr_render_helpers tests still pass. No focused unit tests for:

- `hir_function_returns_sifr_int_with_extra_forced` against staged HirFunction inputs.
- `collect_sifr_int_captured_forced_locals` against various capture shapes.
- The new fixed-point loop's convergence on chained forced detection.
- "Should not promote" sibling: a recursive helper whose body doesn't need SifrInt should stay `-> i64`.

These would harden against future regressions without requiring a full pipeline run. Optional polish.

### N-pass1-4 — Carry-forward open items unchanged

Lexical shadowing, legacy-emission paths, fallible `//` and `%`, function arguments / arg expressions that are already SifrInt, captured-local-only **non-recursive** nested helpers (N-pass1-1 above), and any further recursive variants — all stay tracked under the open INT-1 follow-up. The next tracker PR should:

- Mark this slice complete (closes PR #1835's pass-1 N1 for the recursive case).
- Refine the residual open items to reflect that "local-source recursive capture body coercion" is now closed for recursive, but non-recursive captured-local-only nested helpers are still in the broader function-boundary migration.
