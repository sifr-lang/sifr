# Review: INT-1 SifrInt Multilevel Forced Captures Pass 1

PR: [sifr#1849](https://github.com/sifr-lang/sifr/pull/1849)
Branch: `int-1-sifrint-multilevel-forced-captures`
Commit: `af9ee8af`

## Verdict

**Satisfied.**

This slice closes the N1 follow-up flagged in [reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:127-149](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md): multi-level nested helpers now propagate **forced exact-int locals** transitively, symmetric with the shadowed-module-binding propagation that landed in PR #1847. With this PR, a `big: int = BIG_LIMIT + 1` defined in `outer` and referenced inside `outer → middle → inner` correctly forces `outer`, `middle`, and `inner` to lower through `SifrInt` end-to-end.

This is not just polishing an asymmetry — without the fix, the same shape produces **invalid Rust** that fails to compile (verified in §3 below). Pre-PR, the prior commit emits `fn outer() -> i64 { let big: SifrInt = ...; let middle = || { let inner = || { return big + (0 as i64); }; ... } }`, which fails with `the trait Add<i64> is not implemented for SifrInt`. The PR is closing a correctness bug, not adding ergonomic coverage.

The change is two coordinated edits in [crates/sifr_codegen/src/function_emitter.rs](crates/sifr_codegen/src/function_emitter.rs), plus two new e2e fixture entries and a unit test. Phase-issue checklist line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:461](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:461) explicitly names this gap.

## Findings

No blocking findings.

### 1. The two-line edit is the minimum correct fix

[function_emitter.rs:1198-1201](crates/sifr_codegen/src/function_emitter.rs:1198) routes `collect_sifr_int_captured_forced_locals` through the existing `collect_captured_outer_names_transitively` helper that PR #1847 added — symmetric to [`collect_sifr_int_captured_shadowed_module_bindings`](crates/sifr_codegen/src/function_emitter.rs:1204-1209). One-line swap, exactly the shape recommended in the prior review's N1.

That change alone is sufficient for the **analysis** pipeline (return-promotion fixed-point). It is *not* sufficient for the **codegen** pipeline, because codegen rebuilds the forced-locals state per nested level rather than re-running analysis. The second edit at [function_emitter.rs:387-392](crates/sifr_codegen/src/function_emitter.rs:387) is what threads it through codegen:

```rust
self.sifr_int_local_bindings
    .borrow_mut()
    .extend(sifr_int_captured_forced_locals.iter().cloned());
self.sifr_int_forced_local_bindings        // NEW
    .borrow_mut()
    .extend(sifr_int_captured_forced_locals);
```

Why both extends are needed:

