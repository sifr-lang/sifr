# TypeScript-Go Architecture Transfer: LSP Persistent Session

Persistent LSP session ownership moves LSP analysis out of `DocumentStore` and into the serialized
language-server session. This keeps execution single-threaded while making the
compiler-service owner explicit before later dirty-scope, scheduler, cancellation,
and progress/watchdog work.

## Session-Owned Analysis

`DocumentStore` now owns protocol document state only:

- URI;
- filesystem path;
- latest LSP document version;
- latest editor text;
- workspace settings.

`Session` owns `LspAnalysisWorkspace`, which stores the persistent analysis
handles for open documents. Each handle wraps `AnalysisHost`; each
`AnalysisHost` owns a `WorkspaceSession`. Open, change, and save notifications
first update `DocumentStore`, then feed the latest text/version into the
session-owned analysis workspace.

## Overlay Updates

LSP document analysis now enters the compiler service through workspace
overlays:

- `AnalysisHost::open_single_file_overlay` creates a single-file
  `WorkspaceSession`, installs the open editor buffer as an overlay, and loads
  frontend state from that overlay.
- `AnalysisHost::upsert_overlay_document` updates an existing
  `WorkspaceSession` overlay and reloads the serialized frontend state.
- Load diagnostics remain attached to the session-owned analysis entry so
  diagnostics can report frontend load errors without reintroducing
  `DocumentStore` analysis ownership.

This means unsaved editor buffers are analyzed from overlay text rather than
disk text.

## Snapshot And Version Publication Identity

`Session::with_document_analysis` is now the LSP request boundary for
analysis-backed requests. It captures the document version before semantic work,
executes the request through `AnalysisSnapshot` methods, and rejects publication
if the document version changed before the result is returned. The underlying
snapshot method still rejects stale workspace snapshots by captured
`WorkspaceSnapshot` revision and analysis graph/source revision.

Persistent LSP session ownership does not introduce async scheduling,
cancellation, debounce, or precise dirty scopes. Those remain scheduler,
cancellation/progress, and dirty-scope responsibilities.
