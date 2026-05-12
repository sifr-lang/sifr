

## Code Review: `asyncio.create_task()` Veneer Implementation

### Files Reviewed
- `crates/sifr_hir/src/lower/imports.rs`
- `crates/sifr_hir/src/lower/task_calls.rs`
- `crates/sifr_hir/src/lower/task_scope_calls.rs`
- `lib/sifr/asyncio.sifr`
- `crates/sifr/tests/e2e/pass/asyncio_create_task_subset.sifr`
- `crates/sifr/tests/e2e/fail/asyncio_create_task_outside_scope_rejected.sifr`

---

### What's Correct

**1. Import Registration** (`imports.rs:47-52`)
- `create_task` correctly added to `asyncio_compat_imports` alongside other asyncio compatibility items
- Supports both direct import and aliased import (`create_task as ct`)

**2. Lowering Path** (`task_calls.rs:32`)
- `create_task` routes through `lower_asyncio_create_task_call()` as expected

**3. Scope Detection** (`task_calls.rs:37-58`)
- Uses `active_bindings()` to enumerate task scope bindings
- Pattern `let [(scope_name, scope_ty)] = ... else` correctly handles:
  - **Zero scopes**: rejected with diagnostic
  - **Multiple scopes**: rejected with same diagnostic (exactly-one requirement)
- Creates synthetic `HirExpr::Name` for the scope object to pass to canonical path

**4. Canonical Path Reuse** (`task_calls.rs:56`)
- Calls `lower_task_scope_spawn_from_object()` which is the **exact same canonical path** used by:
  - `scope.spawn()`
  - `group.spawn()`
- This ensures all canonical semantics are preserved:
  - Coroutine-only validation
  - Proper `Task[T, E]` type construction
  - TaskGroup homogeneous error type checking
  - TaskGroup open-state validation
  - Handle observation tracking

**5. Diagnostic Quality**
- Message: `"asyncio.create_task() requires exactly one active task scope; use it inside async with task.scope() or task.TaskGroup()"`
- Clear, actionable, specifies both valid patterns

---

### Verified Behaviors

| Test | Result | Notes |
|------|--------|-------|
| Pass test compiles | ✓ | `asyncio_create_task_subset.sifr` |
| Pass test runs | ✓ | Returns correct value |
| Fail test triggers diagnostic | ✓ | `[type] asyncio.create_task() requires...` |
| Zero scopes → rejected | ✓ | Correct diagnostic |
| Multiple nested scopes → rejected | ✓ | Correct diagnostic |
| `TaskGroup` scope → works | ✓ | Routes through canonical path |
| Aliased import → works | ✓ | `create_task as ct` resolved |
| Non-coroutine arg → rejected | ✓ | Via canonical path |
| Sync function → rejected | ✓ | Via canonical path |
| `cargo fmt` | ✓ | Clean |
| HIR guardrails | ✓ | `check_hir_maintainability_guardrails.py` PASS |

---

### Non-Blocking Observations

**1. Pre-existing e2e_pass failure** (unrelated to this implementation)
- Test `async_with_return_cleanup.sifr` fails with `compile_error!("structured statement emission missing for production path: TryFinally")`
- Confirmed this failure exists on main branch before this change
- Not a blocker for this review

**2. Missing test coverage for alias** (acceptable)
- No explicit test for `from sifr.asyncio import create_task as ct`
- Verified manually it works; alias resolution in `resolve_imports_early()` handles this correctly
- Could add but not required for this phase

---

### Design Intent Verification

The implementation correctly honors the stated design intent:

> "This intentionally does not introduce ambient/orphan tasks or a second runtime model."

✓ The implementation enforces exactly one active scope, preventing orphan tasks.

✓ The implementation reuses the canonical task model rather than creating parallel paths.

---

### Missing Test Cases (Suggestions, Not Blockers)

While not blockers, consider adding:
1. Explicit pass test using `create_task` with `TaskGroup` (currently uses `group.spawn()`)
2. Explicit fail test for multiple active scopes (behavior verified, no dedicated test)

However, these are suggestions rather than requirements since:
- The canonical path is shared and well-tested
- Fail test for zero scopes covers the rejection mechanism
- Manual verification confirmed multiple scopes also rejected

---

### Verdict

**SATISFIED**

The implementation:
1. ✓ Correctly routes `asyncio.create_task()` through the canonical task-spawn path
2. ✓ Requires exactly one active scope binding
3. ✓ Rejects calls outside scope with clear diagnostic
4. ✓ Preserves all canonical validation semantics
5. ✓ Reuses existing validated code (no duplication)
6. ✓ Has adequate test coverage for pass and fail scenarios
7. ✓ Passes all formatting and linting checks

No concrete blockers identified.

**SATISFIED**
