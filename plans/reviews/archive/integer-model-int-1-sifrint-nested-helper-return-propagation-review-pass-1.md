# Review: INT-1 SifrInt Nested Helper Return Propagation Pass 1

## Verdict

Satisfied with non-blocking suggestions.

The slice closes the pass-1-of-#1831 N-pass2-2 follow-up cleanly for the stated narrow scope: a module function whose body contains a direct nested helper whose `-> int` return naturally produces `SifrInt` (via module SifrInt sources or recursive nested-helper chains) now promotes both the helper's recognition and the outer's signature. State isolation across sibling functions is correct, and the fixed-point loop handles transitive nested helper dependencies. Probe matrix and runtime verification confirm the load-bearing cases work and prior cases stay correct.

I flag three non-blocking observations: (1) the slice's coverage is *broader* than the stated "direct" scope — recursive nesting actually works through `hir_function_returns_sifr_int` calling `collect_nested_sifr_int_function_returns` recursively; (2) captured-local-only SifrInt nested helpers still don't promote (pre-PR same, broader closure-capture concern); (3) recursive nested helpers that capture module SifrInt sources hit the broader function-argument migration sharp edge. None gates merge.

## Findings

None blocking.

### 1. Pre-scan and per-function active set

[function_sifr_int_returns_for_body](crates/sifr_codegen/src/function_emitter.rs:165-174) computes the union of module-level returns and direct nested helper returns within the given body. [collect_nested_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:794-832) filters for `HirStmt::NestedFunction`, runs a fixed-point loop calling `hir_function_returns_sifr_int` per nested fn, and returns the *difference* with `outer_function_returns` to avoid double-counting when extended.

[hir_function_returns_sifr_int](crates/sifr_codegen/src/function_emitter.rs:744-790) is enriched: before computing forced locals and walking returns, it itself extends `function_sifr_int_returns` with `collect_nested_sifr_int_function_returns(func.body, ...)`. This is what makes the recursion work — when checking whether outer returns SifrInt, we first compute "what nested helpers does outer have?", then ask "given those helpers, do outer's returns produce SifrInt?". The recursion naturally cascades through arbitrary nesting depth (see N1 below).

The module-level pre-scan at [register_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:131-156) (unchanged in this slice) calls `hir_function_returns_sifr_int` for each module function, so nested helper info is correctly transitively reflected in module-level promotion decisions.

### 2. State isolation across sibling functions

The new save/restore in [emit_function](crates/sifr_codegen/src/function_emitter.rs:619, 737):

```rust
let saved_sifr_int_function_returns = self.sifr_int_function_returns.borrow().clone();
...
let active_function_returns = self.function_sifr_int_returns_for_body(&func.body);
*self.sifr_int_function_returns.borrow_mut() = active_function_returns;
...
*self.sifr_int_function_returns.borrow_mut() = saved_sifr_int_function_returns;
```

…replaces the field with the per-function enriched set during body emit, then restores on exit. I verified isolation with sibling functions sharing a nested helper name:

```sifr
def with_helper() -> int:
    def helper() -> int:
        return BIG_LIMIT + 1
    return helper()

def without_helper() -> int:
    def helper() -> int:
        return 42
    return helper()
```

Emits cleanly: `with_helper -> SifrInt`, `without_helper -> i64`. The two `helper` names don't conflict because each function's emit replaces the field and restores. ✓

### 3. The rewriter sees the right active set during body emit

Inside outer's body emit, the rewriter at [is_sifr_int_returning_function_call](crates/sifr_codegen/src/expr_render_helpers.rs:1406) reads `sifr_int_function_returns`, which now contains the nested helpers. So `helper()` calls inside outer's body get recognized as SifrInt-returning, and downstream coercion (let-retype, BinOp arm, comparison) fires correctly. Verified in the e2e fixture: `returned_big_from_nested_helper`'s `return helper()` is recognized; the call site `let returned_nested_helper: SifrInt = returned_big_from_nested_helper();` retypes correctly.

### 4. Closure return-state isolation (PR #1831's contract) preserved

