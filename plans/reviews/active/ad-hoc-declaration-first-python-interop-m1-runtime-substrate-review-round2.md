## Re-review of round-2 fix — callback_ops.rs invoke_sifr_callback

**Fix location:** `crates/sifr_runtime/src/python/callback_ops.rs:214-223`

### Scenario walk-through

| Scenario | Behavior | Verdict |
|---|---|---|
| `result_handle == arg_handle`, clone succeeds | skips arg-close (`if result_handle != arg_handle` false), then closes `result_handle` once → aliased handle released exactly once | ✓ no double-free |
| `result_handle == arg_handle`, clone fails | skips arg-close in error branch, closes `result_handle` once (ignored if closed) → same aliased handle released at most once | ✓ no double-free |
| stale `result_handle` (malformed callback) | closes `arg_handle` (removed from store → -1), attempts `close_object(result_handle)` which returns `closed_error` (swallowed by `_ignored`) → arg released, stale is a no-op | ✓ balanced |
| user callback closed `arg_handle` before returning new result | outer `close_object(arg_handle)` fails silently (ignored), `close_object(result_handle)` succeeds → net one release each | ✓ balanced |

### Nested attach/close under GIL

`invoke_sifr_callback` runs inside `PyCFunction`, so GIL is held. Every cleanup path goes through `close_object` (object_ops.rs:227), which:
1. acquires OBJECT_STORE, removes entry, drops the guard (scoped block ends at line 239);
2. calls `super::attach(|_py| drop(entry))` — since GIL is already held, PyO3 treats this as a no-op nested attach and runs the drop inline.

Guard is released before attach, so the ordering (GIL → OBJECT_STORE) matches every other call site — no AB/BA cycle. The store lookup keying on `(handle, token)` with monotonic `next_handle` also rules out ABA even if handles are reused.

### Original error preservation

Both cleanup calls use `let _ignored = …`; the returned value is `Err(py_runtime_error(error))` where `error` is the `clone_handle` failure. Cleanup errors cannot shadow it (callback_ops.rs:220-221). ✓

### Regression test

`malformed_callback_result_releases_temporary_argument` (callback_ops.rs:351-370) copies a pre-closed `stale` handle (Copy on `(i64,i64)`), snapshots `resource_diagnostics()` before invoking, and asserts the snapshot is unchanged after the expected `Err`. This is exactly the leak the fix closes: `-1` for `arg_handle` cancels the `+1` from the internal `store_object(arg)` inside `invoke_sifr_callback`. Test aligns with fix.

### Rest of substrate diff

Line counts and non-callback files unchanged from round 1 (`python.rs` 885L, `object_ops.rs` 782L, `object_ops_tests.rs` 116L, `foreign_object.rs` 80L; guardrail respected). Round 1's other non-blocking observations still stand and are still non-actionable.

## Verdict

**SATISFIED** — the clone_handle failure path now releases both `arg_handle` (when distinct) and `result_handle`, handles the alias `result_handle == arg_handle` without double-close, correctly swallows the expected `closed_error` when the returned handle is stale, preserves the original clone error, and respects the GIL/OBJECT_STORE ordering already established elsewhere. The regression test exercises the specific fix path. No remaining actionable issues in the substrate diff.
