# Ad Hoc M4 Async Process Spawn/Wait Review — Pass 1

Scope: working-tree review of the M4 async process spawn/wait wave for the production concurrency/runtime/platform substrate phase.

Reviewer date: 2026-06-08.

## Verdict

`PASS` — implementation is consistent with the narrow wave scope, generated runtime is panic-free under the documented failure modes, the test fixture exercises the documented surface, and ledger/traceability/host-matrix updates match what the wave actually delivers. Residual non-blocking concerns are listed below.

## Files audited

- `lib/sifr/process.sifr`
- `crates/sifr_stdlib/src/process.rs`
- `crates/sifr_codegen/src/intrinsics/registry.rs`
- `crates/sifr_codegen/src/intrinsics/registry/process_async.rs`
- `crates/sifr_codegen/src/preamble/process_async_runtime.rs`
- `crates/sifr_codegen/src/stdlib_filter/implementation.rs`
- `crates/sifr_codegen/src/lib_modules_and_codegen.rs`
- `crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr`
- `verification/stdlib/concurrency_runtime_m4_process_traceability.md`
- `verification/platform/supported_host_matrix.md`
- `verification/validation_lanes/create_pr_e2e_manifest.json`
- `verification/validation_lanes/merge_e2e_manifest.json`
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md`

## What the wave delivers (verified)

1. Public `sifr.process` surface:
   - `class AsyncChild` with `_handle: int` and a `wait(self) -> Awaitable[Result[Status, ProcessError]]` method that delegates to `process_async_wait(self._handle)`.
   - `async_spawn(own command: Command) -> Awaitable[Result[AsyncChild, ProcessError]]` (lib/sifr/process.sifr:408-419), wiring `program`, `arguments`, `env_vars`, `working_dir`, `has_working_dir`, `stdin_mode`, `stdout_mode`, `stderr_mode`, and `has_stdin_data` into `process_async_spawn`.
   - `async_wait(own child: AsyncChild) -> Awaitable[Result[Status, ProcessError]]` (lib/sifr/process.sifr:422-423) consuming the AsyncChild and forwarding `child._handle` to `process_async_wait`.

2. Stdlib intrinsic typing (`crates/sifr_stdlib/src/process.rs:293-316`):
   - `process_async_spawn` takes the 9-tuple `(program, args, env, cwd, has_cwd, stdin_mode, stdout_mode, stderr_mode, has_stdin)` and returns `Awaitable[Result[AsyncChild, ProcessError]]` where the new `process_async_child_class()` matches the public surface field shape.
   - `process_async_wait` takes `(handle: int)` and returns `Awaitable[Result[Status, ProcessError]]`.

3. Lowerers (`crates/sifr_codegen/src/intrinsics/registry/process_async.rs:82-104`):
   - `lower_process_async_spawn` requires 9 args, clones `stdout_mode` and `stderr_mode` consistent with the existing async stdin_mode/cwd cloning convention, passes the `bool` `has_stdin` by value, and boxes a `__sifr_process_async_spawn(...)` call via `Box::pin`.
   - `lower_process_async_wait` requires 1 arg and boxes `__sifr_process_async_wait(handle)`.
   - Registry wiring (`registry.rs:626-633`) flags both with `Some(StdlibFeature::Tokio)` so the Tokio feature is required only when the user actually invokes spawn/wait.

4. Generated runtime (`crates/sifr_codegen/src/preamble/process_async_runtime.rs:222-244, 246-331, 586-620, 728, 774-795`):
   - Private `__SIFR_PROCESS_ASYNC_CHILDREN: LazyLock<Mutex<HashMap<i64, tokio::process::Child>>>` table; emission is gated by `needs_spawn || needs_wait` (table is needed for either side).
   - Private `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID: AtomicI64::new(1)` and helper `__sifr_next_process_async_child_id` are emitted only when `needs_spawn` is true. Wait alone does not require the id helper, which is correct.
   - `__sifr_process_async_spawn` is gated on `needs_spawn`; `__sifr_process_async_wait` is gated on `needs_wait`.
   - Spawn helper guards in this order: (a) reject `has_stdin` with `"async process spawn does not consume Command.stdin_bytes"`, (b) reject any non-`"inherit"` `stdin_mode`/`stdout_mode`/`stderr_mode` with `"async process spawn stdio modes require async owned pipe support"`. Both errors flow as typed `ProcessError`.
   - Spawn inserts the spawned `tokio::process::Child` under a freshly allocated id, then returns `AsyncChild::new(__handle)`. The mutex guard is dropped at the end of the inner block before returning; nothing is awaited while the guard is held.
   - Wait removes the entry from the table inside a scoped block (so the `MutexGuard` is dropped before `.await`), maps a missing entry to `ProcessError { message: format!("async process child handle {} is closed or unknown", handle) }`, then awaits `__child.wait()` and converts via `__sifr_process_status_from_exit` + `__sifr_process_exit_signal` (the same status/signal helpers already used by async run/output).
   - Poisoned-mutex recovery uses `.lock().unwrap_or_else(|__err| __err.into_inner())`, so a panicking sibling cannot tip the lock into an unrecoverable panic for either spawn or wait.

5. Shared-prelude classification (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:335-402, 421-443`):
   - Text-scan derivation uses `__sifr_process_async_spawn(` and `__sifr_process_async_wait(` (paren-suffixed) to disambiguate from each other and from existing async-run/output names.
   - AST collector sets `needs_spawn` on observation of `__sifr_process_async_spawn`, `__sifr_next_process_async_child_id`, or either of the two private statics, and sets `needs_wait` on observation of `__sifr_process_async_wait`.
   - `is_shared_prelude_item` recognizes both statics and all three new functions, so per-module emissions correctly defer to the single shared prelude copy.

