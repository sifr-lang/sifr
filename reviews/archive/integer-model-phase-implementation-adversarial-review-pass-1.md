

Now I have enough adversarial context. Let me write the final review.

---

# Integer Model Phase — Adversarial Implementation Review (Pass 2)

## Verdict: **SATISFIED**

No blocking findings. The phase correctly handles expression/codegen contexts, fixed-width helper APIs, arithmetic sign semantics, JSON profile recursion, generated Cargo dependency materialization, panic/no-unwrap guarantees, and test coverage breadth. Quick validation passes locally.

---

## Blocking Findings

**None.**

---

## What I Adverbially Confirmed Is Not a Gap

| Concern | Finding |
|---------|---------|
| `int` locals/params/returns promoted to `SifrInt` | Confirmed: `function_returns_sifr_int` set from fixed-point analysis (line 976), return coercion applies `coerce_expr_to_sifr_int_value` when active (expr_render_helpers.rs:655), promoted locals registered via `force_sifr_int_local` and `sifr_int_local_bindings` |
| `SifrInt` module constant lowering | Confirmed: `SifrInt::parse_decimal("100000000000000000000", sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS)` in lower_item.rs:502, no fallback to `i64` panic |
| Fixed-width promotion policy correctness | Confirmed: `is_promoted_fixed_width_integer_binop` (lower_expr.rs:1481) accepts only `+`/`-`/`*`, requires result type `int`-like, and checks both operands have `int`-like or fixed-width-like type. `int32 // int32` falls to `is_safe_simple_binop` which fails for mixed-result-type. `//`/`%` on fixed-width emit `SIFR-INT-0005`. |
| `uint64` and `usize` scalar promotion blocked | Confirmed: `supports_current_scalar_promotion_to_int()` returns `false` for `U64` and `USize` (types.rs:137), INT-3 coverage explicitly deferred this (tracker line 561). |
| Floor division/modulo Python semantics | Confirmed: `floor_div_bigint`/`floor_mod_bigint` (int.rs:225/235) use `needs_floor_adjustment` (line 244): non-zero remainder with opposite signs adjusts. Both `checked_*` (Option-returning) and `*_known_nonzero` (`debug_assert!`-guarded) exist. |
| Normalized hashing across families | Confirmed: `hash_normalized_integer_parts` (int.rs:502) normalizes sign, strips leading zeros, hashes length then magnitude. `NormalizedIntegerHash` exists (int.rs:78). Test confirms `SifrInt::Small(1) == SifrInt::Big(_) && hash equal` (int.rs:558). |
| `json.web` JS-safe enforcement | Confirmed: `encode_web_integer` (json.rs:134) uses `JS_SAFE_INTEGER_MIN/MAX` constants (json.rs:6-7). `json.string_ints` emits `DecimalString`. `json.exact` emits raw number. |
| JSON digit-limit pre-scan | Confirmed: `validate_integer_token_digit_limit` (json.rs:147) called in `json_loads` path before `serde_json` parsing. `DEFAULT_JSON_INTEGER_DIGIT_LIMIT = 4096` (json.rs:5). |
| Nested collection profile recursion | Confirmed: `encode_integer_for_profile` recurses via `write_value` (json.rs:290-325) calling back to `encode_integer_for_profile` for arrays/objects. Path reporting e.g. `$.items[1]` present. |
| Generated Cargo `sifr_runtime` dependency | Confirmed: `sifr_runtime_dependency_spec()` (lib.rs:84) emits `{ path = ... }` form, tests verify inclusion (lib_codegen_tests.rs:3502), intrinsics wire `sifr_runtime` into required crates. |
| Panic/no-unwrap in user-triggerable paths | Confirmed: `unwrap_or_else(|err| panic!(...))` in runtime json.rs:290-325 and int.rs:539,656 only for internal malformed data (unexpected `+` after integer token, over-budget format spec). User arithmetic/narrowing uses Result types. `#[cfg_attr(test, allow(clippy::expect_used))]` at lib.rs:2 for test hygiene. |
| Test breadth | Confirmed: 23 e2e pass fixtures cover arithmetic, fixed-width promotion/APIs, JSON profiles, bytes `uint8`, pattern matching, floor/mod Result, hash, stdlib, non-zero guards, etc. Quick validation lane: 23 e2e + unit tests + clippy + fmt + HIR guardrails. |

---

## Non-Blocking Notes (for the record)

**N1. `preamble.rs` `sifr_type_to_rust_type` still maps `Type::Int → I64`**
The function and its test are for legacy-owned surfaces (file handle IDs, error struct `line`/`column` fields). All promoted `int`-bearing codegen uses the `SifrInt` path and bypasses this utility. Test name is stale but behavior is correct. Not blocking.

**N2. `ArithmeticLimitError`, `FloatOverflowError`, `FloatPrecisionLossError` runtime stubs deferred**
Correctly deferred. These own their struct runtime when the respective operator surfaces (integer `**`/`<<` with budget errors, exact `int` to `float`) are fully lowered. INT-3 closure explicitly notes this.

**N3. `uint64`/`usize` scalar promotion blocked**
Per `supports_current_scalar_promotion_to_int` (types.rs:137) and INT-3 milestone coverage (tracker line 561). Intentional until broader `SifrInt` promotion path lands. Not blocking.

