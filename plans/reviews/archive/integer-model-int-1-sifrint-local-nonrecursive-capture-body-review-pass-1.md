# Review: INT-1 SifrInt Local-Source Non-Recursive Capture Body Pass 1

**Verdict: Satisfied. No blockers.**

The slice cleanly closes the non-recursive captured-local case identified in PR #1837's pass-1 N-pass1-1 (and tracked in PR #1838's open follow-up). When a non-recursive nested helper closure captures an outer local already forced to `SifrInt`, the helper now: (a) gets correctly detected as SifrInt-returning by the nested-return pre-scan via the existing `hir_function_returns_sifr_int_with_extra_forced` helper now fed the captured-forced set, and (b) the closure body's references to the captured local get coerced via the BinOp arm because the inner `sifr_int_local_bindings` is unconditionally extended with the captured-forced names.

Recursive captured-local behavior from PR #1837 remains preserved (the recursive-captures path now uses the broader `sifr_int_nested_capture_bindings` union, which is a superset of the prior `sifr_int_recursive_captures`). Closure return-state isolation from PR #1831 is also preserved (small inner helpers that don't capture or naturally produce SifrInt stay `Fn() -> i64`).

## Findings

None blocking.

### 1. Mechanism is correctly state-aware

[function_emitter.rs:231-233](crates/sifr_codegen/src/function_emitter.rs:231) computes `outer_forced_locals` and `sifr_int_captured_forced_locals` at function entry, **before** any inner state clear. So `collect_sifr_int_captured_forced_locals` sees the outer's intact forced set and correctly identifies which captured outer names are SifrInt-forced. This is the same timing pattern PR #1837 established for `sifr_int_recursive_captures`.

The new union `sifr_int_nested_capture_bindings` at [function_emitter.rs:241-242](crates/sifr_codegen/src/function_emitter.rs:241) combines `sifr_int_recursive_captures` (which catches module-source / registered-local / forced-local recursive captures) with `sifr_int_captured_forced_locals` (which catches forced-local captures regardless of recursion). The union is then passed to `hir_function_returns_sifr_int_with_extra_forced` at [function_emitter.rs:250](crates/sifr_codegen/src/function_emitter.rs:250), so for non-recursive helpers (where `recursive_captures` is empty) the captured-forced names still flow into the SifrInt-detection analysis.

### 2. Inner `sifr_int_local_bindings` correctly populated for both paths

For RECURSIVE helpers, the existing [line 305-308](crates/sifr_codegen/src/function_emitter.rs:305) loop iterates `recursive_captures` and inserts captures into `sifr_int_local_bindings` when the membership check against `sifr_int_nested_capture_bindings` passes (broader than the previous `sifr_int_recursive_captures`). Pre-PR-#1839 this loop checked `sifr_int_recursive_captures`; post-PR it checks the broader union, but since `recursive_captures` itself is empty for non-recursive helpers, this loop is a no-op for non-recursive cases.

The new [line 316-318](crates/sifr_codegen/src/function_emitter.rs:316) unconditionally extends `sifr_int_local_bindings` with `sifr_int_captured_forced_locals` regardless of recursion. This is the key new mechanism for non-recursive helpers — it bypasses the `recursive_captures` iteration and directly registers the captured-forced names. For recursive helpers, this extend is a HashSet superset operation over the line 305-308 inserts (no behavioral change since HashSet dedupes).

The overall effect:
- **Non-recursive**: line 305-308 no-op, line 316-318 inserts captured-forced. Body emit sees captured names as registered SifrInt locals. BinOp arm coerces references.
- **Recursive**: line 305-308 inserts (per the broader union), line 316-318 redundantly inserts captured-forced. Body emit sees the same set.

Both paths converge correctly.

### 3. End-to-end verification

I reproduced the pass-1 N-pass1-1 reproducer post-this-PR with [/tmp/nonrecursive_v2.sifr]:

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
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let helper = || {
        return &big + SifrInt::from_i64(1);
    };
    return helper();
}
```

Compiles and runs. The closure body's `big + 1` is correctly coerced to `&big + SifrInt::from_i64(1)`. Outer is `-> SifrInt`. ✓

The new e2e fixture's `returned_big_from_local_nested_helper` round-trips with `'100000000000000000002'`. ✓

### 4. Probe matrix verified

| Probe                                                                | Result |
|----------------------------------------------------------------------|--------|
| Non-recursive helper capturing forced outer SifrInt local — fixture  | ✓ fixed (was pre-PR-#1839 broken) |
| Sibling functions: non-recursive + recursive + small-local           | ✓ state-isolated (`-> SifrInt` / `-> SifrInt` / `-> i64`) |
| Helper has its own non-forced local + captures outer forced          | ✓ inner local stays i64, captured local borrowed |
| Inner local shadowing captured-forced name (`let big: int = 7`)      | ✓ captured-forced detection skips shadowed name (uses `collect_locally_defined_vars` filter) |
| Helper with parameter + captures outer forced local                  | ✓ closure params inferred, captured borrowed |
| Multiple captures (mixed SifrInt + i64)                              | ✓ selective per-capture promotion |
| Closure that just returns the captured local (`return big`)          | ✓ `return big.clone();` (value-position coerce produces Clone) |
| Outer with SifrInt forced local but helper doesn't capture it        | ✓ helper stays `Fn() -> i64`, no over-broad promotion |
| `returned_big_with_nested_small()` (closure return-state isolation from #1831) | ✓ small inner stays `Fn() -> i64` |
| Pure-i64 recursive helper (no SifrInt source)                        | ✓ unaffected |
| Earlier milestone shapes (e2e fixture full asserts)                  | ✓ all still pass; 14 expr_render_helpers tests pass |

`scripts/run_all_tests.sh --profile quick` reproduces `report_signature=e1bf653aaa770517` (same as #1817–#1837), confirming no test deltas elsewhere.

### 5. Lexical shadowing handled correctly

When the helper has its own `let big: int = 7` shadowing an outer `big`:

```sifr
def outer() -> int:
    big: int = BIG_LIMIT + 1
    def helper() -> int:
        big: int = 7
        return big + 1
    return helper()
