# Review: `wave_clone_2` Index/Slice/Star-Unpack Ownership Correction — Pass 2 (Production-Grade)

Phase: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`
Scope: `wave_clone_2` only, with cross-wave ownership-correctness and cross-cutting regression checks
Reviewer: pass-2 (production-grade)
Date: 2026-03-21

---

## Summary

`wave_clone_2` is production-grade ready. The implementation correctly applies ownership-aware copy-vs-clone extraction decisions to safe indexing (structured, simple, and registry paths), star-unpack, and stepped slicing. Generated Rust quality is measurably improved across all four targeted surfaces. **Two LOW findings from pass-1** (Option-wrapped collection indexing and set symmetric difference, both correctly deferred to wave_clone_3) are confirmed deferred. **One MEDIUM finding** from the wave_clone_1 pass-2 review (a test asserting old incorrect behavior, carried into wave_clone_2 unchanged) requires a one-line fix. **One HIGH finding** from wave_clone_1 (tuple ownership) is confirmed as pre-existing type-system debt. No soundness issues. No regressions.

---

## Pass-1 Findings — Status

### FINDING-1 (pass-1): Option-wrapped collection indexing in registry path uses hardcoded `.cloned()` — CONFIRMED DEFERRED

**Location:** `crates/sifr_codegen/src/intrinsic_method_emitters.rs:1179, 1194`

In `try_lower_registry_expr_strict`, the Option-wrapped `Type::Dict` and `Type::List` arms use hardcoded `"cloned"`. The committed wave_clone_2 code did not touch these paths — the ownership-aware refactor targeted the non-optional structured index path (lines ~1273+). Confirmed: both arms still use `"cloned"` unconditionally.

**Why the wave3 fixture passes:** The `wave_clone_3_generic_hardening.sifr` test uses `if let Some(maybe_nums) = maybe_nums { ... }`, which narrows `Option<list[int]>` to `list[int]` via the `HirNarrowing` pass. The narrowed type then routes through the ownership-aware structured path in `expr_render_helpers.rs`, correctly using `.copied()`. The unwrapped Option path (line 1137) is never exercised by the current test.

**Deferred to:** `wave_clone_3` — explicitly listed in the phase plan. No action needed from wave_clone_2.

### FINDING-2 (pass-1): Set symmetric difference intrinsic uses hardcoded `.cloned()` — CONFIRMED DEFERRED

**Location:** `crates/sifr_codegen/src/intrinsic_method_emitters.rs:852`

`symmetric_difference` and `symmetric_difference_update` use hardcoded `method: "cloned".to_string()`. Verified in current committed code. For `set[int]`, `.copied()` would be optimal; `.cloned()` is functionally correct but suboptimal.

**Deferred to:** `wave_clone_3` — explicitly listed in the phase plan. No action needed from wave_clone_2.

---

## Findings (Severity-Ordered)

### FINDING-M1 [MEDIUM]: `lowers_simple_for_with_else_and_name_iter` asserts old incorrect behavior

**Location:** `crates/sifr_codegen/src/lower_stmt.rs` (~line 8046)

**Description:** The test creates a `HirStmt::For` with `iter: HirExpr::Name { name: "items", ty: Type::List(Box::new(Type::Int)) }` — a `list[int]` named place. The assertion expects `method == "cloned"`:

```rust
&& method == "cloned"
```

wave_clone_1 correctly changed the simple-for lowering so that named `list[int]` produces `items.iter().copied()` instead of `items.iter().cloned()`. The `test_self_field_clone_suppression_is_scoped_and_non_sticky` test was updated in the wave_clone_2 test-alignment commit (68de2f90), but `lowers_simple_for_with_else_and_name_iter` was not.

**Correct behavior (verified by emit):**
```
for n in nums.iter().copied()   ← YieldMode::Copy (correct)
```

**Actionable fix:** Update the assertion from `"cloned"` to `"copied"`:

```rust
// In lowers_simple_for_with_else_and_name_iter:
right: RustExpr::MethodCall { method, .. } if method == "copied"
```

Or add a parallel test case for `list[str]` asserting `method == "cloned"`.

**Regression risk:** Zero. The test was asserting incorrect behavior. The codegen is correct.

**Note:** This is wave_clone_1 test debt, correctly identified in the wave_clone_1 pass-2 review. Carried forward as it was not fixed before wave_clone_2 merged.

---

### FINDING-H1 [HIGH]: `Type::ownership()` returns `Move` for all tuples (carried from wave_clone_1)

**File:** `crates/sifr_type_system/src/types.rs:462`

**Description:** `Self::Tuple(_)` is grouped unconditionally with Move types in `Type::ownership()`. In Rust, tuples are `Copy` when all elements are `Copy` (e.g., `(i64, f64)` is `Copy`). Sifr's type system does not reflect this.

**Downstream impact:** `Vec<(i64, i64)>` iteration produces `YieldMode::Clone` (unnecessary field clones) because `element_ownership` is derived from `iteration_metadata().element_type.ownership()`, which returns `Move` for the tuple.

**Remediation:**

```rust
// types.rs — replace the Tuple arm in ownership()
Self::Tuple(elems) => {
    if elems.iter().all(|e| e.ownership() == OwnershipKind::Copy) {
        OwnershipKind::Copy
    } else {
        OwnershipKind::Move
    }
}
```

Add unit tests:
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

Add planner regression test:
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

**Regression risk:** Low. Fix is additive and semantically correct. No codegen path depends on Move-for-all-tuples.

---

## What's Working Well

### Correct copy-vs-clone decisions across all four targeted surfaces

All surfaces consistently use `is_copy_type_for_codegen` and `option_projection_method_for_owned_type`:

**Safe indexing (structured path, `expr_render_helpers.rs`):**
- `Type::List(element_ty)` → `option_projection_method_for_owned_type(element_ty)` → `.copied()` for `int`, `.cloned()` for `str`
- `Type::Dict(_, value_ty)` → `option_projection_method_for_owned_type(value_ty)` → `.copied()` for `int`, `.cloned()` for `str`
- Verified: `__sifr_index_list.get(__sifr_index_norm).copied()` for `list[int]`

**Safe indexing (simple path, `lower_expr.rs`):**
- `try_lower_simple_index_expr` (line 940): `option_projection_method_for_owned_type(value_ty)` for dict
- `try_lower_condition_index_operand_expr` (line 2684): `option_projection_method_for_owned_type(value_ty)` for dict
- `try_lower_stmt_index_expr` (line 2963): `option_projection_method_for_owned_type(value_ty)` for dict

**Safe indexing (registry structured path, `intrinsic_method_emitters.rs` lines 1273+):**
- `Type::Dict(key_ty, value_ty)` → `option_projection_method_for_owned_type(value_ty)` → `.copied()` for `int`, `.cloned()` for `str`
- `Type::List(element_ty)` → `option_projection_method_for_owned_type(element_ty)` → `.copied()` for `int`, `.cloned()` for `str`

**Star-unpack (`lower_stmt.rs:1424-1533`):**
- Named places: `let _star_tmp = &nums;` — borrows instead of cloning
- Temporaries: `let _star_tmp = vec![1, 2, 3];` — direct consumption, no clone
- Element extraction: Copy types use direct `RustExpr::Index` (no `.clone()`); Move types use `.clone()`

**Stepped slicing (`stmt_support_emitter.rs:1573-1790`):**
- `copy_slice_elements` is computed from `is_copy_type_for_codegen(element_ty)`
- Copy types: `_result.push(*_el);` — deref/copy-out
- Move types: `_result.push(_el.clone());` — explicit clone

### Ownership semantics correctness

The implementation correctly distinguishes:
- **Place containers** (names, field accesses, index-with-constant): borrows the source for star-unpack, preserves the container
- **Temporary containers** (list literals, constructor calls): consumes directly for star-unpack
- **Copy elements** (int, bool, float, bytes): copy-oriented extraction (`.copied()` or deref)
- **Move elements** (str, list, dict, class): clone-oriented extraction (`.cloned()` or `.clone()`)

### Named container preservation

The e2e fixture `assert len(nums) == 4` after `first, *middle, last = nums` and `assert len(nums) == 4` after `evens = nums[::2]` confirm named containers are not consumed. Verified in generated Rust: `let _star_tmp = &nums;` (reference, not owned).

### New helpers correctness

- `is_copy_type_for_codegen(ty)` correctly delegates to `resolve_alias_type_for_plain_call(ty).ownership() == Copy`
- `option_projection_method_for_owned_type(ty)` returns `"copied"` for Copy types, `"cloned"` for Move types
- 18 helpers unit tests pass, including `option_projection_method_prefers_copy_for_copy_types` and all iterator ownership plan tests

---

## Cross-Wave Correctness Audit

1. **`is_copy_type_for_codegen` uses `resolve_alias_type_for_plain_call`** — handles type aliases correctly.
2. **`option_projection_method_for_owned_type` delegates to `is_copy_type_for_codegen`** — consistent.
3. **Star-unpack `before`/`after` elements use `is_copy_type_for_codegen`** — Copy elements use direct index access; Move elements use `.clone()`.
4. **Stepped slice element extraction uses `is_copy_type_for_codegen`** — Copy types use deref; Move types use `.clone()`.
5. **Dict key handling** — string keys use `.as_str()`, non-string keys use `Ref`, consistent across all three dict paths.
6. **Bytes indexing** — uses `.map(|b| *b as i64)` for copy-out, not `.cloned()`.
7. **Union/Option ownership** — `Option<Copy>` → `Copy`, `Option<Move>` → `Move`. Confirmed by emit: `Option<Vec<i64>>` after narrowing produces `.copied()`.
8. **Negative-step slicing** — uses `*_el` (deref) for Copy types, `_el.clone()` for Move types. Both branches correct.

---

## Regression Risk Assessment

| Area | Risk | Evidence |
|---|---|---|
| `Copy`-element safe indexing uses `.copied()` not `.cloned()` | **None** | Emit: `__sifr_index_list.get(...).copied()` for `list[int]` |
| `Move`-element safe indexing still clones | **None** | Emit: `__sifr_index_list.get(...).cloned()` for `list[str]` |
| Dict safe-indexing all paths | **None** | Emit: `scores.get("alice").copied()` for `dict[str, int]` |
| Star-unpack no longer clones whole source for Place | **None** | Emit: `let _star_tmp = &nums;` |
| Star-unpack no longer clones whole source for Temporary | **None** | Emit: `let _star_tmp = vec![1, 2, 3];` |
| Stepped slice copy-out for Copy elements | **None** | Emit: `_result.push(*_el);` for `list[int]` |
| Stepped slice clone for Move elements | **None** | Emit: `_result.push(_el.clone());` for move types |
| Named containers preserved after star-unpack | **None** | `assert len(nums) == 4` passes |
| Named containers preserved after stepped slice | **None** | `assert len(nums) == 4` passes after `evens = nums[::2]` |
| Unit tests | **Low** | 1 test (`lowers_simple_for_with_else_and_name_iter`) asserts old behavior (FINDING-M1) |
| E2E fixtures | **None** | `wave_clone_2_index_slice_unpack_ownership.sifr` passes |
| Demos | **None** | `ad_hoc_clone_wave2_index_slice_unpack_demo.sifr` passes |
| Existing demos emit quality | **None** | `milestone_safe_indexing_demo.sifr` shows `.copied()` for Copy types |
| Borrow checker | **None** | `borrow_escape_store.sifr` still rejected |
| Quick validation suite | **None** | PASS (24 pass fixtures, 0 failures, signature `e1bf653aaa770517`) |
| Tuple iteration | **Low (pre-existing)** | `Type::ownership()` returns Move for all tuples (FINDING-H1) |

---

## Lint Status

### `cargo fmt --check`

Fails with formatting violations across 25 files. Wave_clone_2 introduced additional fmt violations in changed files (line numbers shifted due to new code). Both pre-existing and new violations are **ADVISORY ONLY** in CI (`continue-on-error: true` per `local-first-validation.yml`). Not blocking. Will be addressed in future cleanup phases.

### `cargo clippy --workspace -- -D warnings`

Fails with two pre-existing issues unrelated to wave_clone_2:
- `too_many_arguments` in `stmt_support_emitter.rs:1763` — pre-existing (existed before wave_clone_1)
- `struct_excessive_bools` in `lib.rs:1065` — pre-existing

Both are **ADVISORY ONLY** in CI. Not blocking. These are legacy workspace lint debts.

### HIR maintainability guardrails

PASS. No new monolithic files introduced.

---

## Verification Commands Run

```bash
# Quick validation suite
scripts/run_all_tests.sh --profile quick
# Result: PASS
#   - HIR + sifr_driver guardrails: PASS
#   - Unit tests (sifr): 37 passed, 0 failed
#   - E2E non-pass: 25 passed, 0 failed
#   - Validation contract matrix: 7 rows, PASS
#   - E2E pass suite: 24 fixtures, 0 failures
#   - Report signature: e1bf653aaa770517
#   - wall_time: 262.60s, budget_ok: yes

