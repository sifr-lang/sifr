# Review: wave_psp_iter_fix_1 Completion-Gap Check (Pass 1)

**Phase**: `ad-hoc-canonical-iteration-model-and-lazy-parity-closure`
**Wave**: `wave_psp_iter_fix_1` - Type-System Capability Layer
**Review Type**: Completion-gap check (pass 1)
**Date**: 2026-03-20

---

## Review Scope

This review evaluates `wave_psp_iter_fix_1` for:
1. Completion against wave scope
2. Required validation evidence
3. Governance alignment
4. Any gaps in the implementation

---

## Summary Assessment

| Category | Status | Notes |
|----------|--------|-------|
| Type-system implementation | **PASS** | `Reversible[T]` alias with capability-aware typing implemented |
| Positive test fixtures | **PASS** | All pass fixtures execute correctly |
| Negative test fixtures | **PASS** | All fail fixtures correctly reject invalid operations |
| Demo execution | **PASS** | Demo runs as expected |
| CPython traceability | **PASS** | Matrix document present and complete |
| Architecture alignment | **PASS** | Architecture doc updated with Reversible contract |

---

## Detailed Findings

### 1. Wave Scope Review

#### ✅ Implemented per Scope

| Scope Item | Implementation Status |
|------------|----------------------|
| Keep `Iterable[T]` / `Iterator[T]` first-class | **IMPLEMENTED** - Already present in type system |
| Add reversible capability support | **IMPLEMENTED** - `Type::reversible()` constructor and `Reversible[T]` alias added to `sifr_type_system/src/types.rs` |
| Add internal iteration capability metadata | **IMPLEMENTED** - Capability tracking present in type system |
| Align assignability with frozen contract | **IMPLEMENTED** - Assignability rules updated |
| Make tuple rule explicit | **IMPLEMENTED** - `homogeneous_tuple_iter_element_type()` added |

#### Definition of Done Verification

Per the phase document, wave_psp_iter_fix_1 definition of done:

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Reversibility is type-checked explicitly | **PASS** | `reversed(it)` rejects non-reversible iterators with explicit error |
| Type-level iteration semantics no longer depend on erased backend assumptions | **PASS** | `Reversible[T]` is a proper type alias, not erased |
| Tuple iteration behavior is internally consistent | **PASS** | Homogeneous tuples iterate, heterogeneous rejected |

---

### 2. Validation Evidence Review

#### Positive Path Validation

| Test | Command | Result |
|------|---------|--------|
| Pass fixture | `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/phase_psp_iter_fix_1_type_capability_layer.sifr` | ✅ PASS (cache hit) |
| Demo execution | `cargo run -q -p sifr -- run demos/ad_hoc_iter_fix_wave1_type_capability_demo.sifr` | ✅ PASS (outputs: 30, 15, [6, 5, 4]) |

#### Negative Path Validation

| Test | Expected Error | Actual Result |
|------|----------------|---------------|
| Heterogeneous tuple iteration | `iter() tuple argument must have one statically provable element type` | ✅ PASS |
| Reversed on Iterator | `reversed() argument must be reversible, got 'Iterator[int]'` | ✅ PASS |
| Reversible annotation rejects set | `argument 1 ('xs') of function 'consume': expected 'Reversible[int]', got 'set[int]'` | ✅ PASS |

#### Code Changes Summary

The implementation (commit `bd6119d9`) includes changes to:

- `crates/sifr_type_system/src/types.rs` - Added `Reversible[T]` alias and helper functions
- `crates/sifr_type_system/src/infer.rs` - Inference updates
- `crates/sifr_hir/src/lower/builtin_calls.rs` - Lowering for iterator operations
- `crates/sifr_hir/src/lower/expressions.rs` - Expression lowering updates
- `crates/sifr_codegen/src/intrinsic_method_emitters.rs` - Codegen for iterator operations
- `crates/sifr_codegen/src/stmt_support_emitter.rs` - Statement emission updates

---

### 3. Governance Alignment

#### ✅ Architecture Lock Alignment

The implementation aligns with the architecture lock documented in `verification/stdlib/phase_psp_iter_fix_architecture_lock.md`:

