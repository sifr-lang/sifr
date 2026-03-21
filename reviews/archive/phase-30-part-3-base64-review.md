# Phase 30 Part 3: Base64 Implementation Review

**Date:** 2026-03-08
**Focus:** Code Reuse, Quality, and Efficiency

---

## Findings

### 1. Missing Public Exports in Stdlib (Medium)

**Location:** `lib/sifr/base64.sifr:2`

The module imports but does NOT re-export these functions:
- `urlsafe_b64encode`, `urlsafe_b64decode`
- `b32encode`, `b32decode`, `b32hexencode`, `b32hexdecode`

These are imported from `_sifr.crypto` but have no corresponding `def` exports in the module. Tests import them directly from `sifr.base64`, which works via intrinsics resolution - but explicit definitions are needed for API clarity.

**Fix:** Add to `lib/sifr/base64.sifr`:
```sifr
def urlsafe_b64encode(s: str) -> str:
    return _sifr.crypto.urlsafe_b64encode(s)

def urlsafe_b64decode(s: str) -> Result[str, ParseError]:
    return _sifr.crypto.urlsafe_b64decode(s)

def b32encode(s: str) -> str:
    return _sifr.crypto.b32encode(s)

# ... etc
```

### 2. Code Duplication in Codegen (Medium)

**Location:** `crates/sifr_codegen/src/intrinsics/base64.rs`

~10 helper functions duplicated across 7+ intrinsic files:
- `ref_expr`, `arg_expr`, `string_lit`, `int_lit`, `bool_lit`, `char_lit`
- `parse_error`, `err_parse`, `to_string_expr`, `as_bytes`, `parse_map_err`, `ok_expr`

**Fix:** Extract to shared module `crates/sifr_codegen/src/intrinsics/common.rs`:
```rust
// common.rs
pub fn ref_expr(expr: RustExpr) -> RustExpr { ... }
pub fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr { ... }
// etc
```

### 3. Redundant Error Re-raising (Low)

**Location:** `lib/sifr/base64.sifr:30-35`, `37-42`

```sifr
def b16encode(s: str) -> Result[str, ParseError]:
    try:
        raw: str = bytes_to_hex(encode_utf8(s))
        return raw.upper()
    except ParseError as e:
        raise e  # Redundant
```

**Fix:** Remove unnecessary try/except:
```sifr
def b16encode(s: str) -> Result[str, ParseError]:
    raw: str = bytes_to_hex(encode_utf8(s))
    return raw.upper()
```

---

## Verified

- Demo passes: `m30_1a base64 parity demo: pass`
- Encode/decode roundtrips work (b64, urlsafe, b16)
- Error handling returns `ParseError` correctly

## Summary

| Issue | Severity | Effort |
|-------|----------|--------|
| Missing stdlib exports | Medium | Low |
| Codegen helper duplication | Medium | Medium |
| Redundant error handlers | Low | Low |