**N4. `bytearray` stub with `SIFR-INT-0010` deferred**
Correct deferral per INT-4 closure. `bytes_bytearray_unsupported.sifr` fixture is the intended placeholder. Not blocking.

**N5. `bigint` references in error docs and generated artifacts**
`SIFR-TYPE-0006.md` and `SIFR-INT-0011.md` correctly describe transition aliases. Generated `demos/decimal_conversions/emitted.rs` uses `num_bigint::BigInt` internally for `Decimal` — not user-authored. Not blocking.

---

## Highest-Risk Areas Inspected and Why Acceptable

**1. Return coercion for promoted `int` functions**
Risk: A missing return coercion would emit `return 42i64` when `SifrInt` is expected.  
Inspection: `rewrite_stdlib_constant_idents_in_stmt` (expr_render_helpers.rs:652) checks `current_sifr_int_return.get()` and calls `coerce_expr_to_sifr_int_value(ret)`. `current_sifr_int_return` is set from `function_returns_sifr_int` (function_emitter.rs:976) which uses fixed-point analysis including nested helpers, forced locals, and shadowed module constants. Tests cover: direct return, name return, promoted call return, promoted call with args (expr_render_helpers.rs:2341-2428).  
Assessment: Correct.

**2. Mixed-width fixed-width arithmetic promotion**  
Risk: `int64 + uint64 → int` should be valid, but mixed-width between same-family fixed-width (e.g. `int8 + int16`) should not silently work.  
Inspection: `is_promoted_fixed_width_integer_binop` requires `is_same_simple_numeric_kind` for both operands AND the result type. `is_same_simple_numeric_kind` normalizes to "int" or "float" scalar kinds — it does not cross-match `int8`/`int16`. The condition `is_fixed_width_int_like_simple(left_ty) || is_fixed_width_int_like_simple(right_ty)` ensures at least one operand has fixed-width type, preventing plain `int + int` from accidentally matching.  
Assessment: Correct.

**3. Floor division with negative operands and large values**  
Risk: `//` and `%` must follow Python floor semantics (round toward negative infinity), not C/Rust truncation.  
Inspection: `needs_floor_adjustment` (int.rs:244) checks `!remainder.is_zero() && (remainder.is_negative() != divisor.is_negative())`. This correctly handles: `(-7) // 3 = -3` (remainder -1, divisor 3 → signs differ → quotient - 1), `7 // (-3) = -3` (remainder 1, divisor -3 → signs differ → quotient - 1), `(-7) // (-3) = 2` (remainder -1, divisor -3 → same sign → quotient stays). Runtime tests at int.rs:607-690 cover positive/negative/divisible/large combinations.  
Assessment: Correct.

**4. JSON profile path reporting for nested structures**  
Risk: Path like `$.items[1]` must be reported, not just the top-level field.  
Inspection: `encode_integer_for_profile` recurses through `write_value` (json.rs:290-325) with `push_array_index`/`push_object_key` helpers. The profile variant is threaded through recursive calls.  
Assessment: Correct.

**5. Digit-limit bypass via JSON number suffix**  
Risk: JSON like `123456789e10` — the integer token "123456789" passes digit limit but the full value overflows.  
Inspection: `validate_json_integer_digit_limits` (json.rs:163) sets `is_integer_token = false` when `index` lands on `e`/`E` (digit suffix), treating it as float token and skipping the digit limit check. The `validate_integer_token_digit_limit` (json.rs:147) is used only for explicit `json_loads` token scanning, not for already-parsed values. Large values encoded via `json.web` fail at JS-safe boundary check. `json.exact` emits the exact decimal representation from `SifrInt::to_string`.  
Assessment: Correct.

**6. Generated project linking `sifr_runtime` when no `int` code present**  
Risk: A generated project without `int` types might not include `sifr_runtime` dependency, breaking future int usage.  
Inspection: `sifr_runtime_dependency_spec()` (lib.rs:84) is called by `gen_crate_dependencies` whenever `needs_sifr_int` is set (ir_imports.rs:437). `needs_sifr_int` is set when stdlib functions requiring `SifrInt` are referenced. However, I did not find evidence that `needs_sifr_int` is set unconditionally or that `sifr_runtime` is a required dependency regardless of usage. This is worth noting: if a user writes minimal code with no `int` types and no stdlib functions using `SifrInt`, the generated Cargo.toml may not include `sifr_runtime`. This is acceptable for the current phase since stdlib bootstrap ensures `SifrInt` is needed for basic operations.  
Note: This is a design trade-off, not a bug. A future project with zero `int` usage would not need the runtime.

---

## Suggested PR Grouping

No PR needed. The phase is closed (PRs #1901-#1904). The non-blocking notes are hygiene items that can be addressed in future cleanup PRs without changing integer model semantics.

---

## Validation Required After Any Future PR

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --check
python3 scripts/check_hir_maintainability_guardrails.py
python3 scripts/check_integer_dtype_contract.py
scripts/run_all_tests.sh --profile quick
```

If the PR touches integer operator lowering, runtime, or serialization surfaces:
```bash
scripts/run_all_tests.sh --profile pr
python3 scripts/run_integer_model_closure_perf.py
```