- `sifr_int_local_bindings` — read at emit time by `is_registered_sifr_int_local`, drives borrow/clone choices and binop coercion when the body is rewritten. Already populated pre-PR (PR #1839).
- `sifr_int_forced_local_bindings` — read at the *top* of `try_lower_structured_nested_function_stmt` ([function_emitter.rs:287](crates/sifr_codegen/src/function_emitter.rs:287)) as the seed `outer_forced_locals` for the *next* nested level. Without this set being populated during middle's body lowering, when the recursive `try_lower_structured_nested_function_stmt(inner)` runs inside middle, it sees `outer_forced_locals = ∅` and inner fails to recognize `big` as forced.

So the asymmetry between the two `extend`s in pre-PR code is exactly the bug: codegen carried the captured forced local one level deep but did not re-seed it for further descent. The PR fixes both directions of propagation.

### 2. Order of operations is correct

[function_emitter.rs:357-393](crates/sifr_codegen/src/function_emitter.rs:357-393) shows the sequence inside `try_lower_structured_nested_function_stmt`:

1. Clear `sifr_int_local_bindings` and `sifr_int_forced_local_bindings`.
2. Insert recursive captures (if any).
3. Insert `captured_shadowed_module_bindings` into `local_binding_types` as `Type::Int`.
4. Extend both SifrInt sets with `sifr_int_captured_forced_locals` ← *new line lives here*.
5. Call `register_local_body_binding_types(&func.body)` — which itself seeds from the now-populated `sifr_int_forced_local_bindings` (see [function_emitter.rs:125](crates/sifr_codegen/src/function_emitter.rs:125)) before union'ing in body-derived forced locals.

Step 4 must precede step 5; the diff places it correctly.

The `iter().cloned()` on the first extend and move on the second is the right idiom — `sifr_int_captured_forced_locals` is consumed once, no unnecessary clone of the whole set.

### 3. Codegen verified empirically

Three checks against the new e2e fixture entries plus a hand-written probe:

```rust
// returned_big_from_local_multilevel_nested_helper
fn returned_big_from_local_multilevel_nested_helper() -> SifrInt {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let middle = || {
        let inner = || {
            return &big + SifrInt::from_i64(0);   // big captured as SifrInt through 2 closures
        };
        return inner();
    };
    return middle();
}

// returned_big_from_local_multilevel_recursive_nested_helper
fn returned_big_from_local_multilevel_recursive_nested_helper() -> SifrInt {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let middle = || {
        fn inner(remaining: i64, big: SifrInt) -> SifrInt {     // big lowered as SifrInt capture param
            if remaining <= (0 as i64) {
                return &big + SifrInt::from_i64(0);
            }
            return inner(remaining - (1 as i64), big.clone()) + SifrInt::from_i64(1);
        }
        return inner(2 as i64, big.clone());
    };
    return middle();
}
```

Asserts in [crates/sifr/tests/e2e/pass/module_constants.sifr:184-185](crates/sifr/tests/e2e/pass/module_constants.sifr:184) round-trip `'100000000000000000001'` and `'100000000000000000003'` correctly under `cargo run -q -p sifr -- run`.

To confirm the bug-not-just-asymmetry framing, I checked out `af9ee8af~1` for [function_emitter.rs](crates/sifr_codegen/src/function_emitter.rs) and ran the same fixture shape (without the new fixtures, using a hand-crafted `outer → middle → inner` program). Pre-PR emission for the identical Sifr source was:

```rust
fn outer() -> i64 {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let middle = || {
        let inner = || {
            return big + (0 as i64);   // ← Add<SifrInt, i64> not implemented
        };
        return inner();
    };
    return middle();
}
```

`cargo run` on that emission fails with `error[E0277]: the trait Add<i64> is not implemented for SifrInt`. So the multi-level shape was unreachable through pre-PR codegen — any user who happened to write it would hit a rustc error in the generated crate, not a Sifr-side panic. That aligns with INT-1's "if it compiles, it works" guarantee being preserved (no user-triggerable runtime panic, only a build-time rustc error from broken codegen), but it's still a correctness gap that this PR closes.

### 4. Sibling fixtures unchanged

Spot-checked the surrounding helpers in `module_constants.sifr` after `cargo run -q -p sifr -- emit`:

- `returned_big_from_nested_helper` / `returned_big_from_recursive_nested_helper` — still `SifrInt` returns through `__const_BIG_LIMIT()`.
- `returned_big_from_local_nested_helper` / `returned_big_from_local_recursive_nested_helper` (single-level forced-local captures, PR #1837/#1839) — unchanged.
- `shadow_exact_module_constant_with_multilevel_nested_local` / `_recursive_nested_local` / `_param_with_multilevel_nested` (multilevel **shadow** fixtures from PR #1847) — still emit pure `i64` throughout, no `__const_BIG_LIMIT()`.

The change is surgical: it only affects multi-level captures of names that are in the outer's `sifr_int_forced_local_bindings` set. ✓

### 5. Unit test coverage

The new test [`multilevel_nested_helper_captures_forced_local_and_promotes_return_to_sifr_int`](crates/sifr_codegen/src/function_emitter.rs:1583-1598) exercises the analysis side of the change directly:

```rust
let func = middle_with_inner_returning_name("big");   // middle → inner referencing `big`
let forced_locals = HashSet::from(["big".to_string()]);
assert_eq!(
    collect_sifr_int_captured_forced_locals(&func, &forced_locals),
    forced_locals     // {big} surfaces transitively
);
assert!(hir_function_returns_sifr_int_with_extra_forced(
    &func,
    &HashSet::new(),
    &HashSet::new(),
    &forced_locals,
));
```

Reuses `middle_with_inner_returning_name` from PR #1847 — the test DSL helpers from prior slices compose cleanly. All 6 `function_emitter::tests` (including the 5 from prior slices) pass.

### 6. Capture analysis correctness across edge cases

Walked `collect_captured_outer_names_transitively` against the cases the prior pass-1 review enumerated, with `outer_names = forced_locals`:

| Shape | Captured | Promoted? |
|-------|----------|-----------|
| `def middle: def inner: return big + 0` (the new fixture) | `{big}` | ✓ inner SifrInt, middle SifrInt, outer SifrInt |
| `def middle: big = 5; def inner: return big + 0` (middle re-shadows) | `∅` | ✓ middle returns i64, no promotion |
| `def L1: def L2: def L3: return big + 0` (3-level) | `{big}` at every level | ✓ all SifrInt |
| `def helper1: def inner: return big + 0` and sibling `def helper2: return 0` | helper1 captures `{big}`, helper2 captures `∅` | ✓ helper1 SifrInt, helper2 i64 |
| recursive inner referencing `big` (the new recursive fixture) | `{big}` recursive capture, lowered as `SifrInt` capture param | ✓ matches single-level analog from PR #1837 |

The transitive helper's structure — at each level, subtract the level's own params and `collect_locally_defined_vars(body)` from the visible-outer set, then descend into nested defs *only at this level's top* — composes correctly: a re-shadow at any intermediate level correctly hides the name from deeper scopes, and shadowing in deeper levels does not retroactively affect siblings.

### 7. Recursive multilevel capture lowering

The new `returned_big_from_local_multilevel_recursive_nested_helper` fixture lowers `inner` (recursive) inside `middle` (non-recursive closure). Trace at [function_emitter.rs:301-308](crates/sifr_codegen/src/function_emitter.rs:301-308):

- During middle's body lowering, `self.sifr_int_local_bindings = {big}` (from this PR's first extend at line 387).
- When `try_lower_structured_nested_function_stmt(inner)` runs, `recursive_captures` for inner contains `big` (Type::Int convention=own from the HIR, since Type::Int has Copy ownership).
- [`recursive_capture_lowers_to_sifr_int`](crates/sifr_codegen/src/function_emitter.rs:236-244) checks `is_registered_sifr_int_local("big")` ← true (set during middle's body lowering by the PR's first extend) → returns true.
- `lower_recursive_capture_param_type` returns `RustType::Named("SifrInt")` → inner's signature becomes `fn inner(remaining: i64, big: SifrInt) -> SifrInt`. ✓

The borrow/clone semantics inside the recursive body (`&big + SifrInt::from_i64(0)` for the leaf, `inner(..., big.clone())` for the recursive call) are inherited from the existing single-level recursive-capture path (PR #1837) and do not need to be re-derived for multilevel.

### 8. Phase scope alignment

[issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:461](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:461):

> Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: multi-level nested helper capture of outer locals already forced to `SifrInt` still needs transitive capture propagation, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support.

This PR addresses the first half of that line precisely. The second half (augmented assignment / fallible `//` and `%`) is out of scope and untouched.

The integer model design doc [internal_docs/integer_model.md](internal_docs/integer_model.md) does not speak to nesting depth specifically; the model relies on standard lexical scoping with exact-int promotion. A name forced to `SifrInt` at any enclosing scope should propagate to any inner scope that captures it lexically (without re-shadow). The PR enforces that.

## Notes

(Non-blocking observations only.)

### N1 — `register_sifr_int_forced_local_bindings` does not re-seed `collect_sifr_int_forced_locals`

[function_emitter.rs:108-134](crates/sifr_codegen/src/function_emitter.rs:108-134), specifically line 126:

```rust
let mut forced = self.sifr_int_forced_local_bindings.borrow().clone();   // {big} after this PR
forced.extend(collect_sifr_int_forced_locals(
    body,
    &local_int_bindings,
    &shadowed_module_bindings,
    &module_sifr_int_bindings,
    &function_sifr_int_returns,
));     // ← does not pass `forced` as a seed
```

The analysis-side counterpart [`collect_function_sifr_int_forced_locals_with_extra_and_shadowed`](crates/sifr_codegen/src/function_emitter.rs:1036-1081) calls `collect_sifr_int_forced_locals_with_seed` with `extra_forced_locals` as the seed, so a chained-forcing case (`derived: int = big + 1` where `big` is captured-forced) propagates correctly through the analysis. The codegen-side `register_sifr_int_forced_local_bindings` does not — it calls the unseeded `collect_sifr_int_forced_locals` and only union's the body-derived result with the existing forced set.

Constructed case that exercises the gap:

```sifr
def outer() -> int:
    big: int = BIG_LIMIT + 1
    def middle() -> int:
        def inner() -> int:
            derived: int = big + 1     # should be forced because big is forced
            return derived             # uses derived
        return inner()
    return middle()
```

Trace (with this PR's fix in place but N1 unaddressed):
- inner's `sifr_int_forced_local_bindings` is seeded with `{big}` (via this PR).
- `register_local_body_binding_types(inner.body)` runs: `local_int_bindings = {derived}`.
- `collect_sifr_int_forced_locals(inner.body, {derived}, …)` is called with NO seed → its internal `forced` set starts at `∅`. When walking `derived = big + 1`, it tests `hir_expr_needs_sifr_int_storage(big + 1, forced=∅, …)`. The `Name(big)` arm checks `forced.contains("big") || (module_sifr_int_bindings.contains("big") && !shadowed)` — both false → `derived` is NOT marked forced.
- `forced.extend(∅)` → final `sifr_int_forced_local_bindings = {big}`, `derived` missing.

Whether this causes a downstream rustc error depends on what the let-emitter does with `local_binding_types[derived] = Type::Int` when the RHS lowers to `&big + SifrInt::from_i64(1)` (`SifrInt`-typed). Likely a Rust type mismatch on the `let derived: i64 = ...SifrInt...`. I did not construct an end-to-end fixture (it would lock in the bug shape before the fix). The current new fixtures only exercise `return big + 0` directly, not `derived: int = big + 1; return derived`, so this gap is not covered by tests.

Pre-PR same — and the analysis pipeline already handles this correctly via the seeded variant — so it's not a regression introduced here. But it's the natural next slice if it actually triggers in practice.

**Recommendation:** route `register_sifr_int_forced_local_bindings` through a seeded variant analogous to `collect_sifr_int_forced_locals_with_seed`, taking `self.sifr_int_forced_local_bindings.borrow().clone()` as the seed. One-line change on the codegen side. Worth pairing with a fixture (e.g. `returned_chained_forced_from_local_multilevel_nested_helper`) before flipping the codegen.

### N2 — `def`-in-conditional still not recursed into

Carries forward from N2 of the prior pass-1 review ([reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md:151-155](reviews/integer-model-int-1-sifrint-multilevel-shadowing-review-pass-1.md)):

[`collect_captured_outer_names_transitively`](crates/sifr_codegen/src/function_emitter.rs:1235-1271) at line 1261 iterates `func.body.iter().filter_map(NestedFunction)`, which only finds `def` at the body's top level. A `def` inside `if`/`while`/`for`/`match` is not recursed into for transitive capture analysis. `collect_locally_defined_vars` does correctly enter such `def` names into the shadow set (because traversal descends into control-flow blocks for *let-bindings*), but the *transitive walk* does not descend.

I did not exhaustively check whether Sifr permits `def`-in-conditional or whether any e2e fixture exercises that shape. The pattern is consistent with the pre-existing nested-return analysis at [function_emitter.rs:1147-1195](crates/sifr_codegen/src/function_emitter.rs:1147-1195), so this PR is not introducing a new gap. Worth a focused audit if `def`-in-conditional becomes load-bearing.

### N3 — Style consistency

`collect_sifr_int_captured_forced_locals` and `collect_sifr_int_captured_shadowed_module_bindings` ([function_emitter.rs:1197-1209](crates/sifr_codegen/src/function_emitter.rs:1197-1209)) are now perfectly symmetric — both delegate to `collect_captured_outer_names_transitively`. The double-extend at [function_emitter.rs:387-392](crates/sifr_codegen/src/function_emitter.rs:387-392) is a clean addition; the `iter().cloned()` then move idiom is the right shape for consuming the set once.

The new unit test reuses `middle_with_inner_returning_name` from PR #1847 — clean composition.

### N4 — Validation

User-supplied `scripts/run_all_tests.sh --profile quick` `report_signature=e1bf653aaa770517` matches the same signature reported in PR #1817 through #1847. Confirms no test deltas from this slice.

I separately verified the 22 pre-existing `cargo test -p sifr_codegen --lib` failures (parser-related: `Expected an indented block after class definition`, etc.) reproduce on `af9ee8af~1` after temporarily checking out main's [function_emitter.rs](crates/sifr_codegen/src/function_emitter.rs) and [module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr): 580 passed, 22 failed pre-PR vs 581 passed, 22 failed post-PR. The +1 is exactly the new unit test. No new failures.

The orchestrated `scripts/run_all_tests.sh --profile quick` runner is the authoritative gate per AGENTS.md and shows clean.

## Probe matrix

| Probe | Result |
|-------|--------|
| `returned_big_from_local_multilevel_nested_helper` (new e2e fixture) | ✓ outer/middle/inner all SifrInt, asserts `'100000000000000000001'` |
| `returned_big_from_local_multilevel_recursive_nested_helper` (new e2e fixture) | ✓ inner is `fn inner(remaining: i64, big: SifrInt) -> SifrInt`, asserts `'100000000000000000003'` |
| Pre-PR emission for the same multilevel non-recursive shape | ✗ rustc fails: `Add<i64> not implemented for SifrInt` (confirms PR closes a real bug) |
| `multilevel_nested_helper_captures_forced_local_and_promotes_return_to_sifr_int` (new unit test) | ✓ passes |
| All 5 prior `function_emitter::tests` (shadow + capture) | ✓ pass |
| Unshadowed nested helpers (`returned_big_from_nested_helper`, `returned_big_from_recursive_nested_helper`) | ✓ unchanged, still SifrInt-promoted via `__const_BIG_LIMIT()` |
| Single-level forced-local nested helpers (`returned_big_from_local_nested_helper`, `returned_big_from_local_recursive_nested_helper`) | ✓ unchanged |
| Multilevel **shadow** fixtures from PR #1847 (`shadow_exact_module_constant_with_multilevel_*`) | ✓ unchanged, still pure `i64` throughout |
| `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/module_constants.sifr` | ✓ all asserts pass |
| `cargo test -p sifr_codegen --lib` | 581 passed, 22 failed (22 are pre-existing parser-related, reproduce on main) |

## Carry-forward open INT-1 items

After this slice merges, the remaining INT-1 items per [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:461](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:461):

1. **(suggested follow-up from N1)** Multi-level chained-forcing in nested helpers — make `register_sifr_int_forced_local_bindings` seed `collect_sifr_int_forced_locals` with the existing forced set, so locals derived from captured-forced parents (e.g. `derived: int = big + 1` inside an inner) are themselves forced. Pair with a fixture that exercises the chained shape.
2. Unsupported augmented assignment / fallible `//` and `%` exact-int runtime/codegen support.

INT-1 closure remains very close after this slice — the multilevel forced-capture path matches the multilevel shadow path now in both analysis and codegen.
