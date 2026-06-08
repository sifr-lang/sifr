RESULT: PASS

I've reviewed the M4 async process spawn/wait wave against the phase constraints.

Review findings:

**Public API constraints (`lib/sifr/process.sifr:167-174,404-419`)**
- `AsyncChild` exposes only `_handle: int`; `async_spawn`/`async_wait`/`AsyncChild.wait` return `Awaitable[Result[..., ProcessError]]`. No `tokio::*` or `std::process::*` leak through the public surface.
- `async_spawn(own command: Command)` and `async_wait(own child: AsyncChild)` correctly take owned arguments. None of the three are `@blocking_io`-classified, consistent with the other `async_*` family.

**Stdlib metadata (`crates/sifr_stdlib/src/process.rs:53-60,245-271`)**
- `process_async_child_class()` exposes only `_handle: Int`; the returned types are `Awaitable[Result[AsyncChild, ProcessError]]` / `Awaitable[Result[Status, ProcessError]]`.

**Codegen lowerers (`crates/sifr_codegen/src/intrinsics/registry/process_async.rs:43-65`, `registry.rs:616-623`)**
- Arities are checked (9 args for spawn, 1 for wait); both lowerers wrap in `Box::pin(...)` and require `StdlibFeature::Tokio`.

**Generated runtime (`preamble/process_async_runtime.rs:605-776`)**
- Async child table is gated on `needs_spawn || needs_wait`. The wait body removes the child inside an inner `{ ... }` block (lines 756-763) and only then `.await`s — the `MutexGuard` is dropped before suspension. Spawn similarly scopes the insert.
- No data-dependent `unwrap()`/`expect()`: `lock().unwrap_or_else(|err| err.into_inner())` is mutex-poison recovery (no panic), `code().unwrap_or(-1)` provides a default, and missing handles surface as typed `ProcessError("...closed or unknown: {handle}")`. Pipe modes and `has_stdin` return typed `ProcessError` early.

**Prelude gating (`stdlib_filter/implementation.rs:51-65,336-409,425-451`, `lib_modules_and_codegen.rs:327-434,599-614`)**
- `SharedPreludeProcessAsyncNeeds` adds `needs_spawn`/`needs_wait` flags. `needs_process_children` (sync child table) is independent and only triggers on sync `__SIFR_PROCESS_CHILDREN`/`__sifr_process_spawn` references. The async-only fixture imports do not pull the sync child symbols through DCE, matching the user's emission check.

**Fixture (`crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr`)**
- Covers nonzero exit, success, double-`wait` typed error containing "closed or unknown", `Stdio("null")` accepted, `Stdio("pipe")` deferred with "async pipe support" message, and `stdin_bytes(...)` rejected with "stdin_bytes" message. Error-message substrings all match the helper text exactly.

**Manifests / traceability / host matrix**
- `process_async_spawn_wait` is in both create-pr and merge manifests. Traceability row for `async_spawn`/`async_wait`/`AsyncChild.wait` correctly scopes the slice and explicitly defers public async pipes, async termination, cancellation-safe observation, and scoped supervision. Host matrix marks Windows as host-limited.

**File-size guardrail**
- All touched files under 900 lines (largest: `preamble/process_async_runtime.rs` at 812 lines).

**Non-blocking observations (not blockers):**
- `reviews/ad-hoc-production-concurrency-runtime-m4-async-spawn-wait-review-pass-1.md` exists but is empty; if it is meant as the artifact slot for this review, it should be populated before PR.
- The async helper-name path collector matches on exact idents, so `__sifr_process_async_run` vs `__sifr_process_async_run_timeout` correctly route to distinct flags — robust as long as future helpers don't reuse exact idents.
