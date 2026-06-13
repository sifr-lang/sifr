# Review: `wave_clone_1` Iterator/Comprehension Ownership Correction — Pass 2

Phase: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`
Scope: `wave_clone_1` only, post-pass-1 follow-up fixes (revised pass-1 findings incorporated)
Reviewer: pass-2
Date: 2026-03-21

---

## Summary

`wave_clone_1` is in production-grade shape. The pass-1 actionable fix was applied (FINDING-3: `YieldMode::Clone` unit test added). All pass-1 findings are correctly handled: one fixed, the rest deferred. Validation passes across all lanes. **One HIGH finding** identified: `Type::ownership()` incorrectly returns `Move` for all tuples, including all-Copy tuples, causing unnecessary field clones in tuple homogeneous iteration. One MEDIUM finding. One LOW cosmetic finding. No regressions.

---

## Pass-1 Follow-Up Status

### FINDING-3: Missing `YieldMode::Clone` planner unit test — FIXED

`iterator_plan_clones_named_move_element_collection` added to `helpers.rs` tests (lines 773-785). Confirmed present in current codebase:

```rust
#[test]
fn iterator_plan_clones_named_move_element_collection() {
    let source = HirExpr::Name {
        name: "strings".to_string(),
        ty: Type::List(Box::new(Type::Str)),
    };
    let plan = plan_iterator_ownership(&source);
    assert_eq!(plan.value_category, ValueCategory::Place);
    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Clone);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Move));
}
```

Test count increased from 15 (pre-pass-1) to 17. All 17 pass.

### FINDING-1 (pass-1 original): `YieldMode::Borrow` in simple-lowering path — CORRECTLY DEFERRED

`apply_simple_copy_clone_yield_mode` (lower_expr.rs:609-626) and `apply_copy_clone_yield_mode_for_ir` (stmt_support_emitter.rs:4903-4919) both handle `Borrow` as `Move | Borrow => iter_expr`. Conservative and sound. Correctly deferred to `wave_clone_3`.

### FINDING-2 (pass-1 original): `HirExpr::TupleLiteral` in `is_reusable_place_expr` — CORRECTLY DEFERRED

`is_reusable_place_expr` (helpers.rs:38-54) handles `Name`, `FieldAccess`, and `Index` but not `TupleLiteral`. Falls back to `Temporary` safely. Correctly deferred to `wave_clone_3`.

---

## Findings (Severity-Ordered)

### FINDING-H1 [HIGH]: `Type::ownership()` returns `Move` for all tuples, including all-Copy tuples

**File:** `crates/sifr_type_system/src/types.rs:462`

**Description:** `Type::ownership()` groups `Self::Tuple(_)` with all other non-Copy types in the `Move` arm:

```rust
// types.rs:462 — current
Self::Str
| Self::Bytes
| Self::Any
| Self::List(_)
| Self::Dict(_, _)
| Self::Set(_)
| Self::Tuple(_)   // ← Returns Move unconditionally
| Self::Iterable(_)
| Self::Iterator(_) => OwnershipKind::Move,
```

In Rust, tuples are `Copy` when all their elements are `Copy`. For example, `(i64, f64)` is `Copy` in Rust, but `Type::ownership()` returns `OwnershipKind::Move` for it.

**Downstream impact:** In `plan_iterator_ownership_with_element_hint`, the planner derives `element_ownership` from `iteration_metadata().element_type.ownership()`. For `Vec<(i64, i64)>`:
- `element_type` = `(i64, i64)` (a `Type::Tuple`)
- `Type::ownership()` returns `OwnershipKind::Move` for the tuple
- `infer_yield_mode` with `element_ownership = Some(Move)` in `Preserve` mode produces `YieldMode::Clone`
- Tuple homogeneous iteration (`registry_tuple_homogeneous_iter_expr` in `intrinsic_method_emitters.rs:197` and `stmt_support_emitter.rs`) then emits `.clone()` on each field unconditionally

The problematic pattern in `registry_tuple_homogeneous_iter_expr`:
```rust
// intrinsic_method_emitters.rs:197 (stmt_support_emitter.rs mirrors this)
match yield_mode {
    crate::helpers::YieldMode::Copy | crate::helpers::YieldMode::Move => field_expr,
    crate::helpers::YieldMode::Clone | crate::helpers::YieldMode::Borrow => {
        crate::RustExpr::MethodCall {  // ← Unnecessary clone for (i64, i64)
            receiver: Box::new(field_expr),
            method: "clone".to_string(),
            args: vec![],
        }
    }
}
```

**Correct behavior:** `Vec<(i64, i64)>` should produce `YieldMode::Copy`, meaning tuple fields are passed through without cloning.

**Remediation (concrete):**

1. Replace `Self::Tuple(_)` in the `Move` arm with an explicit arm:

```rust
Self::Tuple(elems) => {
    if elems.iter().all(|e| e.ownership() == OwnershipKind::Copy) {
        OwnershipKind::Copy
    } else {
        OwnershipKind::Move
    }
}
```

2. Add `types.rs` unit tests:
```rust
#[test]
fn type_ownership_tuple_all_copy_is_copy() {
    assert_eq!(
        Type::Tuple(vec![Type::Int, Type::Float]).ownership(),
        OwnershipKind::Copy
    );
}
#[test]
fn type_ownership_tuple_with_move_is_move() {
    assert_eq!(
        Type::Tuple(vec![Type::Int, Type::Str]).ownership(),
        OwnershipKind::Move
    );
}
```

3. Add planner regression test in `helpers.rs`:
```rust
#[test]
fn iterator_plan_copies_tuple_of_copy_elements() {
    let source = HirExpr::Name {
        name: "pairs".to_string(),
        ty: Type::List(Box::new(Type::Tuple(vec![Type::Int, Type::Int]))),
    };
    let plan = plan_iterator_ownership(&source);
    assert_eq!(plan.yield_mode, YieldMode::Copy);
}
```

**Regression risk:** Low. No codegen path depends on the current Move-for-all-tuples behavior. Fix is additive and semantically correct.

---

### FINDING-M1 [MEDIUM]: `sorted`/`rev` on named collections use preserve mode unnecessarily

**File:** `crates/sifr_codegen/src/intrinsic_method_emitters.rs:2036-2042` (sorted), `1963` (rev fallback)

**Description:** `sorted(nums)` on a named `Vec<T>` emits `(nums).iter().copied().collect::<Vec<_>>()` instead of using direct consumption. `sorted` always produces a new collection, so the original container is never reused after the call, making the preserve-mode borrow redundant.

**Root cause:** `registry_iterable_to_owned_iter_expr` uses `plan.source_access_mode` to decide between `iter()` and `into_iter()`. For named places, this is `Preserve`. The `sorted` implementation calls `collect::<Vec<_>>()` on the borrowed iter.

**Impact:** Extra indirect borrow + copy for every `sorted` call. Performance only, not correctness.

**Remediation:** Add a dedicated `registry_sorted_iter_expr` that always uses `Consume` mode, or add a special case in the sorted/rev emitter. Deferred to `wave_clone_3`.

---

### FINDING-M2 [MEDIUM]: `normalize_for_iter_expr` `.cloned()` → identity is obsolete for planner paths

**File:** `crates/sifr_codegen/src/lower_stmt.rs:2050-2083`

**Description:** The normalization matches `vec![...].iter().cloned().collect::<Vec<_>>()` → `vec![...]`. The planner now emits `.copied()` for Copy-element collections, so this pattern never fires for planner-derived iterators. It remains useful as a belt-and-suspenders pass for non-planner codegen paths.

**Remediation:** Add a comment documenting this, or add equivalent `.copied()` → identity normalization for completeness. Low effort, deferred to `wave_clone_3`.

---

### OBS-1 [LOW]: `.copied()` before `.collect()` redundancy not normalized

**Location:** `stmt_support_emitter.rs:5188-5197` and `lower_stmt.rs:2050-2083`

**Description:** Both normalization functions strip redundant `.cloned()` when followed by `.collect()` but neither strips redundant `.copied()`. Planner-generated `.copied().collect()` chains are not normalized.

**Regression risk:** None. Functionally correct. Deferred to `wave_clone_3` as optional cleanup.

---

## Deferred Items (Not Wave-1 Scope)

| Item | Deferred To | Reason |
|---|---|---|
| `Type::ownership()` for `Tuple` (FINDING-H1) | **wave_clone_1 follow-up** | Fix before wave_clone_2 to avoid compounding debt |
| `sorted`/`rev` preserve-mode overhead (FINDING-M1) | `wave_clone_3` | Performance only; not correctness |
| `normalize_for_iter_expr` .cloned() normalization comment (FINDING-M2) | `wave_clone_3` | Low effort doc fix |
| `.copied().collect()` redundancy (OBS-1) | `wave_clone_3` | Cosmetic; functionally correct |
| `YieldMode::Borrow` path verification | `wave_clone_3` | Conservative pass-through is safe |
| `HirExpr::TupleLiteral` in `is_reusable_place_expr` | `wave_clone_3` | Falls back to Temporary safely |
| `TypeVar`/`Any` conservative handling | `wave_clone_3` | No change needed for concrete types |
| `Copy`-element indexing/safe-indexing `.clone()` | `wave_clone_2` | Indexing paths not yet refactored |
| Star-unpack whole-source clone | `wave_clone_2` | Star-unpack not yet refactored |
| Stepped slicing clone/copy | `wave_clone_2` | Slicing paths not yet refactored |

---

## Regression Risk Assessment

| Area | Risk | Evidence |
|---|---|---|
| Named `Place` collections not silently consumed | **None** | Borrow checker rejects `borrow_escape_store.sifr` |
| Temporary collections consumed directly | **None** | `vec![...].into_iter().map(...)` in emit |
| `Copy` element paths use copy-oriented iteration | **None** | `nums.iter().copied()` verified |
| `Clone` element paths use `.cloned()` when needed | **None** | New planner unit test passing |
| Range loops have no ownership noise | **None** | Direct range in control flow demo emit |
| Unit tests | **None** | 17 helpers tests, all pass |
| E2E fixtures | **None** | 24 pass fixtures, all pass |
| Quick validation suite | **None** | `scripts/run_all_tests.sh --profile quick` — PASS |
| Full validation suite | **None** | `scripts/run_all_tests.sh` — PASS |
| Borrow checker invariant | **None** | `borrow_escape_store.sifr` still rejected |
| Emit output: no clone-heavy patterns | **None** | Zero clone-heavy patterns in wave-1 emit |
| HIR maintainability guardrails | **None** | Pass |

---

## Verification Commands Run

```
cargo test -p sifr_codegen -- helpers::tests         # 17 tests, all pass
cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_1_iterator_comprehension_ownership.sifr | grep -c '\.clone()\.into_iter\|\.iter()\.cloned'  # 0 matches
cargo run -q -p sifr -- emit demos/milestone_control_flow_demo.sifr | grep -c '\.clone()\.into_iter\|\.iter()\.cloned'  # 0 matches
scripts/run_all_tests.sh --profile quick             # PASS (24 e2e fixtures, 0 failures, signature e1bf653aaa770517)
scripts/run_all_tests.sh                             # PASS (full suite)
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/borrow_escape_store.sifr  # expected compile failure
```

---

## Conclusion

`wave_clone_1` is **conditionally production-grade ready**. The pass-1 actionable fix was applied. All original pass-1 findings are correctly resolved (one fixed, two deferred with sound conservative behavior). Ownership semantics are correct across all concrete-type planner axes.

**One HIGH finding remains**: `Type::ownership()` returns `Move` for all tuples, including all-Copy tuples, causing unnecessary field clones in tuple homogeneous iteration. This is a quality issue with a straightforward fix and should be addressed before `wave_clone_2` lands to avoid compounding debt. It does not block merge but should be fixed in a quick follow-up PR.

**Recommended next action**: Fix FINDING-H1 (`Type::ownership()` tuple arm), add the corresponding unit and planner regression tests, then run full validation before proceeding to `wave_clone_2`.
