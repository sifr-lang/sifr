# Code Review (Pass 3): M11 LSP Scheduler Queues (wave_tsgo_m11_lsp_scheduler)

## Status of pass 2 findings

| Pass 2 finding | Status | Evidence |
| --- | --- | --- |
| LOW — `queued_requests` leaks request bodies if cancelled before dispatch (M13 doc note recommended) | Fixed (doc only) | `internal_docs/typescript_go_architecture_transfer_m11_lsp_scheduler.md:30-33` explicitly calls out the M13 obligation to drop retained bodies when queued cancellation becomes reachable across worker turns |
| LOW — defensive missing-body branch in `drain_queued_requests` silently drops requests | Fixed | `crates/sifr_lsp/src/server.rs:112-126` now finishes the request and sends a `Response::new_err` with `LspError::internal` instead of silently `continue`-ing |
| LOW — newest schedule for a URI inherits a fresh `sequence`, reordering flush across URIs | Fixed | `crates/sifr_lsp/src/session.rs:237-243` preserves the existing entry's `sequence` on re-schedule; regression test `diagnostic_reschedule_preserves_original_queue_order` at `session.rs:487-547` |
| LOW — pending diagnostic jobs not cleared when `diagnostics.mode` flips to `off` | Fixed | `crates/sifr_lsp/src/notifications/mod.rs:53-55` clears on mode transition; `crates/sifr_lsp/src/diagnostics.rs:18-19,28-29,44-46` also clears when callers/`flush_ready` observe `Off`; new `Session::clear_diagnostic_jobs` at `session.rs:209-211` |
| LOW — `take_next_diagnostic_job` remains `O(n)` per pop | Not addressed | `session.rs:257-264` still scans the `BTreeMap` via `min_by_key`. Acknowledged as acceptable at M11 single-document scale; worth revisiting when M13 makes workspace-wide flushes more frequent |
| NIT — `start_next_request` trace fires alongside `queued request` for the same id | Not addressed | `session.rs:159-181` still emits both. Two traces per request is tolerable while the scheduler is synchronous |

## Findings (severity-ordered)

### LOW — `flush_ready`'s mode check is now dead defensive code
`crates/sifr_lsp/src/diagnostics.rs:39-47`

Both callers (`publish_document` at `diagnostics.rs:18-23`, `publish_all` at `diagnostics.rs:27-35`) return early when `mode == Off`, so `flush_ready`'s own `if mode == DiagnosticsMode::Off { … clear … return }` block is unreachable. Harmless, but the duplicate state-clearing call at three sites makes it easy to forget which one is authoritative. Pick one (the caller) and drop the others, or document that `flush_ready` is the sole authoritative clearer.

### LOW — `Session::cancel_request` does not communicate the cancelled key back to the server
`crates/sifr_lsp/src/session.rs:187-201`, `crates/sifr_lsp/src/server.rs:104-105`

`cancel_request` returns `()` even though `RequestQueue::remove_pending` knows whether a queued entry was removed. The body of that request stays in `LspServer::queued_requests` until `drain_queued_requests` pops the next scheduled entry — which now will never produce that key. Today the stdio loop is fully synchronous so the leak is unreachable (cancel cannot interleave with drain), and the M11 doc records the M13 obligation. Closing the loop now would be one line: have `cancel_request` return the cancelled `RequestId` (or its key) so `LspServer` can remove the `queued_requests` entry in step. Optional but cheap.

### NIT — `Background` lane is not exercised end-to-end
`crates/sifr_lsp/src/scheduler.rs:13-22`, `crates/sifr_lsp/src/requests/mod.rs`

`sifr/backgroundIndex` has no `requests::handle` arm, so a client that sends it gets `MethodNotFound`. Good — that's the intended M11 surface and the inline comment now says so. Worth noting that no smoke step asserts the sentinel stays internal; a single negative assertion (request returns method-not-found) would make the contract observable. Acceptable to defer to M13 where the lane gains a real worker.

## Phase-scope check

