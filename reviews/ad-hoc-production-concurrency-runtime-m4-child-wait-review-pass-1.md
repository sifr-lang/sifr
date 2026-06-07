I have the empty review file from the user's scratch — I'll write my pass-1 review report there as well. Let me finalize the review verdict.

## Verdict: PASS (with non-blocking follow-ups)

The M4 sync child wait wave is honestly scoped and implementation-ready. All seven review concerns from the task brief check out:

### Verified

1. **Sync-only child lifecycle, no overclaim of pipes/async.** `spawn(Command) -> Result[Child, ProcessError]`, top-level `wait(Child)`, and `Child.wait()` are all `@blocking_io`, return typed `Result`, and the traceability/manifests explicitly preserve owned pipes, async wait, termination/timeout/cancellation, scoped supervision, and richer text mode as later M4 work. `Stdio` constants remain reserved-only.

2. **Generated process-child table is correctly gated.** I independently re-ran emit for `process_sync_output_text.sifr` (imports only `Command`+`output_text`): no `__SIFR_PROCESS_CHILDREN`/`__sifr_next_process_child_id`/`std::process::Child` matches. The DCE + `derive_shared_needs` path (`crates/sifr_codegen/src/stdlib_filter/implementation.rs:250-306`, `crates/sifr_codegen/src/lib_modules_and_codegen.rs:380-411`) only emits the table when the kept stdlib code references the symbols, which happens iff `spawn` / `wait` / `Child` survives DCE.

3. **`process_wait` is one-shot and typed; no data-dependent panics.** `crates/sifr_codegen/src/intrinsics/registry/process.rs:457-526` removes the handle from the map first, then `.ok_or_else(|| ProcessError { message: "process child handle is closed or unknown: …" })?`. Mutex `.unwrap_or_else(|err| err.into_inner())` is poison recovery (programmer-invariant). `.code().unwrap_or(-1)` is fine — `code()` returns `Option<i32>` only because signal exits have no code; `-1` is a data value, not a panic in a Result path. `Child.wait()` rejects double-call with `"already been waited"` before touching the table.

4. **Direct-async diagnostic works through workload metadata.** Confirmed locally: `cargo run -p sifr -- check crates/sifr/tests/e2e/fail/process_wait_direct_async_rejected.sifr` emits `error[SIFR-ASYNC-0003]: blocking_io function 'wait' called directly from async context`.

5. **Asymmetric observation between top-level `wait(child)` and `Child.wait()`.** Acceptable for this wave. Both surfaces produce `ProcessError`, just with different message text after mixed use. The `_waited` boolean is a local mirror; the runtime table remains the source of truth. The fixture explicitly tests both messages and the safety invariant holds in all four call orderings (`wait→wait`, `wait→method`, `method→wait`, `method→method`). Worth a non-blocking follow-up to unify the asymmetry by either deferring entirely to the runtime table or sharing the message via a helper.

6. **Resource/lifecycle docs are honest enough.** Traceability lists "owned pipe access, async wait, termination, timeout, and scoped supervision remain later M4 work" and "double-close/use-after-close diagnostics, and handle sendability/shareability checks beyond the one-shot sync `Child.wait()` state". Non-blocking: the docs don't explicitly call out that dropping an unwaited `Child` leaves a `std::process::Child` in the process-global table for the process lifetime (potential zombie on Unix). Worth a one-line follow-up in the "Follow-up Boundaries" section noting the leak risk.

7. **Preamble/filtering integration is robust.** `is_shared_prelude_item` (`stdlib_filter/implementation.rs:338-355`) now strips the static + the `__sifr_next_process_child_id` fn so they don't double-emit. `needs_mutex` (`lib_modules_and_codegen.rs:637-641`) correctly OR's in `needs_process_children` so the `use std::sync::Mutex` import is present. `derive_shared_needs_text_scan` mirrors the AST path for the fallback. Pattern is symmetric with existing `__SIFR_FILE_HANDLES` handling.

### Empirical verification

- `cargo run -q -p sifr -- build crates/sifr/tests/e2e/pass/process_spawn_wait_status.sifr` -> compiled successfully; binary execution returns success (asserts pass for all six expected booleans).
- `cargo run -q -p sifr -- emit ...process_sync_output_text.sifr | grep '__SIFR_PROCESS_CHILDREN|__sifr_next_process_child_id|std::process::Child'` -> no matches.
- `cargo run -q -p sifr -- check ...process_wait_direct_async_rejected.sifr` -> SIFR-ASYNC-0003 as expected.
- All four create-pr lane validations claimed in the ledger (cargo check, fmt, file-size + HIR guardrails, fail suite, run_all_tests.sh --profile create-pr with `93 passed`/`0 failed`/`cache_hits=24/25`) line up with the staged diff.

### Non-blocking follow-ups (do not block PR)

1. **Unify the wait-observation asymmetry.** Either remove `_waited` from `Child` and surface only `"closed or unknown"` consistently, or factor a shared helper so top-level `wait(child)` produces the same `"already been waited"` text. Either keeps behavior safe; the choice is cosmetic.

2. **Document the unobserved-Child leak in `verification/stdlib/concurrency_runtime_m4_process_traceability.md` Follow-up Boundaries.** One bullet: dropped-but-unwaited `Child` keeps `std::process::Child` in the process-global table for the process lifetime and may leak a zombie on Unix; deferred to the termination/timeout/cancellation wave.

3. **Importing `Child` (without calling `spawn`/`wait`) emits the runtime table** because the kept `Child::wait` method body references it. Verified empirically. Acceptable — anyone explicitly importing `Child` is by definition a child-handling user — but could be tightened later by lazily lowering `Child::wait` only when one of `spawn`/`wait`/`Child::wait` is referenced.

4. The existing M4 sync-process-foundation follow-up about deleting the unused legacy `_sifr.sys.subprocess_*` paths still stands — neither blocks this wave.

Ready to PR.
