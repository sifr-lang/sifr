

## INT-5 Runtime JSON Profile Slice Review

**Verdict: No blockers. Reviewer is satisfied for this slice.**

### Review Findings

**1. `sifr_runtime::json` — Centralization of JSON Integer Profile Policy**

The `json.rs` module correctly implements the three canonical profiles (`Exact`, `Web`, `StringInts`) with consistent naming via `as_str()` ("json.exact", "json.web", "json.string_ints") and `from_name()` parsing that accepts both short and full forms. The `encode_integer_for_profile` dispatch and `validate_integer_token_digit_limit` helper are appropriately scoped as first runtime primitives.

**2. `json.web` — JavaScript Safe Integer Rejection**

`encode_web_integer` at line 163-176 correctly rejects values outside `JS_SAFE_INTEGER_MIN` (`-9007199254740991`) through `JS_SAFE_INTEGER_MAX` (`9007199254740991`) rather than silently emitting unsafe JSON numbers. The check uses an inclusive range bound: `(JS_SAFE_INTEGER_MIN..=JS_SAFE_INTEGER_MAX).contains(&value)`. Additionally, arbitrary-precision integers that cannot convert to i64 are also rejected via the `value.as_bigint().to_i64()` fallibility path. Tests cover:
- Safe boundary values (min and max) — accepted
- Just-beyond-boundary values — rejected with `JsonIntegerRangeError`
- Large arbitrary-precision values — rejected

**3. `JsonIntegerRangeError` and `JsonLimitError` — Shape and Registration**

Both error types are consistently shaped:
- `JsonIntegerRangeError` has `message`, `path`, `profile` fields, matching the architecture doc requirement for "field path, value range issue, and policy alternatives"
- `JsonLimitError` has `message`, `limit` fields for digit-limit enforcement

Registration is complete across all canonical builtin paths:
- **HIR lowering** (`typing_and_functions.rs`): Both registered as error types with parent `Error`, proper field definitions, and constructor signatures
- **Codegen preamble** (`lib.rs`): Both in `BUILTIN_ERROR_CLASSES`, with field/default emission for generated struct definitions
- **Stdlib filter** (`stdlib_filter.rs`): Both in `GLOBAL_INFRA_TYPES` for DCE suppression

Users and generated code can reference them directly — the e2e test confirms direct instantiation and field access work end-to-end.

**4. Digit-Limit Helper — Appropriateness**

`validate_integer_token_digit_limit` is appropriate as a first runtime primitive for untrusted integer token limits:
- Correctly strips sign prefix before counting
- Rejects non-integer tokens (e.g., "12.0")
- Enforces the configured limit with a typed `JsonLimitError` containing the actual digit count
- Re-exports `DEFAULT_MAX_INTEGER_DIGITS` as `DEFAULT_JSON_INTEGER_DIGIT_LIMIT` (4096 digits per architecture doc)

**5. Test Sufficiency**

- **Runtime unit tests** (7 passing): Profile encoding, safe boundary acceptance, unsafe boundary rejection, arbitrary-precision rejection, digit-limit sign handling, non-integer token rejection
- **E2E test** (`json_integer_error_builtins.sifr`): Verifies end-to-end instantiation and field access of both error types from Sifr source
- **`scripts/run_all_tests.sh --profile quick`**: All 24 e2e pass tests pass

**Pre-existing codegen clippy warnings** in `expr_render_helpers.rs` and `function_emitter.rs` are unrelated to this slice and outside the scope of INT-5.

**No blockers. This slice is ready for PR.**
