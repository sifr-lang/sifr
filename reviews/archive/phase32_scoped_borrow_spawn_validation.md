

Review complete.

**Summary:**

The slice is correctly scoped. I verified:

1. **Fixture** (`spawn_scoped_borrow_deferred.sifr`): Pattern is correct — `async def worker(items: list[int])` called as `worker(items)` inside `scope.spawn()`, with `items` borrowed from the `launch()` parameter. The `expect-error[col=30]: SIFR-TYPE-0002` directive targets the `scope.spawn(...)` call site. The diagnostic correctly fires: `scope.spawn() cannot move borrowed parameter 'items' across a task boundary`.

2. **Phase doc** (`32_async_ecosystem.md`): The new tracker line (587) accurately describes the slice intent: explicit fail fixture for the deferred v1 scoped-borrow model. The fixture is already listed in `milestone_async_4` negative validation at line 576.

3. **Implementation note accuracy**: Confirmed — no compiler changes were made. The existing borrow-across-spawn validation from prior Phase 32 work (PRs #1957, #1959, #1961) correctly catches this pattern.

4. **Validation results**:
   - `cargo test -p sifr_hir scope_spawn`: 3 tests pass
   - `cargo test -p sifr --test e2e test_e2e_fail`: 308 pass (includes new fixture)
   - `cargo run -q -p sifr -- check ...`: SIFR-TYPE-0002 fires at correct location with exit code 1
   - `git diff --check`: No whitespace errors

**SATISFIED**
