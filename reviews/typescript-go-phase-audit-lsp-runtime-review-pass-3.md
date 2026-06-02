# TypeScript-Go Phase Audit — LSP/Runtime Protocol Pass 3

Scope: LSP persistent session integration, overlay event handling, stale result
rejection, scheduler lanes, cancellation/progress/watchdog behavior,
`sifr/debugTrace` output, marker corpus coverage, per-request latency budgets.
Acceptance criteria audited: AC-2, AC-7, AC-12, AC-13, AC-20, AC-24, AC-26..AC-28.

Verdict: CHANGES RECOMMENDED. The LSP code shipped under M5/M11/M12/M13/M16/M17
implements most of the *names* the phase set up (workspace session, scheduler
lanes, cancellation tokens, debounced diagnostic jobs, progress, watchdog,
debugTrace, per-request budgets), but several behaviors are not observable from
the protocol surface and one architectural piece is misaligned with the
documentation. The findings below identify gaps where tests exercise unit-level
internals but not protocol behavior, where ordering/cancellation/debounce
claims are not observable, and where stale results can still publish.

---

## F1 (HIGH) — LSP analysis is **per-document**, not per-workspace; the M5 persistent-session claim is split-brain (AC-2, AC-7)

**Files**:

- `crates/sifr_lsp/src/analysis_workspace.rs:11-13` — `LspAnalysisWorkspace`
  holds `documents: BTreeMap<String, LspDocumentAnalysis>`; each
  `LspDocumentAnalysis` owns its own `AnalysisHost`.
- `crates/sifr_lsp/src/analysis_workspace.rs:96-101` — every document is opened
  via `AnalysisHost::open_single_file_overlay(..., FrontendMode::SingleFile)`.
- `crates/sifr_analysis/src/host/overlay_updates.rs:14` — that helper internally
  calls `WorkspaceSession::single_file(path, mode)` — i.e. a fresh, isolated
  `WorkspaceSession` per file.
- `internal_docs/lsp_server.md:67-73` claims `Session` owns
  `LspAnalysisWorkspace`, "which holds persistent analysis handles for open
  documents. Those handles wrap WorkspaceSession and update unsaved editor
  buffers through WorkspaceSession overlays." Technically true; misleading in
  practice — there is **no single shared workspace session**.

**Consequences**:

- `workspace/symbol` (`crates/sifr_lsp/src/requests/symbols.rs:22-55`) loops
  every open URI, dispatching `with_document_analysis(...)` per URI. Each
  invocation hits its own single-file workspace session whose
  `symbol_index` only knows that one file. Workspace symbol queries are not
  workspace-wide.
- `textDocument/references` and `textDocument/rename`
  (`crates/sifr_lsp/src/requests/navigation.rs:27-95`) only see the file you
  point at: `snapshot.references(host, file, position)` runs against a host
  containing exactly one file. Multi-file references / cross-module renames are
  impossible by construction.
- The LSP never opens a project mode session even when a `sifr.toml` is
  present. The CLI uses `AnalysisHost::open_project`
  (`crates/sifr_analysis/src/host/implementation.rs:37-40`); the LSP uses
  `open_single_file_overlay`. AC-2 requires snapshots **shared by CLI analysis
  and LSP requests** — the two paths are different host kinds.

**Severity**: HIGH (architectural). Closeout reports AC-2 as PASS; the unit
tests prove the per-document path, not multi-document semantics.

**Recommended experiment (`./tmp/tsgo_phase_audit_pass3/multi_file_workspace_symbol.py`)**:
launch `cargo run -q -p sifr -- lsp --stdio`, `initialize` against a root,
`didOpen` two `.sifr` files with distinct top-level `def`s, then send
`workspace/symbol` with an empty query. Assert that BOTH files'
`def` names appear and that each result's `location.uri` resolves to the
correct file. Per F1, results today are per-file isolated.

---

## F2 (HIGH) — `load_diagnostics` fast path bypasses snapshot/version stale-rejection on pull-diagnostics (AC-12)

**Files**:

- `crates/sifr_lsp/src/diagnostics.rs:118-129` —
  `document_diagnostics(session, uri)` checks
  `session.load_diagnostics(uri).is_empty()` first; if non-empty it returns
  those rendered load-time diagnostics **without** going through
  `session.with_document_analysis(...)` (the only place where
  `before_version` / `after_version` and `is_snapshot_current` are checked).
- `crates/sifr_lsp/src/requests/diagnostics.rs:7-13` —
  `text_document_diagnostic` calls `document_diagnostics(session, &uri)`
  directly. No `document_version_matches` guard is run around this call.
