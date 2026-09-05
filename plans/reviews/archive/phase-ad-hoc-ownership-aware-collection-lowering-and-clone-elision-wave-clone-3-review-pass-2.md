# wave_clone_3 Review — Production-Grade Review Pass 2

**Date**: 2026-03-21
**Commit range**: `b4ae56cb` — "wave_clone_3: harden generic iterator ownership and tuple copy semantics (#1402)" through `c19f9c4d` — "wave_clone_3: apply review pass 1 invariants doc note (#1403)"
**Phase**: `ad-hoc-ownership-aware-collection-lowering-and-clone-elision`
**Reviewer**: agent (external review, pass 2)

---

## 0. Executive Summary

**Decision**: APPROVED — production-ready.

All wave_clone_3-specific changes are sound, correctly reasoned, and fully validated. The implementation closes the three identified gaps from earlier waves (unsound `.copied()` for `Any`/`TypeVar`, incorrect tuple `Copy` derivation, tuple literal misclassification) without introducing any regressions. The pre-existing follow-up actions from pass 1 have been addressed. There are no high or medium risk items.

---

## 1. Scope of Changes

| Commit | Change |
|--------|--------|
| `b4ae56cb` | Core wave_clone_3 implementation: `is_conservative_element_type`, hardened `iteration_element_ownership`, simplified `plan_iterator_ownership_with_element_hint`, `HirExpr::TupleLiteral` arm in `is_reusable_place_expr`, `Type::Tuple` arm in `ownership()` |
| `c19f9c4d` | Apply pass 1 observation 1: doc comment on `is_conservative_element_type` |

Files modified: `crates/sifr_codegen/src/helpers.rs`, `crates/sifr_type_system/src/types.rs`
New artifacts: `wave_clone_3_generic_hardening_ownership.sifr` (E2E), `ad_hoc_clone_wave3_generic_hardening_demo.sifr` (demo), `wave_clone_3_generic_hardening_traceability.md` (traceability)

---

## 2. Root-Cause Correctness

### 2.1 Tuple Ownership Derivation — `Type::Tuple` arm in `ownership()`

**Location**: `crates/sifr_type_system/src/types.rs` lines 464–473

```rust
Self::Tuple(elems) => {
    if elems.iter().all(|elem| elem.ownership() == OwnershipKind::Copy) {
        OwnershipKind::Copy
    } else {
        OwnershipKind::Move
    }
}
```

**Assessment**: Correct and complete. The implementation precisely matches Rust's tuple `Copy` semantics: a tuple is `Copy` iff all its elements are `Copy`. The recursive call to `elem.ownership()` correctly handles nesting (e.g., `tuple[tuple[int, int], str]` → `Move` via nested evaluation). This was the root cause of unnecessary clone emissions for `list[tuple[int, int]]` in earlier waves.

**Edge cases handled correctly**:
- Empty tuple `tuple[()]` — `elems.is_empty()` → `elems.iter().all(...)` vacuously returns `true` → `Copy`. In Rust, the empty tuple `()` is indeed `Copy`. ✓
- Nested tuples — recursion through `elem.ownership()` handles arbitrarily deep nesting. ✓
- Union members — `Type::Union`/`Intersection` in `ownership()` checks `any(...Move...)` for Move derivation, consistent with the tuple element-level check. ✓

### 2.2 Conservative Element Type Detection — `is_conservative_element_type`

**Location**: `crates/sifr_codegen/src/helpers.rs` lines 69–83 (nested function inside `iteration_element_ownership`)

```rust
fn is_conservative_element_type(ty: &Type) -> bool {
    match ty.resolve_alias() {
        Type::Any | Type::Unknown => true,
        Type::Union(members) | Type::Intersection(members) => {
            members.iter().any(is_conservative_element_type)
        }
        _ => false,
    }
}
```

**Assessment**: Correct. The function returns `true` precisely for element types where ownership cannot be proven soundly from iteration metadata:
- `Any`: dynamic, no compile-time ownership guarantee
- `Unknown`: unresolved type, conservatively Move
- Unions/intersections containing any of the above