The Closure/ClosureBlock arms in the rewriter still save/clear/restore `current_sifr_int_return`. This slice doesn't change that path. Crucially, the closure save/restore doesn't extend to `sifr_int_function_returns` — but it doesn't need to, because:

- Closure bodies inside a promoted outer should still see the outer's active function_returns (so calls to sibling nested helpers within the closure are recognized).
- Closures themselves never appear in `sifr_int_function_returns` (which only contains named functions), so no name collision risk.

I verified by probing nested helpers that call sibling nested helpers:

```sifr
def outer() -> int:
    def helper_a() -> int:
        return BIG_LIMIT + 1
    def helper_b() -> int:
        return helper_a() + 2
    return helper_b()
```

Emits `helper_b`'s closure body as `return helper_a() + SifrInt::from_i64(2);` — `helper_a()` is recognized as a SifrInt-returning call (via the active set during outer's emit), and the BinOp coercion fires. Round-trips at runtime. ✓

### 5. Probe matrix

| Probe                                                                       | Result |
|-----------------------------------------------------------------------------|--------|
| Direct nested helper (`returned_big_from_nested_helper`) — fixture          | ✓ promoted, retypes call site, runs |
| Sibling functions with same-named nested helpers — different promotion needs | ✓ isolated, no cross-talk |
| Sibling nested helpers where one calls the other (`helper_b` calls `helper_a`) | ✓ both recognized via active set |
| Doubly-nested (`outer` → `middle` → `inner`)                                | ✓ recursive promotion (see N1) |
| Pure-i64 paths (no SifrInt source anywhere)                                 | ✓ untouched |
| Earlier milestone shapes (BinOp, AugAssign, comparison, etc.)               | ✓ all still pass e2e fixture asserts |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1831), confirming no test deltas elsewhere.

## Notes

(Non-blocking observations only.)

### N1 — The slice undersells what it does: arbitrary-depth nesting works

The slice description and tracker phrasing call this "direct nested helper handling versus unsupported deeper nested-nested propagation." But because [hir_function_returns_sifr_int](crates/sifr_codegen/src/function_emitter.rs:744) recursively calls [collect_nested_sifr_int_function_returns](crates/sifr_codegen/src/function_emitter.rs:794) on each nested function's body, arbitrary depth is naturally handled. I verified:

```sifr
def with_double_nested() -> int:
    def middle() -> int:
        def inner() -> int:
            return BIG_LIMIT + 1
        return inner()
    return middle()
```

emits `with_double_nested -> SifrInt` with `middle` and `inner` both correctly recognized in their respective scopes. `let v: SifrInt = with_double_nested();` retypes and runs cleanly.

This is a feature, not a bug. The implementation is *more capable* than the description claims. Worth aligning the tracker bullet's wording in the next tracker PR — instead of "direct nested helpers", say something like "nested helpers (at any depth) whose returns transitively produce SifrInt via module sources or sibling nested helpers". Or leave the conservative wording and treat the deeper-nesting capability as accidental but harmless. Either way, no correctness concern.

### N2 — Captured-local-only nested helpers don't promote

When a nested helper produces `SifrInt` *only* through a captured outer local (rather than a module SifrInt source), the helper isn't recognized:

```sifr
def outer() -> int:
    big: int = BIG_LIMIT + 1     # outer-forced SifrInt local
    def helper() -> int:
        return big + 1            # helper captures big
    return helper()
```

emits `outer -> i64` (not promoted) because [hir_function_returns_sifr_int](crates/sifr_codegen/src/function_emitter.rs:744) on `helper` computes helper's *own* forced locals (which are empty — helper has no `let` for `big`), and `big` isn't in the module SifrInt bindings either. So helper's `big + 1` Return is judged not to need SifrInt storage, and helper isn't promoted. Outer's `return helper()` then sees helper not in the set, so outer isn't promoted either.

Pre-PR-#1833 (verified by `git checkout 6c80feec -- crates/sifr_codegen/`): same emit, same broken Rust. **Not a regression** — pre-PR also failed inside the closure body's `big + (1 as i64)` (SifrInt + i64 has no Add impl).