- The publish path (`DiagnosticsController::flush_ready`,
  `crates/sifr_lsp/src/diagnostics.rs:54-103`) does call
  `document_version_matches` before and after; so the gap is invisible there.

**Behavior**: if a host's open ever fails, `load_diagnostics` holds those
rendered diagnostics indefinitely until the next successful overlay update
(`crates/sifr_lsp/src/analysis_workspace.rs:107-112`,
`crates/sifr_lsp/src/analysis_workspace.rs:115-140`). A pull `textDocument/diagnostic`
arriving after a successful didChange that didn't fully clear them will return
the stale rendered diagnostics labelled as fresh.

**Severity**: HIGH for AC-12 ("LSP must reject stale publications"). The
shortcut wasn't designed to publish; it's a non-snapshot return path.

**Recommended experiment**: introduce a parse error on `didOpen` (e.g. unclosed
paren) so `load_diagnostics` is populated; send `textDocument/didChange` that
fixes the parse; then immediately send `textDocument/diagnostic` and verify
the diagnostic list. If `load_diagnostics` is not cleared between the two
events, stale items leak.

**Fix candidate**: route the shortcut through `with_document_analysis` (or at
minimum call `document_version_matches` against the requested URI before
returning the cached payload), and ensure successful overlay updates always
clear `load_diagnostics` (current update flow leaves
`load_diagnostics: Vec::new()` only on success — but on subsequent failure they
re-populate; the bypass is asymmetric).

---

## F3 (HIGH) — `$/cancelRequest` for an in-flight request is never observed in the serialized server; `InFlight` cancellation is a no-op at the protocol surface (AC-13, AC-28)

**Files**:

- `crates/sifr_lsp/src/server.rs:61-130` — main loop is synchronous. After
  `enqueue_request`, the server calls `drain_queued_requests` (line 128),
  which loops to completion before re-entering `connection.receiver.recv()`.
- `crates/sifr_lsp/src/server.rs:132-145` — `CancellationTarget::InFlight`
  branch only marks the queue's `cancelled` set; it does no IO and no
  message-cancel handling. The serialized loop cannot dequeue
  `$/cancelRequest` while a request runs.
- `crates/sifr_lsp/src/session.rs:159, 165` — the only phase-boundary
  cancellation checks live inside `with_document_analysis`. Workspace-scoped
  loops do not check between modules/documents:
  - `workspace_diagnostic` (`crates/sifr_lsp/src/requests/diagnostics.rs:15-27`)
    iterates `document_uris()` without `session.check_active_request_cancelled`.
  - `workspace_symbol` (`crates/sifr_lsp/src/requests/symbols.rs:29-43`) does
    the same per-URI loop with no inter-iteration cancel check.
- The test `active_request_cancellation_fails_phase_boundary_checks`
  (`crates/sifr_lsp/src/session.rs:451-482`) directly calls `cancel_request`
  inside the same thread, bypassing the message-pump path. It proves the
  mechanism but not protocol observability.

**Severity**: HIGH for AC-13. The phase decision in
`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:738-742`
explicitly requires cancellation checks "between modules/files…before
publishing results, and in worker-loop boundaries"; the current code only
implements the wrap-and-publish guard, not inter-module checks.

**Recommended experiment**: hand-craft a `workspace/diagnostic` over the
stress harness with three open documents; concurrently send `$/cancelRequest`
for that id via a second thread immediately after sending the request. Capture
`sifr/debugTrace` afterwards: no `phase=cancellation` event should appear in
the executing request's window because the cancel notification is buffered
until after the request finishes. Document that finding as evidence the
in-flight cancellation surface is not observable.

**Fix candidate**: spawn a dedicated reader thread that decodes
`$/cancelRequest` immediately into a shared `BTreeSet<RequestKey>` (or
`AtomicBool` per active token), and have workspace-scoped loops call
`check_active_request_cancelled` between modules.

---

## F4 (HIGH) — Watchdog is a no-op on Windows and only fires when a message arrives (AC-24)

**Files**:

- `crates/sifr_lsp/src/watchdog.rs:49-52` — non-Unix builds always return
  `true` from `parent_is_alive`. Editor processes on Windows/macOS native
  Apple Silicon Microsoft Edition (not our case, but Windows definitely) get
  no protection.
