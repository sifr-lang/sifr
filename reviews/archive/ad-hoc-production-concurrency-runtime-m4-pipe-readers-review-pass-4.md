PASS.

Verification against each gate:

**Pass-3 blocker fixed**: `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:785-786` preserve `Merged as PR #2350` and `Merge-ledger validation` lines; the pipe-reader block starts at line 788, after them.

**Traceability matrix** (`verification/stdlib/concurrency_runtime_m4_process_traceability.md`):
- `process_child_wait_method_direct_async_rejected` retained in the `Sync spawn/wait/Child.wait` row and Fail suite row.
- `process_child_kill_method_direct_async_rejected` retained in the `Sync kill/Child.kill` row and Fail suite row.
- `process_spawn_pipe_readers` added to `Command`, `Stdio`, `PipeReader`, `Sync spawn/wait/Child.wait` rows, CPython `test_subprocess.py` mapping, and Create PR + Merge lanes.

**Sync-only scope**: New `lower_process_child_stdout`/`stderr`/`pipe_read_all` (registry/process.rs) and the `process_child_pipes.rs` preamble are sync (`is_async: false`); async run/output is untouched and still rejects stdin bytes with the existing deferral error.

**Stdio API**: `Command.__init__` defaults `stdout_mode`/`stderr_mode` to `"inherit"` (lib/sifr/process.sifr:157-158); `Command.stdout(mode: Stdio)` / `Command.stderr(mode: Stdio)` set the mode (lines 178-182). `Stdio` class exists at lib/sifr/process.sifr:26-30.

**One-shot PipeReader**: `Child.stdout()`/`stderr()` (lib/sifr/process.sifr:108-122) call `process_child_stdout/stderr`, which use `.take()` on the underlying child field (process_child_pipes.rs:142) and return typed `ProcessError` with `"already taken"` on double extraction.

**PipeReader.read_all**: marked `@blocking_io`, sets `self._closed = True` before delegating to `process_pipe_read_all` (lib/sifr/process.sifr:77-81). The generated helper removes the entry from the table, returns `Result<Vec<u8>, ProcessError>`, and uses `read_to_end` + `map_err`.

**Panic-freedom**: All generated functions return `Result<…, ProcessError>`; lock acquisition uses `unwrap_or_else(into_inner)` for poisoned locks (no data-dependent unwrap); missing handles and re-extractions return `ProcessError` via `ok_or_else`; spawn uses `map_err` on the I/O error.

**Gating**: `is_shared_prelude_item` and `derive_shared_needs_text_scan` (stdlib_filter/implementation.rs:311-396) only emit `__SIFR_PROCESS_PIPE_READERS`, `__sifr_process_spawn`, `__sifr_process_child_stdout/stderr`, `__sifr_process_pipe_read_all`, and `__sifr_process_stdio_from_mode` when those identifiers appear in user code. Ordinary `Command.output()` doesn't reference any of them and therefore won't pull the pipe table or helpers — confirmed by the ledger's emission checks at lines 795-796.

**Docs**: traceability explicitly lists `PipeWriter`, owned stdin, streaming reads beyond one-shot `read_all`, async pipes/communicate, timeout, cancellation, and scoped supervision as remaining M4 work; supported-host matrix marks Windows as host-limited requiring a deterministic fixture.

**Validation recorded accurately**: line 804 records `177.84s` warm wall-time, `98 passed/0 failed`, `cache_hits=26/26`, `report_signature=559a90cf856fe902`, advisory `warm wall-time budget exceeded` — matches the request verbatim.

**Other**: File-size guardrail is clear (largest touched file is `intrinsics/registry/process.rs` at 897 lines; `process_runtime.rs` 853, `process_child_pipes.rs` 263). Manifests updated for both create-pr and merge lanes.

No changes required.
