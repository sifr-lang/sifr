

## Review: Phase 32 Spawn Move-Boundary Validation Slice

---

## Review Decision: **SATISFIED**

---

## 1. Spawn Move-Boundary Validation Behavior

**HIR test** (`expressions_tests.rs`): The `test_scope_spawn_consumes_owned_move_argument` test correctly validates that after `scope.spawn(worker(items))` moves `items`, the subsequent `items.append(2)` triggers `OWN_USE_AFTER_MOVE`. Diagnostic code, message, and primary range are all correctly asserted.

**Codegen test** (`lib_codegen_tests.rs`): The `test_scope_spawn_lowers_owned_move_coroutine_arguments` test correctly verifies that list literals passed to `own` parameters lower through `__sifr_spawn_infallible(worker(vec![...]))`.

**Pass fixture** (`spawn_owned_move_value.sifr`): An owned list is moved into a spawned coroutine with no subsequent use of the original binding — correctly passes.

**Fail fixture** (`spawn_mutable_alias_rejected.sifr`): `items.append(2)` after `scope.spawn(worker(items))` is correctly rejected.

---

## 2. Quick-Lane Batch Harness Support for Async Result Fixtures

**Manifest update** (`quick_e2e_manifest.json`): `spawn_owned_move_value` is now included in the quick lane. This directly addresses the prior review's residual risk.

**Harness fix** (`e2e.rs`): The `build_group_sources` refactor correctly:
1. Replaces the fragile `make_entry_function_public` approach with a wrapper function pattern.
2. Adds `__SifrBatchTermination` trait with implementations for `()` (no-op) and `Result<(), E>` (exits with code 1 on `Err`).
3. Wraps each fixture's entry point in `wrapper_fn` which calls `__sifr_finish(entry_fn())`.

**Type safety of the wrapper approach**: The wrapper function is only generated after `build_rust_source_from_module` succeeds, which requires `fn main(` to exist in the generated Rust. The type system then guarantees the return type matches a supported `__SifrBatchTermination` impl.

**Fixture failure semantics preserved**: For `spawn_owned_move_value.sifr`:
- Compiles: `async fn main() -> Result<(), ScopeFailure>`
- Wrapper calls `__sifr_finish` on `Result<(), ScopeFailure>` → `Ok` is no-op, `Err` exits with code 1
- Result: correct behavior for both outcomes

**Unit test** (`test_batch_group_dispatch_uses_entry_termination_trait`): Correctly verifies the generated main contains `impl<E: std::fmt::Debug> __SifrBatchTermination for Result<(), E>`, `__sifr_finish` calls, and no `pub async fn` in the entry wrapper.

---

## 3. Phase Tracker Update

`internal_docs/phases/32_async_ecosystem.md:585` correctly shows "In progress" for the spawn move-boundary validation slice.

---

## Validation Results

| Command | Result |
|---------|--------|
| `cargo test -p sifr_hir scope_spawn -- --nocapture` | ✓ PASS |
| `cargo test -p sifr_codegen scope_spawn -- --nocapture` | ✓ PASS |
| `cargo run -q -p sifr -- run .../spawn_owned_move_value.sifr` | ✓ PASS |
| `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` | ✓ PASS |
| `cargo test -p sifr --test e2e test_batch_group_dispatch_uses_entry_termination_trait -- --nocapture` | ✓ PASS |
| `scripts/run_all_tests.sh --profile quick` | ✓ PASS |

---

## Residual Risks

None. The prior review's identified gap (fixture not in quick lane) has been addressed. The harness fix correctly implements batch termination for async Result fixtures without altering pass/fail semantics.

---

**SATISFIED**
