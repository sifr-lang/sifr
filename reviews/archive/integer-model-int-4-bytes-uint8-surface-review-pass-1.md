Now I have a complete picture. Let me provide my review.

---

## Review: INT-4 Bytes uint8 Surface Slice

### 1. Blocking Findings

**None.** The implementation correctly implements the INT-4 rule.

### 2. Non-Blocking Observations

**Pre-existing type system inconsistency (not introduced by this PR):**

- **File:** `crates/sifr_codegen/src/expr_ref_emitter.rs:82`
- **Code:** `Type::Bytes => return Some(Type::Int)`
- **Issue:** The `display_option_inner_type` function returns `Type::Int` for bytes, but should return `Type::FixedInt(FixedIntType::U8)` per the INT-4 contract.

**Analysis:** This branch is effectively unreachable for bytes indexing expressions because:
1. Bytes indexing produces `uint8 | None` (confirmed at `types.rs:1078-1084`)
2. The `option_inner_type` function correctly extracts `FixedInt(U8)` from unions (lines 44-52)
3. `option_inner_type` returns `Some(Type::FixedInt(U8))` before the indexing branch is evaluated

**No functional impact:** Both `Type::Int` and `Type::FixedInt(_)` return `false` from `uses_debug_display_format` (line 8), so the display format would be identical in either case.

**Recommendation:** Fix as a follow-up cleanup for type system consistency. Not a blocker.

### 3. Verification Summary

| Area | Status | Evidence |
|------|--------|----------|
| Type system contract | ✓ | `types.rs:893, 960, 1078-1084, 1269-1271` — bytes element/iteration/index all use `FixedInt(U8)` |
| Type system tests | ✓ | `types.rs:1505-1520` — 5 assertions covering all contract points |
| HIR lowering | ✓ | `guarded_index.rs:87-96` — guarded bytes index returns `FixedInt(U8)` |
| HIR unit test | ✓ | `expressions_tests.rs:2282-2313` — verifies index and iteration types |
| Codegen (Rust types) | ✓ | 12 instances of `RustType::I64` → `RustType::Named("u8")` for bytes elements |
| Codegen (widening) | ✓ | `intrinsic_method_emitters.rs:2954-2957` — `FixedInt(_)` widening to `i64` |
| Standard library | ✓ | `bytes.sifr` — uses `int(b)` for widening, `uint8` for element types |
| E2E tests | ✓ | 5 test files updated + new `bytes_uint8_surface.sifr` |
| Local validation | ✓ | All quick profile tests pass |

### 4. Final Verdict

**Satisfied.** The INT-4 rule is correctly implemented: bytes indexing and iteration expose `uint8`, while ordinary indexes and lengths remain `int`. No regressions, no user-triggerable panic risks, no contract drift in changed code.
