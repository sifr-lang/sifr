# M4 Async Owned Process Pipes — Review Pass 1

Result: **PASS**

Scope reviewed:
- `lib/sifr/process.sifr`
- `crates/sifr_stdlib/src/process.rs`
- `crates/sifr_codegen/src/intrinsics/registry.rs`
- `crates/sifr_codegen/src/intrinsics/registry/process_async.rs`
- `crates/sifr_codegen/src/preamble/process_async_child_runtime.rs`
- `crates/sifr_codegen/src/preamble/process_async_runtime.rs`
- `crates/sifr_codegen/src/stdlib_filter/implementation.rs`
- `crates/sifr/tests/e2e/pass/process_async_spawn_pipes.sifr`
- `crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr`
- `verification/platform/supported_host_matrix.md`
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md`
- `verification/validation_lanes/create_pr_e2e_manifest.json`
- `verification/validation_lanes/merge_e2e_manifest.json`
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`

## Blocking findings

None.

## Targeted-question walkthrough

### 1. Mutex guard held across await in generated async pipe helpers
Every Tokio-aware helper scopes the `std::sync::Mutex` guard inside an explicit
`{ ... }` block that ends before the `.await` site:

- `__sifr_process_async_pipe_read_all` (`process_async_child_runtime.rs:325-339`):
  `let mut __pipe = { let mut __pipes = ...lock(); __pipes.remove(&__handle)? };`
  drops the lock before `__pipe.read_to_end(&mut __buffer).await`.
- `__sifr_process_async_pipe_read` (`process_async_child_runtime.rs:361-385`):
  initial removal and conditional re-insert each occur in their own lock blocks
  before/after `__pipe.read(...).await`.
- `__sifr_process_async_pipe_write_all`
  (`process_async_child_runtime.rs:431-447`): removal, `write_all(...).await`,
  and unconditional re-insert each have separate lock scopes.
- `__sifr_process_async_pipe_reader_close` and `__sifr_process_async_pipe_close`
  (`process_async_child_runtime.rs:399-412` and `462-475`) are `is_async: false`
  and never see an `.await` point.
- `__sifr_process_async_child_stdin/stdout/stderr`
  (`process_async_child_runtime.rs:256-316`) are `is_async: false`; the lock is
  held only across the synchronous `Option::take` on the `tokio::process::Child`
  field and nothing else.
- `__sifr_process_async_spawn` (`process_async_runtime.rs:485-488` ->
  `process_async_spawn_insert_body`): the only `.await` inside the helper is
  reached after the spawn lock block closes; `tokio::process::Command::spawn`
  itself is synchronous.
- `__sifr_process_async_wait` (`process_async_child_runtime.rs:480-491`),
  `__sifr_process_async_kill` (`process_async_child_runtime.rs:496-505`),
  `__sifr_process_async_terminate` (`process_async_child_runtime.rs:511-538`):
  same pattern; the lock scope contains only synchronous calls
  (`remove`, `get_mut`, `start_kill`, `child.id`) and is dropped before any
  `.await`.

No `MutexGuard` is alive across an `await`.

### 2. Lost handle / double-use behavior on async read/write errors
- `__sifr_process_async_pipe_write_all`
  (`process_async_child_runtime.rs:431-447`) captures the `write_all` result
  *before* re-inserting the writer, then propagates the error. So the writer
  handle survives a transient write error and the user can still call
  `AsyncPipeWriter.close()` to release it.
- `__sifr_process_async_pipe_read_all`
  (`process_async_child_runtime.rs:325-339`) and `__sifr_process_async_pipe_read`
  (`process_async_child_runtime.rs:361-385`) propagate the `?` immediately on
  I/O error, dropping the local `__pipe` so the handle is permanently gone from
  the table. A subsequent retry or `close()` therefore returns the
  "closed or unknown" error.
- EOF on `__sifr_process_async_pipe_read` is handled correctly: when
  `__read == 0` the conditional re-insert is skipped, so the handle disappears
  and the fixture's assertion that the next `read(...)` raises "closed" is
  satisfied (`process_async_spawn_pipes.sifr:70-75`). `read_all` always consumes
  the handle, matching the fixture's "closed" assertion at line 90-97.
- Repeated extraction is guarded by `Option::take()` on the
  `tokio::process::Child` field, producing the "already taken" message the
  fixture relies on (`process_async_spawn_pipes.sifr:36-41`).

