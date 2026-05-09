

## Phase 32 milestone_async_2 Runtime Bootstrap Review

### Scope Verification

The PR is scoped to:
- Auto-detect `async def main()` and emit Tokio bootstrap
- Wire Tokio dependency only when async entrypoints present
- Mechanical HIR guardrail splits

**Verified:**
- `annotate_async_main_entrypoint()` in codegen correctly detects `async fn main()` and inserts `#[tokio::main(flavor = "current_thread")]`
- Tokio added to `required_crates` only when `has_async_main_entrypoint` is true
- `test_async_main_entrypoint_gets_tokio_bootstrap_dependency` and `test_sync_main_does_not_require_tokio` unit tests verify the conditional behavior
- `test_generate_cargo_toml_required_tokio_uses_runtime_features` verifies dependency spec
- HIR code properly split: `async_await.rs`, `async_with.rs`, `external_defs.rs`
- HIR guardrails pass

### Blockers

**1. `Option<&Box<T>>` Clippy Lint (new violation introduced)**
- `crates/sifr_hir/src/lower/async_with.rs:21` has `Option<&Box<Expr>>`
- Clippy `-D warnings` fails due to `clippy::borrowed_box`
- The pattern is new to this file (moved from `statements.rs`, but introduced as function signature)
- **Fix:** Change to `Option<&Expr>` or `Option<&Box<Expr>>` with `as_ref()` call

**2. Pre-existing e2e failure: `nested_function_nonlocal_accumulator`**
- Confirmed failing on main branch, unrelated to this PR
- Not a blocker but should be tracked separately

### What Works

- Async runtime bootstrap with `current_thread` Tokio flavor
- Conditional Tokio dependency (no dependency added for sync code)
- Proper guardrail splits with no functionality change
- Unit tests pass
- E2E `async_runtime_bootstrap.sifr` passes
- `async_basic.sifr` e2e passes

### Verdict: **NOT SATISFIED**

**Reason:** One blocker: the `Option<&Box<Expr>>` parameter in the moved function signature introduces a new `clippy::borrowed_box` lint violation that causes `-D warnings` clippy to fail. The function must be fixed before merge.
