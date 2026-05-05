# Review: milestone_diag_11 expression iterator builtin diagnostics

**Branch:** `codex/diag-11-raw-hir-expression-iter-builtins`
**Scope:** `builtin_calls.rs`, `expression_iter_builtins.rs`, `expressions.rs`, `expressions_tests.rs`, `mod.rs`
**Review goals:** Verify reversed/enumerate raw HIR diagnostics were migrated to structured code/range transport with no fallback behavior; verify helper returning reversible element type is correct; verify primary ranges and DiagnosticCode variants are semantically appropriate; verify maintainability/module split and tests are sufficient.

---

## Verdict: CLEAN — No required fixes

All four review goals are satisfied. The implementation is correct, consistent with established patterns, and well-tested.

---

## 1. Raw HIR diagnostics — migrated with structured code/range transport, no fallback behavior

### `reversed()` — `builtin_calls.rs:1119-1164`

All errors use `ctx.error_with_code_at(DiagnosticCode::..., ..., range)` with `primary_range` pointing at the offending argument token:

- **Wrong arity/keywords** (line 1127): `CALL_WRONG_POSITIONAL_COUNT` at `arity_range(call)` (last arg or function name).
- **Unknown element type** (line 1139): `PROTO_INVALID_ITERATOR_SIGNATURE` at `call.arguments.args[0].range()`.
- **Not reversible** (line 1153): `PROTO_BOUND_NOT_SATISFIED` at `call.arguments.args[0].range()`.

No raw `ctx.error(...)` fallback remains.

### `enumerate()` — `expression_iter_builtins.rs:27-124`

All errors use structured `error_with_code_at`:

- **Wrong arity** (line 29): `CALL_WRONG_POSITIONAL_COUNT` at `call_arity_range(call)`.
- **Unexpected keyword** (lines 40, 48): `CALL_UNEXPECTED_KEYWORD` at keyword range.
- **Duplicate start** (line 56): `CALL_DUPLICATE_ARGUMENT` at `name.range()`.
- **Element type unknown** (line 68): `TYPE_MISMATCH` at `call.arguments.args[0].range()`.
- **Start type (positional)** (line 83): `TYPE_MISMATCH` at `start_expr.range()`.
- **Start type (keyword)** (line 97): `TYPE_MISMATCH` at `keyword.value.range()`.

No raw `ctx.error(...)` fallback.

---

## 2. Helper returning reversible element type — correct, does not hide user-facing diagnostics

**`callable_builtin_element_type`** (`builtin_calls.rs:1108-1110`) delegates to `iterable_element_type_for_builtin` (`builtin_calls.rs:99-107`):

```rust
fn iterable_element_type_for_builtin(arg_ty: &Type) -> Option<Type> {
    arg_ty.iterable_element_type().or_else(|| {
        if matches!(arg_ty.resolve_alias(), Type::Any | Type::Unknown) {
            Some(Type::Any)
        } else {
            None  // <-- None for types with no iteration element type
        }
    })
}
```

For a non-iterable concrete type like `int`, this returns `None` rather than falling back to `Any`. The caller (`lower_builtin_reverseable_arg` for reversed; `lower_enumerate_call` for enumerate) then emits the appropriate user-facing diagnostic with the actual type name in the message — e.g., `"reversed() argument must be an iterable with a statically-known element type, got 'int'"`. This is correct: `Any` would suppress the concrete type information and is correctly reserved for when the type is genuinely indeterminate (`Any`/`Unknown`).

---

## 3. Primary ranges and DiagnosticCode variants — semantically appropriate

### `reversed(1)` (proto invalid signature)

- **Code:** `PROTO_INVALID_ITERATOR_SIGNATURE` — correct for "statically-known element type" violation.
- **Range:** points at `1` — the argument expression. This is the correct anchor for "argument must be an iterable…".

### `enumerate(1)` (element type unknown)

- **Code:** `TYPE_MISMATCH` — see note below.
- **Range:** points at `1` — the argument expression.

**Note on code choice asymmetry:** `enumerate(1)` uses `TYPE_MISMATCH` while `reversed(1)` uses `PROTO_INVALID_ITERATOR_SIGNATURE` for the same "statically-known element type" failure. This is tested and intentional (`test_reversed_and_enumerate_argument_errors_have_codes`, lines 2347–2369). `reversed` goes through `lower_builtin_reverseable_arg` which uses `PROTO_INVALID_ITERATOR_SIGNATURE`. `enumerate` handles the check inline and uses `expression_diagnostics::type_mismatch` → `TYPE_MISMATCH`. The asymmetry is consistent with how the two call sites are structured, and both codes are semantically reasonable. No change required.