The fix would require propagating the outer's forced-locals set into nested-helper analysis, which is a broader closure-capture migration concern. The pass-1 review's tracker bullet at [issues/…/checklist:444](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md) phrases this as "function arguments / arg expressions that are already SifrInt" — captured locals are conceptually the same shape. Worth noting in the next tracker PR's open follow-up that this remains.

### N3 — Recursive nested helpers with module-source captures hit the LocalFn capture-arg issue

When a nested helper is recursive (calls itself), [try_lower_structured_nested_function_stmt](crates/sifr_codegen/src/function_emitter.rs:176) emits it as `RustStmt::LocalFn` with explicit capture parameters. The capture parameter types are determined by `lower_function_param_type` which uses `sifr_type_to_rust_type` — so a captured `BIG_LIMIT` (source-level `int`) becomes an `i64` parameter. But at the call site, the codegen passes `__const_BIG_LIMIT()` which returns `SifrInt`. Mismatch.

```sifr
def outer() -> int:
    def helper(n: int) -> int:
        if n <= 0:
            return BIG_LIMIT + 1
        return helper(n - 1) + 1
    return helper(2)
```

Post-PR emits `fn helper(n: i64, BIG_LIMIT: i64) -> SifrInt { ... return helper(..., __const_BIG_LIMIT()) ... }` — the recursive-capture argument fails because the parameter is i64 but the value is SifrInt.

Pre-PR-#1833 was broken at a different line (helper's body return mismatching the unchanged `-> i64` signature). Post-PR moves the failure to the capture argument. Both versions fail to compile. **Not a regression** — no working program stops working. The fix requires the broader function-argument migration.

### N4 — `current_sifr_int_return` save/restore in `try_lower_structured_nested_function_stmt` is to `false`, not the nested-fn's promotion status

I noted this in pass-1 of #1831 but it remains: `try_lower_structured_nested_function_stmt` sets `current_sifr_int_return.set(false)` rather than `self.function_returns_sifr_int(&func.name)`. For the typical Sifr nested-fn-as-closure path this doesn't matter (closures don't have explicit Returns coerced). But if a recursive nested helper were promoted to `-> SifrInt` (via the LocalFn path), its body's Return would not be value-coerced via `coerce_expr_to_sifr_int_value`. The body's BinOp coercion still produces SifrInt expressions, so the LocalFn signature usually matches. Only edge cases where a Return value is a registered local Ident (which would otherwise be `Clone(Ident)`) might silently degrade. Currently I can't construct a reachable failure from this; it remains a defensive future-proofing concern carried over from #1831.

### N5 — Test coverage for `collect_nested_sifr_int_function_returns` is e2e-only

The slice adds a non-trivial new helper but no focused unit tests for it. A few unit tests would harden against future regressions:

- Single-level nested helper detection (the e2e-pinned shape).
- Multi-level (doubly-nested) detection (the N1 capability).
- Sibling-shadowing isolation (e.g., assert that `function_sifr_int_returns_for_body` returns helper for `with_helper`'s body but not for `without_helper`'s body, even when both have a nested fn named `helper`).
- A "should not promote" sibling: a nested helper whose body is `return 42` shouldn't be added to the set.

The e2e fixture's `returned_big_from_nested_helper` exercises the load-bearing case at runtime, but unit tests would let future contributors catch regressions without a full pipeline run. Optional polish.

### N6 — Carry-forward open items unchanged

Lexical shadowing, legacy-emission paths, fallible `//` and `%`, function arguments / arg expressions that are already SifrInt, and the broader function-boundary migration all remain open in the tracker bullet at [issues/…/checklist:444](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). The next tracker PR should:

- Mark this slice complete (the "nested helpers whose own bodies naturally produce SifrInt" gap is now closed for module-level + module-source cases).
- Refine the residual open items to reflect captured-local-only nested helpers (N2 above) and recursive-capture argument shapes (N3 above) as still part of the broader function-boundary migration.