6. Driver wiring (`crates/sifr_codegen/src/lib_modules_and_codegen.rs:394-401, 424-432, 600-608`):
   - `stdlib_needs_process_async.needs_spawn`/`needs_wait` are OR-aggregated across modules.
   - `needs_process_async` (which gates the entire async helper block including the shared `__sifr_process_status_from_exit`) now considers spawn/wait too.
   - `build_process_async_items` is called with all six needs flags.

7. Fixture and lane manifests:
   - `crates/sifr/tests/e2e/pass/process_async_spawn_wait.sifr` validates: nonzero exit code via `async_spawn`+`async_wait`, success via method-form `AsyncChild.wait()`, second-wait returning `"closed or unknown"` substring through the runtime handle table, `stdin_bytes(...)` rejection (substring `"stdin_bytes"`), and explicit `stdin(Stdio("pipe"))` rejection (substring `"owned pipe"`).
   - `create_pr_e2e_manifest.json` and `merge_e2e_manifest.json` add `process_async_spawn_wait` in lexicographic position immediately after `process_async_output_timeout`.

8. Documentation and platform matrix:
   - `verification/stdlib/concurrency_runtime_m4_process_traceability.md` adds a new dedicated row for `AsyncChild`/`async_spawn`/`async_wait`/`AsyncChild.wait` that explicitly states only inherited stdio is supported, that `stdin_bytes(...)` and explicit `stdin/stdout/stderr` modes return typed `ProcessError` deferrals until public async pipe handles land, and that async kill/terminate, cancellation-safe observation, scoped process supervision, and shell async APIs remain later M4 work. CPython family mapping and validation-coverage tables are extended to reference the new fixture without overclaiming.
   - `verification/platform/supported_host_matrix.md` adds an `Async subprocess spawn/wait` row marked `supported` (Linux/macOS) and `host-limited` (Windows), with notes that mirror the documented deferrals.
   - The phase ledger lists the wave as `in progress`, records targeted local validation evidence, and does not yet claim merge — consistent with the wave's pre-merge status.

## Spot checks I executed against the criteria