### `reversed(it)` where `it: Iterator[int]` (proto bound not satisfied — not reversible)

- **Code:** `PROTO_BOUND_NOT_SATISFIED` — correct semantic match: "argument must be reversible" is a protocol bound check.
- **Range:** points at `it` — the argument expression.

### `enumerate(nums, bogus=1)` (unexpected keyword)

- **Code:** `CALL_UNEXPECTED_KEYWORD` — correct.
- **Range:** points at `bogus=1` — the keyword token anchor. Confirmed by `test_range_and_enumerate_unexpected_keywords_have_call_code` (lines 2280–2308).

### `enumerate(nums, 10, start=1)` (duplicate start keyword)

- **Code:** `CALL_DUPLICATE_ARGUMENT` — correct for "multiple values for argument 'start'".
- **Range:** points at `start` (the keyword name) — correct anchor for the duplicate.

### `enumerate(nums, "bad")` (start type mismatch — positional)

- **Code:** `TYPE_MISMATCH` — correct for type restriction.
- **Range:** points at `"bad"` — correct.

### `enumerate(nums, **kwargs)` (unpacked keyword)

- **Code:** `CALL_UNEXPECTED_KEYWORD` — correct.
- **Range:** points at `**kwargs` via `first_keyword_range(call)`.

---

## 4. Maintainability: module split

**Module boundaries:**
- `expression_iter_builtins.rs`: `lower_reversed_call` and `lower_enumerate_call` (the two expression-level builtin lowering entry points). Also contains `reject_zip_keywords_if_present` (used by expressions.rs for zip lowering).
- `builtin_calls.rs`: Shared helpers `callable_builtin_element_type`, `lower_builtin_reverseable_arg`, `callable_builtin_list_output_type`, `callable_builtin_dict_output_type`, plus all other stdlib builtin call lowering (list/tuple/dict/set constructors, len, isinstance, range, etc.).
- `expressions.rs:1115-1123`: Dispatches to `lower_reversed_call` / `lower_enumerate_call` by function name.

The split is clean: expression-level call lowering is in `expression_iter_builtins.rs`, shared utilities are in `builtin_calls.rs`. No circular dependencies. The `expression_iter_builtins.rs` module is focused (two public entry points + one shared helper function `reject_zip_keywords_if_present`). This matches the pattern established by prior diagnostic migration slices.

---

## 5. Tests — sufficient for this slice

The test file `expressions_tests.rs` covers all diagnostic paths for reversed/enumerate:

| Test | Lines | What it covers |
|---|---|---|
| `test_reversed_enumerate_zip_are_typed_as_iterators` | 2213–2248 | Happy path — correct type inference |
| `test_range_and_enumerate_unexpected_keywords_have_call_code` | 2280–2308 | Unexpected keyword for enumerate |
| `test_enumerate_duplicate_start_keyword_has_call_code` | 2310–2330 | Duplicate start keyword |
| `test_reversed_rejects_non_reversible_iterator_argument` | 2332–2344 | `PROTO_BOUND_NOT_SATISFIED` for non-reversible |
| `test_reversed_and_enumerate_argument_errors_have_codes` | 2346–2369 | `PROTO_INVALID_ITERATOR_SIGNATURE` for reversed, `TYPE_MISMATCH` for enumerate (same symptom, different codes per design) |
| `test_enumerate_start_type_errors_have_codes` | 2371–2404 | Start arg type errors (positional + keyword) |
| `test_enumerate_arity_and_unpacked_keyword_errors_have_codes` | 2406–2439 | Arity and unpacked keyword errors |
| `test_reversible_annotation_accepts_list_and_rejects_set` | 2441–2451+ | Protocol-bound `Reversible[T]` with list (ok) and set (not reversible) |

All tests verify:
- Error code (`error.code == Some(DiagnosticCode::...)`)
- Error message (exact string match)
- `primary_range` (via `range_for_after_anchor` helper)

This is a comprehensive test matrix for the slice.

---

## Prior validation confirmed

Local validation passed before this review (per user statement):
- `cargo fmt`
- `cargo test -p sifr_hir enumerate -- --nocapture`
- `cargo test -p sifr_hir reversed -- --nocapture`
- `cargo check -p sifr_hir`
- `cargo clippy -p sifr_hir -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- `git diff --check`

All checks green.

---

**Conclusion:** The slice is clean. No fallback behavior, no suppressed diagnostics, correct ranges, appropriate diagnostic codes, clean module separation, and sufficient test coverage. Ready to merge.
