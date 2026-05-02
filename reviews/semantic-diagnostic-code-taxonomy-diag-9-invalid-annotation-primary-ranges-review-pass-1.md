# Review: milestone_diag_9 — invalid type annotation primary ranges

## Files in scope (git diff only)

| File | Change summary |
|------|---------------|
| `crates/sifr_hir/src/lower/typing_and_functions.rs` | `invalid_type_annotation` helper gains a `TextRange` param; all 15 call sites updated to pass a precise range |
| `crates/sifr_hir/src/lower/expressions_tests.rs` | Two existing tests refactored to extract source strings and assert `primary_range`; two new tests added |
| 13× `crates/sifr/tests/e2e/fail/*.sifr` | Each fixture gains a `col=N` anchor on its `expect-error` directive |

---

## 1. `typing_and_functions.rs` — implementation

**`invalid_type_annotation` helper (line 390–398):**
- Signature changed from `ctx.error_with_code(...)` to `ctx.error_with_code_at(..., range)`.
- Every call site now provides a concrete `TextRange` instead of a bare diagnostic.

**Range attribution by error message:**

| Error message | Range passed |合理性 |
|---------------|-------------|--------|
| `integer_literal_too_large` | `num.range()` | correct — the literal token itself |
| `only integer literals are supported` | `num.range()` | correct — same token |
| `unsupported type annotation base` | `sub.value.range()` | correct — the base expression |
| `dict type annotation requires exactly 2 type parameters` | `sub.slice.range()` | correct — the `[K, V, ...]` slice |
| `dict type annotation requires [K, V] syntax` | `sub.slice.range()` | correct |
| `Result type annotation requires exactly 2 type parameters` | `sub.slice.range()` | correct |
| `Result type annotation requires [T, E] syntax` | `sub.slice.range()` | correct |
| `Callable type requires exactly 2 type parameters` | `sub.slice.range()` | correct |
| `Callable parameter types must be a list` | `tuple.elts[0].range()` | correct — the offending non-list element |
| `Callable type requires [[param_types], return_type] syntax` | `sub.slice.range()` | correct |
| generic type alias wrong arity | `sub.slice.range()` | correct |
| `class '...' does not declare type parameters` | `sub.value.range()` | correct — the class base name |
| class wrong arity | `sub.slice.range()` | correct |
| `unsupported type annotation expression` | `expr.range()` | correct — catch-all fallback |

**All 15 call sites covered. No raw/hardcoded fallback paths remain.**

---

## 2. `expressions_tests.rs` — unit tests

### Refactored existing tests

- `test_generic_class_subscript_requires_declared_type_params` — source extracted to local variable, `primary_range` assertion added using `range_for_after_anchor` targeting `"LegacyBox"` after the `def f(x: ` anchor. Asserts the `LegacyBox` name in the non-generic class subscript error.
- `test_generic_class_subscript_arity_mismatch_errors` — same refactor pattern; `primary_range` targets `"int, str"` (the over-arity arguments) in the `Pair[int, str]` subscript.

### New tests added

- **`test_invalid_dict_type_annotation_has_primary_range`** — `dict[int]` fixture; asserts error message `"dict type annotation requires [K, V] syntax"`, code `TYPE_INVALID_ANNOTATION`, and `primary_range` pointing to `"int"` (the lone type argument).
- **`test_callable_param_list_annotation_has_primary_range`** — `Callable[int, str]` fixture; asserts message `"Callable parameter types must be a list: Callable[[int, str], bool]"` and `primary_range` pointing to `"int"` after the `Callable[` anchor.

All four tests use `range_for` / `range_for_after_anchor` helpers and follow the established test structure in the file.

---

## 3. E2E fixtures — `col=` anchors

All 13 fixtures migrated from plain `# expect-error: SIFR-TYPE-0007` to `# expect-error[col=N]: SIFR-TYPE-0007`.

| Fixture | `col=` | Points to |
|---------|--------|-----------|
| `integer_literal_too_large_type_annotation` | 20 | the large literal `999...` |
| `invalid_float_literal_type_annotation` | 20 | `1.5` |
| `invalid_type_annotation_expression` | 20 | `int + str` (first token) |
| `invalid_type_annotation_base` | 20 | `make_type()` (first token of base) |
| `dict_type_annotation_wrong_arity` | 25 | first token of over-arity subscript |
| `result_type_annotation_wrong_arity` | 27 | first token of over-arity subscript |
| `result_type_annotation_wrong_syntax` | 27 | first token of under-arity subscript |
| `callable_type_annotation_param_list_required` | 32 | first invalid type arg in `Callable[int, str]` |
| `callable_type_annotation_wrong_arity` | 32 | first over-arity arg in `Callable[[int], str, bool]` |
| `callable_type_annotation_wrong_syntax` | 32 | first invalid type arg in `Callable[int]` |
| `generic_type_alias_wrong_arity` | 17 | first over-arity type arg |
| `generic_class_wrong_arity` | 17 | first over-arity type arg in `Pair[int, str]` |
| `generic_class_non_generic_subscript` | 12 | class name `LegacyBox` being erroneously subscripted |

**Note on two fixtures:** `result_type_annotation_wrong_arity` (`col=27`) and `callable_type_annotation_wrong_arity` (`col=32`) point to the **first** over-arity type-argument token within the subscript slice, not to the opening `[`. This is a deliberate design choice (the unit tests use the same `"int, str"` partial-span approach). The implementation's `sub.slice.range()` covers the full subscript slice, so the e2e `col=` positions land within the reported span and the assertions pass.

---

## 4. Validation summary

The user confirmed all local validations passed:

- `cargo fmt --check` — clean
- `git diff --check` — clean
- `check_hir_maintainability_guardrails.py` — clean
- `cargo clippy --workspace -- -D warnings` — clean
- Unit tests: `invalid_dict_type_annotation_has_primary_range`, `callable_param_list_annotation_has_primary_range`, `generic_class_subscript*` — all pass
- E2E fail suite: all 13 updated fixtures pass
- `scripts/run_all_tests.sh --profile quick` — wall_time=54.19s, profile=e1bf653aaa770517

---

## Verdict

**Satisfied. No further passes required.**

- The `invalid_type_annotation` helper is correctly refactored to accept and forward a `TextRange`.
- Every call site provides a semantically appropriate range (base name, slice, token, or expression tail).
- Two new unit tests assert `primary_range` for the `dict` and `Callable` param-list paths.
- All 13 e2e fixtures carry precise `col=` anchors; the harness accepts them.
- No fallback/raw diagnostic paths remain in the touched helpers.
