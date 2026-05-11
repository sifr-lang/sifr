

## Review: milestone_async_4 closure slice

### Files Reviewed

1. **demos/m32_ownership_concurrency_demo.sifr** (new untracked file)
2. **verification/validation_lanes/quick_e2e_manifest.json** (staged changes)
3. **internal_docs/phases/32_async_ecosystem.md** (staged changes)

---

### 1. Is the demo a valid and honest milestone_async_4 demo?

**Yes.** The demo exercises all three ownership/concurrency scenarios required by milestone_async_4:

- **Completed same-task mutable borrow before await**: `append_before_await(local_items)` mutates the list, then `await task.sleep(0.0)` follows with no live borrow. Verified in generated Rust: `append_before_await(&mut local_items)` is a synchronous call before the `tokio::time::sleep(...).await` point.

- **Owned task-boundary inputs**: `scope.spawn(count_items([1, 2, 3]))` moves a concrete `Vec<i64>` into the child task. Verified in generated Rust: `count_items(vec![1, 2, 3])` is passed by value.

- **Immutable shared task inputs via `sifr.sync.Shared`**: `scope.spawn(read_shared(shared_value))` with `Shared[int]` demonstrates the canonical immutable sharing path. Verified in generated Rust: `read_shared(shared_value)` with `Shared::new(41)` construction.

The demo does not reach for non-send, borrowed, or scoped-borrow paths — it correctly exercises the positive scenarios.

---

### 2. Is it acceptable that task result values are observed but not unwrapped/asserted?

**Yes.** `await Task[T, E]` returns `TaskResult[T, E]`, which is an affine composite carrying either `Ok(T)`, `Err(E)`, or `Cancelled(CancellationError)`. The demo's `_owned_result` and `_immutable_shared_result` bindings consume the handles (as required by affine semantics) and the `TaskResult` types are correct in the generated Rust:

```rust
let _owned_result: __SifrTaskResult<i64, std::convert::Infallible> = owned_handle.join().await;
let _immutable_shared_result: __SifrTaskResult<i64, std::convert::Infallible> = shared_handle.join().await;
```

Both children are infallible (`std::convert::Infallible` error channel), so `Ok(value)` is guaranteed at scope exit. Asserting or destructuring the result is not necessary for validation correctness. Naming the bindings with `_` prefix is appropriate — they confirm the handles are consumed and the scope exit does not panic, but the concrete values are not the point.

The design is consistent with the `TaskResult` surface as documented in the phase doc (locked decision 5: `await Task[T, E]` produces `TaskResult[T, E]`).

---

### 3. Does quick-lane placement cover milestone_async_4 positive validation adequately?

**Yes.** The manifest adds exactly two fixtures to complete the positive coverage:

| Fixture | Previously in quick lane | Now in quick lane |
|---|---|---|
| `spawn_owned_send_value` | No | **Yes (added)** |
| `spawn_owned_move_value` | Yes | Yes |
| `spawn_capture_immutable_shared_ok` | Yes (via #1967) | Yes |
| `await_without_live_borrow` | No | **Yes (added)** |

All three milestone_async_4 positive validation fixtures are now in the quick pass lane. Confirmed by running all three directly:

```
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/spawn_owned_send_value.sifr  → no errors found
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/spawn_capture_immutable_shared_ok.sifr  → no errors found
cargo run -q -p sifr -- check crates/sifr/tests/e2e/pass/await_without_live_borrow.sifr  → no errors found
```

And run to completion:
```
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_owned_send_value.sifr   → cache_hit
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_capture_immutable_shared_ok.sifr  → cache_hit
cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/await_without_live_borrow.sifr  → cache_hit
```

Negative validation fixtures remain in the full suite (e.g., `spawn_non_send_field_rejected`, `spawn_borrowed_value_escapes_rejected`, `borrow_across_await_rejected`) — all still correctly reject with Sifr diagnostics.

---

### 4. Is the phase doc wording accurate for a closure slice?

**Yes.** The implementation progress line reads:

> In progress ownership/concurrency closure slice: the remaining milestone positive fixtures are in the quick lane and `demos/m32_ownership_concurrency_demo.sifr` exercises owned spawn inputs, immutable shared task inputs, and completed same-task mutable borrows before await.

This is accurate:
- "In progress" correctly reflects unmerged state
- "remaining milestone positive fixtures" refers to the two fixtures added to the quick lane (`spawn_owned_send_value`, `await_without_live_borrow`) — `spawn_capture_immutable_shared_ok` was already present
- The demo filename is correct
- The three exercises (owned spawn, immutable shared, completed same-task borrow) match the demo's content exactly

The doc does not mark the milestone complete, correctly deferring that to a tracker PR after this implementation PR merges.

---

### 5. Correctness bugs, missing tests, or docs issues?

**No correctness bugs.** All checks pass:

- `cargo run -q -p sifr -- run demos/m32_ownership_concurrency_demo.sifr` → cache_hit (runs to completion)
- `cargo run -q -p sifr -- emit demos/m32_ownership_concurrency_demo.sifr` → clean codegen output with correct async function signatures, `__SifrTaskScope`, `__SifrTask`, `__SifrTaskResult` types
- Negative validation fixtures still correctly reject: `spawn_non_send_field_rejected` → `SIFR-OWN-0010`, `spawn_borrowed_value_escapes_rejected` → borrow diagnostic, `borrow_across_await_rejected` → borrow diagnostic
- `cargo clippy --workspace -- -D warnings` → clean
- `cargo fmt --check` → clean
- `python3 scripts/check_hir_maintainability_guardrails.py` → PASS
- `git diff --check` → clean (no whitespace errors)

**No missing tests.** The milestone_async_4 positive coverage is complete with the two new quick-lane entries. No fixture gaps identified.

**No docs issues.** The phase doc line is accurate and correctly scoped.

---

VERDICT: SATISFIED
