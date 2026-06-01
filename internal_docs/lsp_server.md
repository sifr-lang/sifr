# Sifr LSP Server Architecture

status: phase36-m36.5-implemented; TypeScript-Go architecture transfer M4 current-state caveats recorded

## Protocol Target

The production protocol is LSP 3.17 over stdio, launched by:

```bash
sifr lsp --stdio
```

The Rust implementation uses `lsp-server` and `lsp-types` directly. m36.5 pins `lsp-server` 0.7.9 and `lsp-types` 0.97.0 in `Cargo.lock`. Notebook synchronization and Python-specific import, interpreter, environment, and settings behavior are explicitly unsupported.

## Server Ownership

`sifr_lsp` is a protocol shell. It owns:

- JSON-RPC/LSP handshake and dispatch
- client capability negotiation
- document synchronization transport
- workspace configuration transport
- URI, range, position, diagnostic, location, and edit conversion
- cancellation, scheduling, and stale-result rejection
- LSP error/log/message behavior

It does not own parser logic, lowering, type checking, HIR construction, symbol-table construction, diagnostics derivation, formatter decisions, linter decisions, generated Rust decisions, or Sifr semantic rules.

## Internal Layers

The LSP implementation is being migrated toward these compiler-service layers:

- `CapabilityRegistry`: resolves position encoding, workspace configuration, pull diagnostics, dynamic registration, related information, semantic tokens, signature label offsets, hierarchical document symbols, completion details/documentation, work-done progress, file watching, and code-action resolve support.
- `DocumentStore`: tracks open `.sifr` documents by URI, version, language id, source text, line index, canonical `FileId`, edit application, and UTF-8/UTF-16/UTF-32 conversion.
- `SifrLspSession`: owns `AnalysisHost`, open-document overrides, workspace folders, settings, diagnostics mode, queues, suspended workspace diagnostics, stale-result state, and command/settings metadata.
- `RequestQueue`: tracks request ids, methods, start times, cancellation tokens, response handlers, and exactly-one-response completion.
- `Scheduler`: separates sync state mutation, background document queries, workspace queries, latency-sensitive requests, and formatting/code-action work.
- `SnapshotLayer`: captures coherent analysis snapshots per request and rejects stale publication.
- `ConversionLayer`: owns all LSP-to-Sifr and Sifr-to-LSP conversions.
- `DiagnosticsController`: handles push diagnostics, pull diagnostics, workspace diagnostics, result ids, clearing, versions, related information, tags, and dynamic registration.
- `CommandRegistry`: owns restart, logs, explain diagnostic, generated Rust preview, check, and test commands.
- `ProtocolTestHarness`: launches `sifr lsp --stdio`, drives JSON-RPC, and records deterministic protocol snapshots.

The concrete m36.5 modules are:

- `crates/sifr_lsp/src/capabilities.rs`
- `crates/sifr_lsp/src/document_store.rs`
- `crates/sifr_lsp/src/session.rs`
- `crates/sifr_lsp/src/request_queue.rs`
- `crates/sifr_lsp/src/scheduler.rs`
- `crates/sifr_lsp/src/conversion.rs`
- `crates/sifr_lsp/src/diagnostics.rs`
- `crates/sifr_lsp/src/commands.rs`
- `crates/sifr_lsp/src/requests/`
- `crates/sifr_lsp/src/notifications/`

The stdio server terminates explicitly on `exit` after a successful `shutdown`
response. This avoids retaining protocol IO threads after the client completes
the required LSP shutdown sequence.

## Current M5 Compiler-Service State

The TypeScript-Go architecture transfer keeps M5 serialized while moving LSP
analysis ownership into the language-server session. Current implementation
reality:

- `DocumentStore` owns only protocol text, URI, path, and version state.
- `Session` owns `LspAnalysisWorkspace`, which holds persistent analysis handles
  for open documents. Those handles wrap `WorkspaceSession` and update unsaved
  editor buffers through `WorkspaceSession` overlays.
- LSP notifications update `DocumentStore` first, then feed the latest document
  text/version into the session-owned analysis workspace before diagnostics or
  semantic requests capture snapshots.
- Analysis-backed requests route through `Session::with_document_analysis`,
  capture `AnalysisSnapshot` values, and reject results if either the captured
  workspace snapshot becomes stale or the document version changes before
  publication.
- `textDocument/didChange` compacts each content-change batch before applying
  it to `DocumentStore` or forwarding the latest document text into analysis.
  Repeated full-document replacements collapse to the latest replacement plus
  any following incremental edits.
