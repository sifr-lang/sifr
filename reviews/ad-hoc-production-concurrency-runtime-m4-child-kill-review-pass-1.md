Now I have enough context. Writing the review.

RESULT: CHANGES_REQUESTED

## Findings

### 1. (Medium / blocker) Unconditional Unix-only signal extraction breaks Windows compile
**Files**: `crates/sifr_codegen/src/intrinsics/registry/process.rs:228-240` and the assertion at `crates/sifr_codegen/src/intrinsics/registry_extended_tests.rs:165`.

`status_signal()` emits `std::os::unix::process::ExitStatusExt::signal(&__status).unwrap_or(-1) as i64` with no `#[cfg(unix)]` gate. Grepping confirms this is the **first and only** unconditional `std::os::unix::*` emission in the entire codegen (`grep std::os::unix crates/{sifr_codegen,sifr_runtime,sifr_stdlib}/src/` returns only this path). All prior M4 work — including the timeout poll loop's `__child.kill()` — uses cross-platform `std::process::Child::kill()` and stays host-portable.

`verification/platform/supported_host_matrix.md:18` lists Windows x86_64 alongside macOS and Linux for "Subprocess spawning and termination" as `blocked-on-concurrency-runtime-m4`, i.e. M4 is the milestone that is supposed to unblock those rows. A Sifr program that calls `sifr.process.kill(child)` or `Child.kill()` would now emit Rust that fails to compile on Windows, breaking the project's "if it compiles, it works" guarantee for the M4 surface.

`verification/stdlib/concurrency_runtime_m4_process_traceability.md` does **not** declare a Unix-only constraint for `kill`/`Child.kill`, and the "Follow-up Boundaries" list does not include a Windows signal-representation deferral. So today the slice is silently Unix-only.

**Options to address (either is acceptable):**
- Gate the signal lookup with `#[cfg(unix)]` in the emitted Rust (e.g., a small `{ #[cfg(unix)] { …signal()…unwrap_or(-1) } #[cfg(not(unix))] { -1i64 } }` block) so the generated code compiles on Windows while `signal` still surfaces on Unix.
- Explicitly add a follow-up boundary in `verification/stdlib/concurrency_runtime_m4_process_traceability.md` noting that Windows signal representation is deferred, and document the host scope of `kind == "signal"` evidence honestly (current Unix-only).

The reviewer's question (4) is exactly this; the slice as-checked-in is honest only if the traceability is updated, otherwise codegen needs a guard.

---

## Non-blocking observations (no change required)

- **One-shot consumption is correct**: `take_child_stmts` at `crates/sifr_codegen/src/intrinsics/registry/process.rs:109-158` removes the handle from `__SIFR_PROCESS_CHILDREN` before `kill()`/`wait()`, and the Sifr-side `_waited = True` is set before the intrinsic call. A failed `process_kill` still consumes the Child (the underlying `std::process::Child` was already removed) — that's the right behavior so the user can't retry on a half-killed handle. Cross-references between top-level `wait`/`kill` and `Child.wait`/`Child.kill` are symmetric (`lib/sifr/process.sifr:80-99, 188-209`).

- **Panic surfaces are clean**: every `Result` is mapped via `process_map_err` to a typed `ProcessError`; `.code()` and `ExitStatusExt::signal(...)` are wrapped in `.unwrap_or(-1)`; mutex acquisition uses `.unwrap_or_else(|err| err.into_inner())`. No new `unwrap`/`expect`/panic paths reach generated runtime code.

- **Top-level `wait(child)` mutation is the right fix**: changing `wait(child: Child)` → `wait(mut child: Child)` with the `_waited` guard (`lib/sifr/process.sifr:188-198`) brings the module-level function in line with `Child.wait()` and gives a clean `"already been waited"` diagnostic instead of the underlying `"closed or unknown"` handle message. The fixture update (`crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr:20`) tracks this and is the right semantic.

- **Test coverage is adequate for the slice**: `process_child_kill_status.sifr` covers both `kill(child)` and `Child.kill()` happy paths, the post-kill `child.wait()` rejection, and the post-`Child.kill()` `kill(method_child)` rejection — 8 assertions match the milestone scope. The fail fixture `process_kill_direct_async_rejected.sifr` follows the same `task.sleep(0.0)`-without-import convention as `process_wait_direct_async_rejected.sifr` and `process_blocking_direct_async_rejected.sifr`, so SIFR-ASYNC-0003 fires before name resolution. Both manifests (create-PR and merge) include `process_child_kill_status`.

- **Traceability is otherwise honest**: deferrals for graceful `terminate`, async process lifecycle, pipe ownership, and scoped supervision are explicit in `verification/stdlib/concurrency_runtime_m4_process_traceability.md:40-51`, and `Status` kind/signal mapping is described correctly. Only the host-scope gap (above) is missing.

- **Stylistic nit (not raised as a finding)**: `try: …; except ProcessError as e: raise e` blocks in `kill`, `wait`, `Child.kill`, `Child.wait` are no-op wrappers. Pre-existing pattern, not introduced by this wave.
