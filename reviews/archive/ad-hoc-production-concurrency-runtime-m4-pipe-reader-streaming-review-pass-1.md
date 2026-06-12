RESULT: PASS

# M4 sync PipeReader streaming reads — review pass 1

Reviewed branch: `codex/concurrency-runtime-m4-pipe-reader-streaming`
Scope: sync `PipeReader.read(max_bytes)` / `PipeReader.close()` on `sifr.process`. Async pipes, sendability/shareability checks, scoped supervision, process-group behavior, and text-mode pipe decoding are intentionally out of scope.

## Review questions

1. **Public surface preserves typed `ProcessError` and avoids panics.** PASS.
   `lib/sifr/process.sifr:89-113` defines `read(max_bytes)` and `close()` as `@blocking_io` methods returning `Result[bytes, ProcessError]` / `Result[None, ProcessError]`. `read()` raises a typed `ProcessError` when `_closed`, calls the intrinsic, marks `_closed = True` on an empty chunk (EOF), and re-raises typed errors via `except ProcessError as e: raise e`. `close()` raises typed `ProcessError` when already closed and otherwise sets `_closed = True` before delegating to the intrinsic. No `.unwrap()`/`.expect()` paths surface to the user. The intrinsic itself handles a poisoned mutex via `lock().unwrap_or_else(|err| err.into_inner())` (`crates/sifr_codegen/src/preamble/process_child_pipes.rs:387,396`), so there is no panic path even on lock poisoning.

2. **Stdlib metadata, intrinsic lowerers, registry dispatch, and helper signatures agree.** PASS.
   - Stdlib metadata: `process_pipe_read` is `(handle: Int, max_bytes: Int) -> Result[Bytes, ProcessError]` and `process_pipe_reader_close` is `(handle: Int) -> Result[None, ProcessError]` (`crates/sifr_stdlib/src/process.rs:150-166`).
   - Lowerers: `lower_process_pipe_read` requires `args.len() == 2` and emits `__sifr_process_pipe_read(arg0, arg1)`; `lower_process_pipe_reader_close` requires `args.len() == 1` and emits `__sifr_process_pipe_reader_close(arg0)` (`crates/sifr_codegen/src/intrinsics/registry/process_pipes.rs:58-76`).
   - Registry dispatch: `"process_pipe_read"` and `"process_pipe_reader_close"` map to the new lowerers (`crates/sifr_codegen/src/intrinsics/registry.rs:607-608`).
   - Generated helpers: `__sifr_process_pipe_read(handle: i64, max_bytes: i64) -> Result<Vec<u8>, ProcessError>` and `__sifr_process_pipe_reader_close(handle: i64) -> Result<(), ProcessError>` (`crates/sifr_codegen/src/preamble/process_child_pipes.rs:361-430`), wired into `build_process_child_items()` (`crates/sifr_codegen/src/preamble/process_runtime.rs:697-698`).
   - `lib/sifr/process.sifr:2-31` exports both names from `_sifr.process`.

3. **`__sifr_process_pipe_read` partial-handle preservation, EOF removal, positive-size validation, 1 MiB cap.** PASS.
   The helper rejects `max_bytes <= 0` with "process pipe read size must be positive" and `max_bytes > 1048576` with "process pipe read size exceeds 1048576 bytes" before touching the reader map. The body then performs `std::io::Read::read(__pipe.as_mut(), __buffer.as_mut_slice())` against the boxed `Read + Send` handle via `get_mut(&__handle)`; the lock is dropped at the end of the `let __read = { ... }` block. On `__read == 0` (EOF) the helper re-acquires the lock and removes the entry. On a non-zero read the entry is preserved so subsequent `read()` calls continue. Buffer truncation uses `Vec::truncate(__read)`, which is panic-free for in-bounds lengths (and `__read <= max_bytes as usize` always holds for `std::io::Read::read`). Missing/closed handles raise typed `ProcessError`; I/O failures are mapped to typed `ProcessError` via `map_err(|e| ProcessError { message: e.to_string() })`. The `1048576` literal matches the `1 MiB` documented cap.

4. **`__sifr_process_pipe_reader_close` releases partial handle and reports missing as typed error.** PASS.
   `crates/sifr_codegen/src/preamble/process_child_pipes.rs:406-430` locks `__SIFR_PROCESS_PIPE_READERS`, performs `remove(&__handle)`, and `ok_or_else(...)?` produces a typed `ProcessError("process pipe reader handle is closed or unknown: ...")` if the entry was already gone. A partially-read handle (which the new `process_pipe_read` keeps in the map after non-zero reads) is therefore released by this call. The Sifr-level `close()` guard at `lib/sifr/process.sifr:108-113` sets `_closed = True` before invoking the intrinsic, so the intrinsic only sees a missing entry under direct intrinsic abuse or a previously-EOF'd reader (which is already raised pre-intrinsic at the Sifr layer).