`TypeVar` is intentionally excluded — it is handled by `Type::ownership()` which returns `Move` (conservative). This separation of concerns is documented in the doc comment added in `c19f9c4d`.

**Soundness argument**: If `is_conservative_element_type` ever returns `false` for a type that should be treated conservatively, the planner would emit `.copied()` or `.cloned()` on potentially non-`Copy` data, leading to unsound Rust code. The function is correct because it uses a whitelist approach (only `Any`/`Unknown`/containing-unions are conservative) rather than a blacklist.

### 2.3 Iteration Element Ownership — `iteration_element_ownership`

**Location**: `crates/sifr_codegen/src/helpers.rs` lines 68–95

```rust
fn iteration_element_ownership(source_ty: &Type) -> Option<OwnershipKind> {
    source_ty
        .resolve_alias()
        .iteration_metadata()
        .and_then(|metadata| {
            if is_conservative_element_type(&metadata.element_type) {
                None
            } else {
                Some(metadata.element_type.ownership())
            }
        })
}
```

**Assessment**: Correct. The function chains two fallible operations:
1. `iteration_metadata()` returns `None` for types without iteration support (heterogeneous tuples, arbitrary `Class` types, etc.)
2. `is_conservative_element_type` returns `false` for types with proven element ownership

When either chain link is `None`, the planner defaults to `YieldMode::Borrow` (conservative).

**Complete decision table**:

| Source type | Element type | `iteration_metadata` | `is_conservative` | Result | Yield |
|---|---|---|---|---|---|
| `list[int]` | `Int` | Some | false | Some(Copy) | Copy |
| `list[str]` | `Str` | Some | false | Some(Move) | Clone |
| `list[Any]` | `Any` | Some | true | None | Borrow |
| `list[TypeVar]` | `TypeVar` | Some | false | Some(Move) | Clone |
| `list[tuple[int,int]]` | `tuple[int,int]` | Some | false | Some(Copy) | Copy |
| `list[tuple[int,str]]` | `tuple[int,str]` | Some | false | Some(Move) | Clone |
| `tuple[int,str]` (direct iter) | heterogeneous | None | N/A | None | Borrow |
| `Class` (no `__iter__`) | N/A | None | N/A | None | Borrow |

All rows produce the expected yield mode. ✓

### 2.4 Element Hint Ignored — `plan_iterator_ownership_with_element_hint`

**Location**: `crates/sifr_codegen/src/helpers.rs` lines 142–160

```rust
pub(crate) fn plan_iterator_ownership_with_element_hint(
    source_expr: &HirExpr,
    element_type_hint: Option<&Type>,
) -> IteratorOwnershipPlan {
    let source_ty = crate::resolve_alias_type_for_plain_call(source_expr.ty());
    let inferred_element_ownership = iteration_element_ownership(source_ty);
    let _ = element_type_hint; // intentionally unused — element hints cannot soundly override iteration metadata
    let element_ownership = inferred_element_ownership;
    // ...
}
```

**Assessment**: Correct. The element type hint is explicitly discarded with `let _ = element_type_hint`. This is the right decision: forcing copy/clone behavior from an element hint without corresponding iteration metadata (e.g., iterating a bare `TypeVar` with a hint of `int`) would be unsound. The planner correctly stays conservative. The comment makes intent explicit.

### 2.5 Tuple Literal Value Category — `HirExpr::TupleLiteral` arm in `is_reusable_place_expr`

**Location**: `crates/sifr_codegen/src/helpers.rs` lines 52–55

```rust
HirExpr::TupleLiteral { elements, ty } => {
    ty.resolve_alias().ownership() == OwnershipKind::Copy
        && elements.iter().all(is_reusable_place_expr)
}
```

**Assessment**: Correct and sound. A tuple literal is a reusable place only when:
1. The tuple type is `Copy` (guaranteed by the `Type::Tuple` ownership arm above)
2. All elements are reusable places (preventing move-type elements from being treated as reusable)

