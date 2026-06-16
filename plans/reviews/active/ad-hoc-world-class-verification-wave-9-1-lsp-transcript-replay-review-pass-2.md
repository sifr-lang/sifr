# Review Pass 2: Wave 9.1 LSP JSON-RPC Transcript Replay

**Verdict: no blockers. Pass-1 concerns are resolved as described — ship it. No further review round required.**

## Verification of pass-1 fixes

| # | Finding | Status |
|---|---|---|
| 1 | Cancellation test does not exercise buffering | **Resolved** — `check_lsp_transcript_replay.py:195-201` now sends id `100`=`textDocument/documentSymbol` (fast lane) and id `101`=`workspace/symbol` (slower workspace lane), awaits `101` first, and asserts `100 in client.responses` *before* draining it. The assertion fails loudly if buffering isn't exercised, so the test now genuinely proves the new code path. |
| 2 | `hover is None` fragility | **Resolved for the affected scenario** — `replay_stale_diagnostics_after_edit` (lines 244-247) discards the hover result; correctness is now established solely by `request()` not raising on `error`. (See note 2.1 below for the unchanged sibling.) |
| 3 | `_wait_for_response` misclassifies server-initiated requests | **Resolved** — `lsp_protocol.py:120-121` guards buffering with `elif "id" in message and "method" not in message`. Server-initiated requests now fall through (silently dropped) instead of polluting `self.responses`. |
| 4 | `validate_manifest` accepts empty `required_categories` | **Resolved** — `check_lsp_transcript_replay.py:73` predicate now includes `not required_categories`. |
| 7 | `send_request` could collide with `next_id` | **Resolved** — `lsp_protocol.py:71-72` advances `self.next_id = max(self.next_id, request_id + 1)` for int ids. |

Pass-1 findings #5 (manifest fields are documentation-only) and #6 (`project-reload-watched-file` assertion is loose) were explicitly tagged as cosmetic in pass 1; both remain unaddressed and are still non-blocking.

## Residual non-blocking observations

### 2.1 (Low) Sibling `hover is None` check in cancellation replay still present
`check_lsp_transcript_replay.py:210-215` keeps the `hover is None` guard. Pass-1 noted this is lower risk because the cursor lands on `helper` (resolved at line 4 char 19 of `SAMPLE`), so a null hover would itself indicate a regression. Not a blocker; mention only for symmetry with the stale-diagnostics fix.

### 2.2 (Low) Server-initiated requests are silently dropped
The new branch structure in `_wait_for_response` correctly avoids misclassifying server requests as responses, but it also doesn't queue them anywhere. If a future server slice starts issuing `workspace/configuration` / `window/showMessageRequest`, those messages are simply discarded inside the wait loop. Today's scenarios don't trigger this, but a follow-up may want a dedicated queue + a `respond_to_server_request` helper. Pure forward-look; nothing to do now.

### 2.3 (Low) `cancellation-and-out-of-order-requests` `required_methods` still drifts
The scenario's `required_methods` in `manifest.json:27-31` does not list `textDocument/didOpen`, `initialize`, `initialized`, `textDocument/hover`, `shutdown`, or `exit`, all of which the replay calls. This is the same pass-1 finding #5 — manifest fields aren't enforced — flagged again only so the drift isn't lost. No fix required for this slice.

### 2.4 (Low) Buffering assertion in cancellation replay relies on lane ordering being stable
The `if 100 not in client.responses` check turns the test into a proof of fast-lane-precedes-workspace-lane ordering. That's the right call (it's better than silently passing), but if `WorkLane::Workspace` ever fast-paths empty/single-file workspaces and overtakes the latency lane, this scenario will fail. Worth documenting as a known dependency on the scheduler's lane prioritization rather than as a bug.

## Validation receipt
Local validations quoted (py_compile + replay + `--self-test` + `lsp-smoke` suite with `variants=6 failures=0 blocking_failures=0`) are consistent with the runner wiring in `runner.py:101-102` and the six expected variants (one per scenario, plus the self-test). No discrepancy detected.

## Recommendation
Merge as-is. The buffering feature is now actually exercised, the false-positive risks are gone, and the remaining items are pass-1 carry-overs already classified as cosmetic. No additional review pass is needed for this slice.
