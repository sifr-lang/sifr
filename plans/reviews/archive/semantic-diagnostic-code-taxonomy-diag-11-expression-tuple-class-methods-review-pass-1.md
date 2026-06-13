# DIAG-11 Review: Expression Method Diagnostics Migration (Pass 1)

**Scope**: `crates/sifr_hir/src/lower/expressions.rs`, `crates/sifr_hir/src/lower/expressions_tests.rs`
**Branch**: `codex/diag-11-raw-hir-expression-tuple-class-methods`

---

## Blocking Issue

### 1. Protocol arity check missing `return None` — behavioral change (bug)

**Location**: `expressions.rs:3215–3230`

```rust
Type::Protocol { name, methods, .. } => {
    if let Some((_, ft)) = methods.iter().find(|(n, _)| n == method) {
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
            // BUG: missing `return None;` here
        }
        Some(canonicalize_class_surface_type(&ft.return_type))
```

**Problem**: After `reject_method_arg_count` fires for wrong arity, execution falls through to `return Some(...)` instead of returning `None`. All other `resolve_method_type` arms (list, dict, set, str, tuple, class, enum, bigint) correctly `return None` after emitting an arity diagnostic. The Protocol arm is the only one missing this.

**Consequence**: When a protocol method is called with the wrong number of arguments, the diagnostic is emitted but type inference continues as if the call succeeded, producing incorrect types downstream. This is an accidental behavioral change introduced by the migration.

**Fix**: Add `return None;` after the `reject_method_arg_count` call inside the Protocol arm, on line ~3229.

---

## Non-Blocking Observations

### A. Protocol type-argument mismatch not checked

The Protocol arm in `resolve_method_type` (lines 3215–3238) validates argument **count** but not argument **types** when a method is found. Compare with the Class arm (lines 3117–3213) which checks both. This appears pre-existing (not introduced by this slice) but worth noting as the Protocol arm is structurally incomplete relative to Class.

### B. Protocol missing-method test absent

The new tests cover tuple, class, newtype, enum, bigint, and generic types. A test for `PROTO_BOUND_NOT_SATISFIED` on a protocol missing method (not just a protocol method call on a type that doesn't satisfy the protocol) is missing. Example pattern covered by `test_reversed_rejects_non_reversible_iterator_argument` for a different diagnostic path; a direct protocol-missing-method test should be added to be consistent with the other surface tests.

### C. `method_count_range` behavior with 0 args

For newtype `.value()`, enum `.name()`, `.value()`, and bigint `.clone()` — when `reject_no_method_args` computes `method_count_range(arg_ranges.len(), 0, arg_ranges, method_range)` — `arg_ranges` is empty (`[]`) and `method_range` is the method identifier range. The range will point at the method identifier, which is correct. However, in the arity-error case for e.g. `port.value(1)`, `method_count_range(1, 0, arg_ranges, method_range)` should point at the offending argument `(1)`, not the method identifier. This is the expected behavior per `method_count_range` and matches all other tests. No issue.

### D. Remaining raw `ctx.error(...)` sites

Confirmed all remaining raw `ctx.error(...)` sites in `resolve_method_type` are for comprehensions/generator/walrus (list comp, set comp, dict comp, generator, walrus operator) — correctly scoped for the next slice. No tuple/class/protocol/newtype/enum/bigint/default method surfaces remain using raw `ctx.error(...)`.

### E. Taxonomy consistency

| Surface | Error Type | Code | Status |
|---------|-----------|------|--------|
| tuple arity | CALL_WRONG_POSITIONAL_COUNT | ✓ | migrated |
| tuple type mismatch | TYPE_MISMATCH | ✓ | migrated |
| tuple missing method | STDLIB_UNSUPPORTED_SURFACE | ✓ | migrated |
| class arity | CALL_WRONG_POSITIONAL_COUNT | ✓ | migrated |
| class type mismatch | TYPE_MISMATCH | ✓ | migrated |
| class callable-field arity | CALL_NOT_CALLABLE_OR_ARITY | ✓ | migrated |
| class non-callable field | CALL_NOT_CALLABLE_OR_ARITY | ✓ | migrated |
| class missing method | CLASS_MISSING_MEMBER | ✓ | migrated |
| protocol missing method | PROTO_BOUND_NOT_SATISFIED | ✓ | migrated |
| newtype value arity | CALL_WRONG_POSITIONAL_COUNT | ✓ | migrated |
| enum name/value arity | CALL_WRONG_POSITIONAL_COUNT | ✓ | migrated |
| enum missing method | CLASS_MISSING_MEMBER | ✓ | migrated |
| bigint clone arity | CALL_WRONG_POSITIONAL_COUNT | ✓ | migrated |
| bigint missing method | STDLIB_UNSUPPORTED_SURFACE | ✓ | migrated |
| generic type missing | STDLIB_UNSUPPORTED_SURFACE | ✓ | migrated |

All taxonomy assignments are correct per the specification.

### F. Test coverage for new tests

All 13 new tests are present and correctly structured:
- `test_tuple_method_wrong_positional_count_has_call_code` — arity ✓
- `test_tuple_method_type_mismatch_has_type_code` — type mismatch ✓
- `test_tuple_missing_method_has_stdlib_code` — unsupported surface ✓
- `test_class_method_argument_type_has_type_code` — class method arg type ✓
- `test_callable_field_wrong_arity_has_call_code` — callable field arity ✓
- `test_class_field_not_callable_has_call_code` — non-callable field ✓
- `test_class_missing_method_has_class_code` — class missing member ✓
- `test_newtype_value_wrong_arity_has_call_code` — newtype arity ✓
- `test_enum_value_wrong_arity_has_call_code` — enum arity ✓
- `test_enum_missing_method_has_class_code` — enum missing member ✓
- `test_bigint_clone_wrong_arity_has_call_code` — bigint arity ✓
- `test_bigint_missing_method_has_stdlib_code` — bigint unsupported ✓
- `test_generic_type_missing_method_has_stdlib_code` — generic unsupported ✓

All verify message text, diagnostic code, and primary range.

---

## Verdict

**One blocking issue**: Protocol arity check is missing `return None;` after `reject_method_arg_count`, causing the function to return `Some(...)` instead of `None` when a protocol method is called with wrong argument count. This must be fixed before merge.

All other aspects are correct: taxonomy is consistent, ranges are precise, test coverage is adequate, and no tuple/class/protocol/newtype/enum/bigint/default method raw diagnostics remain in scope.