Without this check, a tuple literal like `(a: int, b: str)` could incorrectly be classified as `Place`, leading to lowering that reuses the tuple binding and causes double-drop. The check prevents this. ✓

**Important note on iteration over tuple literals**: `HirExpr::TupleLiteral` in the source is typically classified as `Temporary` (not `Place`) because tuple literals contain `IntLiteral`, `StringLiteral`, etc., not `Name` nodes. Therefore, direct iteration over tuple literals uses `SourceAccessMode::Consume` and `YieldMode::Move`. This is correct behavior — iterating a literal tuple should consume it.

---

## 3. Iterator Lowering Behavior

### 3.1 Consumption in `lower_homogeneous_tuple_iter_expr`

**Location**: `crates/sifr_codegen/src/stmt_support_emitter.rs` lines 4998–5043

The tuple iteration lowering uses `source_access_mode` and `yield_mode` from the plan:
- `SourceAccessMode::Preserve` → `.clone()` the tuple source before destructuring
- `SourceAccessMode::Consume` → use source directly
- `YieldMode::Copy | Move` → field access (no clone)
- `YieldMode::Clone | Borrow` → `.clone()` each field

**Assessment**: Correct. The tuple binding and field extraction respect the ownership plan. For `list[tuple[int, int]]`, this produces `.iter().copied()` with no per-field clones. ✓

### 3.2 Consumption in `lower_iter_source_expr_for_ir_with_mode`

**Location**: `crates/sifr_codegen/src/stmt_support_emitter.rs` lines 5046–5234

The main iterator lowering path handles:
- `list[int]` (preserve) → `.iter().copied()` ✓
- `list[Any]` (preserve) → `.iter()` ✓
- `list[TypeVar]` (preserve) → `.iter().cloned()` ✓
- `tuple[int, int]` (preserve) → `lower_homogeneous_tuple_iter_expr` ✓
- `Type::Class` (preserve/consume) → `__iter__()` call chain ✓
- `dict[int]` (preserve) → `.keys().copied()` ✓
- `str` (preserve) → `.chars().map(...)` ✓
- Heterogeneous tuples → falls through to `fallback_iter_expr()` (direct use) ✓

### 3.3 Consumption in `lower_for_loop_iter_expr`

**Location**: `crates/sifr_codegen/src/lower_stmt.rs` lines 2187–2385

Same plan consumption logic as `stmt_support_emitter.rs`. The `Type::Tuple` case (lines 2295–2337) mirrors the `stmt_support_emitter.rs` implementation, using `iter_plan.yield_mode` and `iter_plan.source_access_mode` to determine field cloning vs. direct access and source cloning vs. direct consumption. ✓

---

## 4. Tuple Copy Semantics — Verification of the Complete Chain

The complete chain from source type to emitted Rust for `list[tuple[int, int]]`:

```
Sifr: for pair in pairs:  (where pairs: list[tuple[int, int]])
  │
  ├─ plan_iterator_ownership(source_expr: Name("pairs"), hint=None)
  │   ├─ resolve_alias_type → Type::List(Box::new(Type::Tuple([Int, Int])))
  │   ├─ iteration_element_ownership(Type::List(...))
  │   │   ├─ iteration_metadata → Some(IterationMetadata { element_type = Tuple([Int, Int]) })
  │   │   ├─ is_conservative_element_type(Tuple([Int, Int])) → false  (not Any/Unknown/union)
  │   │   └─ Some(Tuple([Int, Int]).ownership()) = Some(Copy)
  │   │       └─ Tuple([Int, Int]).ownership()
  │   │           ├─ all elements Copy? Int.ownership() == Copy, Int.ownership() == Copy
  │   │           └─ → Copy
  │   ├─ source_access_mode = Preserve  (Name → Place)
  │   ├─ yield_mode = Copy  (Preserve + Some(Copy))
  │   └─ IteratorOwnershipPlan { value_category: Place, source_access_mode: Preserve, yield_mode: Copy, element_ownership: Some(Copy) }
  │
  └─ lower_iter_source_expr_for_ir_with_mode
      └─ Type::List → .iter().copied()
```

