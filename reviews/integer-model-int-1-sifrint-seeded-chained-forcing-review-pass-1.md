# Review: INT-1 SifrInt Seeded Chained-Forcing Pass 1

PR: [sifr#1851](https://github.com/sifr-lang/sifr/pull/1851)
Branch: `int-1-sifrint-seeded-chained-forcing`
Commit: `5ef6771c`

## Verdict

**Satisfied.**

This is a test-only PR (one file changed, +25 lines in [crates/sifr/tests/e2e/pass/module_constants.sifr](crates/sifr/tests/e2e/pass/module_constants.sifr)) that closes the chained-forcing residual called out at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463) and forwarded from [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:179-219](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md) (N1) and [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-tracker-review-pass-1.md:50-59](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-tracker-review-pass-1.md). The author probed first, found that current codegen already lowers the chained shape correctly after PR #1849, and locked the behavior in via two new e2e fixtures plus the corresponding `main()` asserts. The "no codegen change needed" framing is correct — verified empirically below.

The two new fixtures cover the explicit shape from the prior review's N1 trace (`derived: int = big + 1` inside `inner` where `big` is a captured forced `SifrInt`) for both the non-recursive helper-inside-helper shape and the recursive helper-inside-helper shape.

## Findings

No blocking findings.

### 1. The "codegen already handles this" claim is empirically correct

I ran `cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/module_constants.sifr` against `5ef6771c` and inspected the lowering for both new fixtures.

Non-recursive ([module_constants.sifr:133-141](crates/sifr/tests/e2e/pass/module_constants.sifr:133)):

```rust
fn returned_big_from_local_multilevel_chained_nested_helper() -> SifrInt {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let middle = || {
    let inner = || {
    let derived: SifrInt = &big + SifrInt::from_i64(1);
    let derived2: SifrInt = &derived + SifrInt::from_i64(1);
    return derived2.clone();
};
    return inner();
};
    return middle();
}
```

Recursive ([module_constants.sifr:143-152](crates/sifr/tests/e2e/pass/module_constants.sifr:143)):

```rust
fn returned_big_from_local_multilevel_chained_recursive_nested_helper() -> SifrInt {
    let big: SifrInt = __const_BIG_LIMIT() + SifrInt::from_i64(1);
    let middle = || {
    fn inner(remaining: i64, big: SifrInt) -> SifrInt {
        let derived: SifrInt = &big + SifrInt::from_i64(1);
        if remaining <= (0 as i64) {
            return derived.clone();
        }
        return inner(remaining - (1 as i64), big.clone()) + SifrInt::from_i64(1);
    }
    return inner(2 as i64, big.clone());
};
    return middle();
}
```

Both:
- Promote outer `big`, `middle`, and `inner` returns to `SifrInt`.
- Re-type the chained derived locals (`derived`, `derived2`) to `SifrInt`.
- Use `&` on referenced exact-int locals to preserve value semantics.
- Lift the recursive `big` capture as `SifrInt` and clone at recursive call sites.

No `i64`-typed `derived`/`derived2`, no `let derived: i64 = ...SifrInt...` mismatch — the type-mismatch hypothesis from the prior review's N1 [reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md:215](reviews/integer-model-int-1-sifrint-multilevel-forced-captures-review-pass-1.md) does not actually fire. The runtime asserts (`'100000000000000000003'`, `'100000000000000000004'`) match what the code computes (see §3 below).

### 2. Why the codegen works without seeding `register_sifr_int_forced_local_bindings`

The prior review's N1 recommended routing `register_sifr_int_forced_local_bindings` ([function_emitter.rs:108-134](crates/sifr_codegen/src/function_emitter.rs:108)) through a seeded variant of `collect_sifr_int_forced_locals`, anticipating that without seeding, `derived` would never be marked forced and the let-emitter would emit `let derived: i64 = ...SifrInt...`. That register call is in fact still unseeded on `5ef6771c` (line 126 calls the bare `collect_sifr_int_forced_locals`, which delegates to `collect_sifr_int_forced_locals_with_seed(..., &HashSet::new())` per [function_emitter.rs:1280-1287](crates/sifr_codegen/src/function_emitter.rs:1280)).

The reason codegen still emits the right thing is a different path: the post-lowering rewrite at [expr_render_helpers.rs:495-528](crates/sifr_codegen/src/expr_render_helpers.rs:495) is **value-driven**, not forced-set-driven. For each `RustStmt::Let`:

