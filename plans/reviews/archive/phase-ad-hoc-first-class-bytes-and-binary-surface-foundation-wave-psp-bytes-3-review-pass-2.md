# Review: wave_psp_bytes_3 Production-Grade Review (Review Pass 2)

**Wave**: `wave_psp_bytes_3` (Downstream Contract Adoption and Governance Closeout)
**Phase**: `issues/ad-hoc-first-class-bytes-and-binary-surface-foundation.md`
**Reviewer**: Production-grade review
**Date**: 2026-03-19

---

## Executive Summary

**Status**: ⚠️ **CONDITIONAL APPROVAL - Requires Root-Cause Remediation**

The wave demonstrates correct type-signature enforcement and compile-time contract guarantees. However, a **critical production-grade issue** was identified: the `bytes` type is not actually implemented as a first-class runtime type. Instead, it is implemented as `list[int]` (`Vec<i64>`) throughout the codegen pipeline.

This is a **root-cause architectural issue** that affects production readiness:

1. **Semantic Mismatch**: Documentation claims "first-class bytes is shipped" but the runtime representation is `list[int]`
2. **Performance Implications**: Using `list[int]` (heap-allocated `Vec<i64>`) instead of a compact byte buffer (`Vec<u8>`) has memory and performance consequences
3. **Future Compatibility Risk**: Downstream phases anchoring on this contract will inherit the `list[int]` semantics
4. **Governance Ledger Accuracy**: The waivers are incorrectly scoped—they don't capture this fundamental implementation divergence

---

## Review Areas

### 1. Type-Signature Correctness ✅

**Finding**: The HIR type signatures are correctly defined.

**Evidence**:
- `crates/sifr_hir/src/stdlib/sys_fs.rs` lines 426-445:
  ```rust
  // file_read_bytes(handle: int) -> Result[bytes, IOError]
  functions.insert(
      "file_read_bytes".to_string(),
      FunctionType::all_borrow(
          vec![("handle".to_string(), Type::Int)],
          result_ty(Type::Bytes, "IOError"),
      ),
  );

  // file_write_bytes(handle: int, data: bytes) -> Result[None, IOError]
  functions.insert(
      "file_write_bytes".to_string(),
      FunctionType::all_borrow(
          vec![
              ("handle".to_string(), Type::Int),
              ("data".to_string(), Type::Bytes),
          ],
          result_ty(Type::None, "IOError"),
      ),
  );
  ```

**Verification**:
```bash
$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr
type error: argument 1 ('data') of FileHandle.write_bytes(): expected 'bytes', got 'list[int]'

$ cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr
type error: type mismatch: expected 'list[int]', got 'Result[bytes, IOError]'
```

---

### 2. Runtime Implementation ⚠️ CRITICAL ISSUE

**Finding**: ❌ **ROOT-CAUSE ISSUE IDENTIFIED**

The `bytes` type is **NOT** implemented as a first-class type. It is implemented as `list[int]` throughout the codegen pipeline.

**Evidence**:

#### a) Codegen Type Mapping
`crates/sifr_codegen/src/preamble.rs` line 11:
```rust
Type::Bytes => RustType::Vec(Box::new(RustType::I64)),
```

This maps `bytes` to `Vec<i64>`, which is exactly how `list[int]` is represented in Rust.

#### b) File I/O Intrinsics
`crates/sifr_codegen/src/intrinsics/file_handles.rs`:

- **`lower_file_read_bytes`** (lines 720-775): Returns `Vec<i64>`:
  ```rust
  RustStmt::Return(Some(ok_expr(RustExpr::MethodCall {
      receiver: Box::new(RustExpr::MethodCall {
          receiver: Box::new(RustExpr::Ident("__buf".to_string())),
          method: "into_iter".to_string(),
          args: vec![],
      }),
      method: "map".to_string(),
      args: vec![RustExpr::Closure {
          params: vec![RustParam::Named {
              name: "b".to_string(),
              ty: RustType::Named("u8".to_string()),
          }],
          body: Box::new(RustExpr::Cast {
              expr: Box::new(RustExpr::Ident("b".to_string())),
              ty: RustType::I64,  // <-- Converting to i64!
          }),
          is_move: false,
      }],
  })),
  ```