- **Canonical iteration types**: `Reversible[T]` added as type alias
- **Capability model**: Type system now tracks reversibility explicitly
- **Tuple iteration**: Homogeneous tuples supported, heterogeneous rejected

#### ✅ CPython Traceability Matrix

The traceability document (`verification/stdlib/wave_psp_iter_fix_1_cpython_traceability.md`) correctly maps:

| CPython family | Sifr surface direction | State |
|----------------|----------------------|-------|
| `test_iter` reverse-capability behavior | `reversed(...)` rejects non-double-ended iterators | adapted |
| `test_iter` iterable protocol annotations | `Reversible[T]` protocol alias | adapted |
| `test_tuple` homogeneous vs heterogeneous iteration | Homogeneous supported, heterogeneous rejected | adapted |

---

### 4. Completion Gap Analysis

#### ✅ No Critical Gaps Found

1. **Type-system implementation**: Complete - `Reversible[T]` alias with full capability tracking
2. **Positive validation**: Complete - All pass fixtures execute correctly
3. **Negative validation**: Complete - All fail fixtures correctly reject invalid operations
4. **Demo execution**: Complete - Demo runs and produces expected output
5. **Documentation**: Complete - Architecture and traceability documents updated

#### ⚠️ Observations

1. **Test Suite Run**: Full e2e pass suite validation should be run before merging (per workflow)
2. **HIR/Codegen Scope**: Wave 1 focuses on type-system capability layer; HIR and codegen pipeline for iterators are owned by subsequent waves (wave_psp_iter_fix_2, wave_psp_iter_fix_3)

---

## Implementation Details Verified

### Type System Changes

From `sifr_type_system/src/types.rs`:

```rust
pub fn reversible(element_type: Type) -> Self {
    Self::Alias {
        name: "Reversible".to_string(),
        type_args: vec![element_type.clone()],
        body: Box::new(Self::Iterable(Box::new(element_type))),
    }
}

fn reversible_alias_element_type(ty: &Type) -> Option<Type> {
    // Reversible[T] unwrapping logic
}

fn homogeneous_tuple_iter_element_type(elems: &[Type]) -> Option<Type> {
    // Returns element type only if all tuple elements are identical
}
```

### Error Messages Verified

- Heterogeneous tuple: `"iter() tuple argument must have one statically provable element type"`
- Non-reversible iterator: `"reversed() argument must be reversible, got 'Iterator[int]'"`
- Non-reversible set: `"expected 'Reversible[int]', got 'set[int]'"`

---

## Recommendation

**READY FOR EXTERNAL REVIEW**

`wave_psp_iter_fix_1` has completed all type-system capability layer requirements:

1. ✅ `Reversible[T]` type alias implemented
2. ✅ Capability-aware assignability working
3. ✅ Homogeneous tuple iteration supported
4. ✅ Heterogeneous tuple iteration rejected with explicit error
5. ✅ All test fixtures pass as expected

**Next steps per execution workflow**:
1. Run full validation: `scripts/run_all_tests.sh --profile quick`
2. Open PR for wave_psp_iter_fix_1
3. Complete external review loop
4. Merge PR
5. Proceed to `wave_psp_iter_fix_2` (Canonical Iterator HIR)

---

## Review Metadata

- **Reviewer**: agent
- **Review pass**: 1 (completion-gap check)
- **Files examined**:
  - `issues/ad-hoc-canonical-iteration-model-and-lazy-parity-closure.md`
  - `verification/stdlib/phase_psp_iter_fix_architecture_lock.md`
  - `verification/stdlib/wave_psp_iter_fix_1_cpython_traceability.md`
  - `crates/sifr_type_system/src/types.rs` (Reversible implementation)
  - `crates/sifr/tests/e2e/pass/phase_psp_iter_fix_1_type_capability_layer.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_iter_heterogeneous_tuple_unsupported.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_reversed_iterator_not_reversible.sifr`
  - `crates/sifr/tests/e2e/fail/phase_psp_iter_fix_1_reversible_annotation_rejects_set.sifr`
  - `demos/ad_hoc_iter_fix_wave1_type_capability_demo.sifr`
