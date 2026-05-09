# Review: INT-3 Bool/Integer Comparison Diagnostic (PR #1865)

## Summary

Implements `SIFR-INT-0007` for direct bool/integer equality and ordering comparisons without explicit conversion.

## Findings

### Correctness

**Type system (`check.rs`)**

- `is_bool_type`: returns `true` for `Type::Bool` and `Type::LiteralBool(_)` after alias resolution — correct.
- `is_integer_type`: returns `true` for `Type::Int`, `Type::LiteralInt(_)`, `Type::BigInt`, and all `Type::FixedInt(_)` variants — correct. Note that this includes `U64` and `USize` which are blocked from the fixed-width-to-`int` promotion path; this is acceptable as all bool/integer comparisons should be explicit regardless of the fixed-width variant.
- `is_bool_integer_mixed_comparison`: symmetric — `(bool && int) || (int && bool)`. Correct.
- The check fires on both equality (`==`, `!=`) and ordering (`<`, `>`, `<=`, `>=`) arms, before the existing mixed int/bigint block — correct placement.
- `Type::Bool == Type::Bool` is preserved (verified by explicit assertion at `check.rs:809`) — correct.

**Registry (`codes.rs`)**

- `INT_BOOL_INTEGER_COMPARISON` is added as a public constant, registered in `DIAGNOSTIC_REGISTRY` with `owner: "sifr_type_system"`, and included in `ACTIVE_DIAGNOSTIC_CODES` — all correct.

**Documentation**

- `docs/errors/SIFR-INT-0007.md` is generated content, appropriately marked as auto-generated.
- `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` indexes are updated with the new entry in INT-family order — correct.

### Test Coverage

| Surface | Coverage |
|---|---|
| Type system unit (`test_bool_integer_comparison_blocked_with_int_diagnostic`) | 4 bool/integer pairs: `Bool==Int`, `LiteralBool!=LiteralInt`, `FixedInt(U8)<Bool`, `BigInt>=LiteralBool` — covers both equality and ordering, all 4 integer shapes |
| HIR unit (`test_bool_integer_equality_has_int0007`) | `True == 1` — bool literal vs. int literal equality |
| HIR unit (`test_bool_fixed_width_ordering_has_int0007`) | `uint8 < bool` — fixed-width ordering |
| E2E fail fixture | `True == 1` with `# expect-error: SIFR-INT-0007` — single representative case |

The type system test covers `LiteralBool` vs. `LiteralInt` but not `Type::Int` vs. `Type::Bool` directly. The HIR test `True == 1` exercises `LiteralBool(true) == LiteralInt(1)` which is covered. A direct `Type::Int < Type::Bool` or `Type::Int == Type::Bool` type-system test would be more explicit but is non-blocking.

## Required Changes

**None.** The implementation is correct and the full quick validation (106.59s, report signature `e1bf653aaa770517`) passes.

## Non-blocking Notes

1. **Type system test gap**: `Type::Int == Type::Bool` and `Type::Int < Type::Bool` are not explicitly covered in the type system unit test. They are exercised indirectly through the HIR test (`True == 1` infers to `LiteralBool`/`LiteralInt`), and the full quick validation confirms end-to-end correctness. A future pass could add a direct pair, but it is not required for this milestone.

2. **E2E fixture minimalism**: The e2e fail fixture covers only `True == 1`. Ordering e2e coverage (`True < 1`) is absent from the fixture but is validated by the HIR unit test. Given that the harness enforces `expect-error` columns per-line and the unit tests are authoritative for HIR lowering behavior, this is acceptable.

3. **`FixedInt(U64)` and `FixedInt(USize)` in `is_integer_type`**: These variants are included in the bool/integer check even though `fixed_width_promotes_to_current_int` blocks them from the ordinary arithmetic promotion path. This means `uint64_value == True` emits INT-0007. Per the integer model, all bool/integer comparisons should be explicit regardless of fixed-width variant, so this behavior is correct by design.

## Verdict

The PR correctly implements `SIFR-INT-0007` for direct bool/integer equality and ordering comparisons across exact `int`, `bigint` (transition alias), integer literals, and all fixed-width integer families. `bool`/`bool` comparisons and unrelated type comparisons are preserved. The diagnostic is registered, documented, and covered by unit and e2e tests.

**Approved for this milestone.**