5. **Prelude filtering and gating include the new helpers without dragging async process child state into sync-only fixtures.** PASS.
   - `is_shared_prelude_item` filters out the two new function definitions (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:450-451`) and the `__SIFR_PROCESS_PIPE_READERS` static (line 435).
   - `derive_shared_needs_text_scan` flags `__sifr_process_pipe_read(` (parenthesized to disambiguate from `__sifr_process_pipe_read_all`) and `__sifr_process_pipe_reader_close` (lines 331-332).
   - `SharedNeedsCollector::visit_path` lists both new helper names so AST scans set `needs_process_children` (lines 382-383).
   - `build_process_child_items()` is gated on `needs_process_children` (`crates/sifr_codegen/src/lib_modules_and_codegen.rs:620-622`) and emits only the sync child runtime; Tokio async child state lives in a separate gated branch. The reported emission check confirms no async process child state appears in the generated Rust for the sync streaming fixture.

6. **Fixture coverage is honest.** PASS.
   `crates/sifr/tests/e2e/pass/process_pipe_reader_streaming.sifr` covers, in one binary:
   - Invalid size rejection: `reader.read(0)` raises `ProcessError` containing "positive".
   - Chunk ordering: `read(2)`, `read(3)`, `read(10)` yield `b"ab"`, `b"cde"`, `b"f"` from `printf abcdef`.
   - EOF close behavior: a subsequent `read(1)` returns `b""`, marking the reader closed.
   - Read-after-EOF rejection: a subsequent `read(1)` raises typed "closed".
   - Wait succeeds after pipe drain.
   - Explicit close after a partial read: a second child has `read(1) == b"x"`, then `close()`, then `read_all()` raises typed "closed".
   - Adjacent-pipe behavior is validated by the reported PASS of the existing `process_spawn_pipe_readers` and `process_spawn_pipe_writer` fixtures (regression coverage is appropriate here rather than re-implementing them inside the streaming fixture).

7. **Traceability, host matrix, manifests, and execution ledger are honest about scope.** PASS.
   - `verification/stdlib/concurrency_runtime_m4_process_traceability.md:5,15,40,41,48` records the slice's `PipeReader.read(max_bytes)`/`close()` evidence, the partial-handle preservation and EOF close behavior, the explicit-close path, and explicitly defers async pipes and sendability/shareability.
   - `verification/platform/supported_host_matrix.md:20` adds the row for sync PipeReader streaming reads with `host-limited` on Windows and the deterministic-fixture caveat.
   - Both manifests list `process_pipe_reader_streaming` adjacent to the other sync pipe fixtures (`verification/validation_lanes/create_pr_e2e_manifest.json:97`, `verification/validation_lanes/merge_e2e_manifest.json:112`).
   - The execution ledger records the implementation summary, the targeted local validation list, and the pending Claude review (`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:430,991-1014`).

8. **File-size guardrails.** PASS. Touched hand-maintained files remain under the 900-line cap (`process_child_pipes.rs` 563, `stdlib_filter/implementation.rs` 789, others unchanged in size class).

## Non-blocking follow-ups

- **PR hygiene.** The working tree on this branch contains uncommitted edits to `issues/ad-hoc-production-network-http-platform-substrate.md` (+145/-30) and `issues/ad-hoc-production-network-http-platform-substrate-execution.md` (+17), plus two untracked network/HTTP readiness review files under `reviews/`. These are outside the M4 PipeReader streaming slice scope. Before opening the slice PR, exclude or stash these to avoid contaminating the diff — the prior precedent is `reviews/ad-hoc-production-concurrency-runtime-m2-sync-review-pass-2.md` (NOT PASS for unrelated network/http contamination). Not a code blocker for the slice itself.
- **Helper construction style.** `process_pipe_read_item` and `process_pipe_reader_close_item` emit their bodies as a single multi-statement `RustExpr::Ident(...)` raw string (`crates/sifr_codegen/src/preamble/process_child_pipes.rs:377-401,416-426`) instead of the structured `RustStmt`/`RustExpr` builders used by `process_pipe_read_all_item`, `process_pipe_write_all_item`, and `process_pipe_close_item` in the same file. The emitted Rust compiles and the run/emit checks pass, but the inconsistency makes future edits harder. A later cleanup pass could rebuild these bodies through the IR builders the rest of the file uses.
- **Lock scope during streaming read.** `process_pipe_read_item` holds `__SIFR_PROCESS_PIPE_READERS.lock()` for the duration of `std::io::Read::read(...)` (`crates/sifr_codegen/src/preamble/process_child_pipes.rs:386-393`). The existing `process_pipe_read_all_item` releases the lock before its read (it `remove`s the pipe out of the map first). For the current sync, single-threaded `@blocking_io` contract this is fine, but once concurrent sync readers across multiple PipeReaders become a use case (e.g., blocking-pool offload), one slow reader can starve all others through the global pipe-reader map. A later wave could switch `read()` to `remove`-and-reinsert (or to per-handle ownership transfer) before that use case ships.
- **Negative-test fail-suite coverage.** This wave does not add a `.../fail/...` fixture for direct-async-call rejection of `PipeReader.read(max_bytes)`, mirroring the existing pattern for other `@blocking_io` methods such as `process_pipe_writer_method_direct_async_rejected`. If the lowering already rejects direct async calls on `read`, a small fail fixture would lock that in as future-proof evidence.

## Summary

The slice is implementation-clean and faithful to its declared scope: typed errors throughout, no user-triggerable panics, correct partial-vs-EOF handle lifecycle, validated 1 MiB cap and positive-size rejection, agreement across stdlib metadata / lowerers / registry / generated helpers, honest deferral of async pipes, sendability/shareability, supervision, and text-mode decoding, and honest traceability/host-matrix/manifest/ledger updates. The fixture covers the requested cases without dragging unrelated surfaces in. Local validation evidence (cargo fmt, cargo check across the relevant crates, file-size and HIR guardrails, the new fixture, adjacent pipe regressions, emission check, and the 425-fixture fail suite) is consistent with the diff.

No blockers. Recommend proceeding to PR after addressing the PR-hygiene follow-up so the slice diff stays scoped.
