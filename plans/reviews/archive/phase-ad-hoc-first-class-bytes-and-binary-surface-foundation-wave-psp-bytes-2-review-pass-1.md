# wave_psp_bytes_2 Implementation Review (Pass 1 - Completion Gap)

**Phase**: `ad-hoc-first-class-bytes-and-binary-surface-foundation`
**Wave**: `wave_psp_bytes_2` (Conversion Surfaces and Compatibility Migration)
**Reviewer**: agent
**Date**: 2026-03-19
**Status**: In Review

---

## Executive Summary

The `wave_psp_bytes_2` implementation introduces conversion surfaces for the first-class `bytes` type, including UTF-8 encode/decode, hex conversion, and `sifr.bytes` compatibility migration. The implementation is **substantially complete** and all core functionality works correctly.

**Key Features Implemented:**
- `bytes(size)` constructor with negative size rejection
- `bytes.from_ints(list[int])` factory with out-of-range validation
- `bytes.from_hex(str)` conversion with hex validation
- `str.encode(encoding?)` with UTF-8-only enforcement
- `bytes.decode(encoding?)` with UTF-8-only enforcement
- Compatibility layer migration in `lib/sifr/bytes.sifr`

**Status**: ✅ APPROVED - No blocking issues identified.

---

## 1. Implementation Correctness

### 1.1 Type System: ✅ CORRECT

| Component | Implementation | Assessment |
|-----------|---------------|------------|
| `Type::Bytes` enum variant | `crates/sifr_type_system/src/types.rs:15` | ✅ Present |
| `bytes(size)` return type | `Result[bytes, ValueError]` | ✅ Correct |
| `bytes.from_ints()` return type | `Result[bytes, ValueError]` | ✅ Correct |
| `bytes.from_hex()` return type | `Result[bytes, ParseError]` | ✅ Correct |
| `str.encode()` return type | `Result[bytes, ParseError]` | ✅ Correct |
| `bytes.decode()` return type | `Result[str, ParseError]` | ✅ Correct |

### 1.2 HIR Lowering: ✅ CORRECT

| Operation | Location | Assessment |
|-----------|----------|------------|
| `bytes(size)` constructor | `crates/sifr_hir/src/lower/builtin_calls.rs:497-531` | ✅ Lowered as `bytes_with_size` intrinsic |
| `bytes.from_hex()` | `crates/sifr_hir/src/lower/builtin_calls.rs:578-598` | ✅ Lowered as `bytes_from_hex` intrinsic |
| `bytes.from_ints()` | `crates/sifr_hir/src/lower/builtin_calls.rs:600-624` | ✅ Lowered as `bytes_from_ints` intrinsic |
| `str.encode()` method | `crates/sifr_hir/src/lower/bytes_methods.rs:17-44` | ✅ Returns `Result[bytes, ParseError]` |
| `bytes.decode()` method | `crates/sifr_hir/src/lower/bytes_methods.rs:122-148` | ✅ Returns `Result[str, ParseError]` |

### 1.3 Codegen Intrinsics: ✅ CORRECT

| Intrinsic | Location | Assessment |
|-----------|----------|------------|
| `lower_bytes_with_size` | `crates/sifr_codegen/src/intrinsics/bytes.rs:566-612` | ✅ Returns `Result<Vec<i64>, ValueError>`, rejects negative |
| `lower_bytes_from_ints` | `crates/sifr_codegen/src/intrinsics/bytes.rs:614-698` | ✅ Validates 0-255 range, rejects out-of-range |
| `lower_bytes_from_hex` | `crates/sifr_codegen/src/intrinsics/bytes.rs:420-564` | ✅ Validates hex chars, handles whitespace |
| `lower_str_encode_utf8_result` | `crates/sifr_codegen/src/intrinsics/bytes.rs:158-164` | ✅ UTF-8 encodes string to bytes |
| `lower_str_encode_utf8_result_with_encoding` | `crates/sifr_codegen/src/intrinsics/bytes.rs:166-176` | ✅ UTF-8-only guard, rejects others |
| `lower_decode_utf8` | `crates/sifr_codegen/src/intrinsics/bytes.rs:189-312` | ✅ Validates bytes in range, then UTF-8 |
| `lower_decode_utf8_with_encoding` | `crates/sifr_codegen/src/intrinsics/bytes.rs:178-188` | ✅ UTF-8-only guard, rejects others |
| `lower_bytes_to_hex` | `crates/sifr_codegen/src/intrinsics/bytes.rs:314-418` | ✅ Converts bytes to hex string |

