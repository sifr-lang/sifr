# M13 Review — SATISFIED with non-blocking residual risks

## Decision: SATISFIED

The implementation lands the milestone surface area required by the scope: per-request `CancellationToken`, queued vs in-flight cancellation distinction in `RequestQueue`, phase-boundary checks inside `Session::with_document_analysis` and before response publication, delayed `$/progress` begin/end notifications for multi-document workspace diagnostics, and `sifr lsp --parent-pid` with a parent-liveness watchdog at message boundaries. Behavior is consistent with the locked decision that M13 keeps execution serialized; cancellation propagation is naturally limited to phase boundaries on either side of each compiler-service call.

Validation passes (23 sifr_lsp tests, clippy clean, fmt clean, stress + smoke + M1 guardrail + closeout scripts all PASS).

## Residual non-blocking risks

1. **`ProgressState.events` grows unbounded in production** — `crates/sifr_lsp/src/progress.rs:59,90,99` push a `ProgressEvent` for every begin/end. The only reader is `events()` at line 107, which is `#[cfg(test)]`. In a long-lived editor session, this leaks memory each time `publish_all` runs for ≥2 documents. Either gate the `events` field behind `#[cfg(test)]` or remove it and rely on `Session::trace` for observability. Note: `Session.traces` (`session.rs:24,284`) has the same pre-existing pattern — worth fixing together rather than expanding it.

2. **`CancellationToken` is a tautological wrapper** — `cancellation.rs` stores only a cloned `RequestId` and exposes only `request_id()`. The locked architecture decision says the token should expose `is_cancelled()` and `check_cancelled() -> Result<(), Cancelled>`; the actual cancellation predicate lives on `Session`/`RequestQueue`. Today the token adds indirection without behavior. Either:
   - move the predicate onto the token by giving it an `Arc<AtomicBool>` (sets up the future async story), or
   - replace `active_request: Option<CancellationToken>` with `active_request_id: Option<RequestId>` until M14+ needs real propagation.
   This is structural, not a correctness bug, but it diverges from the locked decision shape.

3. **Parent-death exit is misclassified as `INTERNAL_COMPILER_PANIC`** — `crates/sifr/src/cli_model_and_entrypoint.rs:660-670` maps any `run_stdio_with_options` error to `INTERNAL_COMPILER_PANIC` with exit code 3. The watchdog returns `LspError::request_cancelled("parent process N is no longer alive")` (`watchdog.rs:31`), which is an expected operational shutdown, not a panic. Recommend distinguishing watchdog-initiated termination so the server exits 0 (or a dedicated "parent gone" code) rather than rendering a misleading internal-failure diagnostic to a stream nobody is reading anyway.

4. **Dead `$/cancelRequest` branch in `notifications::handle`** — `notifications/mod.rs:15-20` still handles `$/cancelRequest`, but `server.rs:73-77` intercepts the same notification with `continue` before generic dispatch. The notifications branch is unreachable and lacks the `send_cancelled_response`/`queued_requests.remove` cleanup. Remove the dead arm to avoid confusion about which path is authoritative.

5. **Watchdog spawns `kill -0` per message** — `watchdog.rs:39-47` `Command::new("kill").arg("-0")` runs once per LSP message in the receive loop (`server.rs:61`). At editor message rates (semantic tokens, hover, completion deltas during typing) this is a meaningful per-message cost on top of the actual request. A direct `libc::kill(pid, 0)` (via `rustix` to stay clear of `unsafe_code`) would be one syscall instead of a fork+exec. Non-blocking but worth a follow-up.

6. **Watchdog is a no-op on Windows** — `watchdog.rs:50-52` returns `true` unconditionally for non-unix. VS Code on Windows is a real consumer; the `--parent-pid` flag provides no protection there. Document the limitation in the M13 doc, or use `OpenProcess`/`GetExitCodeProcess` to make it real cross-platform.

7. **Progress threshold is work-unit-based, not time-based** — `DIAGNOSTICS_PROGRESS_DOCUMENT_THRESHOLD = 2` (`progress.rs:3`) means every multi-document publish fires begin/end even when the work is fast. The M13 doc claims "fast editor paths stay quiet" but with two open files a quick `didChangeWatchedFiles` always emits progress. The locked decision wording ("only after a threshold") is ambiguous between time and count, and TypeScript-Go uses time. Consider gating on `Instant::now()` elapsed at the begin call, or accept this trade-off and update the doc to say the threshold is document-count, not latency.

8. **Reserved-but-unused `ProgressKind` variants** — `References`, `IndexWarming`, `WorkspaceLoad` are declared (`progress.rs:9-14`) with `#[allow(dead_code)]` but no call site emits them. The M13 doc acknowledges this is intentional for future milestones; just flagging so it stays visible in the closeout tracker.

9. **Stress test does not exercise a real cancel race** — `lsp_protocol_stress.py:44` sends `$/cancelRequest` for id `99999` which was never issued. It verifies "unknown id cancel is harmless," not "an in-flight cancel produces `-32800`." Given the serialized model, a deterministic in-flight cancellation race may be hard to drive over stdio, but adding a queued-request cancel test (enqueue a workspace request, immediately cancel by its id, expect a `-32800` response) would tighten the protocol-level guarantee. The unit test at `request_queue.rs:267-281` exercises the queue but not the publication path.

## Summary

The milestone is implementable and shippable as-is; the residual items above are quality, not correctness. Item 1 (unbounded `events` Vec) is the closest to blocking — recommend fixing before closeout — and item 4 (dead `$/cancelRequest` branch) is a quick removal that prevents future maintainer confusion.