- `crates/sifr_lsp/src/server.rs:62` — `self.watchdog.check()?` runs ONLY at
  the top of each message-loop iteration; `connection.receiver.recv()` blocks
  indefinitely if no message arrives. If the parent dies and the editor stops
  sending requests/notifications, the server lives forever.

**Severity**: HIGH for AC-24. The stress test
(`verification/tooling/lsp_protocol_stress.py:26`) always sends messages so
the gap is invisible there. The M13 pass-1 reviewer (
`reviews/typescript-go-m13-lsp-cancellation-progress-watchdog-review-pass-1.md`
line 24) already flagged the Windows hole. Pass-2/3 did not close it; the
"idle-loop" hole has not been raised before.

**Recommended experiment**: spawn `target/debug/sifr lsp --stdio --parent-pid
<doomed-pid>`, send `initialize`, kill the parent helper, then send nothing
else. Sleep for 30 s. Verify the server is still alive
(`kill -0 <child>` succeeds). Then send a no-op notification — only then will
the watchdog detect the dead parent.

**Fix candidates**:
- Spawn a separate timer thread that calls `parent_is_alive` periodically and
  closes the connection sender on failure.
- Implement Windows `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,...)` +
  `GetExitCodeProcess`.

---

## F5 (MEDIUM) — LSP never opens project-mode workspace; CLI/LSP analysis paths are not unified (AC-2)

**Files**: `crates/sifr_lsp/src/analysis_workspace.rs:96-101` always passes
`FrontendMode::SingleFile`. CLI analysis uses
`AnalysisHost::open_project(root)`
(`crates/sifr_analysis/src/host/implementation.rs:37-40`).

**Consequences**: imports that resolve into other workspace modules,
workspace-level diagnostics (cross-module type checks), workspace-level symbol
completion, and `workspace/diagnostic` results all behave as if every file is
an orphan. AC-2 specifically calls for "snapshots shared by CLI analysis and
LSP requests"; the two consumers use different host constructors and so cannot
share state. This is the same root cause as F1 but framed at the boundary
between LSP and CLI consumers.

**Recommended experiment**: open a 3-file project under
`verification/performance/query_projects/lsp/main.sifr` and run
`textDocument/diagnostic` for each open buffer. Compare LSP output against
`cargo run -q -p sifr -- check demos/project_graph/main.sifr` output for the
same project; the LSP will silently miss cross-module diagnostics that
`check` reports.

---

## F6 (MEDIUM) — Cross-host FileId collisions silently overwrite `uri_map` / `source_map` (AC-7)

**Files**:

- `crates/sifr_lsp/src/analysis_workspace.rs:57-76` —
  `uri_map`/`source_map` collect `(analysis.file.as_u32(), uri/source)` from
  ALL hosts into a single `BTreeMap<u32, String>`. Because each host
  independently allocates FileIds starting from 0, the first FileId from each
  host (typically 0) collides; `BTreeMap::insert` silently overwrites.
- `crates/sifr_lsp/src/requests/navigation.rs:114-128` — `locations(...)` uses
  these maps to translate every result's `file.as_u32()` back into a URI or
  source string. Any cross-host result hits the wrong entry.

**Severity**: MEDIUM. Masked today because most queries are within one host's
single file. Becomes a correctness bug as soon as F1/F5 are fixed (or if a
single host ever holds multiple files).

**Recommended experiment**: add a unit-level test that opens two documents
through `Session::open_document`, asserts `session.uri_map().len() == 2`. With
the current per-host architecture and BTreeMap collapse, the test should
either pass by luck (because each host's `documents` only contains one entry
each producing distinct FileIds when those happen to differ) or surface the
collision when they collide.

---

## F7 (MEDIUM) — Watcher-storm degradation runs per-host, not workspace-wide (AC-20)

**File**: `crates/sifr_lsp/src/analysis_workspace.rs:43-49` —
`record_watcher_events` iterates all documents and calls
`host.record_watcher_events(event_count, WATCHER_STORM_THRESHOLD)` on each
host. The storm threshold (64) is applied per single-file session.

**Severity**: MEDIUM. The locked decision
(`issues/ad-hoc-typescript-go-compiler-architecture-transfer.md:701-707`)
treats `WatcherStorm` as a workspace-scope reason, not a per-file degradation.
The per-host distribution means a 200-event burst across one workspace looks
like a 200-event burst to each open file's host, so each host re-evaluates
storm degradation independently — but since each host's workspace is one file,
the "storm" is meaningless at the document level.

**Recommended fix**: introduce a single workspace session (F1 fix), then
record watcher events once.

---

