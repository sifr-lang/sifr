

**Review: Failure type surface slice — Phase 32**

After thorough review of the staged diff against the async concurrency model contract:

---

**VALIDATION: `Failure[E]` is correctly rejected as a Result error channel**

- `is_valid_error_type` only accepts `Type::Class` → `Failure[E]` is implicitly rejected. ✓
- E2E test `failure_not_result_error.sifr` covers the rejection path. ✓
- Unit test `failure_type_is_not_valid_result_error` in `result_diagnostics_tests.rs` verifies the diagnostic fires with correct range. ✓

**VALIDATION: Codegen produces correct private `__SifrFailure<E>` surface**

- `__SifrFailure<E>` struct with `primary: E` and `secondary: Vec<SecondaryError>` fields. ✓
- Visibility is `Private` (not exposed as a public API surface). ✓
- Function signatures lower to `fn observe(failure: &__SifrFailure<ValueError>)`. ✓
- Module-uses detection wired: `module_uses_failure_type`, `function_uses_failure_type`, `type_contains_failure`, `type_contains_by`. ✓

**VALIDATION: Type system surface is complete**

- `Type::Failure(Box<Type>)` added with all required trait implementations (`is_never_false`, `display_name`, `rust_type`, `type_name`, `contains_any`, `is_assignable_to`). ✓
- Type variable substitution handles `Failure`. ✓
- Generic inference handles `Failure`. ✓
- Union type sort key updated (Failure at 21, TimeoutResult bumped to 22). ✓
- `SecondaryError` referenced for codegen deps in `error_refs.rs`. ✓
- `Failure` added to all Debug-display-opt-out locations (`expr_ref_emitter.rs`, `intrinsic_method_emitters.rs`, `stmt_support_emitter.rs`). ✓

**VALIDATION: Scope is appropriately scoped as type/codegen surface**

This is a type annotation and codegen surface slice. The implementation correctly stops at:
- `Failure[E]` annotation parsing → HIR lowering
- Private `__SifrFailure<E>` struct codegen
- Validation that `Failure[E]` is evidence not an error channel

It does **not** pretend to include runtime `TaskResult -> Failure` wrapping, which is a separate runtime concern. ✓

---

**REQUIRED FIXES (not satisfied):**

1. **`TimeoutResult` should also be rejected as a Result error type** — `TimeoutResult[E]` is defined as `enum TimeoutResult[E]: Inner(E), Timeout(TimeoutError)` and is evidence semantics, not an ordinary error channel. Currently `is_valid_error_type` implicitly rejects it (only matches `Type::Class`), but there is **no test** verifying that `Result[T, TimeoutResult[Error]]` is invalid. Add a unit test mirroring `failure_type_is_not_valid_result_error` for `TimeoutResult`:

```rust
#[test]
fn timeout_result_type_is_not_valid_result_error_channel() {
    let source = "\
def broken() -> Result[None, TimeoutResult[ValueError]]:
    return None
";
    let result = lower_source(source);
    let errors = result.expect_err("TimeoutResult should not be a Result error type");
    assert!(errors.iter().any(
        |error| error.code == Some(DiagnosticCode::RESULT_INVALID_ERROR_TYPE)
    ), "expected SIFR-RESULT-0002 on TimeoutResult error channel");
}
```

2. **`format_type_name` should handle `TimeoutResult`** — `diagnostics.rs:format_type_name` handles `Type::Failure` but not `Type::TimeoutResult`. If a `TimeoutResult` appears in an error message, it falls through to `format!("{ty:?}")` and produces an ugly debug string. Add:

```rust
Type::TimeoutResult(inner) => format!("TimeoutResult[{}]", format_type_name(inner)),
```

---

**Remaining assessment:**

The core design contract is satisfied. `Failure[E]` is correctly surfaced as evidence-with-secondary rather than an error channel. The private struct codegen, type system plumbing, and HIR lowering are coherent. With the two fixes above (1 test + 1 format helper), the slice would be complete.
