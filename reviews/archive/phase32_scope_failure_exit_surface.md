

Based on my detailed code review, I've identified a critical bug and several observations:

## Critical Bug: Missing `ScopeFailure` in Intrinsics Error List

The `collect_referenced_builtin_error_classes` function in `error_refs.rs` correctly marks `TimeoutError` as an intrinsic error (because the runtime generates `TimeoutError::new(...)` in the `TimeoutAwait` render), but it does NOT mark `ScopeFailure` as intrinsic.

**This causes `demos/m32_task_core_demo.sifr` to fail with `cannot find type ScopeFailure in this scope`** because:
- The demo returns `Result[None, Error]`, not `Result[None, ScopeFailure]`
- The `error_refs.rs` text scanner doesn't find `ScopeFailure` in the source code
- The codegen then emits `scope.__sifr_join_all()` returning `Result<(), ScopeFailure>`, but `ScopeFailure` struct is never generated

**Fix needed** in `crates/sifr_codegen/src/error_refs.rs`, add `ScopeFailure` alongside `TimeoutError`:

```rust
if !intrinsic_functions.is_empty() {
    for error_name in [
        "IOError",
        "ParseError",
        "ValueError",
        "JSONDecodeError",
        "JsonIntegerRangeError",
        "JsonLimitError",
        "TOMLDecodeError",
        "RegexError",
        "TimeoutError",
        "ScopeFailure",  // <-- ADD THIS
    ] {
        referenced.insert(error_name.to_string());
    }
}
```

## Other Findings

**Correct behavior confirmed:**
- HIR layer (`async_with.rs` lines 611-622) correctly enforces that spawning scopes require Result[..., ScopeFailure] or Result[..., Error]
- Codegen (`stmt_support_emitter.rs` lines 7347-7358) correctly generates scope failure propagation code
- All `e2e/pass` fixtures use `Result[None, ScopeFailure]` as return type (the safe, explicit pattern)
- The `observed` tracking via `Arc<AtomicBool>` correctly marks child handles as observed

**Actionable change:** Add `"ScopeFailure"` to the intrinsics error list in `error_refs.rs` before this slice can be considered satisfied. Without this fix, any async function using `task.scope()` with `Result[None, Error]` (rather than `Result[None, ScopeFailure]`) will fail to compile.