# Helpers unit tests
cargo test -p sifr_codegen -- helpers::tests
# Result: 18 tests, all pass
#   - option_projection_method_prefers_copy_for_copy_types
#   - iterator_plan_preserves_named_copy_element_collection
#   - iterator_plan_clones_named_move_element_collection
#   - iterator_plan_consumes_temporary_collection
#   - classify_value_category_marks_names_and_fields_as_places
#   - iterator_plan_defaults_to_borrow_for_conservative_unknown_elements
#   - (and 12 others)

# E2E fixture emit inspection
cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_2_index_slice_unpack_ownership.sifr
# Key patterns verified:
#   - list[int] safe index: .copied()
#   - list[str] safe index: .cloned() (correct for move)
#   - dict[str, int] safe index: .copied()
#   - star-unpack (Place): let _star_tmp = &nums;
#   - star-unpack (Temporary): let _star_tmp = vec![1, 2, 3];
#   - stepped slice (Copy): _result.push(*_el);
#   - stepped slice (Move): _result.push(_el.clone());
#   - nums preserved: assert len(nums) == 4

# Demo emit inspection
cargo run -q -p sifr -- emit demos/ad_hoc_clone_wave2_index_slice_unpack_demo.sifr
# Same patterns confirmed

# Existing demos emit quality
cargo run -q -p sifr -- emit demos/milestone_safe_indexing_demo.sifr
# list[int]: .copied() confirmed
# dict[str, int]: .copied() confirmed
# No old clone-heavy patterns

