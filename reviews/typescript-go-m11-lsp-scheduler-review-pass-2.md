I have enough context. Let me produce the review.

```markdown
# Code Review (Pass 2): M11 LSP Scheduler Queues (wave_tsgo_m11_lsp_scheduler)

## Status of pass 1 findings

| Pass 1 finding | Status | Evidence |
| --- | --- | --- |
| HIGH — post-shutdown requests tear down the run loop | Fixed | `crates/sifr_lsp/src/server.rs:93-103` builds a `Response::new_err(RequestCancelled)` and continues the loop; matched by `lsp_protocol_stress.py:111-117` |
| MEDIUM — close-with-pending diagnostic job poisons later flushes | Fixed | `crates/sifr_lsp/src/session.rs:111-115` drops the URI from `diagnostic_jobs` in `close_document`; covered by `close_document_discards_pending_diagnostic_job` at `session.rs:451-475` |
| LOW — `remove_pending` corrupts `queued_keys` on invariant violation | Fixed | `request_queue.rs:99-109` searches the lane queues first and only removes `queued_keys` after confirming a removal |
| LOW — duplicate dispatch trace | Fixed | `requests/mod.rs:19-20` no longer emits a per-dispatch trace; the single trace now lives at `session.rs:172-181` |
| NIT — background lane has no explanatory comment | Fixed | `scheduler.rs:17-18` explains the lane is a sentinel until M13 |
| NIT — cached-diagnostics shortcut bypassed version capture | Fixed (comment only) | `diagnostics.rs:83-85` documents why the shortcut is safe under the new version guard |

## Findings (severity-ordered)

### LOW — `LspServer::queued_requests` leaks request bodies if a request is cancelled before dispatch
`crates/sifr_lsp/src/server.rs:104-129`, `crates/sifr_lsp/src/session.rs:187-201`,
`crates/sifr_lsp/src/request_queue.rs:99-109`

`handle_request` now records the original `Request` in
`self.queued_requests` keyed by `request_key(&id)`, and `drain_queued_requests`
only removes that entry when `start_next_request` returns the same key. But
`Session::cancel_request` (via `$/cancelRequest`) goes through
`RequestQueue::remove_pending`, which only touches the scheduler's lane queue
plus `queued_keys` — it never reaches `LspServer::queued_requests`. If a
request is cancelled while still queued, its `Request` body sits in
`queued_requests` forever. The current stdio loop is fully synchronous, so the
cancel notification can't arrive before `drain_queued_requests` empties the
queue, and the leak is unreachable today. It becomes reachable as soon as M13
introduces a worker. Either move the request body into the scheduler entry,
or have `Session::cancel_request` return the cancelled key so the server can
discard the body in step.

### LOW — Defensive `else` branch in `drain_queued_requests` silently drops requests
`crates/sifr_lsp/src/server.rs:111-115`

```rust
let Some(request) = self.queued_requests.remove(scheduled.key()) else {
    self.session.finish_request(scheduled.id());
    continue;
};
```

If the scheduler ever yields a key whose `Request` body is missing, the
server clears `in_flight` and continues without responding — violating LSP's
exactly-one-response contract. The path is unreachable in the current
synchronous design (every enqueue is followed by an insert, which is followed
by a drain), but the silent drop will be a debugging nightmare if a future
refactor pulls enqueue and drain apart. Recommend emitting an
`InternalError` response on this branch (and/or a `debug_assert!`) so a
broken invariant is loud rather than silent.

### LOW — `Session::take_next_diagnostic_job` remains `O(n)` per pop
`crates/sifr_lsp/src/session.rs:247-254`

Pass 1's `O(n²)` `min_by_key` scan was acknowledged but not addressed: each
`flush_ready` iteration still walks the full `BTreeMap<String,
ScheduledDiagnosticJob>` to find the smallest `sequence`. Invisible at
single-document scale; observable when `workspace/didChangeWatchedFiles` fans
out across a large open workspace. A parallel `BTreeMap<u64 /* sequence */,
String /* uri */>` (kept in sync with `diagnostic_jobs`) would make this
`O(log n)` and preserve the URI-keyed debounce.

### LOW — Newest schedule for a URI inherits a new `sequence`, reordering flush across URIs
`crates/sifr_lsp/src/session.rs:228-245`

`schedule_document_diagnostics` always bumps `next_diagnostic_sequence` and
overwrites the existing entry, so if URI A is scheduled (seq 1), URI B is
scheduled (seq 2), and URI A is re-scheduled (seq 3), `flush_ready` now
publishes B before A even though A was queued first. The version guard keeps
the published *content* correct, but it does change the order
`publishDiagnostics` notifications arrive at the client. Cheap fix: when an
entry already exists, keep its existing `sequence` and only refresh
`version`. Worth a comment if the new ordering is intentional.

### LOW — Pending diagnostic jobs are not cleared when `diagnostics.mode` flips to `off`
`crates/sifr_lsp/src/diagnostics.rs:11-35`,
`crates/sifr_lsp/src/notifications/mod.rs:47-52`

`workspace_did_change_configuration` swaps settings but doesn't drop the
`diagnostic_jobs` accumulated under the previous mode. The
`mode == Off` early-return in `publish_document` / `flush_ready` then leaves
those jobs idling until the next event re-enables the mode, at which point
they all flush at once. The version guard keeps them safe, but it's strictly
more state than necessary. Either clear `diagnostic_jobs` when mode becomes
`Off`, or document why holding them is intended.

### NIT — `start_next_request` trace fires even for the request the same call enqueued
`crates/sifr_lsp/src/session.rs:159-181`

`enqueue_request` traces `queued request …` and `start_next_request` then
traces `dispatching request …` for the same id within a single
`handle_request`, so every request still produces two traces (down from
three). Tolerable, but worth folding into one trace when the scheduler is
synchronous, e.g. only trace the dispatch.

## Residual risks / missing tests

- **`scripts/run_all_tests.sh --profile quick` is still absent from M11
  validation** (`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:66-77`
  and the user-provided validation list). Pass 1 already flagged this; every
  prior milestone records it; AGENTS.md calls it the "authoritative gate".
  This needs to land before the milestone closes — current entries only
  cover `cargo test -p sifr_lsp` plus the verification scripts.
- **Workspace-wide clippy was narrowed to `-p sifr_lsp`**. The diff is
  LSP-scoped so this is unlikely to regress other crates, but every prior
  milestone records `cargo clippy --workspace -- -D warnings`. Re-run that
  before merge to stay consistent.
- **Latent cancellation leak (LOW above)** has no test because the sync
  loop can't reach it. Document the expectation in
  `internal_docs/typescript_go_architecture_transfer_m11_lsp_scheduler.md`
  so M13 picks it up — today the doc states `request execution remains
  serialized` but does not name the cancel-while-queued path that M13 will
  unlock.
- **No end-to-end test for the version-guard skipping a stale publish.**
  Unit tests cover the predicates and `flush_ready`'s consumers, but no
  stress-test step actually races `didChange` against a queued diagnostic
  job to observe `skipped stale diagnostics …`. A best-effort variant would
  schedule diagnostics, mutate the document, and assert that the resulting
  `publishDiagnostics` carries the latest version. The current stress test
  asserts the *latest* version field but never demonstrates a stale capture
  being dropped (because the sync loop also can't reach that state in
  practice).
- **Background lane still has no live consumer.** The architecture doc
  acknowledges this; just make sure the `sifr/backgroundIndex` sentinel
  remains internal until M13 — today nothing forbids a client from
  triggering the path with a manual JSON-RPC message, and `requests::handle`
  would simply return `MethodNotFound` (correct, but worth a deliberate
  smoke check before adding any real client surface).

## Verdict

**SATISFIED**

The HIGH protocol regression and MEDIUM orphan-job bug from pass 1 are both
fixed with focused regression tests (`lsp_protocol_stress.py:111-117`,
`session.rs::close_document_discards_pending_diagnostic_job`), the LOW
`remove_pending` invariant is restored, the duplicate dispatch trace is
gone, and the small documentation/comment gaps are addressed. The remaining
items are LOW/NIT defensive cleanups and one process gap (the
`--profile quick` validation entry) that should be settled before the
milestone is marked merged, but none of them block the code change itself.
```
