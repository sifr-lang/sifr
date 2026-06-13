

Now I have a complete picture. Here is my review:

---

## Review: INT-4 Bytes uint8 Surface Slice — Pass 2

### 1. Blocking Findings

**None.** The implementation is correct and consistent.

### 2. Verification

**Pass-1 consistency issue — resolved:**

- `crates/sifr_codegen/src/expr_ref_emitter.rs:82`: `Type::Bytes => return Some(Type::FixedInt(sifr_type_system::FixedIntType::U8))` — now correctly returns `U8` instead of `Type::Int`. The `option_inner_type` path (lines 44–53) already handled the union correctly via the type system; this fix aligns the index-expression fallback path.

**Contract enforcement — verified across all layers:**

| Layer | File:Line | Change |
|---|---|---|
| Type system | `types.rs:893` | `Bytes.iterable_element_type()` → `FixedInt(U8)` |
| Type system | `types.rs:963` | `Bytes.iteration_metadata().element_type` → `FixedInt(U8)` |
| Type system | `types.rs:1080-1084` | `Bytes.index_result_type(Int)` → `FixedInt(U8) \| None` |
| Type system | `types.rs:1139` | `Bytes.option_element_type()` → `FixedInt(U8)` |
| Type system | `types.rs:1271-1273` | `Bytes → Iterable(T)` requires `T = FixedInt(U8)` |
| Type system test | `types.rs:1501-1523` | 5 assertions covering all contract points |
| HIR lowering | `guarded_index.rs:25,90-96` | Guarded bytes index → `FixedInt(U8)` |
| HIR test | `expressions_tests.rs:2282-2318` | Verifies index/iteration types |
| Codegen | 12 × `RustType::I64` → `RustType::Named("u8")` | Bytes element → `u8` |
| Codegen widening | `intrinsic_method_emitters.rs:2954-2957` | `FixedInt(_)` cast to `i64` for arithmetic contexts |
| Stdlib | `lib/sifr/bytes.sifr` | `int(b)` for widening, `uint8` annotations |
| E2E | 5 existing tests + `bytes_uint8_surface.sifr` | New test covers surface behavior |

**No regressions:** All local validation tests passed (quick profile, focused unit/E2E).

**No user-triggerable panic risks:** No new `unwrap()`/`expect()` on data-dependent paths.

### 3. Final Verdict

**Satisfied.** The INT-4 rule is correctly implemented: bytes indexing and iteration expose `uint8`, ordinary indexes and lengths remain `int`. The pass-1 consistency issue is resolved without introducing new issues.