### 1.4 Intrinsic Registry: ✅ VERIFIED

| Intrinsic | Registration Location | Assessment |
|-----------|----------------------|------------|
| `encode_utf8` | `crates/sifr_codegen/src/intrinsics/mod.rs:200` | ✅ Registered |
| `str_encode_utf8_result` | `crates/sifr_codegen/src/intrinsics/mod.rs:201` | ✅ Registered |
| `str_encode_utf8_result_with_encoding` | `crates/sifr_codegen/src/intrinsics/mod.rs:202-203` | ✅ Registered |
| `bytes_from_hex` | `crates/sifr_codegen/src/intrinsics/mod.rs:208` | ✅ Registered |
| `bytes_with_size` | `crates/sifr_codegen/src/intrinsics/mod.rs:209` | ✅ Registered |

---

## 2. Safety Guarantees

### 2.1 Negative Size Rejection: ✅ VERIFIED

```python
# bytes(-2) should fail with ValueError
try:
    _neg: bytes = bytes(-2)
except ValueError as e:
    bad_size_rejected = True
assert bad_size_rejected
```

**Codegen Implementation** (`bytes.rs:578-590`):
```rust
if __size < 0 {
    return Err(ValueError { message: "bytes(size) requires a non-negative size".to_string() })
}
```

### 2.2 Out-of-Range Byte Validation: ✅ VERIFIED

```python
# bytes.from_ints([0, 256]) should fail with ValueError
try:
    _bad_values: bytes = bytes.from_ints([0, 256])
except ValueError as e:
    bad_value_rejected = True
assert bad_value_rejected
```

**Codegen Implementation** (`bytes.rs:647-683`): Validates each byte is in range 0-255.

### 2.3 Invalid UTF-8 Rejection: ✅ VERIFIED

```python
# b"\xff".decode() should fail with ParseError
try:
    _invalid: str = b"\xff".decode()
except ParseError as e:
    bad_utf8_rejected = True
assert bad_utf8_rejected
```

**Codegen Implementation** (`bytes.rs:211-257`): Validates each byte is 0-255, then uses `String::from_utf8` for validation.

### 2.4 Invalid Hex Rejection: ✅ VERIFIED

```python
# bytes.from_hex("GG") should fail with ParseError
try:
    _bad_hex: bytes = bytes.from_hex("GG")
except ParseError as e:
    bad_hex_rejected = True
assert bad_hex_rejected
```

**Codegen Implementation** (`bytes.rs:462-478`): Validates each character is ASCII hex digit.

### 2.5 Non-UTF-8 Codec Rejection: ✅ VERIFIED

```python
# "abc".encode("latin-1") should fail with ParseError
codec: str = "latin-1"
try:
    _encoded: bytes = "abc".encode(codec)
except ParseError as e:
    bad_encode_codec_rejected = True
assert bad_encode_codec_rejected
```

**Codegen Implementation** (`bytes.rs:71-116`): UTF-8-only guard checks encoding is "utf-8" or "utf8".

---

## 3. Compatibility Migration

### 3.1 lib/sifr/bytes.sifr: ✅ CORRECTLY DELEGATES

| Function | Implementation | Status |
|----------|---------------|--------|
| `decode_utf8(data)` | `return data.decode()` | ✅ Delegates to first-class |
| `bytes_from_hex(s)` | `return bytes.from_hex(s)` | ✅ Delegates to first-class |
| `bytes_from_ints(values)` | `return bytes.from_ints(values)` | ✅ Delegates to first-class |
| `bytes_with_size(size)` | `return bytes(size)` | ✅ Delegates to first-class |
| `encode_utf8_result(s)` | `return s.encode()` | ✅ Delegates to first-class |

---

## 4. Regression Analysis

### 4.1 Existing Bytes Tests: ✅ NO REGRESSION

| Test | Result |
|------|--------|
| `cpython_bytes_subset.sifr` | ✅ PASS |
| `phase_psp_bytes_0_architecture_lock.sifr` | ✅ PASS |
| `phase_psp_bytes_1_core_type_support.sifr` | ✅ PASS |
| `stdlib_bytes.sifr` | ✅ PASS |
| `stdlib_bytes_safety.sifr` | ✅ PASS |

