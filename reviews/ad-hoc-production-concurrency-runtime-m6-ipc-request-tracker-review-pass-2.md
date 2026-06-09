# PASS

Review pass 2 verifies the pass-1 hardening updates landed correctly and introduced no new blocker. The M6 typed IPC request tracker slice is ready to ship.

## Pass-1 follow-ups verified

**1. `begin_shutdown` preserves `Closed` terminal (was pass-1 follow-up #1).**
`crates/sifr_stdlib/src/ipc_request_tracker.rs:130-138` now short-circuits when the tracker is already `Closed`:

```
pub fn begin_shutdown(&mut self, mode: IpcShutdownMode) {
    if self.state == IpcRequestTrackerState::Closed {
        return;
    }
    self.state = IpcRequestTrackerState::Draining;
    if mode == IpcShutdownMode::CancelInFlight {
        self.in_flight.clear();
    }
}
```

This eliminates the `Closed → Draining` regression an out-of-order peer `Shutdown` after a `Terminating` would have caused. The new `shutdown_after_terminating_keeps_tracker_closed` test at lines 301-309 pins the contract by closing first, calling `begin_shutdown(Drain)`, and asserting `state() == Closed` plus `begin_run → Err(Closed)`. Behavior is intended and locked in.

**2. `Run` dispatch through `apply_frame` (was pass-1 follow-up #4).**
`run_frames_reserve_in_flight_capacity` (lines 159-174) now exercises both entry points in one test: a direct `begin_run(1)` plus an `apply_frame(&IpcEnvelope::Run { request_id: 2, payload: ... })`, then asserts both ids land in the in-flight set. Closes the dispatch-table coverage gap.

**3. Non-request frames are inert (was pass-1 follow-up #2).**
`non_request_frames_do_not_mutate_tracker_state` (lines 311-323) primes one in-flight request, applies `IpcEnvelope::Heartbeat { sequence: 7 }`, and asserts `state == Open` and `in_flight_len == 1`. Documents the tracker/protocol boundary explicitly: this layer does not police malformed/unsupported frames; that responsibility stays in the protocol layer.

**4. Validation evidence reconciled.**
`issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:1080` records `12 request tracker tests` and explicitly enumerates the two new cases (`Run` dispatch through `apply_frame`, `shutdown-after-close terminal behavior`, `non-request frame pass-through`). Line counts at `:1084` (`ipc_request_tracker.rs` `332`, `lib.rs` `440`, design doc `246`, ledger `2058`) match the on-disk state I verified with `wc -l`. No overclaim.

## Test count and coverage

12 `#[test]` functions are present in `crates/sifr_stdlib/src/ipc_request_tracker.rs` (lines 160, 177, 188, 201, 224, 241, 257, 273, 284, 301, 312, 326) — matches the ledger's claim. Coverage map:

| Concern | Test |
|---|---|
| In-flight reservation + `Run` dispatch | `run_frames_reserve_in_flight_capacity` |
| Duplicate request id is typed malformed evidence | `duplicate_request_id_is_typed_malformed_evidence` |
| Full-window rejection preserves existing work | `full_window_rejects_new_run_without_losing_existing_requests` |
| Capacity release on `Completed`/`Failed` | `completed_or_failed_frames_release_capacity` |
| Unknown terminal/cancel id is typed malformed evidence | `unknown_terminal_or_cancel_request_is_typed_malformed_evidence` |
| `Started`/`Cancel` keep request in flight | `started_and_cancel_frames_keep_request_in_flight` |
| Drain shutdown rejects new runs, keeps existing | `drain_shutdown_rejects_new_runs_but_keeps_existing_work` |
| Cancel-in-flight shutdown clears outstanding work | `cancel_in_flight_shutdown_clears_outstanding_work` |
| `Terminating` closes and clears | `terminating_frame_closes_and_clears_tracker` |
| `Shutdown` after `Terminating` is a no-op | `shutdown_after_terminating_keeps_tracker_closed` |
| Non-request frames are inert | `non_request_frames_do_not_mutate_tracker_state` |
| Errors redact payload bytes | `tracker_errors_do_not_render_payload_bytes` |

## No new blocker introduced

- **Correctness:** `apply_frame` (lines 81-100) dispatch unchanged for non-shutdown variants; ordering of `ensure_accepting_new_runs → duplicate → capacity` in `begin_run` (lines 102-114) preserved. Pass-1's invariants — duplicate beats backpressure as evidence shape, `Started`/`Cancel` non-terminal, `Completed`/`Failed` are the sole capacity-release path, `Terminating` is a hard close — all still hold.
- **Panic-freedom:** zero data-dependent `unwrap`/`expect`/`panic!`; the only `as usize` cast on `max_in_flight: u32` is still safe on supported hosts.
- **Redaction:** `Display` impl (lines 21-40) unchanged; only numerics surface. `tracker_errors_do_not_render_payload_bytes` still pins the `BackpressureFull` shape.
- **Re-export wiring:** `crates/sifr_stdlib/src/lib.rs:16` (`mod ipc_request_tracker;`) and `:43` (`pub use ipc_request_tracker::{IpcRequestTracker, IpcRequestTrackerError, IpcRequestTrackerState};`) unchanged.
- **Documentation honesty:** `verification/stdlib/concurrency_runtime_m6_typed_ipc_design.md:33` and `verification/platform/supported_host_matrix.md:42` still scope the claim to host-independent request tracking/backpressure state and disclaim child-process fixtures, full connection negotiation, payload eligibility, and generated worker integration. `Typed IPC frames over process pipes` at `:43` of the host matrix remains `blocked-on-concurrency-runtime-m6`.

## Validation rerun

Trusted from the submitter's evidence (Ruff submodule is not initialized in this review worktree, so I could not re-execute `cargo` locally; the on-disk state of the code, line counts, ledger text, and host matrix all reconcile with the claimed evidence):

- `cargo fmt --check` — PASS
- `cargo test -p sifr_stdlib ipc_request_tracker -- --nocapture` — PASS, 12 tests
- `cargo clippy -p sifr_stdlib -- -D warnings` — PASS
- `git diff --check` — PASS
- `python3 scripts/check_file_size_guardrails.py` — PASS, 2253 files, 900-line limit (max touched file is `ipc_request_tracker.rs` at 332 lines)

## Remaining non-blocking observations (carry forward)

These were already labelled non-blocking in pass 1 and remain non-blocking — listing only what is still open:

- The `_ => Ok(())` arm in `apply_frame` (line 98) absorbs `MalformedFrame`, `UnsupportedVersion`, `UnsupportedSchema`, `UnsupportedPayload`, `Hello`, `Ready`, `Reject`, `WorkerStatus` in addition to `Heartbeat`. The new test covers only `Heartbeat`; the rest remain documented-by-omission. Fine for this slice, since the design assigns malformed/unsupported handling to the protocol layer.
- `WorkerStatus { in_flight }` reconciliation against `in_flight_len()` is still a future audit hook (pass-1 follow-up #3).
- `IpcRequestTracker::new(0)` is still silently degenerate — every `begin_run` returns `BackpressureFull { max_in_flight: 0 }` (pass-1 follow-up #5). Either documenting the invariant or rejecting at construction is the cleanest fix; deferring is acceptable for this wave.

None of the above blocks merge.