```

emits

```rust
fn outer() -> i64 {
    let big: SifrInt = …;
    let helper = || {
        let big: i64 = 7 as i64;
        return big + (1 as i64);
    };
    return helper();
}
```

Helper isn't promoted. `collect_sifr_int_captured_forced_locals` filters via `collect_locally_defined_vars(&func.body)` ([function_emitter.rs:976](crates/sifr_codegen/src/function_emitter.rs:976)) — the inner `big` is in `locally_defined`, so the captured-forced detection correctly skips it. ✓

### 6. Closure return-state isolation preserved

I verified the e2e fixture's `returned_big_with_nested_small` (the load-bearing case from PR #1831's closure return-state isolation):

```rust
fn returned_big_with_nested_small() -> SifrInt {
    let small_inner = || {
        return 42 as i64;        // <-- not coerced; small_inner doesn't naturally produce SifrInt
    };
    let value: i64 = small_inner();
    return __const_BIG_LIMIT() + SifrInt::from_i64(value);
}
```

`small_inner` doesn't capture any outer forced locals and its body just returns `42`. The slice's `nested_returns_sifr_int` for `small_inner`:
- `sifr_int_recursive_captures` empty (non-recursive).
- `sifr_int_captured_forced_locals` empty (`small_inner` references no outer forced names).
- `sifr_int_nested_capture_bindings` = empty.
- `hir_function_returns_sifr_int_with_extra_forced(small_inner, ..., {})` returns false (body's `return 42` doesn't need SifrInt storage).
- `nested_returns_sifr_int` = false. ✓

So `current_sifr_int_return.set(false)` for `small_inner`'s body emit, and the closure stays `Fn() -> i64`. ✓ No regression.

## Notes

(Non-blocking observations only.)

### N1 — Captured-forced detection is narrower than the recursive-captures filter

`collect_sifr_int_captured_forced_locals` checks only `outer_forced_locals` (forced-only). The companion `recursive_capture_lowers_to_sifr_int` checks three sources: module helpers, registered-local, AND forced-local. For the non-recursive path, this means a captured outer local that's *registered* but somehow *not forced* wouldn't be picked up by the new line 316-318 extend.

In practice this is hard to construct: pre-scan via `collect_sifr_int_forced_locals` typically forces any name whose value is SifrInt-shaped (LargeIntLiteral, Name in module/forced/function-returns, Type::Int BinOp/UnaryOp with sifr-int operand). The two states (registered and forced) tend to be correlated for SifrInt locals.

If a future case surfaces where a name is registered without being forced (e.g., re-Let where the new value isn't SifrInt-shape but the name was already registered — though the existing Let arm's else branch correctly removes the registration in that case), the slice's non-recursive captured-local detection would miss it. Worth noting but no current reproducer. Optional broadening.

### N2 — Line 316-318 extend is redundant for the recursive path

For recursive helpers, the line 305-308 loop already inserts captured names into `sifr_int_local_bindings` (via the broader `sifr_int_nested_capture_bindings` membership check that includes captured-forced). The new line 316-318 extend then re-inserts captured-forced names. HashSet dedupes, so this is a no-op for recursive helpers. Redundant but correct.

A future cleanup could either (a) inline the captured-forced insert into the line 305-308 loop, or (b) clarify with a comment that line 316-318 is "primarily for the non-recursive path; redundant for recursive helpers because the line 305-308 loop already covers them via the union membership check". Optional polish.

### N3 — No focused unit tests added

The e2e fixture's `returned_big_from_local_nested_helper` covers the load-bearing case at runtime, and the existing 14 expr_render_helpers tests still pass. No focused unit tests for:

- The `collect_sifr_int_captured_forced_locals` predicate against staged HirFunction inputs with various capture/shadow shapes.
- The line 316-318 extend's interaction with the line 305-308 recursive loop.

These would harden against future regressions without requiring a full pipeline run. Optional.

### N4 — Carry-forward open items unchanged

Lexical shadowing, legacy-emission paths, fallible `//` and `%`, function arguments / arg expressions that are already SifrInt — all stay tracked under the open INT-1 follow-up at [issues/…/checklist:450](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md). With this slice, the "captured-local-only non-recursive nested helpers" gap (the residual from PR #1837's tracker) is now closed. The next tracker PR should reflect that.