### 4.2 Base64 Tests: ✅ NO REGRESSION

| Test | Result |
|------|--------|
| `cpython_base64_rfc4648_vectors.sifr` | ✅ PASS |
| `stdlib_base64_intrinsics.sifr` | ✅ PASS |

### 4.3 Quick Validation: ✅ PASS

- **Command**: `scripts/run_all_tests.sh --profile quick`
- **Result**: ✅ PASS (wall_time=39.94s, max_rss=105.0MiB)
- **Report signature**: `e1bf653aaa770517`

---

## 5. Feature Completeness

### 5.1 Wave 2 Scope (per architecture lock)

| Feature | Status | Location |
|---------|--------|----------|
| `bytes(size)` constructor | ✅ DONE | `builtin_calls.rs:497-531`, `intrinsics/bytes.rs:566-612` |
| `bytes.from_ints(list[int])` | ✅ DONE | `builtin_calls.rs:600-624`, `intrinsics/bytes.rs:614-698` |
| `bytes.from_hex(str)` | ✅ DONE | `builtin_calls.rs:578-598`, `intrinsics/bytes.rs:420-564` |
| `str.encode(encoding?)` | ✅ DONE | `bytes_methods.rs:17-44`, `intrinsics/bytes.rs:158-176` |
| `bytes.decode(encoding?)` | ✅ DONE | `bytes_methods.rs:122-148`, `intrinsics/bytes.rs:178-312` |
| UTF-8-only enforcement | ✅ DONE | `intrinsics/bytes.rs:71-116` |
| `lib/sifr/bytes.sifr` migration | ✅ DONE | `lib/sifr/bytes.sifr` |

### 5.2 Negative-Path Coverage: ✅ COMPLETE

| Test | Expected Error | Status |
|------|----------------|--------|
| `bytes(-2)` | `ValueError` | ✅ Verified |
| `bytes.from_ints([0, 256])` | `ValueError` | ✅ Verified |
| `bytes.from_hex("GG")` | `ParseError` | ✅ Verified |
| `b"\xff".decode()` | `ParseError` | ✅ Verified |
| `"abc".encode("latin-1")` | `ParseError` | ✅ Verified |
| `b"abc".decode("latin-1")` | `ParseError` | ✅ Verified |

### 5.3 Compile-Time Type Checking: ✅ COMPLETE

| Test | Expected Error | Actual Error | Status |
|------|----------------|--------------|--------|
| `bytes("4")` | Type error | `bytes(size) expects 'int' size, got 'str'` | ✅ PASS |
| `bytes.from_hex(123)` | Type error | `bytes.from_hex() expects 'str', got 'int'` | ✅ PASS |
| `bytes.from_ints(["a"])` | Type error | `bytes.from_ints() expects 'list[int]', got 'list[str]'` | ✅ PASS |

---

## 6. Demos and Fixtures

### 6.1 New Demos: ✅ WORKING

| Demo | Result |
|------|--------|
| `ad_hoc_bytes_wave2_conversion_surface_demo.sifr` | ✅ PASS (outputs "ok") |
| `ad_hoc_bytes_wave2_negative_boundary_demo.sifr` | ✅ PASS (outputs "ok") |

### 6.2 New Pass Fixtures: ✅ WORKING

| Fixture | Result |
|---------|--------|
| `phase_psp_bytes_2_conversion_surfaces.sifr` | ✅ PASS |
| `phase_psp_bytes_2_conversion_negative_paths.sifr` | ✅ PASS |

### 6.3 New Fail Fixtures: ✅ CORRECTLY FAIL

| Fixture | Expected | Result |
|---------|----------|--------|
| `phase_psp_bytes_2_constructor_non_int.sifr` | Type error | ✅ Correctly fails |
| `phase_psp_bytes_2_from_hex_non_string.sifr` | Type error | ✅ Correctly fails |
| `phase_psp_bytes_2_from_ints_non_int_list.sifr` | Type error | ✅ Correctly fails |
| `phase_psp_bytes_2_encode_non_string_codec.sifr` | Type error | ✅ Correctly fails |
| `phase_psp_bytes_2_decode_non_string_codec.sifr` | Type error | ✅ Correctly fails |

### 6.4 Targeted Unit Tests: ✅ PASS