The read-side asymmetry (write preserves the handle on error, read does not) is
worth a non-blocking note (see below) but is internally consistent with the
EOF-consumes-handle invariant and with the sync `PipeReader` lifecycle the
docs reference.

### 3. Stdio mode semantics for `async_spawn` vs one-shot `async_output`
- `async_spawn` (`process_async_runtime.rs:485-488`,
  `process_async_child_runtime.rs:223-254`) no longer rejects pipe/null modes.
  It rejects `has_stdin == true` (i.e. `Command.stdin_bytes(...)`) up front and
  then maps each of `stdin_mode`/`stdout_mode`/`stderr_mode` through
  `__sifr_process_async_stdio_from_mode`, which only accepts
  `"pipe"`/`"inherit"`/`"null"` and returns a typed `ProcessError` for anything
  else. Fixture `process_async_spawn_wait.sifr:27-33` retains the
  `stdin_bytes` rejection coverage.
- `async_run` / `async_run_timeout` / `async_output` / `async_output_timeout`
  still call `process_async_stdin_mode_guard()` (`process_async_runtime.rs:87-102`)
  which insists `stdin_mode == "inherit"`. Their bodies still hard-code
  `__cmd.stdout(Stdio::piped()); __cmd.stderr(Stdio::piped());` and only force
  `__cmd.stdin(Stdio::piped())` when `has_stdin` is set
  (`process_async_runtime.rs:332-339` and `423-431`). The traceability table
  now correctly frames this as a one-shot-capture restriction rather than as
  "until owned pipes land".

### 4. Generated prelude gating / dedup correctness
The two new statics, the two new pipe tables, and every new helper function
appear consistently in:
- AST visitor `SharedNeedsCollector` (`stdlib_filter/implementation.rs:412-427`)
  marking `process_async.needs_spawn = true` on any reference to the pipe
  statics or any of the new `__sifr_process_async_child_*` /
  `__sifr_process_async_pipe_*` helpers.
- Text-scan fallback (`stdlib_filter/implementation.rs:340-356`) gating
  `needs_spawn` on the same set of strings.
- `is_shared_prelude_item` statics arm (`stdlib_filter/implementation.rs:449-460`)
  stripping `__SIFR_PROCESS_ASYNC_PIPE_READERS` and
  `__SIFR_PROCESS_ASYNC_PIPE_WRITERS` alongside the existing children/id
  statics.
- `is_shared_prelude_item` fn arm (`stdlib_filter/implementation.rs:462-493`)
  stripping all eight new helpers
  (`__sifr_process_async_child_stdin/stdout/stderr`,
  `__sifr_process_async_pipe_read_all`, `__sifr_process_async_pipe_read`,
  `__sifr_process_async_pipe_reader_close`,
  `__sifr_process_async_pipe_write_all`,
  `__sifr_process_async_pipe_close`).

`__sifr_process_async_stdio_from_mode` is a nested function declared inside the
spawn body (`process_async_child_runtime.rs:234-241`), not a top-level item, so
it is correctly excluded from the dedup table and cannot collide between
modules.

Intrinsic wiring matches: every new `_sifr.process` name is registered in
`sifr_stdlib/src/process.rs:357-417` (with the right `Awaitable` vs sync
shape — synchronous `Result[int, ProcessError]` for the child-pipe transfer
intrinsics, `Awaitable[Result[bytes, ProcessError]]` for the async reads,
`Awaitable[Result[None, ProcessError]]` for `write_all`, sync `Result[None,
ProcessError]` for the two close intrinsics), and every name has a matching
arm in `intrinsics/registry.rs:646-677` routed to the `process_async` lowerer
with `StdlibFeature::Tokio`. The intrinsic-side lowerers
(`intrinsics/registry/process_async.rs:126-207`) use
`boxed_async_process_helper_call` for the awaitable shapes and plain
`path_call` for the synchronous transfer/close shapes, which matches the
Sifr-side return-type annotations.

### 5. Fixture adequacy and manifest/docs honesty
`process_async_spawn_pipes.sifr` covers, in order:
- stdin/stdout/stderr pipe transfer on a Stdio("pipe") spawn,
- two consecutive async `write_all` calls followed by `close()`,
- async `read_all` of stdout and stderr with content assertions,
- write-after-close rejection ("closed") and double-extraction rejection
  ("already taken"),
- final wait observation,
- bounded `read(max_bytes)` partial reads with handle preservation, EOF
  yielding empty bytes, post-EOF read returning "closed",
