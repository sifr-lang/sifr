## Add Comprehensive Tests for M3 Type System Features

#### **Current Situation**

- E2E tests exist in `crates/sifr/tests/e2e/pass/` (23 tests) and `e2e/fail/` (5 tests) covering M1 and M2 features.
- No tests exist for union types, literal types, type narrowing, optional handling, Unknown type, or type aliases.
- The test harness (`e2e.rs`) supports `# expect-stdout:` for pass tests and `# expect-error:` for fail tests.

#### **Desired Situation**

- Comprehensive E2E pass tests covering all M3 features:
  - `union_basic.sifr`: Basic union type parameter and return
  - `union_multi.sifr`: 3+ type unions
  - `literal_types.sifr`: Literal string/int/bool types with type aliases
  - `optional_narrowing.sifr`: `str | None` with `is not None` check
  - `isinstance_narrowing.sifr`: isinstance-based narrowing in if/else
  - `equality_narrowing.sifr`: Equality-based narrowing for literals
  - `truthiness_narrowing.sifr`: Truthiness-based narrowing
  - `type_alias.sifr`: `type X = ...` statement usage
  - `unknown_type.sifr`: Unknown with isinstance narrowing
  - `reveal_type.sifr`: reveal_type() output verification
- Comprehensive E2E fail tests:
  - `union_no_narrowing.sifr`: Using union member operation without narrowing
  - `unknown_no_narrowing.sifr`: Using Unknown without narrowing
  - `literal_mismatch.sifr`: Assigning wrong literal to literal type
  - `union_type_mismatch.sifr`: Assigning incompatible type to union
  - `non_exhaustive.sifr`: Missing union variant in narrowing

#### **Suggested Solution**

**New files:**
- `crates/sifr/tests/e2e/pass/union_basic.sifr`
- `crates/sifr/tests/e2e/pass/union_multi.sifr`
- `crates/sifr/tests/e2e/pass/literal_types.sifr`
- `crates/sifr/tests/e2e/pass/optional_narrowing.sifr`
- `crates/sifr/tests/e2e/pass/isinstance_narrowing.sifr`
- `crates/sifr/tests/e2e/pass/equality_narrowing.sifr`
- `crates/sifr/tests/e2e/pass/truthiness_narrowing.sifr`
- `crates/sifr/tests/e2e/pass/type_alias.sifr`
- `crates/sifr/tests/e2e/pass/unknown_type.sifr`
- `crates/sifr/tests/e2e/pass/reveal_type.sifr`
- `crates/sifr/tests/e2e/fail/union_no_narrowing.sifr`
- `crates/sifr/tests/e2e/fail/unknown_no_narrowing.sifr`
- `crates/sifr/tests/e2e/fail/literal_mismatch.sifr`
- `crates/sifr/tests/e2e/fail/union_type_mismatch.sifr`
- `crates/sifr/tests/e2e/fail/non_exhaustive.sifr`

Each test file includes `# expect-stdout:` or `# expect-error:` comments for automated verification.

**Modified files:**
- `crates/sifr/tests/e2e.rs`: May need updates if new test patterns are needed (e.g., `# expect-diagnostic:` for reveal_type).

All existing M1/M2 tests must continue to pass (regression safety).