| Test | Result |
|------|--------|
| `cargo test -p sifr_codegen lowers_bytes_intrinsics_via_registry` | ✅ PASS |

---

## 7. Diagnostics Quality

### 7.1 Type Error Messages: ✅ CLEAR AND ACTIONABLE

| Operation | Error Message | Assessment |
|-----------|--------------|------------|
| `bytes("4")` | `bytes(size) expects 'int' size, got 'str'` | ✅ Clear |
| `bytes.from_hex(123)` | `bytes.from_hex() expects 'str', got 'int'` | ✅ Clear |
| `bytes.from_ints(["a"])` | `bytes.from_ints() expects 'list[int]', got 'list[str]'` | ✅ Clear |

### 7.2 Runtime Error Messages: ✅ CLEAR AND ACTIONABLE

| Operation | Error Message | Assessment |
|-----------|--------------|------------|
| `bytes(-1)` | `bytes(size) requires a non-negative size` | ✅ Clear |
| `bytes.from_ints([256])` | `byte out of range at index 0: 256` | ✅ Clear with index |
| `bytes.from_hex("GG")` | `invalid hex character: G` | ✅ Clear |
| `b"\xff".decode()` | UTF-8 validation error | ✅ Clear |

---

## 8. Architecture Alignment

### 8.1 Phase Contract: ✅ CORRECT

The implementation aligns with the phase specification in `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`:

| Requirement | Implementation | Status |
|-------------|---------------|--------|
| `bytes(size) -> Result[bytes, ValueError]` | Returns `Result<Vec<i64>, ValueError>` | ✅ |
| `bytes.from_ints(data) -> Result[bytes, ValueError]` | Validates 0-255 range | ✅ |
| `bytes.from_hex(s) -> Result[bytes, ParseError]` | Validates hex, handles whitespace | ✅ |
| `str.encode(encoding?) -> Result[bytes, ParseError]` | UTF-8 only | ✅ |
| `bytes.decode(encoding?) -> Result[str, ParseError]` | UTF-8 only | ✅ |
| Negative sizes fail with `ValueError` | Implemented | ✅ |
| No implicit coercion between str/bytes | Enforced | ✅ |

### 8.2 Downstream Contract: ✅ PREPARED

The implementation prepares for wave_psp_bytes_3:
- First-class `bytes` is now the canonical binary carrier
- `lib/sifr/bytes.sifr` delegates to first-class implementation
- Later phases can consume `bytes` without conversion

---

## 9. Review Summary

| Category | Status | Notes |
|----------|--------|-------|
| Type system correctness | ✅ APPROVED | All conversion types properly defined |
| HIR lowering correctness | ✅ APPROVED | All constructors/methods correctly lowered |
| Codegen correctness | ✅ APPROVED | All intrinsics generate correct Rust |
| Safety guarantees | ✅ APPROVED | All negative paths verified |
| Compatibility migration | ✅ APPROVED | lib/sifr/bytes.sifr correctly delegates |
| No regressions | ✅ APPROVED | All existing tests pass |
| Feature completeness | ✅ APPROVED | All wave 2 features implemented |
| Demo execution | ✅ APPROVED | All demos run correctly |
| Diagnostics quality | ✅ APPROVED | Clear and actionable error messages |

---

## 10. Recommendation

**APPROVE** for completion-gap review.

The implementation is correct, complete, and safe. All wave 2 features are implemented according to the architecture lock specification. All negative-path scenarios are properly handled with appropriate error types. The compatibility migration correctly delegates to the first-class implementation.

### Observations (Not Issues)

1. **Representation**: `bytes` remains `Vec<i64>` internally (consistent with wave 1 design choice for iteration and equality).

2. **Hex handling**: The implementation correctly:
   - Strips whitespace from hex input (useful for "53 69 66 72" format)
   - Requires even number of hex digits
   - Rejects invalid hex characters

3. **UTF-8 enforcement**: Both literal and runtime encoding arguments are validated. For literal arguments, a compile-time error is emitted. For runtime arguments, a runtime `ParseError` is returned.

---

## 11. Next Steps

1. [x] Verify all positive-path tests pass
2. [x] Verify all negative-path tests pass
3. [x] Verify compile-time type errors work
4. [x] Run quick validation suite
5. [ ] Proceed to Pass 2 (production-grade review)
6. [ ] Continue to wave_psp_bytes_3 (downstream contract adoption)
