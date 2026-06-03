I have enough information to produce the review.

```markdown
# Code Review: M11 LSP Scheduler Queues (wave_tsgo_m11_lsp_scheduler)

## Findings (severity-ordered)

### HIGH — Post-shutdown requests now terminate the server instead of being rejected
`crates/sifr_lsp/src/server.rs:90-100`

`handle_request` now propagates `enqueue_request`'s `Err` out of the run loop:

```rust
self.session
    .enqueue_request(&id, &request.method, lane)
    .map_err(LspError::request_cancelled)?;   // ← breaks the recv loop
```

`RequestQueue::enqueue` returns `Err("server is shutting down")` once
`begin_shutdown` has been called (`request_queue.rs:60`). In the previous
implementation (`session.start_request` failure inside `and_then`), that
error was converted into a `Response::new_err(id, code, message)` and shipped
back to the client; the loop continued. Now the `?` propagates a boxed
`LspError` out of `LspServer::run`, the loop exits via `self.finish()`, and
the client never gets a response.

This contradicts the documented LSP contract in
`verification/tooling/lsp_protocol_matrix.json:51`
(`"requests after shutdown fail deterministically"`) and is the existing
behavior the matrix is asserting. No automated test exercises the
shutdown→request→exit sequence (smoke/stress both end with
`shutdown` then `exit` with no requests in between), so the regression is
silent.

Fix: keep request-cancelled errors inside the response branch — e.g.
build a `Response::new_err` and send it instead of `?`-ing out of the
recv loop. Failing to enqueue is per-request, not per-server.

### MEDIUM — Diagnostic jobs are not cleared when a document closes (orphan jobs poison later flushes)
`crates/sifr_lsp/src/session.rs:111-114, 202-206, 227-244`
`crates/sifr_lsp/src/diagnostics.rs:45-52`

`close_document` only touches `analysis` and `store`; it does not remove
the URI from `Session::diagnostic_jobs`. If a job for URI `X` is queued
and the surrounding flush errors out partway (e.g. transport send fails,
or `document_diagnostics` returns `Err`), the entry survives. Once
`X` is closed, the next `flush_ready` invocation pops that stale job and
calls `session.document_version_matches(&job.uri, job.version)?` —
`store.document(uri)` returns `LspError::invalid_params("document is not
open: …")`, the `?` propagates, and the new flush fails with an error
unrelated to whatever triggered it.

Also: there is no test covering a close-with-pending-job; both the new
unit tests assume the document stays open between schedule and take.

Fix options: drop `diagnostic_jobs` entries for the URI inside
`close_document`, or treat an `invalid_params` lookup inside `flush_ready`
as "drop the orphaned job and continue".

### LOW — `RequestQueue::remove_pending` corrupts `queued_keys` on invariant violation
`crates/sifr_lsp/src/request_queue.rs:99-113`

```rust
let removed_queued = if self.queued_keys.remove(&key) {
    for lane_queue in self.queued.values_mut() {
        if let Some(index) = lane_queue.iter().position(|r| r.key == key) {
            lane_queue.remove(index);
            return true;
        }
    }
    false           // queued_keys says yes, lane queues say no → key dropped
} else { false };
```

If the `queued_keys` set and the lane queues ever fall out of sync, we
silently drop the `queued_keys` entry without removing anything from the
queue. Today the data structure stays consistent, so this is a defensive
issue rather than a live bug — but it's safer to check the lane queues
first and only mutate `queued_keys` after confirming removal, mirroring
the way `start_next` updates both atomically.

### LOW — `take_next_diagnostic_job` is `O(n)` per pop
`crates/sifr_lsp/src/session.rs:246-253`

`min_by_key` scans the whole `BTreeMap` for each pop, so draining `n`
diagnostic jobs is `O(n²)`. With M11's single-document common path this
is invisible, but `publish_all` from `workspace/didChangeWatchedFiles`
fans out across every open document. A `BTreeMap<u64 /* sequence */,
String /* uri */>` ordered index alongside the URI map would keep the
debounce semantics and pop in `O(log n)`.

### LOW — Duplicate lane classification and triple trace per request
`crates/sifr_lsp/src/server.rs:92`, `crates/sifr_lsp/src/requests/mod.rs:21`,
`crates/sifr_lsp/src/session.rs:165-167, 173-178`

`Scheduler::lane_for_method` runs in `handle_request` (for enqueue) and
again in `requests::handle` (for the dispatch trace). Each request now
emits three traces: `queued …`, `dispatching request …`, and
`dispatching {method} on {lane:?} lane`. Cosmetic, but worth tidying —
e.g. drop the older trace in `requests::handle` now that the new
`start_next_request` trace already records method + lane.

### NIT — `Background` lane is dead until M13
`crates/sifr_lsp/src/scheduler.rs:13-20`

`sifr/backgroundIndex` isn't handled by `requests::handle`; it exists
only so the scheduling tests can prove fairness. That's fine per the
M11 plan ("background index work is represented by a scheduler lane and
fairness tests, but no background worker is started"), but worth a
one-line comment near `lane_for_method` so a future reader does not try
to wire it to a handler. Optional.

## Residual risks / missing tests

- **No protocol-level test for post-shutdown requests** — the very
  scenario the matrix promises ("requests after shutdown fail
  deterministically"). Add a stress-test step that issues a non-shutdown
  request between `shutdown` and `exit` and asserts an error response is
  returned and the server still processes `exit` cleanly. This would
  have caught the HIGH finding above.
- **No test exercising close-with-pending-job.** Add a session unit test
  that schedules diagnostics, closes the document, then calls
  `flush_ready` (directly or via another `publish_document`) and asserts
  no error.
- **`scripts/run_all_tests.sh --profile quick` is not listed in M11
  validation** in `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`
  (lines 66–76). Every prior milestone records it; AGENTS.md calls it
  the "authoritative gate". Run it and append the result before merge.
- **Fairness/debounce is only proved by unit tests.** Acceptable for
  M11 since the stdio loop drains synchronously, but worth a comment in
  `internal_docs/typescript_go_architecture_transfer_m11_lsp_scheduler.md`
  noting that the data-structure guarantees become observable only once
  M13 introduces a worker.
- **Cached diagnostics shortcut bypasses version capture** (existing
  behavior at `diagnostics.rs:83-90`). `document_diagnostics` returns
  cached `load_diagnostics(uri)` without re-running analysis or
  consulting the captured version. Today the cache is invalidated on
  every document update so it is harmless, but the new version-guard
  story only holds because of that invariant. Worth at least an
  inline comment so a future cache-tuning change doesn't silently
  break the "captured-version" contract.

## Verdict

**CHANGES_REQUESTED**

The HIGH finding is a real protocol regression: after the client issues
`shutdown`, the very next request now tears the server loop down instead
of returning an error response, contradicting the LSP contract recorded
in `lsp_protocol_matrix.json`. The MEDIUM orphan-job issue is narrower
but lives along an error path the new debounce design introduced.
Neither has a regression test. Please address the HIGH finding, add the
missing post-shutdown stress step, and run `scripts/run_all_tests.sh
--profile quick` before this milestone closes.
```

Findings summary: one HIGH regression in `server.rs` post-shutdown handling, one MEDIUM around orphaned diagnostic jobs after close, plus several low/nit items and process gaps (missing protocol test, missing quick-profile validation). Verdict: **CHANGES_REQUESTED**.