- `workspace/didChangeWatchedFiles` summarizes each watcher batch once before
  invalidation. Normal watcher batches select graph-structure dirty scope;
  watcher storms degrade to workspace dirty scope with `WatcherStorm`.
- `RequestQueue` tracks pending request ids and shutdown state; it is not yet a
  cancellation-token registry.
- `Scheduler` maps methods to lane labels only; it does not yet run priority
  queues, debounce, background workers, delayed progress, or cancellation.

M7 owns dependency-sensitive signature invalidation. M11 owns priority
queues/debounce. M13 owns cancellation, progress, and parent-process watchdog
behavior.

## Capability Matrix

The checked-in capability source of truth is `verification/tooling/lsp_protocol_matrix.json`. A capability must not be advertised unless the matrix maps it to:

- a Sifr owner
- positive coverage
- negative coverage
- unsupported-feature behavior where applicable
- a performance budget id where latency matters

The matrix locks all required Phase 36 methods and workspace commands before implementation begins. `verification/tooling/check_tooling_contract_lock.py` validates required matrix coverage.

## Settings

Required Sifr LSP settings:

- `sifr.diagnostics.mode`: `off`, `open-files`, or `workspace`
- `sifr.lsp.trace.server`: `off`, `messages`, or `verbose`
- `sifr.format.enable`: boolean
- `sifr.lint.enable`: boolean

Unknown initialization options and settings produce deterministic warnings and logs while continuing with defaults unless the workspace cannot be loaded safely.
When `sifr.format.enable` is false during initialization, the server does not
advertise document or range formatting capabilities. If the setting is disabled
after initialization, formatting requests are rejected without bypassing
`sifr_analysis` or invoking an editor-side formatter.

Document and range formatting load formatter options from the same `sifr.toml`
`[format]` discovery path used by `sifr fmt`. LSP `FormattingOptions.lineLength`
and `options.sifr.preview` may override the discovered line length and preview
setting for a request. The LSP server remains a protocol adapter: formatter
decisions live in `sifr_format`, and editor integrations must not provide a
separate formatter implementation.

Lint diagnostics are published by `sifr_analysis` after it combines frontend
hard diagnostics with `sifr_lint` policy diagnostics. LSP diagnostic `data`
must include a typed diagnostic class:

- `hard`: compiler/frontend diagnostics that are not suppressible,
  downgradeable, or fixable through lint code actions.
- `policy`: Sifr lint diagnostics with a `ruleId` matching the Sifr-owned rule
  registry.

Suppression actions, safe per-diagnostic fixes, and source fix-all actions are
offered only for `policy` diagnostics. The server must not infer this from
diagnostic-code prefixes. Deferred policy fix-all actions use
`codeAction/resolve`, carry the expected document version, recompute edits
through `sifr_analysis`, and fail closed if the document version changed.

## Commands

Required command identifiers:

- `sifr.restartServer`
- `sifr.showServerLogs`
- `sifr.explainDiagnostic`
- `sifr.showGeneratedRust`
- `sifr.checkWorkspace`
- `sifr.runTests`

Command payloads are Sifr-owned and versioned through this document and the protocol matrix.

Lint-related code actions use standard LSP code-action kinds:

- `quickfix`: insert an explicit `# sifr: ignore[rule-id]` suppression for a
  policy diagnostic or apply an individual safe policy fix.
- `source.fixAll.sifr`: resolve and apply all currently safe policy fixes for
  the document.

No editor command may implement a separate Sifr linter or rewrite Sifr source
for policy fixes outside the native LSP/CLI surfaces.

## Versioning Policy

LSP 3.17 is the Phase 36 target. Any `lsp-types` version bump requires a reviewed PR that records adopted capabilities, deferred capabilities, compatibility impact, and protocol matrix changes. Silent adoption of new LSP surfaces is forbidden.

## m36.5 Protocol Coverage

`verification/tooling/lsp_protocol_smoke.py` initializes the server, opens a
`.sifr` buffer, validates versioned push diagnostics, runs the required
document/workspace query families, exercises generated-Rust and test commands,
and shuts down through `shutdown` plus `exit`.

`verification/tooling/lsp_protocol_stress.py` covers cancellation
notifications, full sync, stale-version rejection, invalid-range protocol
errors, workspace configuration, watched-file refresh, save/close flows,
unknown command errors, and closed-document query rejection.