- explicit `close()` of a partially-read reader and subsequent `read_all`
  returning "closed",
- invalid-size rejection ("positive").

`process_async_spawn_wait.sifr` retains the `stdin_bytes` rejection for
`async_spawn` and the spawn/wait happy/sad paths.

Both fixtures are listed in the create-pr and merge manifests
(`verification/validation_lanes/create_pr_e2e_manifest.json:95-96`,
`verification/validation_lanes/merge_e2e_manifest.json:110-111`).

The traceability table updates correctly:
- a new `AsyncPipeReader / AsyncPipeWriter` row references
  `process_async_spawn_pipes` and explicitly leaves sendability/shareability,
  cancellation-safe observation, and scoped supervision out of scope,
- the `AsyncChild` row now lists `process_async_spawn_pipes` and drops the
  inherited-stdio-only language,
- the CPython mapping replaces "Public async pipes" with "Cancellation and
  scoped supervision" in the open list,
- the "Public async owned pipes" follow-up boundary is correctly removed.

The supported-host matrix adds an "Async subprocess owned pipes" row with
macOS arm64 / Linux x86_64 marked `supported` and Windows `host-limited`,
matching the Unix-only fixture coverage. The execution ledger records the
work as "in progress" without claiming a merged PR.

### 6. User-triggerable panic / data-dependent unwrap
Every `.lock()` on the new pipe tables uses
`.unwrap_or_else(|__err| __err.into_inner())`, which is the established
poison-recovery pattern and not a panic. The remaining `Option::ok_or_else`
sites and `Result::map_err` sites all surface typed `ProcessError`. The
`max_bytes as usize` cast in `__sifr_process_async_pipe_read` is gated by an
explicit `max_bytes <= 0` rejection and a `max_bytes > 1048576` upper bound
before the cast, so even on 32-bit hosts the buffer allocation is
deterministic. `Box::new(__pipe)` coercion to `Box<dyn AsyncRead/AsyncWrite +
Unpin + Send>` relies on Tokio's `ChildStdin/ChildStdout/ChildStderr` already
satisfying those bounds; this is statically checked by the compiler. No new
`.unwrap()`, `.expect()`, or `assert!` is introduced in generated code.

## Non-blocking notes

1. **Read-error vs. write-error handle survival asymmetry**.
   `__sifr_process_async_pipe_write_all` re-inserts the writer even when
   `write_all` errors (`process_async_child_runtime.rs:431-447`), so the user
   can still call `close()`. `__sifr_process_async_pipe_read_all` and
   `__sifr_process_async_pipe_read` drop the reader on the first I/O error
   (`process_async_child_runtime.rs:335-337` and `377-379`), so the
   subsequent `close()` returns the same "closed or unknown" error as a
   genuine double-close. This is internally consistent with the EOF-consumes-
   handle invariant but worth recording when the cancellation-safe / scoped
   supervision wave revisits these helpers.

2. **`AsyncPipeReader.close()` after EOF returns a typed error**.
   `read(max_bytes)` removes the handle on EOF
   (`process_async_child_runtime.rs:381-384` skips the re-insert when
   `__read == 0`), so a defensive `reader.close()` after EOF will raise
   "closed or unknown". Unlike sync `PipeReader`, `AsyncPipeReader` has no
   Sifr-side `_closed: bool` to make `close()` idempotent on the client side.
   The fixture only exercises `close()` after a partial read (where the handle
   is still alive), so the EOF-then-close path is currently unexercised.
   Consider either adding a `_closed` flag on the Sifr-side wrappers or
   adding a fixture line to lock the EOF-then-close contract in.

3. **Shared id counter**.
   `__sifr_next_process_async_child_id` is shared between async children and
   async pipe readers/writers, so child handles and pipe handles draw from the
   same monotonically-increasing space. Lookups are per-table so there is no
   collision risk, but if a future wave wants to type-tag handles in
   diagnostics the shared counter will make e.g. "handle 7" ambiguous between
   a child and a pipe. Worth keeping in mind for the cancellation-safe wave.

4. **`process_async_spawn_pipes.sifr` writes 11 bytes total**. The `cat;
   printf err >&2` fixture stays well under typical pipe-buffer limits, so it
   does not stress the case where the child blocks on a full stdout buffer
   while the parent is still writing stdin. That is the realistic deadlock
   shape these owned-pipe helpers will eventually need a fixture for; not
   needed for this slice, but recording the gap.
