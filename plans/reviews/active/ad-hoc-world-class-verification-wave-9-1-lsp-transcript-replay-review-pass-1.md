Verified — I have what I need. Here are the findings.

# Review: Wave 9.1 LSP JSON-RPC Transcript Replay

**Blockers: none.** The gate, runner wiring, and validation flow look correct, validation already passed locally, and the scenarios honestly target the documented stable behaviors. Below are non-blocking concerns ranked by importance.

## Non-blocking findings

### 1. (Medium) Cancellation test likely does not exercise the new response-buffering path

`replay_cancellation_and_out_of_order_requests` in `check_lsp_transcript_replay.py:195-199` sends:
- id `100` → `workspace/symbol` (the new `WorkLane::Workspace` lane — `crates/sifr_lsp/src/scheduler.rs:16`)
- id `101` → `textDocument/documentSymbol` (the latency-sensitive lane)

It then awaits `101` first. Because `documentSymbol` runs on the fast lane and `workspace/symbol` on the slower workspace lane, id `101` almost always returns first, so `wait_for_response(101)` reads it directly without ever buffering. The new `responses` dict path in `lsp_protocol.py:111-119` is the headline feature of this PR, but the test passes whether or not it works. To genuinely cover it, swap the lanes assigned to the two ids (e.g. id `100` = `documentSymbol`, id `101` = `workspace/symbol`), so awaiting `101` first forces id `100` to be buffered and recovered later. The assertion text ("early responses are retained until their request id is awaited") would then actually be tested.

### 2. (Medium) `hover is None` is a fragile assertion when the cursor is on an unresolved identifier

`replay_stale_diagnostics_after_edit` (lines 240-247) reverts the document back to `BROKEN_SAMPLE` (version 1 after version 2) and immediately calls `hover` on `missing_value`. `client.request("textDocument/hover", ...)` returns `result` which the LSP spec allows to be `null`. If the Sifr server starts returning `null` for hover on unresolved names (a reasonable behavior change), this assertion fails spuriously even though the assertion text only claims "stale edits do not corrupt later hover queries." A more honest check is to verify the request did not error (the `request()` helper already raises on `error`) and not depend on a non-null payload — or hover on a definitely-resolved position. Same fragility applies in `replay_cancellation_and_out_of_order_requests:208-213`, though there the position lands on `helper` (resolved) so it's lower risk.

### 3. (Low) `_wait_for_response` now misclassifies server-initiated requests

In `lsp_protocol.py:116-119`, any inbound message with an `id` is dropped into `self.responses`. LSP servers can initiate requests (e.g. `workspace/configuration`, `window/showMessageRequest`, `client/registerCapability`) which carry both `method` and `id`. Pre-change these were silently dropped; post-change they are stored as if they were responses, keyed by the server-chosen id. If the server's request id ever collides with a client id (both sides commonly start at 1), `wait_for_response` could return the server's request as a response. None of these scenarios trigger server-initiated requests today, so it's latent — but worth a guarded branch: `elif "id" in message and "method" not in message: self.responses[...]`.

### 4. (Low) `validate_manifest` error message claims "non-empty" but accepts empty list

`check_lsp_transcript_replay.py:73-74`:
```python
if not isinstance(required_categories, list) or required_categories != sorted(required_categories):
    raise LspProtocolError("required_categories must be a sorted non-empty list")
```
`[] == sorted([])`, so an empty `required_categories` would pass. Either add `not required_categories` to the predicate or drop "non-empty" from the message.

### 5. (Low) Manifest `required_methods` / `assertions` are documentation-only

The runner does not cross-check that each scenario's replay actually invokes its declared `required_methods` (e.g. `project-reload-watched-file` lists `initialize`, `workspace/didChangeWatchedFiles`, `workspace/symbol`, `textDocument/publishDiagnostics`, but the replay also relies on `textDocument/didOpen`, `shutdown`, `exit` which aren't declared) and assertions are free-form prose. This drifts easily as scenarios change. Consider either lowering these fields to "informational" in docs/comments or wiring a lightweight check (e.g. record methods used by the client during a replay and intersect with `required_methods`).

### 6. (Low) `project-reload-watched-file` assertion is loose

`replay_project_reload_watched_file:270-276` takes the *first* `textDocument/publishDiagnostics` and accepts either main.sifr or helper.sifr as the URI. That matches the scope you intentionally limited to (the PR description acknowledges this), but the manifest assertion "watched-file changes revalidate the open project" is only weakly evidenced — it would still pass if the server published a stale diagnostic for `main.sifr` *before* processing the watched-file change. If `open_document` already drained the open's initial diagnostics, this is fine in practice; just be aware the test is asserting "some publishDiagnostics arrived for one of these files" rather than the stronger "republished after the watched file event."

### 7. (Low) Auto-generated `next_id` could one day collide with the manual ids `100`/`101`

`send_request` does not bump `self.next_id`, and the manual ids `100`/`101` are far above the current counter, so no collision today. If future scenarios chain many `client.request(...)` calls before `send_request`, the counter could overrun. Inexpensive guard: advance `self.next_id = max(self.next_id, int(request_id) + 1)` inside `send_request` when the id is an int.

## What looks correct / good

- Manifest schema gate (sorted ids, sorted required_categories, required_methods, profile pinning, runner registration completeness) is sound, and the self-test exercises the failure path.
- Runner wiring (`runner.py:101-102`) adds both replay and self-test variants in the `lsp-smoke` suite, which is what `--profile create-pr` consumes; the variant count (6) matches the validation receipt.
- Response buffering implementation in `_wait_for_response` (`lsp_protocol.py:108-119`) is correct in principle for the client-id case and preserves notification handling.
- Scenarios honestly scope to documented stable wire behavior (workspace folder advertisement, MethodNotFound for unsupported/disabled methods, version-bearing diagnostics, watched-file revalidation, hover survives stale edits).
- Each scenario uses its own `TemporaryDirectory` and `LspClient`, giving good isolation; `close_client` correctly distinguishes the exception path from clean shutdown via `sys.exc_info()`.

## Recommendation

Ship as-is for this slice. Track finding **#1** as a follow-up before declaring the buffering feature itself verified — swapping the two manual request ids is a one-line change that turns the test into a real proof of the new code path. Findings **#2**, **#3**, and **#5** are worth one cleanup pass in a subsequent transcript-replay slice; **#4**, **#6**, **#7** are cosmetic.
