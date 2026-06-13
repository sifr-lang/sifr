I've inspected all the listed files. Let me record my findings.

**Generated Rust async kill/terminate/wait bodies — verified correct (`crates/sifr_codegen/src/preamble/process_async_runtime.rs:614-670`):**
- `kill_body` uses `__children.get_mut(&handle)` (not `remove`) + `__child.start_kill()` — non-async, lock dropped at the `}` before any `await`. Subsequent `wait()` still observes the killed child. ✓
- `terminate_body` is `#[cfg(unix)]`-gated: scopes the mutex around `__child.id()` only, extracts the PID, drops the guard, then `.status().await` on the Tokio `kill -TERM` command. `#[cfg(not(unix))]` returns typed `ProcessError("async process terminate is unsupported on this host; use async_kill for forceful termination")`. No Windows overclaim. ✓
- `wait_body` removes the entry via a scoped block before any `.await`, no guard across `.await`. ✓
- No data-dependent `.unwrap()`/`.expect()`: lookup uses `.ok_or_else(...)`, errors use `.map_err(...)`, mutex uses poison recovery (`.unwrap_or_else(|e| e.into_inner())`). ✓

**Helper emission/gating (`stdlib_filter/implementation.rs:338-419, 422-461`, `lib_modules_and_codegen.rs:399-438, 609-619`):**
- Path-scan flags `needs_kill`/`needs_terminate` only when `__sifr_process_async_kill`/`__sifr_process_async_terminate` appear in the filtered stdlib IR.
- `process_async_child_table_items` is `needs_spawn || needs_wait || needs_kill || needs_terminate`, so any single async-child surface still pulls in the table; only `needs_spawn` mints the ID allocator (kill/terminate/wait don't allocate IDs). ✓
- `is_shared_prelude_item` recognizes the new fn names (`__sifr_process_async_kill`/`terminate`) so the stdlib filter strips them and the preamble path conditionally re-emits them. ✓
- `process_async_run_output.sifr` (fixture #2) imports neither `AsyncChild` nor `async_spawn`/`async_wait`, so DCE leaves the stdlib helper paths unreferenced and no async child table/spawn/wait/kill/terminate state is emitted. ✓

**Sifr surface (`lib/sifr/process.sifr:169-183`, intrinsic registry `sifr_stdlib/src/process.rs:321-334`, lowerer `intrinsics/registry/process_async.rs:106-124`, registry binding `intrinsics/registry.rs:636-643`):**
- `AsyncChild.kill()` / `AsyncChild.terminate()` declared as `Awaitable[Result[None, ProcessError]]`, not `@blocking_io`. ✓
- No top-level async kill/terminate helpers added. ✓
- Async lifecycle stays method-form-only as advertised.

**Fixture (`crates/sifr/tests/e2e/pass/process_async_child_kill_terminate.sifr`):** exercises method-form kill → wait observes `signal == 9`, terminate → wait observes `signal == 15`, and post-wait kill returns typed `ProcessError` with `"closed or unknown"`. ✓

**Docs/manifests/host matrix:**
- `supported_host_matrix.md:24` adds an "Async subprocess kill/terminate" row that is supported on macOS arm64 / Linux x86_64 and `host-limited` on Windows, naming `process_async_child_kill_terminate` as evidence — no Windows or non-Unix terminate overclaim. ✓
- `concurrency_runtime_m4_process_traceability.md:19, 34, 40-41` updates the production-surface and CPython-family rows for method-form `AsyncChild.kill`/`AsyncChild.terminate` with the explicit Unix-only-SIGTERM and unsupported-on-non-Unix wording, and the fixture is listed in both create-pr and merge validation rows. ✓
- `create_pr_e2e_manifest.json:90` and `merge_e2e_manifest.json:105` include `process_async_child_kill_terminate`. ✓
- Issue ledger (`issues/…:429, 990-1010`) records the wave as in-progress, method-form-only, and explicitly defers top-level async kill/terminate helpers, public async owned pipes, cancellation-safe observation, scoped process supervision, async shell APIs, text mode, and Windows terminate support.

**File-size guardrail:** all touched files under 900 lines — `process_async_runtime.rs` 875, `stdlib_filter/implementation.rs` 783, `lib_modules_and_codegen.rs` 834, others well under. ✓

PASS
