

Based on my thorough review of the diff, surrounding code, and generated output, here is my assessment:

---

## Review: Phase 32 channel_cancel_pending_receive slice

### 1. `is_async` in `RustExpr::ClosureBlock` — correctly propagated

- **rust_ir.rs:238**: Added `is_async: bool` to the enum. ✅
- **render.rs:850**: Renders `{async_kw}{move_kw}|{params}| {{...}}` — correct Rust syntax order. ✅
- All 16 constructor sites explicitly pass `is_async: false`. ✅
- **expr_render_helpers.rs:469-474**: `is_async` passes through the rewrite unchanged. ✅
- **stmt_support_emitter.rs:8739**: Uses `closure_is_async` in the new try/except path. ✅

### 2. `rust_stmts_contain_await` / `rust_expr_contains_await` — correct heuristics

The implementation is conservative and correct:
- `Await`, `TimeoutAwait`, and `.await`-containing idents are all detected. ✅
- Nested `Closure` delegates to `rust_expr_contains_await(body)`; `ClosureBlock` delegates to `rust_stmts_contain_await(body)`. ✅
- All statement and expression variants are handled recursively. ✅
- The `Ident` check for `.await` via string contains is a heuristic matching existing patterns (used elsewhere in the codebase for the same purpose). Acceptable. ✅

### 3. Async closure ordering is valid Rust

Generated output shows: `(async || { ... })().await` — which is syntactically valid Rust. The `async` keyword appears before `move` (if present), which is correct. ✅

### 4. Return-capture paths remain correct when async

- `direct_return_capture=true`: `unreachable!()` after closure body — unreachable regardless of async, since `return` in a closure doesn't escape the closure. ✅
- `direct_return_capture=false`: `Ok(None)` returned, match arm unwraps `Ok(Some(__sifr_ret_val))` — same behavior in both sync and async paths. ✅

### 5. Fixture validates the intended behavior

The test:
1. Creates an unbuffered channel (`capacity=-1`)
2. Times out a pending receive with `task.timeout(0.0)` — timeout always fires
3. Confirms the receiver is not poisoned: `sender.send(5)` succeeds, `receiver.receive()` returns `Ok(5)`
4. Asserts `timed_out == True` (timeout path was taken)

This directly validates the stated intent: "timing out a pending same-task receive leaves the receiver usable." The `str()` assertions are not variant checks, but the subsequent assertions confirm the actual values, so the fixture is acceptable. ✅

### 6. The 21 failing tests are pre-existing

The author correctly identified these as unrelated (parse-shape assertions, decomposition guard, etc.). The new test passes and the authoritative `run_all_tests.sh --profile quick` passes (42 fixtures, 147s). ✅

### 7. Missing test coverage (acceptable)

The test verifies async closure + await rendering but doesn't explicitly cover:
- Return capture with async try/except (would require a more complex fixture)
- Sync try/except continues to work (covered by existing tests at line 32 and line 50 in `lib_codegen_tests.rs`)

These are reasonable gaps for a focused regression test. ✅

### Minor observation (non-blocking)

The `rust_expr_contains_await` heuristic for `Ident` checking `value.contains(".await")` could theoretically false-positive on an identifier named `foo.await` (unlikely but possible). However, this matches existing patterns in the codebase and is conservative (would only make a closure async when not strictly needed, which is safe).

---

**No blocking issues identified. All correctness concerns have been addressed.**

REVIEW_STATUS: SATISFIED