## F8 (MEDIUM) — Diagnostic "debounce" is structural batching only; jobs flush immediately and per-event (AC-20)

**Files**:

- `crates/sifr_lsp/src/diagnostics.rs:14-26` — `publish_document` calls
  `schedule_document_diagnostics` and then immediately calls `flush_ready`.
  No timer / sleep / wait.
- `crates/sifr_lsp/src/session.rs:333-356` — `schedule_document_diagnostics`
  coalesces only when the same URI is scheduled before any flush. Once
  `flush_ready` drains it, the next schedule starts a new entry.

**Effect**: every `didOpen`/`didChange`/`didSave` performs a synchronous
diagnostic pass and publishes inside the same notification handler call. The
"debounce" claim in `internal_docs/lsp_server.md:90-94` is structural
(URI dedupe + version capture), not temporal. M11 review pass-2 / pass-3
satisfaction was based on the in-process queue, not a real time window.

**Severity**: MEDIUM. AC-20 ("repeated edits do not enqueue unbounded
redundant work") is preserved within a single notification batch but not
across a sequence of `didChange` notifications — a typist gets one full pass
per keystroke.

**Recommended experiment**: send 10 consecutive `didChange` notifications and
count `textDocument/publishDiagnostics` responses; expect 10 (one per change),
not 1.

---

## F9 (MEDIUM) — Protocol smoke/stress harnesses do not exercise cross-file editor queries (AC-27)

**Files**:

- `verification/tooling/lsp_protocol_smoke.py:202-219` — opens a single
  `main.sifr` and a separate `formatting.sifr`; never sends a query that
  expects results from both.
- `verification/tooling/lsp_protocol_stress.py:30-32` — opens a `secondary.sifr`
  only to test diagnostic publication; no cross-file `workspace/symbol`,
  `textDocument/references`, or `textDocument/rename` against both files.

**Severity**: MEDIUM. AC-27 ("marker-based editor fixtures cover multi-file
hover, completion, definition, references, rename, diagnostics, semantic
tokens, formatting, and stale snapshots") is covered by analysis-layer marker
corpus tests
(`crates/sifr_analysis/src/host/m17_tests.rs:marker_editor_corpus_covers_multifile_queries_and_stale_snapshots`)
but those don't exercise the protocol path. The protocol cannot demonstrate
multi-file editor behavior the phase promises.

**Recommended experiment**: add `verification/tooling/lsp_protocol_multifile.py`
that drives `workspace/symbol`, cross-file references, and cross-file rename
through stdio. Today the test should FAIL (per F1), which is the point.

---

## F10 (MEDIUM) — `perf.lsp.navigation.symbol` collapses five protocol surfaces into one budget (AC-26)

**File**: `verification/performance/lsp_query_budget_ids.md:96` —
`lsp-definition`, `lsp-document-highlights`, `lsp-document-symbols`,
`lsp-folding-ranges`, `lsp-workspace-symbols` all map to the single budget
`perf.lsp.navigation.symbol` / manifest case `lsp-query-008-navigation`.

**Severity**: MEDIUM. AC-26 requires "per-request editor latency budgets
enforced separately for each request family". Five distinct request families
share one threshold (p95 10.954 ms). A regression in definition latency could
be masked by improvements in document-symbol latency or vice-versa.

**Recommended action**: split `lsp-query-008-navigation` into five separate
manifest cases each mapped to one reserved budget id
(`perf.lsp.definition.local_symbol`, `perf.lsp.document_symbols.module`,
`perf.lsp.folding_ranges.module`, etc.).

---

## F11 (LOW) — `sifr/debugTrace` events carry `snapshot_id: None`; ring buffer drops oldest events silently (AC-27)

**File**: `crates/sifr_lsp/src/session.rs:314-325` — every trace event is
built with `snapshot_id: None`. The ring buffer is capped at 256 events
(line 18); when full, the oldest event is removed (line 316). No flag is
emitted to indicate truncation.

**Severity**: LOW. Hampers post-hoc analysis — a cancellation event cannot
be correlated to the snapshot/request it interrupted, and long sessions lose
context invisibly. Reviewer should consider:
- correlating each `Scheduler`/`Cancellation`/`StaleRejection` event with the
  workspace `snapshot_id` captured at the time;
- adding a `truncated_events` counter the trace renderer surfaces.

---

## F12 (LOW) — Cancellation token is just an `RequestId` clone; not a real flag (AC-13)

**File**: `crates/sifr_lsp/src/cancellation.rs:1-18` — `CancellationToken`
wraps `RequestId`; `check_active_request_cancelled` consults the queue's
hashset.

**Severity**: LOW within the serialized server, but blocks the future
background-worker case the phase contemplates. Worth a tracked follow-up to
use `Arc<AtomicBool>` per token before AC-13 is claimed beyond serial
execution.

---

## F13 (LOW) — Reserved `perf.lsp.*` IDs remain unmapped; deferred but not tracked (AC-26)

**File**: `verification/performance/lsp_query_budget_ids.md:13-42` reserves
28+ budget IDs. Manifest enforces 18 (counting `lsp-query-001..018`). The
following reserved IDs have no manifest case:

- `perf.lsp.completion.auto_import`
- `perf.lsp.definition.local_symbol`
- `perf.lsp.rename.prepare`
- `perf.lsp.semantic_tokens.delta`
- `perf.lsp.document_symbols.module`
- `perf.lsp.folding_ranges.module`
- `perf.lsp.document_sync.unchanged_did_change`
- `perf.lsp.document_sync.changed_did_change`
- `perf.lsp.document_sync.parse_error_recovery`
- `perf.lsp.transport.initialize`
- `perf.lsp.transport.shutdown`

The doc itself calls these "intentionally deferred" (line 107-111) but does
not link them to an open follow-up. AC-26 is met for what is enforced; the
gap is documentation/follow-up hygiene.

**Severity**: LOW.

---

## Summary table

| # | AC | Severity | Theme |
| - | -- | -------- | ----- |
| F1 | AC-2/7 | HIGH | LSP analysis is per-document, not per-workspace |
| F2 | AC-12 | HIGH | `load_diagnostics` fast path skips stale-rejection in pull diagnostics |
| F3 | AC-13/28 | HIGH | In-flight cancellation never observed in serial server; no inter-module checks |
| F4 | AC-24 | HIGH | Watchdog dead on non-Unix and idle main loop |
| F5 | AC-2 | MEDIUM | CLI uses project mode, LSP uses single-file mode |
| F6 | AC-7 | MEDIUM | FileId collisions in `uri_map`/`source_map` |
| F7 | AC-20 | MEDIUM | Watcher-storm decided per host |
| F8 | AC-20 | MEDIUM | Diagnostic "debounce" is one publish per keystroke |
| F9 | AC-27 | MEDIUM | Protocol harnesses lack multi-file coverage |
| F10 | AC-26 | MEDIUM | One navigation budget covers five surfaces |
| F11 | AC-27 | LOW | Trace events lack snapshot_id; silent ring-buffer truncation |
| F12 | AC-13 | LOW | Cancellation token is RequestId clone, not a flag |
| F13 | AC-26 | LOW | Reserved budget IDs deferred but untracked |

The highest-leverage fixes are F1/F5 (one shared workspace session for the LSP,
in project mode when available) and F2/F3 (snapshot-guarded pull diagnostics,
plus real mid-request cancellation). F4 is a small Rust patch but should not
ship as "watchdog implemented" without it.

## Recommended ./tmp experiments

Create `tmp/tsgo_phase_audit_pass3/` with the following:

1. `multi_file_workspace_symbol.py` — drives `workspace/symbol` after opening
   two `.sifr` files with disjoint `def`s; asserts both appear; per F1 expect
   results biased to whichever file was queried last.
2. `cross_file_references.py` — opens `main.sifr` and `helper.sifr` where
   `main.sifr` imports a function from `helper.sifr`; sends
   `textDocument/references` on the imported function in `main.sifr`; asserts
   the helper definition site is in the result.
3. `pull_diagnostic_stale_race.py` — induces a parse-error `didOpen`, fixes
   it via `didChange`, then sends `textDocument/diagnostic`; verifies the
   returned diagnostics reflect the fixed text and not the cached
   load-diagnostics.
4. `in_flight_cancel_observability.py` — sends a long-running
   `workspace/diagnostic`, immediately sends `$/cancelRequest` for that id;
   captures the `sifr/debugTrace` response and asserts at least one
   `phase=cancellation` event occurred BEFORE the response was sent. Today
   this fails per F3.
5. `idle_watchdog.sh` — spawns `sifr lsp --stdio --parent-pid <doomed>`,
   completes `initialize`, then sleeps 30 s without sending anything; expects
   the child to exit. Today the child stays alive.

All five experiments are expected to either fail or produce surprising output;
recording the actual transcripts under `tmp/tsgo_phase_audit_pass3/` would
give the next reviewer concrete protocol-level evidence to attach to a
follow-up issue.
