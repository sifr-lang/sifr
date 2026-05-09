

Review complete. The implementation is sound. No blockers found.

---

## INT-5 JSON Load Digit-Limit Slice Review

### Verdict: Satisfied

The slice correctly implements the INT-5 scope: parser digit limits and typed errors for untrusted JSON integer tokens.

---

### What was reviewed

| File | Changes |
|------|---------|
| `sifr_runtime/src/json.rs` | `validate_json_integer_digit_limits` scanner, `JsonLimitError` type |
| `sifr_codegen/src/intrinsics/json.rs` | `lower_json_loads`, `lower_json_validate_integer_digit_limits` |
| `sifr_codegen/src/intrinsics/mod.rs` | Registry entries with `sifr_runtime` in `additional_required_crates` |
| `sifr_hir/src/stdlib/io_json.rs` | `validate_integer_digit_limits` wrapper, `JsonLimitError` type definition |
| `lib/sifr/json.sifr` | `validate_integer_digit_limits` public API |
| `crates/sifr/tests/e2e/pass/stdlib_json_consolidated.sifr` | Quick-lane fixture coverage |

---

### Implementation correctness

**Scanner** (`validate_json_integer_digit_limits`, json.rs:163-207)
- Correctly skips strings before scanning number tokens
- `is_json_number_start_context` correctly identifies `[{` or `,` prefixes
- Integer-token termination is correct: `,`, `]`, `}`, or whitespace
- Fraction/exponent tokens delegate to `skip_json_number_suffix`
- `digit_count` computed from unsigned portion (sign not counted)

**Codegen wiring** (json.rs:463-754)
- `json_loads` calls `validate_json_integer_digit_limits` *before* `serde_json::from_str`
- `JsonLimitError` is converted to `JSONDecodeError` via `__sifr_json_limit_error_as_decode_error`
- `json_validate_integer_digit_limits` returns `JsonLimitError` directly
- Both intrinsics declare `sifr_runtime` in `additional_required_crates`

**Type system** (mod.rs:40-50, 114-121)
- `JsonLimitError` has `message: str`, `limit: int`, parent `Error`
- `json_validate_integer_digit_limits` has `str -> Result[None, JsonLimitError]`

**Boundary contract alignment**
- `DEFAULT_JSON_INTEGER_DIGIT_LIMIT = DEFAULT_MAX_INTEGER_DIGITS = 4096` ✓
- `JsonLimitError` for digit budget violations ✓
- `JsonIntegerRangeError` for profile-enforced range violations ✓
- Doc contract: "untrusted integer token budget violations return `JsonLimitError`" ✓

---

### Edge cases noted (acceptable)

**`+`-prefixed numbers** (json.rs:177): The scanner counts `+` as a digit character and would not terminate the integer token at `+`. However, `+` is not valid JSON, so `serde_json` will reject it. The error still surfaces (as `JSONDecodeError` from the parse phase), just not with the digit-limit message. This is acceptable for the INT-5 scope.

**Leading zeros**: `01` has `digit_count = 2`. This is correct behavior — the scanner enforces digit budget, not JSON spec compliance. `serde_json` handles the parse error.

---

### Test coverage

| Test | Purpose |
|------|---------|
| `json_digit_limit_rejects_integer_tokens_outside_strings` | Over-budget int outside string ✓ |
| `json_digit_limit_ignores_string_digits_and_fractional_numbers` | Strings/exponents not counted ✓ |
| `json_digit_limit_checks_nested_array_numbers` | Nested structures ✓ |
| `integer_token_limit_ignores_sign_and_enforces_digits` | Sign not counted ✓ |
| `integer_token_limit_rejects_non_integer_tokens` | `12.0` rejected ✓ |
| `e2e: stdlib_json_consolidated.sifr` | Quick-lane fixture ✓ |

---

### Pre-existing issues (unrelated to this slice)

Clippy errors in `function_emitter.rs:1309,1325,1517` — `filter_map_bool_then` — exist in `sifr_codegen` but are not introduced by this slice.

---

### No blockers. Slice is acceptable as one INT-5 milestone PR.
