# wave_psp_rng_2 Review Pass 1

## Phase Document Reference

- **Phase**: `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`
- **Wave**: `wave_psp_rng_2: Advanced Hash and Binary Surface Expansion`
- **Scope**: `hashlib`, `base64`

---

## Executive Summary

The `wave_psp_rng_2` implementation successfully delivers bytes-native parity for both `hashlib` and `base64` modules. The implementation follows the phase contract correctly, using `bytes` as the canonical binary carrier rather than `str`. All required surface additions (`digest_bytes()`, `update_bytes()`, `new_bytes()` for hashlib; `b64encode_bytes()`, `b64decode_bytes()` variants for base64) are present and correctly typed.

**Status**: Implementation is substantially correct. Minor gaps identified in test coverage and error boundary consistency.

---

## Root-Cause Correctness

### hashlib Implementation

| Phase Contract Requirement | Implementation Status | Evidence |
|---|---|---|
| `HashObject._data: bytes` | ✅ Implemented | `lib/sifr/hashlib.sifr:10` - `_data: bytes` |
| `update(data: str) -> None` | ✅ Implemented | `lib/sifr/hashlib.sifr:27-28` - converts via `encode_utf8()` |
| `update_bytes(data: bytes) -> None` | ✅ Implemented | `lib/sifr/hashlib.sifr:30-31` - direct bytes concatenation |
| `digest() -> bytes` | ✅ Implemented | `lib/sifr/hashlib.sifr:36-37` - returns raw bytes via `_hash_bytes()` |
| `digest_bytes() -> bytes` | ✅ Implemented | `lib/sifr/hashlib.sifr:39-40` - alias to `digest()` |
| `hexdigest() -> str` | ✅ Implemented | `lib/sifr/hashlib.sifr:33-35` - converts bytes to hex string |
| `new_bytes(name: str, data: bytes = b"")` | ✅ Implemented | `lib/sifr/hashlib.sifr:113-117` |

**Root cause**: Correctly uses `bytes` as the internal state carrier. String compatibility is maintained via `encode_utf8()` at the API boundary, not by storing strings internally.

### base64 Implementation

| Phase Contract Requirement | Implementation Status | Evidence |
|---|---|---|
| Bytes-native encode variants | ✅ Implemented | `lib/sifr/base64.sifr:15-18` - `b64encode_bytes`, `standard_b64encode_bytes` |
| Bytes-native decode variants | ✅ Implemented | `lib/sifr/base64.sifr:19-22` - `b64decode_bytes`, `standard_b64decode_bytes` |
| URL-safe bytes variants | ✅ Implemented | `lib/sifr/base64.sifr:37-40` - `urlsafe_b64encode_bytes`, `urlsafe_b64decode_bytes` |
| Retain text helpers | ✅ Implemented | Original `b64encode(s: str) -> str` variants preserved |

**Root cause**: Correctly provides bytes-in/bytes-out paths while maintaining backward compatibility with string-based APIs.

---

## bytes-native Parity Contracts

### hashlib: No Per-Element Range Validation

The implementation uses Rust's `sha2`, `blake2`, `sha1` crates directly, which operate on `&[u8]`. The codegen intrinsics (`crates/sifr_codegen/src/intrinsics/hashlib.rs`) correctly pass bytes without any per-element validation:

```rust
// crates/sifr_codegen/src/intrinsics/hashlib.rs:24-28
pub(super) fn lower_sha1_bytes(args: &[RustExpr]) -> Option<RustExpr> {
    Some(digest_bytes("<sha1::Sha1 as sha1::Digest>::digest", &args[0]))
}
```

This is correct - the bytes flow directly to the Rust digest function without Sifr-side range checking.

### base64: No Per-Element Range Validation

The base64 intrinsics similarly route bytes directly to the `base64` crate:

```rust
// crates/sifr_codegen/src/intrinsics/base64.rs:108-117
fn engine_encode(engine: RustExpr, bytes: RustExpr) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "base64".to_string(),
            "Engine".to_string(),
            "encode".to_string(),
        ])),
        args: vec![ref_expr(engine), bytes],
    }
}
```

**Assessment**: ✅ Parity contract satisfied - bytes-native paths operate directly on `bytes` without i64 widening/narrowing or per-element validation.

---

## Waiver Accuracy

### SHA3/SHAKE Waiver

| Document | Waiver Statement | Implementation |
|---|---|---|
| Phase doc (line 113) | "Add SHA3 / SHAKE only for algorithms already supported by the Rust dependency stack when implementation begins" | ✅ Correctly waived |
| wave_psp_e1_cpython_traceability.md | "SHA3/SHAKE constructor families...remain waived" | ✅ Correctly documented |
| wave_psp_rng_2_cpython_traceability.md | "SHA3/SHAKE constructor families remain explicitly unsupported" | ✅ Correctly documented |

