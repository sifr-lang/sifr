Audit written to `reviews/typescript-go-phase-audit-lsp-runtime-review-pass-3.md`. Summary of what I found:

**Verdict: CHANGES RECOMMENDED — 4 HIGH, 6 MEDIUM, 3 LOW findings.**

**HIGH-severity gaps:**

- **F1 (AC-2/AC-7)** — `LspAnalysisWorkspace` keeps **one `AnalysisHost`+`WorkspaceSession` per document** (`analysis_workspace.rs:11-13, 96-101` → `host/overlay_updates.rs:14`). Despite `lsp_server.md:67-73` claiming a persistent shared session, `workspace/symbol`, `references`, and `rename` only see one file at a time.
- **F2 (AC-12)** — `diagnostics.rs:118-129` returns cached `load_diagnostics` without snapshot/version guard; pull-based `textDocument/diagnostic` (`requests/diagnostics.rs:7-13`) calls this directly with no `document_version_matches` wrap, so stale rendered diagnostics can publish.
- **F3 (AC-13/AC-28)** — server is fully serialized (`server.rs:128`), so `$/cancelRequest` cannot reach an in-flight request; workspace-scoped loops (`workspace_diagnostic`, `workspace_symbol`) have no inter-module cancellation checks. The "phase-boundary cancellation" test calls `cancel_request` inside the same thread — it doesn't prove protocol-level cancellation.
- **F4 (AC-24)** — `watchdog.rs:49-52` always returns `true` on non-Unix; `server.rs:62` only checks the watchdog at the top of `recv()`, so an idle dead-parent server stays alive forever.

**MEDIUM gaps:** LSP never opens project mode (F5), `FileId` collisions across hosts silently overwrite `uri_map`/`source_map` (F6), watcher-storm degradation runs per host (F7), "debounce" is one publish per keystroke (F8), protocol harnesses lack multi-file tests (F9), and `perf.lsp.navigation.symbol` aggregates 5 protocol surfaces into one budget (F10).

**LOW gaps:** debugTrace events carry `snapshot_id: None` with silent ring-buffer truncation (F11), cancellation token is an `RequestId` clone not a flag (F12), reserved `perf.lsp.*` IDs are deferred without tracking (F13).

The report includes 5 concrete experiments to drop in `tmp/tsgo_phase_audit_pass3/` (multi-file workspace symbol, cross-file references, pull-diagnostic stale race, in-flight cancel observability, idle watchdog) that would fail today and produce protocol-level evidence for a follow-up.
