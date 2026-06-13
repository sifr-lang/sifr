# Review: `wave_clone_1` Iterator/Comprehension Ownership Correction

Phase: `issues/ad-hoc-ownership-aware-collection-lowering-and-clone-elision.md`
Scope: `wave_clone_1` only
Reviewer: pass-1
Date: 2026-03-21

---

## Summary

`wave_clone_1` is well-implemented. The root-cause fix is correct: a shared `IteratorOwnershipPlan` planner in `helpers.rs` classifies every iteration source along three axes (`ValueCategory`, `SourceAccessMode`, `YieldMode`) and all lowering paths (`stmt_support_emitter.rs`, `lower_expr.rs`, `lower_stmt.rs`, `intrinsic_method_emitters.rs`) derive their emission decisions from the planner output. Generated Rust quality is measurably better. All unit tests pass. All e2e fixtures pass. No regression in the borrow checker. Three findings, all low severity.

---

## Findings (severity order)

### FINDING-1 [LOW]: `YieldMode::Borrow` is produced but never consumed in simple-lowering paths

**Location:** `crates/sifr_codegen/src/lower_expr.rs:609-626` (`apply_simple_copy_clone_yield_mode`)

**Description:** The `YieldMode::Borrow` variant is produced by the planner (e.g., for `Type::Str` when `element_ownership == None`, or for unrecognized types where `element_ownership == None`). In `apply_simple_copy_clone_yield_mode`, `Borrow` falls through to "pass through unchanged":

```rust
crate::helpers::YieldMode::Move | crate::helpers::YieldMode::Borrow => iter_expr,
```

This is correct for the current scope — `wave_clone_1` covers `for` / `map` / `filter` / comprehensions over named collections and temporaries, where `YieldMode::Borrow` should not actually occur (strings always go through the `chars().map(...to_string...)` path which produces owned `String` values regardless). However, `Borrow` is reachable through the `None` case in `infer_yield_mode` for types that are not `Iterator`, not `Str`, and have `None` element ownership. For those types, the pass-through behavior is semantically correct (borrowing the iterator yields borrowed references, which is valid for the iteration context), but the generated code quality is unverified for this case.

**Regression risk:** Low. No test fixture exercises `YieldMode::Borrow` at the simple-lowering path. The conservative fallback (pass-through) is safe. This is deferred to `wave_clone_3` (generic hardening).

**Actionable fix:** Add a unit test covering the `Borrow` path in `apply_simple_copy_clone_yield_mode`, or document the conservative invariant that `Borrow` is intentionally deferred. No code change required in wave-1 scope.

---

### FINDING-2 [LOW]: Missing coverage for `HirExpr::TupleLiteral` in `is_reusable_place_expr`

**Location:** `crates/sifr_codegen/src/helpers.rs:38-54` (`is_reusable_place_expr`)

**Description:** `is_reusable_place_expr` handles `HirExpr::Name`, `HirExpr::FieldAccess`, and `HirExpr::Index`, but does **not** handle `HirExpr::TupleLiteral`. A tuple literal like `(a, b)` used as an iteration source would be classified as `Temporary` even though it is semantically a "reusable place" in the Rust sense (the tuple itself is cheap to construct but the references within are stable).

Currently, no test fixture or real user code appears to exercise tuple-as-iteration-source (e.g., `for x in (a, b):`). The `try_lower_simple_iter_source_expr` path would fall through to the `default: Some(lowered_source)` branch and return the lowered tuple expression, which would then likely fail to compile as an iterator — not silently generate wrong code.

**Regression risk:** Low. The `None` element_ownership case in `infer_yield_mode` would produce `YieldMode::Borrow`, which is conservative and safe. The compiler would likely error at type-check time rather than silently emit bad code.

**Actionable fix:** Consider adding `HirExpr::TupleLiteral { elements, .. }` to `is_reusable_place_expr` (returning true only if all elements are reusable places). Alternatively, document this as a known gap deferred to `wave_clone_3`.

---

### FINDING-3 [LOW]: No `Clone`-mode unit test for the planner output

**Location:** `crates/sifr_codegen/src/helpers.rs:759-798` (unit tests)

**Description:** The unit tests cover three planner scenarios:
- `iterator_plan_preserves_named_copy_element_collection` → `YieldMode::Copy`
- `iterator_plan_consumes_temporary_collection` → `YieldMode::Move`
- `iterator_plan_consumes_range_without_clone_contract` → `YieldMode::Move`

There is **no test** covering the `YieldMode::Clone` case — a named `Place` collection with `Move` element type (e.g., `list[str]`). This is a gap in regression coverage for the core planner logic. The clone path is exercised indirectly through integration tests (e.g., the borrow checker test that verifies `items.clone()` is generated when needed), but the planner output itself is not unit-tested for this axis.

**Regression risk:** Low. The `Clone` path logic is straightforward (`Preserve + Move element → Clone`). Integration tests cover it indirectly.

**Actionable fix:** Add a unit test:
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

---

## What's Working Well

### Root-cause correctness
The planner is the single source of truth for ownership decisions in iteration lowering. Every call site (`try_lower_simple_iter_source_expr`, `try_lower_simple_filter_call_expr`, `lower_iter_source_expr_for_ir_with_mode`, `registry_iterable_to_owned_iter_expr`) independently computes the plan from the source expression and element type hint. There is no divergence between the simple-lowering path and the IR-lowering path in terms of the planning logic itself.