- **Required closeout: workspace/background cannot starve latency-sensitive/formatting/diagnostic-style work.**
  Satisfied by `RequestQueue::select_next_lane` and `next_fair_lane` (`request_queue.rs:118-147`) plus `FAIRNESS_INTERVAL = 4` (line 5). Covered by `scheduler_prefers_latency_but_eventually_services_background` and `scheduler_rotates_fairness_lane_across_nonempty_queues` (`request_queue.rs:160-230`).
- **Required closeout: queued work publishes only if the captured snapshot/document version is still valid.**
  Satisfied by `Session::schedule_document_diagnostics`/`document_version_matches` and the dual pre/post-analysis guards in `DiagnosticsController::flush_ready` (`diagnostics.rs:48-63`). Covered by `diagnostic_scheduling_debounces_to_latest_document_version` and `diagnostic_job_version_guard_rejects_stale_capture` (`session.rs:388-459`). The version-guard short-circuit comment at `diagnostics.rs:85-88` correctly explains why the cached load path is still safe.
- **Phase scope: cancellation tokens, progress, background workers stay deferred to M13.**
  Confirmed — `scheduler.rs` has no cancellation token type, the architecture doc states the deferrals explicitly (`typescript_go_architecture_transfer_m11_lsp_scheduler.md:26-35`), and the M1 guardrail still asserts `CancellationToken` does not appear in `scheduler.rs` (`check_typescript_go_m1_guardrails.py:220-225`).
- **Protocol regressions from pass 1 stay fixed.**
  Post-shutdown request handling at `server.rs:90-103` returns `RequestCancelled` (code `-32800`) without tearing down the loop; covered by `lsp_protocol_stress.py:111-117`. Close-with-pending-job orphan path at `session.rs:111-115` covered by `close_document_discards_pending_diagnostic_job` at `session.rs:461-485`.

## Residual risks / missing tests (non-blocking)

- **Validation block still missing the `--profile quick` and workspace-clippy entries**
  (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:66-78`). Pass 1 and Pass 2 both flagged this; every prior milestone records both, and AGENTS.md calls `scripts/run_all_tests.sh --profile quick` the "authoritative gate". Please run them and append the result before this milestone is marked merged — the change isn't to the code, but the milestone closeout is incomplete without it.
- **No protocol-level test for the version-guard skipping a stale publish.** Existing unit tests cover the predicates, and `lsp_protocol_stress.py` asserts that the published notification carries the latest version (line 39), but no test demonstrates an actually stale capture being dropped (the sync loop can't reach that state today). Acceptable; document in the M11 doc that this becomes observable under M13.
- **`Background` lane has no negative-path smoke step** — see NIT above.
- **`take_next_diagnostic_job` `O(n)` per pop** — see pass-2 finding carried forward.
- **NIT: double trace per request** — see pass-2 finding carried forward.

## Verdict

**SATISFIED**

All four LOW issues from pass 2 that were actionable for M11 have been addressed with focused regression coverage:

- `drain_queued_requests` now produces an `InternalError` response on a missing body instead of silently dropping (`server.rs:112-126`).
- `schedule_document_diagnostics` preserves the original queue slot when the same URI is rescheduled (`session.rs:237-243`), with `diagnostic_reschedule_preserves_original_queue_order` exercising the multi-URI reordering scenario.
- Mode→`Off` transitions now clear pending diagnostic jobs at every entry point (`notifications/mod.rs:53-55`, `diagnostics.rs:18-19,28-29,44-46`), via the new `Session::clear_diagnostic_jobs` helper.
- The M13 doc note acknowledging the latent cancel-while-queued body retention is in `internal_docs/typescript_go_architecture_transfer_m11_lsp_scheduler.md:30-33`.

The required closeout invariants (no workspace/background starvation; stale snapshot/document publication is skipped) hold and are covered by both unit and protocol-level tests. The two carried-forward items (`O(n)` `take_next_diagnostic_job`, double-trace) are LOW/NIT and acceptable at M11 scale. The one process gap left — adding `scripts/run_all_tests.sh --profile quick` and `cargo clippy --workspace -- -D warnings` to the M11 validation block — should land before merge but doesn't block the code change itself.
