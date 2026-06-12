## PASS

Behavior-preserving split verified end-to-end. Generated Rust output is unchanged because every moved helper is referentially transparent and the `__sifr_process_async_spawn` param list is identical to the pre-split shape.

### Verification highlights

- **`process_async_spawn_params` shape (the only non-trivial moving part)** — old version returned `process_async_params(false)` (program, args, env, cwd, has_cwd, stdin_mode) and pushed `stdout_mode`, `stderr_mode`, `has_stdin`. New version at `crates/sifr_codegen/src/preamble/process_async_child_runtime.rs:9-48` builds those same 9 named params in the same order with the same types. Resulting `Vec<RustParam>` is bit-identical.
- **Body builders** (`process_async_spawn_body`, `process_async_spawn_insert_body`, `process_async_wait_body`, `process_async_kill_body`, `process_async_terminate_body`) — moved verbatim; each returns the same `RustStmt`/`Vec<RustStmt>` literal-string payload as the pre-split inline blocks (diff confirms no text drift, only the wrapping function indirection).
- **`process_async_child_table_items`** — moved verbatim; `needs_spawn` still gates `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID` and `__sifr_next_process_async_child_id`.
- **Visibility/imports**: `mod process_async_child_runtime;` is private at `preamble.rs:17` (correct, nothing outside `preamble` needs it). All exported helpers are `pub(super)` (minimum sibling access). `process_async_runtime.rs:3-7` imports via `super::process_async_child_runtime::{...}`. Remaining imports (`RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, Visibility`) all still referenced in the slimmed parent file (21 hits).
- **No stale call sites**: grep confirms the moved symbols are referenced only by the new file (defs) and the parent (consumers).
- **File-size guardrail**: `process_async_runtime.rs` 875 → 693, new file 236. Both well under 900.
- **Docs honesty**: execution log line counts and "behavior unchanged" claim match the diff; traceability line lists this wave as "in the current implementation wave" (not as merged).

### Non-blocking notes

- `string_ty()` is duplicated as a private helper in both files (`process_async_child_runtime.rs:5-7` mirrors the parent). Acceptable as a localized convenience, but a follow-up could move the tiny ty constructors to a shared sub-module if more files start needing them.
- Builder return-type shape is mixed: `process_async_spawn_body()` returns `Vec<RustStmt>` (extend) while `process_async_spawn_insert_body()` returns a single `RustStmt` (push). It matches the call sites cleanly, but uniformity (always `Vec<RustStmt>`) would slightly future-proof the split if either body grows.

Neither is a regression vs. pre-split.
