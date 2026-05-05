# Review: semantic-diagnostic-code-taxonomy diag-9 slice 5
## milestone_diag_9 — FLOW break/continue primary ranges

**Reviewer:** Claude Code review pass 1 retry
**Status:** SATISFIED — no blockers

---

## Summary

Attaches HIR primary ranges to `SIFR-FLOW-0001` (`break` outside loop) and `SIFR-FLOW-0002` (`continue` outside loop) by threading real AST statement ranges through the diagnostic helpers and call sites.

---

## Changes reviewed

### `crates/sifr_hir/src/lower/flow_diagnostics.rs`

Both helpers now take `TextRange` and use `error_with_code_at` instead of the spanless `error_with_code`:

```rust
// before
pub(super) fn break_outside_loop(ctx: &mut LowerCtx) {
    ctx.error_with_code(DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP, ...);
}

// after
pub(super) fn break_outside_loop(ctx: &mut LowerCtx, range: TextRange) {
    ctx.error_with_code_at(DiagnosticCode::FLOW_BREAK_OUTSIDE_LOOP, ..., range);
}
```

Same pattern for `continue_outside_loop`.

### `crates/sifr_hir/src/lower/statements.rs` (lines 207–220)

Call sites extract the range from the AST node and pass it forward:

```rust
Stmt::Break(break_stmt) => {
    if !ctx.in_loop() {
        super::flow_diagnostics::break_outside_loop(ctx, break_stmt.range());
        return None;
    }
    Some(HirStmt::Break)
}
Stmt::Continue(continue_stmt) => {
    if !ctx.in_loop() {
        super::flow_diagnostics::continue_outside_loop(ctx, continue_stmt.range());
        return None;
    }
    Some(HirStmt::Continue)
}
```

Note: `break_stmt` and `continue_stmt` are `&StmtBreak` / `&StmtContinue` which implement `ruff_text_size::Ranged`, so `.range()` returns `TextRange`.

### `crates/sifr_hir/src/lower/expressions_tests.rs` (lines 1949–1972)

Tests updated to assert `primary_range` matches the keyword span:

```rust
&& e.primary_range == Some(range_for(source, "break"))  // for break test
&& e.primary_range == Some(range_for(source, "continue"))  // for continue test
```

---

## Review criteria — each verified

### 1. Helpers use `error_with_code_at` and require `TextRange`

**VERIFIED.** Both `break_outside_loop` and `continue_outside_loop` in `flow_diagnostics.rs` now:
- Accept a `range: TextRange` parameter
- Call `ctx.error_with_code_at(code, message, range)` (NOT the spanless `error_with_code`)

The `error_with_code_at` implementation (`mod.rs:247`) correctly sets `primary_range: Some(range)`.

### 2. Statement lowering passes `.range()` from AST

**VERIFIED.** Call sites in `statements.rs:209` and `:216` pass:
- `break_stmt.range()` for the `break` diagnostic
- `continue_stmt.range()` for the `continue` diagnostic

These are real `TextRange` values from the AST, not synthetics.

### 3. Range choice is semantically appropriate; no spanless production path

**VERIFIED.** The range covers the `break` / `continue` keyword itself, which is:
- Semantically correct: the keyword is the syntactic construct that is erroneous
- Unique entry point: `FLOW_BREAK_OUTSIDE_LOOP` and `FLOW_CONTINUE_OUTSIDE_LOOP` are defined **only** in `flow_diagnostics.rs` and called **only** from `statements.rs` — no other code path can emit these diagnostics
- No fallback: there is no call to `error_with_code` (spanless) for these codes anywhere in the codebase

### 4. HIR tests and e2e column anchors cover the behavior

**VERIFIED.**

- **HIR unit tests** (`expressions_tests.rs:1949–1972`): explicitly assert `primary_range == Some(range_for(source, "break"))` and `range_for(source, "continue")`, which extracts the exact `TextRange` of the keyword
- **E2E fixtures**:
  - `break_outside_loop.sifr` → `# expect-error[col=5]: SIFR-FLOW-0001`
  - `continue_outside_loop.sifr` → `# expect-error[col=5]: SIFR-FLOW-0002`
  - Column 5 is correct: 4 leading spaces + `break`/`continue` starts at 1-indexed column 5

### 5. Validation already run locally

All listed validations passed:
- `cargo fmt --check` ✅
- `git diff --check` ✅
- `cargo test -p sifr_hir outside_loop -- --nocapture` ✅
- `cargo test -p sifr --test e2e test_e2e_fail -- break_outside_loop continue_outside_loop --nocapture` ✅
- `cargo clippy -p sifr_hir --no-deps -- -D warnings` ✅

---

## Blocker assessment

**None.**

The implementation is minimal, correct, and covers all required aspects:
- Correct API usage (`error_with_code_at` vs `error_with_code`)
- Correct range source (real AST `.range()`)
- Correct test coverage (HIR unit test + e2e column anchor)
- No spanless fallback path for these two codes

---

## Minor note for future reference

The `import ruff_text_size::TextRange` in `flow_diagnostics.rs` was added in this slice — it was already present in `statements.rs` (via `ruff_text_size::Ranged`), so the import is consistent with the rest of the codebase.
