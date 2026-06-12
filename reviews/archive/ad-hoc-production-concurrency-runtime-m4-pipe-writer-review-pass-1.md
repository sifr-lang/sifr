RESULT: PASS

## Findings

### Correctness (Q1)
- `__sifr_process_pipe_write_all` calls `Write::write_all(__pipe.as_mut(), data.as_slice())`, wraps `io::Error` via `process_map_err` → typed `ProcessError`, and returns `Ok(())`. `__sifr_process_pipe_close` removes the entry under the lock then `?` the `Option` → typed error; dropping the boxed `ChildStdin` closes the fd and signals EOF to the child. Both functions use `ok_or_else` for missing handles, `?` for IO failures, and never `.unwrap()`/`.expect()` on data-dependent paths.
- `PipeWriter.write_all`/`close` guard `_closed` before delegating; the `_closed = True` flip happens before `process_pipe_close(...)` is invoked. This mirrors the established `PipeReader.read_all` pattern (best-effort, single attempt) — intentional, not a defect.

### One-shot stdin + repeated writes (Q2)
- `__sifr_process_child_stdin` (`process_child_pipe_writer_item`) takes `child.stdin` via `Option::take`, so a second `Child.stdin()` returns the `"already taken"` `ProcessError`. The pass fixture exercises this.
- Writer-table entry persists across `write_all` (`get_mut`, not `remove`), so repeated `write_all` calls before `close()` all hit the same `ChildStdin`. Fixture validates `b"pipe-"` then `b"writer"` round-trip plus `"err"` on stderr.

### Arity / order across layers (Q3)
- Stdlib source (`lib/sifr/process.sifr` `spawn`): passes `stdin_mode, stdout_mode, stderr_mode` last.
- Intrinsic metadata (`crates/sifr_stdlib/src/process.rs::process_spawn`): params end with `stdin_mode, stdout_mode, stderr_mode` (`Type::Str`).
- Lowerer (`registry/process.rs::lower_process_spawn`): now `args.len() != 8`, clones args 5/6/7.
- Runtime preamble (`process_runtime.rs::process_spawn_item`): function params end with `stdin_mode, stdout_mode, stderr_mode`; body invokes `__cmd.stdin(...)` → `stdout(...)` → `stderr(...)` in that order using the named identifier. Consistent end-to-end.

### Panic surface in generated runtime (Q4)
- `poisoned_lock_expr` recovers from poisoning via `unwrap_or_else(|e| e.into_inner())` — no panic on poisoned mutex.
- All other operations use `?`, `ok_or_else`, and `map_err`. No `.unwrap()`/`.expect()` introduced. Drops are infallible.

### Writer-table mutex held during `write_all` (Q5)
Acceptable for this M4 sync slice — not blocking.
- The new entry sits under one global `Mutex<HashMap<i64, Box<dyn Write + Send>>>`, held across the blocking `Write::write_all` call. That serializes every other writer operation (including `Child.stdin()` insertion and `close`) until the child drains its pipe buffer.
- For a single-threaded sync caller — the documented user model for this slice — that is fine. There is no deadlock risk: the only multi-lock ordering is `children → writers` (in `child_stdin`), and `write_all`/`close` take only `writers`.
- Follow-up improvement (non-blocking): swap to per-handle `Mutex` or temporarily `remove`/reinsert around the write so a stalled writer cannot block table-wide progress. Worth noting in the M4 follow-up boundaries when streaming/async pipes land.

### Preamble gating + file-size guardrails (Q6)
- `stdlib_filter/implementation.rs` adds gating for `__SIFR_PROCESS_PIPE_WRITERS`, `__sifr_process_child_stdin`, `__sifr_process_pipe_write_all`, `__sifr_process_pipe_close` in all three locations (`derive_shared_needs_text_scan`, `SharedNeedsCollector::visit`, `is_shared_prelude_item`). Symmetric with the existing reader gating.
- File-size guardrail PASS (script run reported). Local check: `process_runtime.rs` 568, `process_child_pipes.rs` 492, `process.rs` 868, new `process_pipes.rs` 79, `stdlib_filter/implementation.rs` 744, `lib/sifr/process.sifr` 464 — all under 900. Splitting the pipe lowerers into `process_pipes.rs` was the right move; `process.rs` is now at 868 and would have exceeded the cap otherwise.

### Docs / traceability / manifests honesty (Q7)
- Traceability `PipeWriter` row accurately scopes: `Stdio("pipe")` stdin required, repeated writes, explicit close → EOF, write-after-close and double-extraction rejected. Marks async pipes/communicate and sendability/shareability as still open.
- Reader row's stale "stdin `PipeWriter`" qualifier removed.
- `Sync spawn, wait, Child.wait` row updated to mention stdin pipe writing.
- Follow-up Boundaries no longer claims PipeWriter as open work; streaming reads, async pipes/communicate, scoped supervision, sendability/shareability remain explicitly listed.
- Supported host matrix: new row marks Linux/macOS supported, Windows `host-limited` with fixture caveat — matches the matrix policy used for the reader row.
- Manifests: `process_spawn_pipe_writer` added next to the readers row in both `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json`; fail entry `process_pipe_writer_method_direct_async_rejected` listed in the fail-suite row of the traceability doc.

### Non-blocking nits
1. Global writer mutex during blocking write — see Q5; worth a follow-up issue when streaming/async pipes are wired.
2. Traceability `Status:` line still ends with "sync stdin pipe writer support is in progress." Update to the merged-PR phrasing when this PR lands (existing convention in the file).
3. `process_pipes.rs` is a thin lowerer (six `path_call` wrappers); fine for now, but if more pipe intrinsics appear, consider a tiny helper to drop the boilerplate `arg_expr(args, 0)` pattern.
