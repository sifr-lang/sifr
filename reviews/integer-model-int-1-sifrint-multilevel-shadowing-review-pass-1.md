# Review: INT-1 SifrInt Multilevel Shadowing Pass 1

PR: [sifr#1847](https://github.com/sifr-lang/sifr/pull/1847)
Branch: `int-1-sifrint-multilevel-shadowing`
Commit: `f2e76017`

## Verdict

**Satisfied.**

This slice closes N1 from the [single-level nested shadowing review pass 1](reviews/integer-model-int-1-sifrint-nested-shadowing-review-pass-1.md): multi-level nested helpers (helper inside helper) now correctly propagate outer locals/parameters that shadow exact-int module constants, so `BIG_LIMIT + 1` evaluated inside a doubly-nested closure no longer falls back to the `__const_BIG_LIMIT()` SifrInt helper.

The mechanism is a single new helper, [`collect_captured_outer_names_transitively`](crates/sifr_codegen/src/function_emitter.rs:1232), routed in for shadowed-module-binding capture analysis at [function_emitter.rs:1205](crates/sifr_codegen/src/function_emitter.rs:1205). It walks nested function bodies recursively, narrowing the visible outer set at each level by subtracting the level's own params and locally defined names, and collects which of those still-visible names are referenced anywhere in the subtree. The function `collect_captured_outer_names` (used for forced-locals capture) is left non-transitive — see Notes N1 below.

Unshadowed exact-int module constant captures (e.g. `returned_big_from_nested_helper`, `returned_big_from_recursive_nested_helper`) still promote to `SifrInt` after the change. Verified by `cargo run -q -p sifr -- emit` against `crates/sifr/tests/e2e/pass/module_constants.sifr`.

## Findings

No blocking findings.

### 1. `collect_captured_outer_names_transitively` semantics are correct

[function_emitter.rs:1232-1268](crates/sifr_codegen/src/function_emitter.rs:1232) computes:

1. `shadowed_in_func = params ∪ locally_defined_vars(body)` — names this function level introduces.
2. `visible_outer_names = outer_names − shadowed_in_func` — outer names still visible at this level.
3. Returns `collect_captured_outer_names(func, visible_outer_names) ∪ ⋃ recurse(nested_fn, visible_outer_names)`.

Two important details:

- The recursion threads `visible_outer_names` (not the original `outer_names`) into the next level. So a name re-shadowed by an intermediate scope is correctly hidden from deeper scopes. I traced this against an `inner` that shadows `BIG_LIMIT` inside `middle` (which itself sees `BIG_LIMIT` as outer-shadowed): inner's shadow correctly causes `BIG_LIMIT` to be excluded from the captured set above inner, while inner's own `local_binding_types` insertion at [function_emitter.rs:384-386](crates/sifr_codegen/src/function_emitter.rs:384) keeps the rewriter from emitting `__const_BIG_LIMIT()` inside it.
- `collect_locally_defined_vars` uses `TraversalConfig::LOCAL_SCOPE_ONLY` ([traversal.rs:12](crates/sifr_codegen/src/hir_analysis/traversal.rs:12)), which descends into if/for/match bodies but stops at nested function boundaries. This is the right choice: the current level's `let`s and for-targets shadow outer names, but nested fn locals do not.

### 2. Routing through `collect_sifr_int_captured_shadowed_module_bindings` covers both pipelines

The single call site [function_emitter.rs:1205](crates/sifr_codegen/src/function_emitter.rs:1205) is consumed by both the analysis pipeline and the codegen pipeline:

- **Analysis** — [`collect_nested_sifr_int_function_returns`](crates/sifr_codegen/src/function_emitter.rs:1166-1169) calls it when fixed-pointing nested helper return promotion. With the transitive fix, the captured shadow set propagates into `hir_function_returns_sifr_int_with_extra_forced_and_shadowed`, which extends the helper's `shadowed_module_bindings` and suppresses promotion.
- **Codegen** — [`try_lower_structured_nested_function_stmt`](crates/sifr_codegen/src/function_emitter.rs:296-299) calls it before lowering each nested function, then inserts the captured shadow names into `local_binding_types` at [function_emitter.rs:384-386](crates/sifr_codegen/src/function_emitter.rs:384). For multi-level cases, this means when `middle` is lowered, `local_binding_types["BIG_LIMIT"] = Type::Int` is set *during middle's body lowering*. When inner is then lowered (via `try_lower_structured_nested_function_stmt(inner)`), `outer_shadowed_module_bindings` for inner correctly observes `BIG_LIMIT` as already shadowed in the active scope ([function_emitter.rs:290-295](crates/sifr_codegen/src/function_emitter.rs:290)), and the recursive analysis again resolves to `{BIG_LIMIT}`.

The same routing also makes the [`recursive_capture_lowers_to_sifr_int`](crates/sifr_codegen/src/function_emitter.rs:236-244) gate work correctly for the recursive multi-level fixture: when `inner` is being lowered inside `middle`, `self.local_binding_types.contains_key("BIG_LIMIT")` is `true` (inserted by middle's pass), so `BIG_LIMIT` is NOT promoted to a `SifrInt` recursive capture parameter. Codegen confirms `fn inner(remaining: i64, BIG_LIMIT: i64) -> i64`.

### 3. Codegen output verified

`cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr`:

```rust
fn shadow_exact_module_constant_with_multilevel_nested_local() -> i64 {
    let BIG_LIMIT: i64 = 5 as i64;
    let middle = || {
        let inner = || {
            return BIG_LIMIT + (1 as i64);   // <-- outer's i64 5, no __const_BIG_LIMIT()
        };
        return inner();
    };
    return middle();
}

fn shadow_exact_module_constant_with_multilevel_recursive_nested_local() -> i64 {
    let BIG_LIMIT: i64 = 5 as i64;
    let middle = || {
        fn inner(remaining: i64, BIG_LIMIT: i64) -> i64 {     // <-- i64 capture, not SifrInt
            if remaining <= (0 as i64) {
                return BIG_LIMIT + (1 as i64);
            }
            return inner(remaining - (1 as i64), BIG_LIMIT) + (1 as i64);
        }
        return inner(2 as i64, BIG_LIMIT);
    };
    return middle();
}

fn shadow_exact_module_constant_param_with_multilevel_nested(BIG_LIMIT: i64) -> i64 {
    let middle = || {
        let inner = || {
            return BIG_LIMIT + (1 as i64);   // <-- outer's param i64, no __const_BIG_LIMIT()
        };
        return inner();
    };
    return middle();
}
```

All three round-trip with `'6'`, `'8'`, `'6'` respectively.

Sibling unshadowed helpers in the same fixture still emit `SifrInt` returns and `__const_BIG_LIMIT()` calls, e.g.

```rust
fn returned_big_from_nested_helper() -> SifrInt {
    let helper = || { return __const_BIG_LIMIT() + SifrInt::from_i64(1); };
    return helper();
}

fn returned_big_from_recursive_nested_helper() -> SifrInt {
    fn helper(remaining: i64, BIG_LIMIT: SifrInt) -> SifrInt { ... }
    return helper(2 as i64, __const_BIG_LIMIT());
}
```

So the change is surgical — it only affects the multi-level shadow case. ✓

### 4. Test coverage

- New unit test [`multilevel_nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int`](crates/sifr_codegen/src/function_emitter.rs:1549) builds outer→middle→inner with `BIG_LIMIT` as outer's `let` and inner referencing it; asserts `hir_function_returns_sifr_int` returns `false`. Ran and passes.
- New e2e fixture entries cover three shapes:
  - non-recursive multi-level local shadow (`'6'`)
  - recursive multi-level local shadow (`'8'`)
  - parameter shadow with multi-level nested (`'6'`)
- All four sibling single-level shadow tests from PR #1845 (`shadowed_module_const_local_does_not_promote_return_to_sifr_int`, `shadowed_module_const_param_does_not_promote_return_to_sifr_int`, `nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int`, `unshadowed_module_const_still_promotes_return_to_sifr_int`) still pass.

The new helper `middle_with_inner_returning_name` (lines 1445-1466) is a small DSL extension that mirrors the existing `helper_returning_name` pattern, keeping the test data terse.

### 5. Alignment with phase scope

The phase issue checklist line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:459](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:459) explicitly names this exact gap:

> multi-level nested helper lexical shadowing for outer locals that shadow exact-int module constants still needs scope-safe coverage

The PR addresses precisely that gap. The other two open INT-1 sub-items (unsupported augmented assignment / fallible `//` and `%`) are out of scope and untouched.

The integer model design doc [internal_docs/integer_model.md](internal_docs/integer_model.md) doesn't speak to multi-level nesting directly; it relies on standard lexical scoping. The PR's behavior — a name introduced in any enclosing scope shadows the module constant of the same name for all references inside that scope's nested functions — matches lexical scoping.

## Notes

(Non-blocking observations only.)

### N1 — Asymmetry: forced-locals capture is still non-transitive

[`collect_sifr_int_captured_forced_locals`](crates/sifr_codegen/src/function_emitter.rs:1194-1199) still uses the non-transitive `collect_captured_outer_names`. This is consistent with pre-PR behavior, but it creates an asymmetry with the now-transitive shadow capture.

A constructed multi-level case that exercises the asymmetry:

```sifr
def outer() -> int:
    big: int = BIG_LIMIT + 1   # forced SifrInt local
    def middle() -> int:
        def inner() -> int:
            return big + 0      # transitively captures big through middle
        return inner()
    return middle()
```

Trace: outer's `forced_locals = {big}`. `collect_sifr_int_captured_forced_locals(middle, {big}) = ∅` (middle has no direct `Name(big)` ref). middle's analysis runs with `extra_forced = ∅`, so when it walks inner via `collect_nested_sifr_int_function_returns`, `outer_forced_locals = ∅` is passed in, and inner's analysis doesn't observe `big` as forced. Inner's `big + 0` does not register as needing SifrInt storage; inner doesn't promote; middle doesn't promote; outer doesn't promote. The generated outer would return `i64`, but the actual captured `big` value is `SifrInt` from outer's binding, which is a Rust type mismatch in the emitted closure body.

I did not add this fixture in this review — it's a hypothetical that mirrors the existing `returned_big_from_local_recursive_nested_helper` shape but with one more level. The current fixture only exercises the single-level analog. Whether this case actually triggers depends on whether anyone has exercised it; the existing single-level fixture works because `collect_sifr_int_captured_forced_locals` finds the direct ref one level deep.

**Recommendation:** make `collect_sifr_int_captured_forced_locals` transitive in a follow-up by routing it through the same `collect_captured_outer_names_transitively` helper. The change would be a one-line swap symmetric to this PR's. Worth adding a fixture (`returned_big_from_local_multilevel_nested_helper`) to pin the runtime shape before flipping the analysis.

Not a regression — pre-PR same — and not strictly in scope for "respect multilevel shadows for SifrInt module constants."

### N2 — Top-level-only iteration of nested function statements

Both the new `collect_captured_outer_names_transitively` (line 1258) and the existing `collect_nested_sifr_int_function_returns` (line 1151) iterate `body.iter().filter_map(... NestedFunction ...)`, which only finds nested function definitions at the body's top level. A `def` defined inside an `if`/`while`/`for`/`match` block is *visible* to `collect_locally_defined_vars` (so it correctly enters `shadowed_in_func`) but is *not* recursed into for transitive shadow/return discovery.

Whether Sifr permits `def` inside conditional blocks (Python does), and whether any e2e fixture exercises that shape, was not exhaustively investigated. The pattern is consistent with the pre-existing nested-return analysis, so this PR is not introducing a new gap. Worth a focused audit if `def-in-conditional` becomes load-bearing.

### N3 — Style consistency

The new function `collect_captured_outer_names_transitively` is placed adjacent to the non-transitive `collect_captured_outer_names`, and the routing change at line 1205 is a single-line swap. The pattern is consistent with the existing `_with_extra_forced` → `_with_extra_forced_and_shadowed` pairing introduced in PR #1845. Clean refactor.

### N4 — Validation report signature

The user-supplied `scripts/run_all_tests.sh --profile quick` `report_signature=e1bf653aaa770517` matches the same signature reported in PR #1817 through #1845. This confirms no test deltas — neither new failures nor new flakes introduced by this slice.

Note: `cargo test -p sifr_codegen --lib --skip test_e2e_pass` shows 22 pre-existing parser-related failures (`Expected an indented block after class definition` etc.) on this branch. These are not specific to this change and reproduce on prior commits. The user's validation set (which uses the orchestrated `scripts/run_all_tests.sh --profile quick` runner) is the authoritative gate per AGENTS.md and shows clean.

## Probe matrix

| Probe | Result |
|-------|--------|
| Multi-level non-recursive nested local shadow (e2e fixture) | ✓ all `i64`, no `__const_BIG_LIMIT()` |
| Multi-level recursive nested local shadow (e2e fixture) | ✓ `i64` capture param, `i64` throughout, returns `8` |
| Multi-level non-recursive nested param shadow (e2e fixture) | ✓ uses outer param `i64`, returns `6` |
| Single-level nested shadow shapes from PR #1845 | ✓ unchanged (i64 throughout) |
| Unshadowed nested helpers (`returned_big_from_*`) | ✓ still SifrInt-promoted |
| Mixed sibling functions (one shadowed, one unshadowed) | ✓ each function lowers per its own shadow status |
| Unit test `multilevel_nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int` | ✓ passes |
| Unit tests `shadowed_module_const_*`, `nested_helper_*`, `unshadowed_module_const_*` | ✓ all 5 pass |
| `cargo run -q -p sifr -- run` on `module_constants.sifr` | ✓ all asserts pass |

## Carry-forward open INT-1 items

After this slice merges, the remaining INT-1 items per [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:459](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:459):

1. **(suggested follow-up from N1)** Multi-level forced-local capture propagation — make `collect_sifr_int_captured_forced_locals` transitive, paired with a `returned_big_from_local_multilevel_nested_helper` fixture.
2. Unsupported augmented assignment / fallible `//` and `%` exact-int runtime/codegen support.

INT-1 closure remains very close after this slice.
