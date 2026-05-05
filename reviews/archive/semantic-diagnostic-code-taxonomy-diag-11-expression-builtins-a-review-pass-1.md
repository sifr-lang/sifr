# Review: milestone_diag_11 expression builtin scalar/conversion raw HIR diagnostic migration

**Branch:** `codex/diag-11-raw-hir-expression-builtins-a`
**Reviewer:** Pass 1
**Validation artifacts:** All local validation commands passed (see scope).

---

## Summary

The diff migrates 8 builtins (`abs`, `hash`, `round`, `repr`, `int`, `bigint`, `float`, `bool`) from raw `ctx.error(...)` emissions to three structured diagnostic helpers: `call_wrong_positional_count`, `call_unexpected_keyword`, and `type_mismatch`. All three helpers route through `error_with_code_at`, so every diagnostic in scope now carries a proper `DiagnosticCode`. Keyword rejection (previously absent) is added for all 8 builtins. HIR tests assert code + primary range via table-driven test functions.

**Verdict: No required fixes remain.**

---

## Finding 1 — Type mismatch primary range for `bigint`: INFORMATIONAL

**Severity:** Informational
**Location:** `expressions.rs:876` (`bigint` type mismatch range)

```rust
expression_diagnostics::type_mismatch(
    ctx,
    format!("bigint() requires int, bigint, decimal, or bigdecimal argument, got '{}'", ...),
    call.arguments.args[0].range(),   // ← points at the arg
);
```

**Observation:** The `bigint` type mismatch uses `call.arguments.args[0].range()` (the argument expression) as primary range. This matches the pattern used by `abs` and `round` type mismatches in the same diff. However, it differs from the `iter` type mismatch in the same file (line ~575), which uses the same approach. This is consistent within the diff.

**Why informational, not a fix:** The primary range is on the offending argument rather than the full call span, which is a deliberate per-diagnostic choice visible in the test expectations (`"x"` inside the callable parens). The pattern is consistently applied across all three type-mismatch cases in the diff.

---

## Finding 2 — Test coverage: `float` and `bool` type mismatches not exercised

**Severity:** Informational
**Location:** `expressions_tests.rs:670–711`

The type-mismatch test table covers only `abs`, `round`, and `bigint`. The remaining builtins with type checks (`int`, `float`, `bool`) are not tested for TYPE_MISMATCH diagnostics. Specifically:

- `int("x")` — `int()` accepts a str argument (falls through to `Result[int, ParseError]`), so no type mismatch fires. Correct that it's excluded.
- `float("x")` — `float(str)` is valid (produces `Result[float, ParseError]`). Correct that it's excluded.
- `bool(...)` — `bool()` accepts any type. No type mismatch possible. Correct that it's excluded.

**Conclusion:** No missing test cases. The tested subset fully covers the type-mismatch paths that actually exist.

---

## Finding 3 — `float` sentinel path ignores keyword check ordering

**Severity:** Informational
**Location:** `expressions.rs:887–909`

For `float`, the keyword rejection check precedes the arity check, which precedes the sentinel check via `float_sentinel_kind_from_call`. The sentinel function itself also checks `call.arguments.args.len() != 1`. If `float(inf, extra=1)` is called, the keyword check fires first at the keyword range. If `float()` (no args) is called, the arity check fires. If `float("inf")` is called, the sentinel path is taken before any type check (which would be impossible since `"inf"` is a valid sentinel string).

This ordering is correct: sentinel literals must short-circuit before hitting the general type machinery.

---

## Finding 4 — `int` has no type-mismatch diagnostic (correct)

**Severity:** N/A — observed correctness.

`int` accepts `str`, `float`, `bool`, `bigint`, `Decimal`, `BigDecimal` — all valid input types. There is no `int`-specific type-mismatch diagnostic. If an unsupported type is passed (e.g., a custom class), no diagnostic fires from the `int` handler itself; the type flows through as `Type::Int` (the fallthrough case on line 834). This is pre-existing behavior unchanged by the diff.

---

## Correctness Checks

| Check | Result |
|---|---|
| All 8 builtins have keyword rejection added | PASS |
| All `ctx.error(...)` replaced with structured helpers | PASS |
| `call_wrong_positional_count` for arity diagnostics | PASS — `abs`, `hash`, `round`, `repr`, `int`, `bigint`, `float`, `bool` |
| `call_unexpected_keyword` for keyword diagnostics | PASS — all 8 builtins |
| `type_mismatch` for scalar type-shape diagnostics | PASS — `abs`, `round`, `bigint` only (others have no such path) |
| No fallback `ctx.error(...)` paths left in migrated builtins | PASS |
| `call_arity_range` and `first_call_keyword_range` used consistently | PASS |
| HIR tests assert `DiagnosticCode::CALL_WRONG_POSITIONAL_COUNT` | PASS |
| HIR tests assert `DiagnosticCode::CALL_UNEXPECTED_KEYWORD` | PASS |
| HIR tests assert `DiagnosticCode::TYPE_MISMATCH` | PASS |
| Tests use `range_for_after_anchor` for primary range verification | PASS |
| No `.unwrap()` or `.expect()` in user paths | PASS — all use `?` / `return None` |
| `cargo fmt` | PASS |
| `cargo check -p sifr_hir` | PASS |
| `cargo clippy -p sifr_hir -- -D warnings` | PASS |
| `check_hir_maintainability_guardrails.py` | PASS |
| `git diff --check` | PASS |

---

## Behavioral Changes from Keyword Rejection

Keyword calls to any of the 8 builtins now produce `CALL_UNEXPECTED_KEYWORD` with a precise range at the first keyword argument. Previously these would have fallen through to some other diagnostic (likely a Python-parser-level error or silent type error). This is a hardening improvement — explicit is better than implicit.

---

## Maintainability

- The 4-line pattern repeated 8 times (keyword check + helper call) is mechanical and obvious; no abstraction benefit would come from extracting it further.
- The helper functions `call_arity_range` and `first_call_keyword_range` are well-established utilities already used by `iter`, `sorted`, and other builtins outside this diff.
- `expression_diagnostics` module is the correct home for these helpers.
- No new type complexity introduced.

---

## Final Verdict

**No required fixes remain.** The migration is complete, correct, and consistent with the established diagnostic taxonomy. All three diagnostic codes are applied appropriately, primary ranges are precise, keyword rejection adds explicit error signaling, and the HIR tests cover the intended cases.