- **`lower_file_write_bytes`** (lines 777-833): Accepts `Vec<i64>`:
  ```rust
  RustStmt::Let {
      mutable: false,
      name: "__data".to_string(),
      ty: Some(RustType::Vec(Box::new(RustType::Named("u8".to_string())))),
      value: RustExpr::MethodCall {
          receiver: Box::new(RustExpr::MethodCall {
              receiver: Box::new(RustExpr::MethodCall {
                  receiver: Box::new(args[1].clone()),  // <-- Input is Vec<i64>
                  method: "iter".to_string(),
                  args: vec![],
              }),
              method: "copied".to_string(),
              args: vec![],
          }),
          method: "map".to_string(),
          args: vec![RustExpr::Closure {
              params: vec![RustParam::Named {
                  name: "b".to_string(),
                  ty: RustType::I64,  // <-- Treating as i64!
              }],
              body: Box::new(RustExpr::Cast {
                  expr: Box::new(RustExpr::Ident("b".to_string())),
                  ty: RustType::Named("u8".to_string()),
              }),
              is_move: false,
          }],
      }),
      method: "collect".to_string(),
      args: vec![],
  },
  ```

#### c) Bytes Intrinsics
`crates/sifr_codegen/src/intrinsics/bytes.rs`:

- **`lower_encode_utf8`** (lines 117-156): Returns `Vec<i64>`:
  ```rust
  method: "collect::<Vec<i64>>".to_string(),  // <-- list[int]!
  ```

- **`lower_bytes_with_size`** (lines 566-612): Returns `Vec<i64>`:
  ```rust
  method: "collect::<Vec<i64>>".to_string(),  // <-- list[int]!
  ```

---

### 3. Fixture Coverage ✅

**Finding**: ✅ **COMPLETE** - All fixtures pass correctly.

**Positive-path fixtures**:
- `crates/sifr/tests/e2e/pass/phase_psp_bytes_3_downstream_contract_alignment.sifr` - ✅ PASS
- `demos/ad_hoc_bytes_wave3_downstream_contract_adoption_demo.sifr` - ✅ PASS

**Negative-path fixtures**:
- `crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr` - ✅ Type error correctly produced
- `crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr` - ✅ Type error correctly produced

---

### 4. Governance Ledger Accuracy ⚠️

**Finding**: ⚠️ **INACCURATE** - Documentation claims "first-class bytes is shipped" but implementation uses `list[int]`.

**Evidence from governance documents**:

1. `verification/stdlib/milestone_psp_7_parity_governance_inventory.md` line 49:
   > "Immutable first-class `bytes` is now shipped and used as the canonical binary carrier"

2. `verification/stdlib/wave_psp_bytes_3_cpython_traceability.md` line 16:
   > "first-class `bytes` at `FileHandle.read_bytes()` / `FileHandle.write_bytes(...)` boundaries"

**Issue**: The documentation claims a first-class bytes type is shipped, but the runtime representation is `list[int]`. This is a **semantic inaccuracy** that needs to be corrected.

---

### 5. Waiver Scope Assessment ⚠️

**Finding**: ⚠️ **WAIVERS ARE INCORRECTLY SCOPED**

The current waivers in `wave_psp_bytes_3_cpython_traceability.md` (lines 20-28) cover:
- `bytearray` mutable object-model parity
- `memoryview` and buffer protocol families
- Non-UTF-8 codec matrices
- `hashlib` bytes-native digest families
- Direct bytes-oriented base64 entrypoints

**Missing Waiver**: The fundamental implementation divergence (bytes as `list[int]`) is **NOT** documented as a waiver. This is a critical oversight because:

1. The phase claims to have shipped "first-class bytes" but hasn't
2. The runtime uses `list[int]` semantics (heap-allocated `Vec<i64>`) instead of compact byte buffer (`Vec<u8>`)
3. This has performance implications for any code relying on binary data