```rust
let force_sifr_int = self.is_forced_sifr_int_local(&name);
let value_is_sifr_int = self.is_sifr_int_expr(&value);
let (ty, value) = if is_legacy_i64_type(&ty) && (value_is_sifr_int || force_sifr_int) {
    let value = self.coerce_expr_to_sifr_int_value(value);
    self.sifr_int_local_bindings.borrow_mut().insert(name.clone());
    (Some(crate::RustType::Named("SifrInt".to_string())), value)
} else { ... }
```

Trace for `inner` in the new non-recursive fixture:
- After PR #1849, `sifr_int_local_bindings` for `inner` already contains `big` (extended at [function_emitter.rs:387-389](crates/sifr_codegen/src/function_emitter.rs:387)).
- Stmt `let derived: i64 = big + SifrInt::from_i64(1)` is processed. `is_sifr_int_expr(&big + SifrInt::from_i64(1))` walks the BinOp and recognizes `Ident("big")` via `is_registered_sifr_int_local("big")` ([expr_render_helpers.rs:1399-1401](crates/sifr_codegen/src/expr_render_helpers.rs:1399)) → `value_is_sifr_int = true`. Rewrite triggers: type becomes `SifrInt`, and `derived` is **inserted** into `sifr_int_local_bindings`.
- Next stmt `let derived2: i64 = derived + SifrInt::from_i64(1)`: same chain — `derived` is now registered, so `value_is_sifr_int = true`, rewrite triggers, `derived2` is registered.

So chained-forcing is achieved iteratively at let-statement granularity by the rewrite pass mutating `sifr_int_local_bindings` as it walks. The analysis-time `sifr_int_forced_local_bindings` set is technically incomplete (it contains `big` but not `derived`/`derived2`), but every code path that consumes it that I could find — `is_forced_sifr_int_local` callers in [function_emitter.rs:243](crates/sifr_codegen/src/function_emitter.rs:243) (recursive capture lowering, only checks the captured name) and [expr_render_helpers.rs:507/551/569](crates/sifr_codegen/src/expr_render_helpers.rs:507) (let/assign/aug-assign rewrites, ORed with `value_is_sifr_int` or `is_registered_sifr_int_local`) — already has a value-driven fallback that covers the chained case. The unseeded register is benign for current behavior.

This makes PR #1851 a coverage-only PR that pins down the value-driven mechanism rather than a code change. The prior review's "one-line fix on the codegen side" recommendation is no longer needed in practice — though the cosmetic asymmetry remains (see N1 below).

### 3. Asserts are arithmetically correct

| Fixture | Expression | Expected | Why |
|---------|------------|----------|-----|
| `returned_big_from_local_multilevel_chained_nested_helper` | `derived2 = (big+1)+1` where `big = BIG_LIMIT+1 = 1e20+1` | `1e20+3 = 100000000000000000003` | matches assert at [module_constants.sifr:209](crates/sifr/tests/e2e/pass/module_constants.sifr:209) ✓ |
| `returned_big_from_local_multilevel_chained_recursive_nested_helper` | `inner(2)` unfolds to `((big+1)+1)+1 = big+3 = (1e20+1)+3` | `1e20+4 = 100000000000000000004` | matches assert at [module_constants.sifr:210](crates/sifr/tests/e2e/pass/module_constants.sifr:210) ✓ |

Recursive trace:
- `inner(0)` returns `derived = big+1` (base case).
- `inner(1)` returns `inner(0) + 1 = (big+1) + 1 = big+2`.
- `inner(2)` returns `inner(1) + 1 = (big+2) + 1 = big+3`.
- `big = BIG_LIMIT+1 = 1e20+1` → final value `1e20+4`. ✓

The author's `cargo run -q -p sifr -- run` validations confirm the runtime values match the asserts; the orchestrated `scripts/run_all_tests.sh --profile quick` (`report_signature=e1bf653aaa770517`) is the same signature that has been clean since PR #1817, so no other suite shifted.

### 4. Both helper-inside-helper derived-local shapes are covered

The user explicitly asked whether non-recursive and recursive helper-inside-helper derived-local shapes are both covered.

| Shape | Fixture |
|-------|---------|
| Non-recursive `outer → middle → inner` with `derived: int = big + 1; derived2: int = derived + 1; return derived2` | `returned_big_from_local_multilevel_chained_nested_helper` ✓ |
| Recursive `outer → middle → inner(remaining)` with `derived: int = big + 1` and recursive return path `inner(remaining - 1) + 1` | `returned_big_from_local_multilevel_chained_recursive_nested_helper` ✓ |

