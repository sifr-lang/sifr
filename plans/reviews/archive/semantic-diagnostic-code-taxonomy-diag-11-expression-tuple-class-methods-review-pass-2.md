# DIAG-11 Expression Method Diagnostics — Review Pass 2

## Verdict: No blocking issues remain.

## Pass 1 Blocker — Confirmed Fixed

**Blocker:** Protocol wrong-arity branch returning `Some(_)` instead of `None` after emitting CALL_WRONG_POSITIONAL_COUNT.

**Verification:** `crates/sifr_hir/src/lower/expressions.rs:3217-3229`
```rust
if args.len() != ft.params.len() {
    reject_method_arg_count(
        ctx,
        format!(
            "{}.{}() takes {} argument(s), got {}",
            name,
            method,
            ft.params.len(),
            args.len()
        ),
        method_count_range(args.len(), ft.params.len(), arg_ranges, method_range),
    );
    return None;  // <-- Correctly returns None
}
```
- Returns `None` after emitting CALL_WRONG_POSITIONAL_COUNT.
- Direct tests `test_protocol_method_wrong_arity_has_call_code` and `test_protocol_missing_method_has_protocol_code` both pass.

## Taxonomy Check

| Case | Code | Status |
|------|------|--------|
| method arity | CALL_WRONG_POSITIONAL_COUNT | ✅ All uses use `ctx.error_with_code_at` |
| callable field arity / non-callable call | CALL_NOT_CALLABLE_OR_ARITY | ✅ Via `expression_diagnostics::call_not_callable_or_arity` |
| argument/bounds type mismatch | TYPE_MISMATCH | ✅ Via `ctx.error_with_code_at` |
| tuple/bigint/default unsupported surface | STDLIB_UNSUPPORTED_SURFACE | ✅ Via `ctx.error_with_code_at` |
| class/enum missing method | CLASS_MISSING_MEMBER | ✅ Via `ctx.error_with_code_at` |
| protocol missing method | PROTO_BOUND_NOT_SATISFIED | ✅ Via `ctx.error_with_code_at` |

## Raw `ctx.error(...)` Inventory (Comprehensions/Generator/Walrus Only)

All 13 remaining `ctx.error(...)` calls are in the next slice's scope:

| Line | Message |
|------|---------|
| 3429 | "list comprehension must have at least one generator" |
| 3453 | comprehension inner expression error |
| 3461 | "comprehension target must be a simple name or tuple" |
| 3469 | comprehension final expr type mismatch |
| 3541 | "set comprehension target must be a simple name" |
| 3547 | set comprehension inner expression error |
| 3599 | comprehension inner expression error |
| 3608 | set comprehension final expr type mismatch |
| 3658 | "only single-generator generator expressions are supported" |
| 3667 | "generator target must be a simple name" |
| 3673 | generator expression type mismatch |
| 3728 | "walrus operator target must be a simple name" |

All raw `ctx.error(...)` calls are confined to comprehensions/generator/walrus handling. No method diagnostics use raw `ctx.error(...)`.

## Validation Results

```
cargo fmt          ✅ No output (clean)
cargo check -p sifr_hir  ✅ Finished dev profile
cargo clippy -p sifr_hir -- -D warnings  ✅ Finished dev profile
python3 scripts/check_hir_maintainability_guardrails.py  ✅ PASS
git diff --check  ✅ No output (clean)

cargo test -p sifr_hir protocol_    -- --nocapture  ✅ 21 passed
cargo test -p sifr_hir method_has   -- --nocapture  ✅ 11 passed
cargo test -p sifr_hir has_call_code-- --nocapture  ✅ 26 passed
cargo test -p sifr_hir has_type_code-- --nocapture  ✅ 22 passed
```

## New Direct Tests (Pass 2)

Two direct tests added in `expressions_tests.rs` verify the Pass 1 fix:

- `test_protocol_method_wrong_arity_has_call_code` (line 3411): Resolves a protocol method with wrong argument count — expects `CALL_WRONG_POSITIONAL_COUNT` with correct primary range, result is `None`.
- `test_protocol_missing_method_has_protocol_code` (line 3433): Resolves a non-existent protocol method — expects `PROTO_BOUND_NOT_SATISFIED` with correct primary range, result is `None`.

Both tests pass with `--nocapture`.

## Conclusion

The Pass 1 blocker is fully resolved. The slice is clean across all validation gates. No blocking issues remain.