**Result**: `for pair in pairs.iter().copied()` — exactly correct. ✓

For `list[Any]`:

```
Sifr: for _v in anys:  (where anys: list[Any])
  │
  ├─ iteration_element_ownership(Type::List(Box::new(Any)))
  │   ├─ iteration_metadata → Some(IterationMetadata { element_type = Any })
  │   ├─ is_conservative_element_type(Any) → true
  │   └─ None  (conservative deferral)
  ├─ source_access_mode = Preserve
  ├─ yield_mode = Borrow  (Preserve + None → Borrow)
  └─ IteratorOwnershipPlan { ..., yield_mode: Borrow, element_ownership: None }
      └─ .iter()  (no .copied()/.cloned())
```

**Result**: `for _v in anys.iter()` — exactly correct. ✓

---

## 5. Regression Coverage

### 5.1 Unit Tests

All 25 helper tests pass, including 7 wave_clone_3-specific additions:

| Test | Validates |
|------|-----------|
| `classify_value_category_treats_copy_tuple_literal_of_places_as_place` | `(a: int, b: bool)` → Place |
| `classify_value_category_treats_move_tuple_literal_as_temporary` | `(a: int, b: str)` → Temporary |
| `iterator_plan_copy_hint_does_not_force_unknown_source_to_copy` | `Type::Any` + hint → Borrow |
| `iterator_plan_preserved_list_any_uses_borrow_not_clone` | `list[Any]` → Borrow |
| `iterator_plan_typevar_hint_stays_conservative` | `TypeVar` + hint → Borrow |
| `iterator_plan_list_typevar_uses_clone_yield` | `list[TypeVar]` → Clone |
| `iterator_plan_copies_tuple_of_copy_elements` | `list[tuple[int,int]]` → Copy |

Both type system tests pass:
- `test_tuple_ownership_all_copy_is_copy` — `tuple[int, float]` → Copy
- `test_tuple_ownership_with_move_is_move` — `tuple[int, str]` → Move

### 5.2 E2E Fixtures

All wave_clone E2E fixtures pass (24 total, including wave_clone_0 through wave_clone_3):
- `wave_clone_0_architecture_lock.sifr` → PASS
- `wave_clone_1_iterator_comprehension_ownership.sifr` → PASS
- `wave_clone_2_index_slice_unpack_ownership.sifr` → PASS
- `wave_clone_3_generic_hardening_ownership.sifr` → PASS
- `ad_hoc_clone_wave3_generic_hardening_demo.sifr` → PASS

### 5.3 Quick Validation Profile

```
Validation lane report
  profile=quick
  e2e=compile=670ms plan=3ms build=23775ms run=2523ms cache_hits=0/6
  24 pass tests completed (24 passed, 0 failed)
  wall_time=85.60s cpu=70.71s
```

All 24 E2E pass fixtures compile, build, and run correctly.

### 5.4 Emitted Rust Verification

Confirmed from `wave_clone_3_generic_hardening_ownership.sifr`:

```rust
// Tuple of copy elements → .iter().copied()
for pair in pairs.iter().copied() { ... }

// Any element type → .iter() (no .cloned()/.copied())
for _v in anys.iter() { ... }
```

Confirmed from demo:
```rust
// Mixed tuple in list → .iter().copied()
for pair in pairs.iter().copied() { ... }

// Any list → .iter() (no clone)
for _value in mixed.iter() { ... }
```

---

## 6. Architecture Consistency

wave_clone_3 is fully consistent with the architecture documented in `internal_docs/architecture.md`:
- Planner contract (value category, source access mode, yield mode) is maintained
- The four yield modes (`Copy`, `Clone`, `Move`, `Borrow`) are correctly selected by the decision tree
- Conservative generic handling for `TypeVar`/`Any`/move unions is enforced
- Tuple ownership correctly propagates element ownership through the type system
- The wave_clone_3 traceability doc accurately records all validation evidence