The non-recursive case exercises a **two-step chain** (`big → derived → derived2`) inside the deepest helper. The recursive case exercises a **single-step derivation** (`big → derived`) plus the recursive call+const path that already had coverage in PR #1849. The split is reasonable: the non-recursive fixture validates that the iterative rewrite chains forward across multiple let statements; the recursive fixture validates that derived-local promotion composes with the recursive hidden-capture parameter lowering. See N2 below for an optional symmetry tweak.

### 5. Sibling fixtures unchanged

Spot-checked surrounding helpers via the same emit:

- `returned_big_from_nested_helper`, `returned_big_from_recursive_nested_helper` — still SifrInt-promoted via `__const_BIG_LIMIT()`. Unchanged.
- `returned_big_from_local_nested_helper`, `returned_big_from_local_recursive_nested_helper` (single-level forced-local captures, PR #1837/#1839) — unchanged.
- `returned_big_from_local_multilevel_nested_helper`, `returned_big_from_local_multilevel_recursive_nested_helper` (multilevel forced captures from PR #1849, no derived chain) — unchanged.
- `shadow_exact_module_constant_with_multilevel_*` (PR #1847) — still pure `i64` throughout. Unchanged.

The change is surgical: it only adds two new functions and two assert pairs.

### 6. Test will catch a regression

The two natural regressions for chained-forcing are:

- **Removal of the value-driven let rewrite at [expr_render_helpers.rs:507-515](crates/sifr_codegen/src/expr_render_helpers.rs:507)**. Without it, `let derived: i64 = &big + SifrInt::from_i64(1)` would emit, and rustc would fail with `Add<...> not implemented for &SifrInt` or a let-type mismatch. The new fixtures exercise this exact let shape under e2e-pass, so a regression would surface as a build failure in `scripts/run_e2e_pass.sh`.
- **Regression in PR #1849's `sifr_int_local_bindings.extend(sifr_int_captured_forced_locals)` at [function_emitter.rs:387-389](crates/sifr_codegen/src/function_emitter.rs:387)**. Without it, `is_registered_sifr_int_local("big")` would return false inside `inner`, the rewrite at `derived = big + 1` would not trigger, and the same rustc error would surface. This is the same regression covered by the existing PR #1849 fixtures, but the chained shape adds a defense-in-depth check at the next let level.

### 7. Phase scope alignment

[issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463):

> Continue the broader `Type::Int` codegen migration beyond direct helper/local expression rewrites: multi-level nested helpers with locals derived from captured forced `SifrInt` parents still need seeded chained-forcing in codegen, and unsupported augmented assignment/fallible `//` and `%` still need exact-int runtime/codegen support.

The first half is what PR #1851 addresses. The framing was "still need seeded chained-forcing in codegen", which implied a code change. The PR's interpretation is that the residual is satisfied if probing shows current codegen handles it; the seeded register is an internal-consistency improvement, not a behavior fix. That interpretation is defensible — see §2 — and the test addition is the right way to lock the behavior in.

The second half (augmented assignment / fallible `//` and `%`) is untouched and remains open.

The integer model design doc [internal_docs/integer_model.md](internal_docs/integer_model.md) does not speak to derived locals or chain depth specifically. The expected behavior — exact-int through any number of derivation steps — is implied by exact-int's value semantics.

## Notes

(Non-blocking observations only.)

### N1 — `register_sifr_int_forced_local_bindings` cosmetic asymmetry persists

[function_emitter.rs:108-134](crates/sifr_codegen/src/function_emitter.rs:108) still calls the unseeded `collect_sifr_int_forced_locals` rather than `collect_sifr_int_forced_locals_with_seed(..., self.sifr_int_forced_local_bindings.borrow().clone())`. Behaviorally this is fine (see §2) because every consumer of the forced set has a value-driven fallback. Structurally it is asymmetric with the analysis-side `collect_function_sifr_int_forced_locals_with_extra_and_shadowed` ([function_emitter.rs:1036-1081](crates/sifr_codegen/src/function_emitter.rs:1036)) which does seed.

The risk if a future code path consumes `is_forced_sifr_int_local` without an `||` to `is_registered_sifr_int_local` or `is_sifr_int_expr`, it would silently miss `derived`/`derived2`. The new fixtures would not catch that hypothetical regression because their chain works through the value-driven rewrite, not through forced-set membership.

A one-line seeded variant would harden against this by making the analysis and codegen forced sets equivalent. Worth a follow-up if the forced-set is ever load-bearing in a new code path. Tracker line 463's "seeded chained-forcing in codegen" framing technically refers to this exact change — either reword the residual to acknowledge the value-driven mechanism, or land the seeded variant alongside the test coverage. This PR does neither; it relies on the tracker entry being subsequently updated to reflect "satisfied via coverage" rather than "satisfied via seeded register".

### N2 — Recursive fixture chain depth could match the non-recursive one

The non-recursive fixture chains `big → derived → derived2` (two derivation steps). The recursive fixture chains only `big → derived` (one step). If `derived2: int = derived + 1` were also added to the recursive `inner` body before the `if remaining <= 0` check, the recursive case would exercise the same iterative rewrite depth as the non-recursive case, and the assert would shift from `1e20+4` to `1e20+5`.

Not blocking — the existing single-step shape composes correctly with hidden-capture parameter lowering (which is the unique aspect of the recursive case), and the non-recursive fixture already validates the iterative rewrite depth. Optional symmetry polish.

### N3 — Tracker bookkeeping for residual closure

The phase tracker [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463) still names "seeded chained-forcing in codegen" as an open item. After this PR merges, the natural follow-up tracker PR should either:

1. Mark the chained-forcing residual closed and shift line 463 to mention only "unsupported augmented assignment / fallible `//` and `%`", or
2. Reword line 463 to acknowledge that codegen handles chained-forcing via the value-driven rewrite, with the seeded register remaining as cosmetic-only.

The choice depends on whether N1 is filed as a separate follow-up or accepted as an intentional design. Either is fine — but the tracker should not be left implying that codegen is still broken for this shape.

### N4 — `def`-in-conditional gap unchanged

Carries forward from N2 of the previous review pass and N1 of the prior tracker review. [`collect_captured_outer_names_transitively`](crates/sifr_codegen/src/function_emitter.rs:1235-1271) still does not recurse into `def`s nested inside `if`/`while`/`for`/`match`. This PR neither widens nor narrows that gap.

### N5 — Validation

User-supplied `scripts/run_all_tests.sh --profile quick` `report_signature=e1bf653aaa770517` matches the same signature reported in PRs #1817 through #1849. Wall time 69.10s is consistent with the rest of the chain. No test deltas implied.

I separately verified that the fixture names (`returned_big_from_local_multilevel_chained_nested_helper`, `returned_big_from_local_multilevel_chained_recursive_nested_helper`) follow the same `returned_big_from_local_multilevel_*` convention as the PR #1849 fixtures and slot in correctly under the existing assert ordering at [module_constants.sifr:181-210](crates/sifr/tests/e2e/pass/module_constants.sifr:181) (declaration-order matched by lexicographic `insta` discovery — assert order at lines 184-185 mirrors helper definition order at lines 133-152).

## Probe matrix

| Probe | Result |
|-------|--------|
| `cargo run -q -p sifr -- emit module_constants.sifr` for `returned_big_from_local_multilevel_chained_nested_helper` | ✓ outer/middle/inner all SifrInt; `derived` and `derived2` both lowered to `let X: SifrInt = ...` |
| `cargo run -q -p sifr -- emit` for `returned_big_from_local_multilevel_chained_recursive_nested_helper` | ✓ recursive `inner` is `fn inner(remaining: i64, big: SifrInt) -> SifrInt`; `derived` lowered to `let derived: SifrInt = &big + ...` |
| Asserts arithmetically check out | non-recursive `1e20+3`, recursive `1e20+4` ✓ |
| `cargo run -q -p sifr -- run module_constants.sifr` (cache hit) | ✓ no failure surfaced |
| Sibling fixtures emit unchanged | ✓ |
| `register_sifr_int_forced_local_bindings` still unseeded on `5ef6771c` | ✓ confirmed at line 126; behavior fine due to value-driven let rewrite |
| Value-driven rewrite path verified at [expr_render_helpers.rs:507-515](crates/sifr_codegen/src/expr_render_helpers.rs:507) | ✓ |
| `scripts/run_all_tests.sh --profile quick` (per PR validation) | report_signature=e1bf653aaa770517, 69.10s ✓ |

## Carry-forward open INT-1 items

After this slice merges, the remaining INT-1 items per [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:463):

1. Unsupported augmented assignment / fallible `//` and `%` exact-int runtime/codegen support.
2. (Optional, surfaced by N1) Make `register_sifr_int_forced_local_bindings` seeded so the analysis and codegen forced sets stay in sync; pair with tracker reword at line 463 to reflect the "satisfied via coverage" outcome of this PR.

INT-1 milestone closure is now blocked only on the augmented assignment / fallible `//` / `%` work — the multi-level forced-capture and chained-forcing shapes are covered end-to-end.
