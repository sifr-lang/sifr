# M13 Review Pass 2 — SATISFIED with non-blocking residual risks

## Decision: SATISFIED

The two close-to-blocking items from pass 1 are fixed and the working tree
re-validates cleanly.

Verification re-run on the working tree (matches the validation list in
`internal_docs/typescript_go_architecture_transfer_m13_lsp_cancellation_progress_watchdog.md`):

- `python3 verification/tooling/check_typescript_go_m1_guardrails.py` -> PASS
- `cargo test -p sifr_lsp` -> PASS, 23 tests
- `cargo clippy -p sifr_lsp -p sifr -- -D warnings` -> PASS
- `git diff --check` -> PASS

## What pass 1 raised and what changed

- Pass 1 item 1 (unbounded `ProgressState.events` Vec). Fixed.
  `events: Vec<ProgressEvent>` is now `#[cfg(test)]`-only
  (`crates/sifr_lsp/src/progress.rs:62`); production state retains only a
  bounded `completed: u64` counter (`progress.rs:60,107`) with
  `saturating_add`. No production-path Vec growth on multi-document publishes
  remains. The matching test-only `events()` accessor and
  `ProgressEvent`/`ProgressPhase` types are also gated.
- Pass 1 item 4 (dead generic `$/cancelRequest` arm in `notifications::handle`).
  Fixed. `notifications/mod.rs:13-31` no longer contains a `$/cancelRequest`
  branch; `server.rs:73-77` is the only path and runs the
  `send_cancelled_response` / `queued_requests.remove` cleanup via
  `cancel_request` (`server.rs:128-141`). Confirmed by reading both files end
  to end.
- Pass 1 items 6 and 7 (Windows watchdog no-op; progress threshold is
  document-count not time) are now explicitly documented in the M13 doc
  (`internal_docs/typescript_go_architecture_transfer_m13_lsp_cancellation_progress_watchdog.md:19,25-29`)
  rather than reworked. Reasonable scope choice; the user-visible behavior
  matches the doc.

## Residual non-blocking risks

Carried forward unchanged from pass 1 (still accurate, still non-blocking):

1. **`CancellationToken` is still a tautological wrapper** —
   `crates/sifr_lsp/src/cancellation.rs:3-18` stores only a cloned `RequestId`
   and exposes only `request_id()`. The locked architecture decision text says
   the token should expose `is_cancelled()` /
   `check_cancelled() -> Result<(), Cancelled>`. Today the predicate lives on
   `Session::check_request_cancelled` / `Session::check_active_request_cancelled`
   (`session.rs:240-255`). Structural divergence from the locked decision shape,
   not a correctness bug.
2. **Parent-death exit is still classified as
   `INTERNAL_COMPILER_PANIC`** —
   `crates/sifr/src/cli_model_and_entrypoint.rs:661-670` maps any
   `run_stdio_with_options` error to `INTERNAL_COMPILER_PANIC` /
   `EXIT_INTERNAL_COMPILER_FAILURE`, including the
   `LspError::request_cancelled("parent process N is no longer alive")` raised
   by `ParentWatchdog::check` (`watchdog.rs:31`). Cosmetic for a stream nobody
   is reading after the parent dies, but the diagnostic is misleading.
3. **Watchdog still forks `kill -0` per message** — `watchdog.rs:39-47` runs
   `Command::new("kill").arg("-0")` from the LSP message loop
   (`server.rs:61`). Per-message fork+exec overhead. Worth a follow-up with
   `rustix::process::test_kill_process` to stay clear of `unsafe_code`.
4. **Reserved-but-unused `ProgressKind` variants** —
   `References`, `IndexWarming`, `WorkspaceLoad` remain declared with
   `#[allow(dead_code)]` (`progress.rs:8-14,27-42`) with no emit sites. The
   M13 doc acknowledges this is intentional for future milestones.
5. **Stress test does not drive a real cancel race** —
   `verification/tooling/lsp_protocol_stress.py:44` still sends
   `$/cancelRequest` for id `99999`, which was never issued. Given the new
   pass-2 observation in the next section, a true in-flight cancel race cannot
   be driven over stdio in the M13 serialized model; the queue-only unit test
   at `request_queue.rs:267-281` continues to cover the queued path.

## New pass-2 observations

These were not in pass 1. Neither is blocking.

A. **`ProgressState.completed` is a write-only field.**
`progress.rs:60` declares `completed: u64`; `progress.rs:107` increments it on
every `end`. No reader exists in the crate (`grep completed crates/sifr_lsp`
finds only the two lines). It is bounded (`saturating_add`), so this is not the
pass-1 leak, but it is dead state per AGENTS.md "don't add features beyond what
the task requires." Either wire it into the future M16 status surface or drop
the field. Non-blocking.

B. **`CancellationTarget::InFlight` is unreachable through the LSP protocol
in M13's serialized model.** The message loop at `server.rs:60-104` is
single-threaded; `Message::Request` is dispatched through
`handle_request` -> `drain_queued_requests`, which blocks on the synchronous
`requests::handle` until `finish_request` runs. A `$/cancelRequest`
notification sitting in the connection receiver cannot be polled until the
outer `connection.receiver.recv()` runs again — by which time the in-flight
request has already cleared from both `queued` and `in_flight`, so
`RequestQueue::cancel` (`request_queue.rs:109-124`) returns
`CancellationTarget::None`. The same logic means
`Session::check_active_request_cancelled` (`session.rs:250-255`) inside
`with_document_analysis` (`session.rs:146-165`) cannot observe a cancel during
analysis. The in-flight branch is exercised by direct-API unit tests
(`session.rs:404-424`), so the mechanism works; it just cannot fire from the
protocol surface today. This is consistent with the locked decision that M11-M13
stay serialized, but the M13 doc currently says "phase-boundary checks observe
before and after compiler-service work" without flagging that those checks are
dormant scaffolding for M14+ concurrency. Worth a one-line clarification in the
M13 doc on the next pass through the docs.

C. **Stress assertion on multi-document progress is structural, not
content-bound.** `lsp_protocol_stress.py:35-40` correctly asserts
`begin` and `end` `$/progress` notifications around the multi-document
`workspace/didChangeWatchedFiles`. It does not assert the progress message body
("checked N document(s)"). Acceptable — body text is a UI string and changing
it should not be a protocol-level break — but worth knowing that future copy
edits will not be caught by the stress harness.

## Summary

M13 is shippable as-is. Pass 1's two flagged-quality items (unbounded `events`
Vec, dead `$/cancelRequest` arm) are resolved. The remaining residuals are
quality and forward-looking concerns: a structural-shape divergence on
`CancellationToken`, a cosmetic exit-code misclassification on parent death,
per-message `kill -0` cost, and the dormant in-flight cancellation path that
cannot fire from stdio until concurrency lands. None block closing the
milestone.