The phase execution checklist in `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision-execution.md` is correctly updated with wave_clone_3 status and review pass 1 application notes.

---

## 7. Pre-Existing Issues (Unchanged by wave_clone_3)

### 7.1 E2E Fixture: `phase_psp_iter_fix_7_user_defined_iterable_protocol.sifr`

Still produces `error[E0515]` — dangling reference from `values` local variable returned via `Box::new((values).iter().copied())`. Confirmed pre-existing at wave_clone_2 (`56267838`). Unrelated to wave_clone_3.

### 7.2 8 Pre-Existing Failing Unit Tests

Confirmed failing at both wave_clone_2 (`56267838`) and wave_clone_3 (`c19f9c4d`):

```
hir_analysis::queries::tests::collect_mutated_vars_ignores_nested_function_scope
lib_codegen_tests::test_generate_rust_multi_assembles_single_rust_file
lib_codegen_tests::test_generate_rust_iterable_binding_from_iterator_materializes_once
lib_codegen_tests::test_generate_rust_iterable_return_from_iterator_materializes_for_signature
lib_codegen_tests::test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs
lib_codegen_tests::test_structured_stmt_path_wraps_non_optional_string_index_into_option_local
lib_codegen_tests::test_stmt_path_handles_recursive_nested_function_with_structured_captures
lower_stmt::tests::lowers_simple_for_with_dict_iter_to_keys_cloned
```

526 tests pass (up from 519 at wave_clone_2 — the 7 additional passing tests are the wave_clone_3 unit tests). The 8 failures existed before wave_clone_3 and are tracked separately.

### 7.3 Clippy Warnings

Pre-existing pedantic warnings in files not modified by wave_clone_3:
- `struct_excessive_bools` in `lib.rs:1065`
- `too_many_arguments` in `lower_stmt.rs:2003`

### 7.4 Pass 1 Observation 3 (Typos)

Pass 1 noted a typo in `wave_clone_0_architecture_lock.sifr` print statement. Upon re-examination, the file at HEAD (`c19f9c4d`) contains no print statement — it uses `assert_eq` calls. The pass 1 observation appears to have been erroneous (possibly referring to a file version or a stale analysis artifact). No action needed.

---

## 8. Risk Assessment

| Risk | Likelihood | Impact | Assessment |
|------|-----------|--------|------------|
| Unsound `.copied()` for `list[Any]` | Eliminated | High | Fixed by `is_conservative_element_type` |
| Unsound `.copied()` for `list[Unknown]` | Eliminated | High | Fixed by `is_conservative_element_type` |
| Unsound `.copied()` via union containing `Any` | Eliminated | High | Fixed by `is_conservative_element_type` recursion |
| Incorrect tuple `Copy` derivation | Eliminated | High | Fixed by `Type::Tuple` ownership arm |
| Element hints overriding conservative planner | Eliminated | Medium | Fixed by discarding hints |
| Tuple literal misclassified as reusable place | Eliminated | Medium | Fixed by ownership check in `is_reusable_place_expr` |
| Regression in existing iterator lowering | Low | Medium | 25 unit tests + 24 E2E fixtures + quick profile all pass |

**Overall risk**: Negligible. wave_clone_3 is purely additive hardening that removes unsafe optimizations rather than introducing new lowering paths.

---

## 9. Conclusion

wave_clone_3 is **production-ready** and ready for closure.

The implementation correctly and completely addresses all three gap categories identified for this wave:
1. Conservative ownership planning for `Any`/`Unknown`/`TypeVar` element types
2. Correct tuple ownership derivation from element ownership
3. Hardened tuple literal value category classification

All validation gates pass. The code is well-structured, sound, and consistent with the architecture. The pre-existing follow-up actions from pass 1 have been addressed.

**Action items for follow-up** (unrelated to wave_clone_3):
1. Fix the dangling-reference bug in `phase_psp_iter_fix_7_user_defined_iterable_protocol.sifr` codegen (separate issue)
2. Fix the 8 pre-existing failing unit tests (separate issues)