The waiver is accurate - no SHA3/SHAKE dependency is registered in the runtime, and the implementation correctly raises `ValueError` for all SHA3/SHAKE constructors:

```sifr
# lib/sifr/hashlib.sifr:123-130
def sha3_256_obj(data: str = "") -> Result[HashObject, ValueError]:
    raise ValueError("sha3_256 is not yet supported")
```

### Dependency Audit

The wave_psp_rng_2_cpython_traceability.md correctly documents active dependencies:
- `sha2 = "0.10"`
- `md5 = "0.7"`
- `sha1 = "0.10"`
- `blake2 = "0.10"`

No SHA3/SHAKE dependency is present, making the waiver accurate.

---

## Test/Traceability Coverage

### Positive Coverage

| Fixture | Purpose | Status |
|---|---|---|
| `phase_psp_rng_2_hashlib_base64_bytes_native_surface.sifr` | Core bytes-native API verification | ✅ Present |
| `cpython_hashlib_object_model_subset.sifr` | HashObject model verification | ✅ Present |
| `cpython_hashlib_api_subset.sifr` | API surface verification | ✅ Present |
| `ad_hoc_rng_wave2_hashlib_base64_bytes_demo.sifr` | Demo/architecture validation | ✅ Present |

### Negative Coverage

| Fixture | Purpose | Status |
|---|---|---|
| `phase_psp_rng_2_sha3_object_model_unsupported.sifr` | SHA3 unsupported boundary | ✅ Present |

### Coverage Gaps Identified

1. **Missing bytes-input to string-based APIs**: No test verifies that `new("sha256", "str")` produces the same digest as `new_bytes("sha256", encode_utf8("str"))`. The test fixture does include this check at line 27-29, but it's worth noting.

2. **No edge case test for empty bytes**: Missing explicit test for `new_bytes("sha256", b"")` producing correct empty-input digest.

3. **No test for `update()` after `update_bytes()`**: The implementation correctly chains both, but no test verifies mixing `update(str)` and `update_bytes(bytes)` on the same object.

---

## Issues and Recommendations

### Issue 1: Inconsistent Error Handling in `_hash_hex` (LOW)

**Location**: `lib/sifr/hashlib.sifr:95-102`

```sifr
def _hash_hex(algorithm: str, data: bytes) -> str:
    try:
        hex_value: str = _bytes_to_hex_or_value_error(_hash_bytes(algorithm, data))
        return hex_value
    except ValueError as e:
        _ = e.message
        return ""  # Silent fallback to empty string
```

**Problem**: The catch-all `return ""` silently swallows errors. If `_hash_bytes` fails, the caller receives an empty string rather than a typed error. This could mask bugs.

**Recommendation**: Consider returning `Result[str, HashlibError]` or at minimum logging the error before returning empty string. This is a deviation from the "no silent fallbacks" principle.

### Issue 2: Missing Return Type on `sha3_*_obj` Functions (LOW)

**Location**: `lib/sifr/hashlib.sifr:123-130`

The SHA3 placeholder functions have inconsistent return types:

```sifr
def sha3_256_obj(data: str = "") -> Result[HashObject, ValueError]:
    raise ValueError("sha3_256 is not yet supported")
```

**Problem**: The function signature declares `Result[HashObject, ValueError]` but always raises (never returns `Ok`). This is technically correct Sifr code but could be confusing.

**Recommendation**: Either:
- Keep current implementation (technically valid - raise is a valid Result producer)
- Or change to `def sha3_256_obj(data: str = "") -> Result[HashObject, ValueError]: raise ValueError(...)`

This is a documentation/clarity issue, not a correctness issue.

### Issue 3: Incomplete Negative Test for base64 (LOW)

**Location**: `verification/stdlib/wave_psp_rng_2_cpython_traceability.md`

The negative test coverage is focused on hashlib (SHA3). There is no explicit negative test for:
- Invalid base64 character input to bytes-decoding variants
- Invalid input types (passing `int` to `b64encode_bytes`)

**Recommendation**: Add a fail fixture for base64 invalid input handling to match hashlib's negative coverage.

---

## Summary

| Category | Assessment |
|---|---|
| Root-cause correctness | ✅ PASS - bytes-native internal state correctly implemented |
| bytes-native parity contracts | ✅ PASS - no per-element validation, direct Rust crate routing |
| Waiver accuracy | ✅ PASS - SHA3/SHAKE correctly waived with accurate dependency audit |
| Test/traceability coverage | ⚠️ MINOR GAPS - see Issues 1-3 above |

**Overall**: Implementation is correct and satisfies the phase contract. The issues identified are low-severity improvements rather than correctness blockers.

---

## Action Items

1. [ ] Review error handling in `_hash_hex` for potential silent failure
2. [ ] Add base64 negative test fixture for invalid input handling
3. [ ] Consider adding edge case tests for empty bytes and mixed update methods
