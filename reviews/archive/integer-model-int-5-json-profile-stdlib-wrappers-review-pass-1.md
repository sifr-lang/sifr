

I've completed a thorough review of the INT-5 stdlib JSON integer profile wrapper slice. Here are my findings:

## Review Summary

**No blockers found.** The reviewer is satisfied for this INT-5 stdlib JSON profile wrapper slice.

### Review Details

**1. Policy delegation to sifr_runtime::json (no duplication)**
- `dumps_exact`, `dumps_web`, `dumps_string_ints` in `lib/sifr/json.sifr` are thin wrappers that delegate to the runtime intrinsics (`json_dumps_value_exact`, `json_dumps_value_web`, `json_dumps_value_string_ints`).
- The actual policy lives in `sifr_runtime/src/json.rs` via `encode_integer_for_profile()` and `JsonIntegerProfile` variants. No duplication outside the runtime.

**2. dumps_web recursive rejection with useful error context**
- The test in `json_integer_profiles.sifr` confirms `dumps_web` rejects 9007199254740992 (outside JS safe range) and surfaces `JsonIntegerRangeError` with `profile == "json.web"` and `path == "$.items[1]"`.
- The path construction via `child_array_path_expr()` and `child_object_path_expr()` in `intrinsics/json.rs` correctly builds JSON pointer notation.

**3. dumps_string_ints encodes integers as decimal strings, dumps_exact preserves numeric JSON**
- `dumps_string_ints` emits `"9007199254740992"` as a string (verified in test).
- `dumps_exact` emits `9007199254740991` as a JSON number, preserving current JsonValue int storage behavior.

**4. HIR intrinsic types, codegen, and error refs completeness**
- `intrinsics/mod.rs` registers all three new intrinsics with correct signatures including `JsonIntegerRangeError` type definition.
- `additional_required_crates()` in `intrinsics/mod.rs` correctly declares `sifr_runtime` for the profile intrinsics.
- `error_refs.rs` already includes `JsonIntegerRangeError` in the builtin error list that gets emitted when intrinsics are used.
- `stdlib_filter.rs` has `JsonIntegerRangeError` in `GLOBAL_INFRA_TYPES`.

**5. Test sufficiency**
- The e2e test in `json_integer_profiles.sifr` covers:
  - Happy path (JS-safe integers with `dumps_web`)
  - Sad path (JS-unsafe integer rejection with correct error fields)
  - `dumps_string_ints` encoding behavior
  - `dumps_exact` numeric preservation
- Existing tests (`stdlib_json_consolidated.sifr`, `cpython_json.sifr`) still pass, confirming no regression.

**Validation already run confirms:**
- `cargo fmt --check` passes
- `cargo check -p sifr_codegen` passes  
- E2E tests for json_integer_profiles, stdlib_json_consolidated, and cpython_json all pass

The slice is ready for PR.
