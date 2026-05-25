

Based on my review, here are the findings:

---

# Phase 36 m36.5 Review — Production Native LSP Server

## Blocking Findings

### 1. EXIT code negation logic is inverted (server.rs:57)

```rust
let code = i32::from(!self.session.shutdown_requested());
```

This produces `exit(0)` when `shutdown_requested` is `false` (no shutdown sent) and `exit(1)` when `true` (shutdown sent correctly). Per LSP, the server must exit with non-zero code if it exits **without** receiving `shutdown` first. The negation is backward.

**Fix**: Flip to `!self.session.shutdown_requested()` returns `true` when shutdown was NOT requested, so `exit(1)` when shutdown was skipped.

### 2. Stress test never sends `exit` (lsp_protocol_stress.py:82)

The phase contract requires "cancellation, shutdown/exit ordering, server logging" in stress tests. The stress test sends `shutdown` but never sends `exit` notification. The `close()` in `lsp_protocol.py` sends `exit`, but the stress test path `run_stress()` calls `client.request("shutdown", {})` and returns without sending `exit`. This means the shutdown/exit ordering contract point is not validated.

**Fix**: Add `client.notify("exit", {})` before `client.request("shutdown", {})` or after shutdown, per LSP ordering, to verify the server processes both in correct sequence.

### 3. `completionItem/resolve` is a stub (requests/mod.rs:28)

```rust
"completionItem/resolve" => Ok(params),
```

Returns the input unchanged. Phase contract says "completionItem/resolve" is required capability. The implementation should enrich completion items with documentation/detail from `sifr_analysis`, not return a no-op. This is a protocol conformance gap for any client that relies on `resolve` for completion metadata.

**Fix**: Implement resolution through `host.completion_resolve()` or map existing completion detail/docs.

### 4. Scheduler lane classification is unused (requests/mod.rs:21, scheduler.rs:12-18)

```rust
let _lane = Scheduler::lane_for_method(method);
```

The lane result is discarded (`_lane`). The `Scheduler` module defines `WorkLane` variants but `LspServer::handle_request` never uses them. Latency-sensitive vs formatting vs workspace separation is declared but not enforced. This is dead infrastructure that could cause starvation once workloads grow.

**Fix**: Either wire the scheduler into actual request dispatch, or remove the dead code until it can be properly implemented.

### 5. Document close never clears diagnostics (document_store.rs:141-143)

```rust
pub(crate) fn close(&mut self, uri: &str) -> bool {
    self.documents.remove(uri).is_some()
}
```

When a document is closed, diagnostics are not cleared. The client still shows stale diagnostics for the closed file. Per LSP spec, a `textDocument/didClose` should trigger clearing of that file's published diagnostics.

**Fix**: Send a `publishDiagnostics` notification with empty diagnostics array for the closing document's URI.

---

## Phase-Contract Gap

### 6. 35 of 36 reserved budget IDs have no benchmark evidence

`lsp_query_budget_ids.md` reserves 36 budget IDs (perf.lsp.cold_start, perf.lsp.completion.*, perf.lsp.hover.*, etc.). `manifest.json` only has `lsp-query-001-request-families`. The other 35 are documented but unimplemented. The phase contract says: "Phase 36 extends Phase 35 verification/performance/ with protocol-level lsp-query benchmark cases" and "Every LSP budget id must map to one manifest case".

This is a significant gap for m36.8 performance closeout.

---

## Low/Informational Findings

### 7. `traces` in Session is never used for `window/logMessage`

The `Session::trace()` method collects messages, `sifr.showServerLogs` returns them, but `window/logMessage` is never sent to the client even when trace level is non-off. The trace infrastructure is incomplete.

### 8. `work_done_progress` capability is advertised but never sent

The capability includes `workDoneProgress` but no request handler actually sends progress notifications to the client. Progress tracking is a stub.

### 9. `prepared_rename` placeholder calculation may overflow (navigation.rs:69)

```rust
.saturating_add(u32::try_from(target.symbol.name.len()).unwrap_or(0))
```

Uses `saturating_add` but then `unwrap_or(0)`. If the length can't convert to u32, you get 0 anyway. The outer `saturating_add` won't prevent the unwrap from panicking. Consider removing the try_from/unwrap and just using `u32::try_from(...).unwrap_or(position.character)` or similar.

### 10. No negative test for cancellation of in-flight requests

`lsp_protocol_stress.py` sends `$/cancelRequest` (line 30) but never asserts the server aborted or retried the request. The negative test seed for "cancellation of queued and running requests" is not covered.

### 11. Blanket `#[allow(clippy::needless_pass_by_value)]` and `#[allow(clippy::unnecessary_wraps)]` in lib.rs

These suppress warnings across the whole crate without file-level justification. Prefer targeted suppression on the specific items.

### 12. `run_tests` returns empty array on missing document (commands.rs:71-81)

```rust
if let Some(test_id) = arguments.first().and_then(Value::as_str) {
    if let Some(document) = store.documents_mut().next() {
        // ...
    }
}
let mut tests = Vec::new();
for document in store.documents_mut() {
    // ...
}
```

If no documents are open and no test_id is provided, returns `[]` silently. The phase contract says "must fail closed when a project has no test surface rather than guessing". An empty array with no error message for `sifr.runTests` on a project with no tests is ambiguous — is there no test surface, or no discovered tests?

---

## What Looks Correct

- **Split-brain architecture**: `sifr_lsp` has no direct imports of `sifr_frontend` parser/lower/type-check. All semantic answers flow through `sifr_analysis`. The `check_lsp_split_brain.py` guardrail passes.
- **Protocol error codes**: All error responses use correct LSP `ErrorCode` values (-32600 through -32000 range).
- **Stale version rejection**: Document version checks in `document_store.rs` `reject_stale()` are correct.
- **Incremental sync bounds checking**: The edit range validation (start <= end, end <= text.len()) is correct.
- **Settings handling**: Both `sifr/diagnostics/mode` and `sifr.diagnostics.mode` forms are accepted (for LSP and VS Code clients respectively).
- **Workspace diagnostics**: `workspace_diagnostic` iterates all open documents and reports pull diagnostics correctly.
- **Diagnostic conversion**: Source maps for span ranges, severity mapping, related information and tags are correct.
- **Performance baseline**: `perf.lsp.request_families` baseline shows median 1.342ms / p95 1.477ms — well within budget.
- **Self-test coverage**: All scripts have `--self-test` paths.

---

## Verdict

**CHANGES_REQUESTED**

Two blocking items require fixes before merge:
1. Fix `server.rs:57` exit code negation
2. Fix `lsp_protocol_stress.py` to include `exit` notification in shutdown/exit ordering test
3. Implement `completionItem/resolve` through `sifr_analysis`
4. Clear diagnostics on document close

The remaining items (scheduler lanes, progress, tracing, budget coverage) are lower priority but should be addressed in m36.6/m36.8 before the phase closes.