- Guardrail semantics: spawn helper code explicitly rejects `has_stdin` before constructing the Tokio command; explicit stdio modes are rejected before `__cmd.spawn()` ever runs. Neither path silently consumes `stdin_bytes(...)`. ✔
- Single-observation: wait removes from the table before awaiting; the mutex guard is dropped at the end of the inner `{ ... }` block, so `.await` runs without holding the lock. Second wait on the same handle hits `ok_or_else` and returns a typed closed/unknown error. The fixture exercises this path. ✔
- Emission gating: `__SIFR_PROCESS_ASYNC_CHILDREN` is gated on `needs_spawn || needs_wait`; `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID` and `__sifr_next_process_async_child_id` are gated on `needs_spawn`; `__sifr_process_async_spawn` and `__sifr_process_async_wait` are gated on their respective flags. Async run/run_timeout/output/output_timeout helpers do not reference any of the spawn/wait identifiers, so existing async helpers continue to emit independently. ✔
- Public Tokio types: the user-facing `AsyncChild` carries only `_handle: int` (and an unused `_waited: bool`, see residual notes). No `tokio::process::Child` is exposed across the generated public surface. ✔
- Panic freedom: `Mutex::lock()` failures recover via `into_inner`; `child.wait()` errors and `Command::spawn()` errors are mapped to typed `ProcessError`; `?` is used only on `Result`s that are mapped to typed errors first. No `unwrap`/`expect` on data-dependent paths. ✔

## Residual non-blocking notes

1. `AsyncChild._waited` (lib/sifr/process.sifr:164, 168) is declared and initialized but is never read or written anywhere. The sync sibling `Child._waited` is used to fail fast inside `Child.wait()`; the async sibling deliberately relies on the runtime handle table to detect double-wait. Either drop the field, or wire it through `AsyncChild.wait()` analogously to sync. Not a blocker because the runtime check is sound and the fixture exercises it, but leaving dead state on a public class invites confusion.

2. The AST-visit collector in `stdlib_filter/implementation.rs:394-402` includes `__SIFR_PROCESS_ASYNC_CHILDREN`, `__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID`, and `__sifr_next_process_async_child_id` in the `needs_spawn` matcher. Those identifiers only ever appear inside the helper bodies themselves (which are pulled out as shared prelude items), so under current codegen they cannot appear in per-module user-visible AST and the match branches are effectively dead. This mirrors the existing sync-child pattern (`__SIFR_PROCESS_CHILDREN` et al.), so leaving it as-is is consistent; just worth a note.

3. The new fixture validates `success` and `nonzero` Status flow through async wait but does not directly exercise Unix signal-status flow through async wait. Signal-status evidence remains carried by `process_signal_status` (sync). The traceability table now lists `process_async_spawn_wait` as one of the Status surface witnesses, which is technically accurate (it does observe `Status` kinds) but does not extend signal coverage. Consider tightening the per-fixture wording if a future review wants stricter Status sub-aspect mapping; not a wave blocker.

4. `crates/sifr_codegen/src/preamble/process_async_runtime.rs` is now 798 lines (well under the 900-line cap, but the trajectory is clear). The next async-process slice — public async owned pipes, async kill/terminate, cancellation-safe wait — will likely push past 900. Plan a responsibility-based split (e.g. `process_async_spawn`, `process_async_run`, `process_async_status`) before the next wave rather than after the cap trips.

5. The fixture only exercises explicit `stdin(Stdio("pipe"))` for the stdio-mode rejection. The runtime guard is symmetric across `stdin_mode`/`stdout_mode`/`stderr_mode` (`!= "inherit"` on any one triggers), so a follow-up could add an explicit `stdout(Stdio("pipe"))`/`stderr(Stdio("pipe"))` rejection case. Not a blocker because the rejection is a single boolean expression covering all three.

## Bottom line

Implementation, generated runtime, fixture, intrinsic metadata/lowerers, prelude classification, host matrix, traceability, and phase ledger are all consistent with each other and with the wave's stated narrow scope. Documentation does not overclaim public async pipes, cancellation, termination, scoped supervision, async shell, text-mode closeout, or Windows. The wave is ready to proceed to PR; the residual notes are cleanup, not correctness.