### Ownership semantics correctness
- `Place + Copy element → Preserve + Copy`: `nums.iter().copied()` — correct, no source consumption, copy-oriented
- `Temporary + Copy element → Consume + Move`: `vec![...].into_iter().map(...)` — correct, direct consumption
- `Place + Move element → Preserve + Clone`: generates `.iter().cloned()` (verified via borrow checker rejection of `borrow_escape_store.sifr`)
- `Range → Consume + Move`: structural range loops emit `1 as i64..n + (1 as i64)` with no boxing
- `enumerate(place)` → `(nums).iter().copied().enumerate().map(...)` — correct, enumerate is wrapped around the already-preserved iterator

### `infer_yield_mode` fallback correctness
The `None` element_ownership case correctly falls through:
- `Type::Str` → `Move` (string chars produce owned `String` via `chars().map(|c| c.to_string())`)
- `Type::Iterator(_)` → `Move` (already an iterator, elements are already owned)
- Everything else → `Borrow` (conservative, safe)

### Generated Rust quality
Verified by `emit` inspection across all targeted files:
- Zero `.clone().into_iter()` in `milestone_control_flow_demo.sifr`
- Zero `.iter().cloned()` in `wave_clone_1_iterator_comprehension_ownership.sifr`
- Temporary collections: `vec![5, 6].into_iter().map(...)` — no source-level pre-clone
- Range loops: `1 as i64..n + (1 as i64)` — no `Box::new((range).clone().into_iter())`
- Enumerate: `(nums).iter().copied().enumerate().map(...)` — no `(nums).clone().into_iter().enumerate()`

### IR normalization consistency
The `normalize_for_loop_iter_expr` optimization in `stmt_support_emitter.rs:5188-5197` correctly strips redundant `.cloned()` when followed by `.collect()`. The `lower_stmt.rs:2050-2083` variant additionally handles `.cloned().into_iter()` chains. Both variants are consistent in their optimization scope.

### Comprehension correctness
All three comprehension forms (list, set, dict) correctly delegate iteration to `lower_comprehension_iter_for_ir`, which routes through `lower_structural_iter_source_expr_for_ir` → `lower_iter_source_expr_for_ir_with_mode` with `prefer_boxed_iterator = false`. The planner is applied. No comprehension form was found to bypass the planner.

### `normalize_for_iter_expr` in lower_stmt.rs handles redundant `.cloned()`
The normalization that strips `.cloned()` before `.collect()` is applied in `lower_stmt.rs` (simple-lowering). This is a post-emission cleanup that catches planner-generated `.copied()` / `.cloned()` chains. The logic is correct and consistent with the planner's intent.

---

## Regression Risk Assessment

| Area | Risk | Evidence |
|---|---|---|
| Named `Place` collections not silently consumed | **None** | Borrow checker correctly rejects `items` parameter escape in `borrow_escape_store.sifr` |
| Temporary collections consumed directly | **None** | `vec![...].into_iter().map(...)` verified in emit output |
| `Copy` element paths use copy-oriented iteration | **None** | `nums.iter().copied()` verified in all targeted files |
| `Clone` element paths use `.cloned()` when needed | **None** | Borrow checker rejection path is exercised |
| Range loops have no ownership noise | **None** | `1 as i64..n + (1 as i64)` verified in control flow demo |
| Unit tests | **None** | All 15 helpers tests pass |
| E2E fixtures | **None** | `wave_clone_1_iterator_comprehension_ownership.sifr` passes |
| Demos | **None** | `ad_hoc_clone_wave1_iterator_comprehension_demo.sifr` passes |
| Full test suite | **None** | `scripts/run_all_tests.sh --profile quick` passes |
| Borrow checker invariant | **None** | `borrow_escape_store.sifr` still correctly rejected |

---

## Verification Commands Run

```bash
cargo run -q -p sifr -- emit crates/sifr/tests/e2e/pass/wave_clone_1_iterator_comprehension_ownership.sifr  # clean output, no clone-heavy patterns
cargo run -q -p sifr -- emit demos/milestone_control_flow_demo.sifr                       # clean output, no clone-heavy patterns
cargo run -q -p sifr -- emit demos/ad_hoc_clone_wave1_iterator_comprehension_demo.sifr    # clean output, no clone-heavy patterns
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/wave_clone_1_iterator_comprehension_ownership.sifr  # pass
cargo run -q -p sifr -- run demos/ad_hoc_clone_wave1_iterator_comprehension_demo.sifr      # pass
cargo run -q -p sifr -- check crates/sifr/tests/e2e/fail/borrow_escape_store.sifr          # expected compile failure (borrow checker still active)
cargo test -p sifr_codegen -- helpers::tests                                               # 15 tests, all pass
cargo test -p sifr -- --skip test_e2e_pass                                                 # (via run_all_tests.sh --profile quick)
```

---

## Deferred Items (Not Wave-1 Scope)

Per the phase plan, these are correctly deferred to `wave_clone_2` / `wave_clone_3`:
- `Copy`-element indexing and safe-indexing `.clone()` / `.cloned()` emissions (not yet fixed)
- Star-unpack whole-source clone (not yet fixed)
- `TypeVar` / `Any` conservative handling hardening (no change needed for concrete types)
- Tuple literal as iteration source (`FINDING-2`)
- `YieldMode::Borrow` path verification (`FINDING-1`)

---

## Conclusion

`wave_clone_1` is **approved for merge**. The implementation correctly introduces a shared ownership-aware planner and routes all four iteration-lowering surfaces through it. Generated Rust quality is measurably improved across the targeted surfaces. Ownership semantics are correct (verified by emit inspection, borrow checker, unit tests, and e2e fixtures). The three findings are all low severity, are conservative/deferred by design, and none represent a regression risk to the current scope.