---

### 6. Regression Analysis ✅

**Finding**: ✅ **NO REGRESSIONS** - The implementation maintains backwards compatibility.

The type-system correctly enforces:
- Users MUST use `bytes` type for `read_bytes()` return values
- Users MUST use `bytes` type for `write_bytes()` arguments
- `list[int]` is correctly rejected at compile time

This ensures that even though the runtime uses `list[int]`, users are forced to use the `bytes` type, maintaining API consistency for future migration to a true first-class bytes type.

---

### 7. Successor Phase Alignment ⚠️

**Finding**: ⚠️ **POTENTIAL FUTURE ISSUE**

Successor phases (`issues/ad-hoc-runtime-and-file-object-parity-expansion.md` and `issues/ad-hoc-stateful-rng-crypto-and-polish-parity-expansion.md`) are anchoring on the `bytes` contract. If the current implementation (using `list[int]` semantics) is not corrected before those phases, they will inherit the wrong semantics.

---

## Summary

| Review Area | Status | Notes |
|-------------|--------|-------|
| Type-signature correctness | ✅ Complete | HIR correctly defines `bytes` type |
| Compile-time contract enforcement | ✅ Complete | `list[int]` correctly rejected |
| Fixture coverage | ✅ Complete | All fixtures pass |
| Runtime implementation | ❌ **CRITICAL** | Uses `list[int]`, not first-class bytes |
| Governance documentation | ⚠️ Inaccurate | Claims first-class bytes, but uses list[int] |
| Waiver scope | ⚠️ Incorrect | Missing fundamental implementation divergence |
| Regression risk | ✅ Low | Type system enforces correct API usage |
| Successor phase alignment | ⚠️ Risk | Phases inherit incorrect semantics |

---

## Root Cause Analysis

The implementation took a **shortcut approach**:
1. Added `Type::Bytes` to the type system enum ✅
2. Added type signatures using `Type::Bytes` ✅
3. But mapped `Type::Bytes` to `Vec<i64>` (`list[int]`) in codegen ❌

This approach provides **type safety** (users must use `bytes` type) but does NOT provide the **claimed first-class bytes implementation** (compact byte buffer with `Vec<u8>` semantics).

---

## Verification Commands Run

```bash
# Positive path tests
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_bytes_3_downstream_contract_alignment.sifr  # PASS
cargo run -q -p sifr -- run demos/ad_hoc_bytes_wave3_downstream_contract_adoption_demo.sifr                 # PASS

# Negative path tests
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_write_bytes_rejects_int_list.sifr  # Type error: expected 'bytes', got 'list[int]'
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/phase_psp_bytes_3_read_bytes_not_list.sifr          # Type error: expected 'list[int]', got 'Result[bytes, IOError]'
```

---

## Recommendation

### Option A: Accept Current Implementation (Not Recommended)

**Conditions**:
- Document the bytes-to-list[int] mapping as an explicit waiver
- Update governance documents to accurately reflect the implementation
- Accept performance implications of using `list[int]` semantics

### Option B: Implement True First-Class Bytes (Recommended)

**Required Changes**:
1. Create a dedicated `SifrBytes` Rust struct for the bytes runtime representation
2. Update `crates/sifr_codegen/src/preamble.rs` to use the new struct instead of `Vec<i64>`
3. Update `file_handles.rs` intrinsics to work with `SifrBytes`/`Vec<u8>`
4. Update `bytes.rs` intrinsics to work with `SifrBytes`/`Vec<u8>`
5. Update fixture expectations if needed
6. Update governance documents to reflect the completed implementation

---

## Decision Required

The wave implementers must decide whether to:
1. **Proceed with Option B**: Implement true first-class bytes (requires additional work)
2. **Accept with Documentation**: Document the current implementation as a "type-safe list[int] adaptation" with explicit waivers

**Current Status**: ⚠️ **BLOCKED** - Cannot approve for production until the implementation divergence is resolved or explicitly documented as a waiver.