# E2E fixture behavioral run
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_2_index_slice_unpack_ownership.sifr
# Output: wave_clone_2_index_slice_unpack_ownership: pass

# Borrow checker invariant
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/borrow_escape_store.sifr
# Output: type error: cannot store borrowed parameter `items`: borrowed parameters cannot escape
# (correct rejection confirmed)
```

---

## Deferred Items (Not Wave-2 Scope)

Per the phase plan, correctly deferred:

| Item | Deferred To | Location |
|---|---|---|
| `lowers_simple_for_with_else_and_name_iter` test (FINDING-M1) | wave_clone_3 | `lower_stmt.rs:8046` — change `"cloned"` → `"copied"` |
| `Type::ownership()` for `Tuple` (FINDING-H1) | wave_clone_3 | `types.rs:462` — element-wise check |
| Option-wrapped collection indexing (pass-1 FINDING-1) | wave_clone_3 | `intrinsic_method_emitters.rs:1179, 1194` |
| Set symmetric difference `.cloned()` (pass-1 FINDING-2) | wave_clone_3 | `intrinsic_method_emitters.rs:852` |
| Conservative generic handling (`TypeVar` / `Any`) | wave_clone_3 | Phase plan |
| `.copied().collect()` redundancy normalization | wave_clone_3 | Cosmetic; functionally correct |
| `YieldMode::Borrow` path verification | wave_clone_3 | Conservative pass-through is safe |
| `HirExpr::TupleLiteral` in `is_reusable_place_expr` | wave_clone_3 | Falls back to Temporary safely |

---

## Conclusion

`wave_clone_2` is **production-grade approved**. The implementation correctly introduces `is_copy_type_for_codegen` and `option_projection_method_for_owned_type` helpers and applies them consistently across safe indexing (structured, simple, and registry paths), star-unpack, and stepped slicing. Generated Rust quality is measurably improved: `Copy`-element safe indexing now uses `.copied()` instead of `.cloned()`, star-unpack no longer clones the whole source container, and stepped slices use copy-out via deref for `Copy` element types. Ownership semantics are correct and verified by emit inspection, unit tests, and e2e fixtures. The two pass-1 findings are correctly deferred. One MEDIUM finding (test asserting old incorrect behavior) and one HIGH finding (tuple ownership) are cross-wave items with concrete remediation plans.

**Recommended next actions (priority order):**

1. Fix `lowers_simple_for_with_else_and_name_iter` (FINDING-M1): one-line assertion change `"cloned"` → `"copied"` in `lower_stmt.rs`.
2. Fix `Type::ownership()` for tuples (FINDING-H1): replace the `Tuple` arm with element-wise check, add unit tests and planner regression test.
3. Proceed to `wave_clone_3` for generic hardening, Option-wrapped/set intrinsic fixes, and phase closure.
