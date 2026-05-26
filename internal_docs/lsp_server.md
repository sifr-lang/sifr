# Sifr LSP Server Architecture

status: phase36-m36.5-implemented

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

The LSP implementation must expose these layers:

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

## Commands

Required command identifiers:

- `sifr.restartServer`
- `sifr.showServerLogs`
- `sifr.explainDiagnostic`
- `sifr.showGeneratedRust`
- `sifr.checkWorkspace`
- `sifr.runTests`

Command payloads are Sifr-owned and versioned through this document and the protocol matrix.

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
