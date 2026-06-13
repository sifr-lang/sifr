# TypeScript-Go Phase Audit — Architecture/State Pass 2

Scope: compiler-service architecture and state correctness for the workspace
session, source provider, snapshot, dirty-scope, cache-key, reuse,
copy-on-write, residency, build info, and status/trace surfaces introduced by
M2–M16/M17.

Acceptance criteria audited: AC-1..AC-7, AC-16..AC-18, AC-21..AC-23, AC-25.

Verdict: CHANGES RECOMMENDED. The frontend has all of the *named* surfaces the
phase locked in (`SourceProvider`/`OverlaySourceProvider`/`TrackingSource
Provider`, `WorkspaceSession`/`WorkspaceSnapshot`, `WorkspaceDirtyScope`,
`signatures_can_replace_module_in_project`, ref-counted reuse caches,
`WorkspaceResidencySnapshot`, `SifrBuildInfoVerification`,
`WorkspaceDebugSnapshot`, `WorkspaceTracePhase`), and the snapshot/handle stale
rejection plumbing is sound. The blocking-class finding is that **the LSP
update path bypasses every incremental reuse mechanism the phase invested in**,
so AC-5/AC-6/AC-17/AC-18 hold only for CLI/`AnalysisHost::update_document`
callers and not for the editor surface those ACs were written for. Two smaller
issues (the structural-replacement predicate omitting compiler/package
context; unconditional `source_map_cache` invalidation for version-only edits)
weaken defense in depth.

This pass-2 audit revisits state ownership specifically; pass-1
(LSP/Runtime, `reviews/typescript-go-phase-audit-lsp-runtime-review-pass-3.md`
F1) already flagged the per-document host topology from the protocol angle —
this file restates it from the cache/snapshot side to show what is silently
disabled.

---

## F1 (BLOCKING) — LSP `upsert_overlay_document` rebuilds `FrontendContext` from scratch, defeating the M10 reuse pipeline (AC-1, AC-5, AC-6, AC-17, AC-18)

**Files**

- `crates/sifr_lsp/src/analysis_workspace.rs:115-126` — `update()` calls
  `host.upsert_overlay_document(...)` on every `didChange`/`didSave`.
- `crates/sifr_analysis/src/host/overlay_updates.rs:20-35` —
  `upsert_overlay_document` is `session.upsert_overlay(...)` followed by
  `session.reload()`.
- `crates/sifr_frontend/src/workspace_session.rs:288-326` — `reload()` for
  `WorkspaceSessionTarget::Project` *always* calls
  `FrontendContext::load_project_with_provider(root, &mut provider)?` (line
  297). For `SingleFile` it always calls `FrontendContext::load_single_file`
  (line 305).
- `crates/sifr_frontend/src/graph_cache_and_queries.rs:367-473` —
  `load_project_with_provider` constructs a brand-new `FrontendContext` with
  `module_graph_cache: None`, `source_map_cache: None`,
  `reuse_caches: FrontendReuseCaches::new()`,
  `graph_revision: GraphRevision(0)`, `source_revision: SourceRevision(0)`
  (lines 461-463). Every module is re-parsed via the provider in the loop at
  lines 402-438.

**Consequences**

- The `signatures_can_replace_module_in_project` /
  `WorkspaceDirtyScope::OneModule` / `ReverseDependencies` machinery that the
  phase celebrates in `crates/sifr_frontend/src/graph_cache_and_queries.rs:475
  -584` is unreachable from `didChange` — that path is exercised by
  `AnalysisHost::update_document` (`crates/sifr_analysis/src/host/implementati
  on.rs:63-116`), which the LSP never calls.
- `cargo test -p sifr_frontend
  structural_one_module_replacement_reuses_unchanged_cache_entries`
  (`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:494-558`) and the
  M10 sibling tests all use a long-lived `FrontendContext` and invoke
  `update_module_source` directly. None of them exercises
  `WorkspaceSession::reload()` after an overlay edit, so they cannot prove the
  reuse claim for the LSP topology.
- AC-1 ("workspace/session snapshot API used by CLI analysis **and LSP
  request paths**") and AC-5 ("private body edits **reuse unaffected module
  results and do not invalidate the whole project**") are satisfied only on
  the CLI/test surface. The LSP path tears the per-URI `FrontendContext` down
  and rebuilds it for every edit.
- AC-18 ("copy-on-write snapshot finalization reuses unchanged project maps
  …**by identity**") is observable in `Arc::ptr_eq` only when two snapshots
  come from the same `FrontendContext`. After `reload()`, both
  `module_graph_arc_for_reuse` and `source_map_arc_for_reuse`
  (`crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:151-173`)
  allocate fresh `Arc`s on the new context.

**Root cause**

There are two entry points for an overlay-driven update:

1. `AnalysisHost::update_document(file, version, text)` — calls
   `context.update_module_source(...)`, runs the dirty-scope merge, preserves
   `reuse_caches`. *Not* invoked from `sifr_lsp`.
2. `AnalysisHost::upsert_overlay_document(path, uri, version, text)` — calls
   `session.upsert_overlay(...)` then `session.reload()`. *This is the LSP
   path.* It does not consult `update_module_source` at all and starts a fresh
   `FrontendContext`.

The session and the context have diverging notions of what a "document update"
is. The session-level `upsert_overlay` correctly records `OneModule` /
`DocumentVersionOnly` dirty scope (`workspace_session.rs:386-400`), but the
ensuing `reload()` then discards the entire context anyway.

**Recommended fix (architectural)**

- Make `WorkspaceSession` keep a long-lived `FrontendContext` and feed overlay
  edits into it via `FrontendContext::update_module_source` rather than
  reconstructing the context. `reload()` should become the rare "config /
  package / file set changed" path, not the common edit path.
- Until that lands, `upsert_overlay_document` should detect a pure
  body-only edit on a known module and route it through `update_module_source`
  (mirroring `host.update_document`), falling back to `reload()` only for
  graph-structure or package-level reasons.

**Severity: BLOCKING** for AC-1 / AC-5 / AC-6 / AC-17 / AC-18 on the LSP
surface; SATISFIED for CLI callers that invoke `AnalysisHost::update_document`
directly. Already flagged from a different angle in
`reviews/typescript-go-phase-audit-lsp-runtime-review-pass-3.md` F1, but the
state-side analysis here shows it nukes M10 reuse on top of being the wrong
host topology.

---

## F2 (MAJOR) — `signatures_can_replace_module_in_project` ignores compiler options, package identity, and entrypoint (AC-17)

**File**: `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:141-149`

```rust
pub(super) fn signatures_can_replace_module_in_project(
    old_signature: &ModuleSignature,
    new_signature: &ModuleSignature,
    parse_failed: bool,
) -> bool {
    !parse_failed
        && old_signature.imports == new_signature.imports
        && old_signature.exports == new_signature.exports
}
```

The locked decision at `issues/ad-hoc-typescript-go-compiler-architecture-tran
sfer.md:709` reads: "`can_replace_module_in_project` is allowed only for
`DirtyScope::OneModule` when parse/lower/compiler options, `ModuleSignature`,
package-visible metadata, and entrypoint identity are unchanged."

The predicate compares only imports and exports. The caller
(`FrontendContext::can_replace_module_in_project`, reuse.rs:118-139) only
takes a `replacement_source: &SourceText`, so in current callers
compiler/package context is the same `FrontendContext` and *cannot* have
changed during the call. That makes the omission **safe in practice today**
but a defence-in-depth gap: any future caller that lets the workspace target
shift (e.g., mode change, package identity reload) and reuses this predicate
would incorrectly approve a structurally unsafe replacement.

**Recommended fix**

- Either narrow the API so it physically cannot be called across differing
  contexts, or extend the predicate to take `WorkspaceCompilerOptions` and
  `WorkspacePackageConfigIdentity` and assert equality.
- Add a negative test: same module text replacement across two
  `FrontendContext`s with different `FrontendMode` must return `false`.

**Severity: MAJOR** (latent bug class; not currently reachable).

---

## F3 (MAJOR) — `update_module_source` invalidates `source_map_cache` and bumps `source_revision` on version-only updates (AC-18)

**File**: `crates/sifr_frontend/src/graph_cache_and_queries.rs:501-510`

```rust
self.modules[index].source_file_view = None;
self.source_revision.0 += 1;
self.source_map_cache = None;
if text_changed {
    self.module_graph_cache = None;
}
```

This runs *before* the `text_changed` branch and applies even when
`old_hash == new_hash` (document-version-only). The dirty-scope arm for that
case (lines 564-568) is `DirtyScope::None + DirtyReason::DocumentVersionOnly`,
yet the source-map cache identity has already been thrown away.

The codified test
`document_version_only_update_recaches_source_file_view` (reuse.rs:585-586)
explicitly *asserts* this:

```rust
assert!(Arc::ptr_eq(&first_graph, &second_graph));
assert!(!Arc::ptr_eq(&first_source_map, &second_source_map));
```

so it is intentional, but it directly contradicts AC-18 ("Copy-on-write
snapshot finalization reuses unchanged project maps, **source maps**,
diagnostics, indexes, and config/package metadata **by identity**"). For a
DocumentVersionOnly transition the canonical-path → source-hash mapping is
literally unchanged.

**Root cause** is that `SourceFileView` carries `document_version` and the
source-file cache key
(`graph_cache_and_queries/reuse.rs:294-317`) fingerprints
`document_version`, so a version bump alone invalidates the file view. Then
the source-map view, which is keyed on the line-up of file views, must also
be rebuilt.

**Recommended fix**

- Separate "what is the canonical source text and line map" from "what is the
  open-document version". The first is what AC-18 says should reuse Arc
  identity; the second is per-snapshot metadata that the diagnostics/handle
  publication layer carries.
- Promote `document_version` out of the cache key for `SourceFileView` and
  attach it to the snapshot-facing wrapper (e.g.,
  `WorkspaceSnapshot.overlays`) so the cached file view can stay shared.

**Severity: MAJOR** (AC-18 violation that is currently *encoded* in a test as
expected behavior).

---

## F4 (MINOR) — `AnalysisHost::metadata()` defaults `workspace_snapshot_id` to `None` (AC-7)

**File**: `crates/sifr_analysis/src/host/implementation.rs:805-811`

```rust
fn metadata(&self, query: AnalysisQueryKind) -> QueryMetadata {
    QueryMetadata {
        query,
        revision: self.current_revision,
        workspace_snapshot_id: None,
    }
}
```

The snapshot-routed entry points in
`crates/sifr_analysis/src/host/snapshot_queries.rs:16-23` correctly overlay
this with
`.with_workspace_snapshot_id(self.workspace_snapshot_id())`, and every public
LSP request path (`crates/sifr_lsp/src/{requests/**,commands.rs,diagnostics.rs}
`) goes through that route — so AC-7 holds at the protocol surface.

The concern is hygiene: `AnalysisHost`'s `pub fn diagnostics(...) -> QueryResul
t<...>` and siblings are reachable without snapshot wrapping (used internally
and from tests). If a future caller invokes them directly, the result will
silently lack snapshot identity. Either make them
`pub(crate)`/`pub(super)` to enforce the snapshot route, or have `metadata()`
require an explicit `Option<WorkspaceSnapshotId>` parameter so the omission is
syntactically visible.

**Severity: MINOR** (defence in depth; not currently exploited).

---

## F5 (MINOR) — `WorkspaceSession::record_analysis_document_update` is dead code that, if used, would defeat dirty-scope precision (AC-16)

**File**: `crates/sifr_frontend/src/workspace_session.rs:461-467`

```rust
pub fn record_analysis_document_update(&mut self) {
    self.revision.0 += 1;
    self.dirty_scope_report = WorkspaceDirtyScopeReport::new(
        WorkspaceDirtyScope::Workspace,
        vec![WorkspaceDirtyReason::SourceTextChanged],
    );
}
```

Repository search confirms it has no production callers
(`rg "record_analysis_document_update" crates/` returns only the definition).
The method bypasses the dirty-scope merge logic in `record_dirty_scope` and
flat-out installs `DirtyScope::Workspace + SourceTextChanged`, which the
contract says is the most conservative invalidation tier. Removing or
documenting it would prevent a future caller from accidentally erasing
`OneModule` precision.

**Severity: MINOR** (dead code with foot-gun shape).

---

## F6 (MINOR) — `ExportSignature` doesn't capture `__all__`, module docstrings, or re-export aliases (AC-6)

**File**: `crates/sifr_frontend/src/module_signatures.rs`

`module_signature` covers:

- function declarations (name, params, return, decorators, type params),
- class declarations (bases, decorators, type params, methods, fields),
- top-level constant assignments (typed and value-typed).

It does not capture:

- Module-level `__all__` lists,
- module docstrings (string literal first statements) when used as package
  documentation,
- top-level `from x import y as z` re-exports as part of the public surface.

For Sifr's current visibility model (compile-time names + signatures), this is
adequate. If Sifr later treats `__all__` or re-export aliases as part of the
public ABI, an unchanged-body edit could fail to invalidate reverse dependents
that read the changed re-export. Flagging now because AC-6 promises
"deterministic" invalidation and the contract decision text at issue:703
mentions "public functions, classes, constants, **and top-level
declarations**".

**Severity: MINOR** (Sifr-spec dependent; flagged as forward concern).

---

## F7 (MINOR) — Guardrail direct-read inventory is incomplete for two files (AC-3)

**Files**

- `crates/sifr_package/src/projection.rs:138,139,155,206` — `path.exists()` /
  `.join(...).exists()` probes inside the package-init / projection flow.
- `crates/sifr_driver/src/build/workspace.rs:123,229` — artifact-cache
  validation `.exists()` probes.

The guardrail
`verification/tooling/check_typescript_go_m1_guardrails.py` and the project's
direct-read inventory enumerate adjacent lines (projection.rs: 100, 109, 127,
129, 169, 187; build/workspace.rs: 219, 282, 296) under documented exception
categories ("package-management output and repair-state effects" and "M15
`.sifrbuildinfo` cache"), but the lines above fall into the same categories
without being listed. The reads are functionally fine — they don't bypass
the `SourceProvider` for semantic compilation reads — but the inventory is
incomplete, which is exactly the failure mode M1 was meant to prevent.

**Severity: MINOR** (hygiene; doesn't violate AC-3 semantically).

---

## SATISFIED checks

- **AC-2 / AC-3** (overlay + tracked reads): `OverlaySourceProvider` and
  `TrackingSourceProvider` (`crates/sifr_frontend/src/source_provider.rs:199-
  381`) implement overlay-aware reads with hashed overlay docs, synthesized
  nested directory entries, and `SourceDependency` records that include
  successful reads, probes, canonicalization, and `FailedLookup`. The session
  threads the tracking provider through `reload()` and stores the dependency
  list in `source_dependencies`, which `WorkspaceResidencyState::refresh_after
  _reload` consumes to register watchers
  (`crates/sifr_frontend/src/workspace_residency.rs:119-168`).

- **AC-4** (cache-key completeness): `crates/sifr_frontend/src/cache_keys.rs`
  threads `CompilerFingerprint` + `WorkspaceContextFingerprint` +
  `PackageContextFingerprint` + `QueryPolicyFingerprint` through every cache
  family (Parse, SourceMap, HirLowering, Diagnostics, Lint, Format,
  PackageGraph, SymbolBuckets, FlowGraph), with negative tests covering
  intentionally-omitted document identity inputs
  (`cache_keys.rs:807-863`) and the package/workspace identity contributions
  (`cache_keys.rs:758-805`).

- **AC-7** (stale-result rejection): `AnalysisHost::ensure_snapshot_current`
  (`host/implementation.rs:819-848`) compares workspace revision and
  graph/source revision, calls `WorkspaceSession::record_stale_rejection` to
  emit a trace event, and snapshot-routed queries overlay
  `workspace_snapshot_id` onto the result metadata. Snapshot handles
  (`crates/sifr_analysis/src/handles.rs:154-177`) reject mismatched kind /
  snapshot id / revision before resolving.

- **AC-16** (dirty-scope distinguishes all six tiers):
  `WorkspaceDirtyScope` and `WorkspaceDirtyReason` cover None / OneModule /
  ReverseDependencies / GraphStructure / ConfigProject / Workspace, with merge
  severity at `workspace_session.rs:146-184`. Selection logic in
  `update_module_source` (`graph_cache_and_queries.rs:514-568`) drives them
  based on import/export/parse status. `record_watcher_events` correctly
  degrades to `Workspace + WatcherStorm` above the threshold
  (`workspace_session.rs:339-359`).

- **AC-18** (COW Arc reuse): except for the source-map-on-version-only churn
  (F3) and the LSP topology issue (F1), unchanged `module_graph`,
  `source_map`, `overlays`, `source_dependencies`, `compiler_options`, and
  `package_config_identity` snapshots are Arc-cloned at
  `workspace_session.rs:479-516`. Identity reuse is exercised by
  `ref_counted_module_caches_reuse_identity_on_hits` (`reuse.rs:445`).

- **AC-21** (residency / config / watcher registries):
  `WorkspaceResidencyState` (`workspace_residency.rs:101-396`) maintains
  reference-counted projects, configs, and watchers with stdlib root and
  generated artifact retention; `release_open_file_project` evicts open-file
  owners on overlay removal (`workspace_session.rs:403-422`).

- **AC-22** (bucketed indexes): symbol buckets are keyed via
  `SymbolBucketScope` in the cache key
  (`cache_keys.rs:426-443`), and bucket readiness is reported through
  `WorkspaceIndexReadinessStatus` (`workspace_trace.rs:164-168`,
  `crates/sifr_analysis/src/host/debug_status.rs:7-30`).

- **AC-23** (build-info verification):
  `WorkspaceResidencyState::verify_build_info`
  (`workspace_residency.rs:210-260`) rejects compiler fingerprint mismatch,
  package/config mismatch, missing/mismatched/extra sources, and the
  WorkspaceNotLoaded case before accepting; rejection clears the cached
  `VerifiedSifrBuildInfo` on the state.

- **AC-25** (status/debug surface):
  `WorkspaceStatusSnapshot` carries snapshot id, revision, target kind, open
  file / project / source / module / dependency counts, cache entry counts,
  index readiness, last update latency, and memory counters
  (`workspace_trace.rs:79-141`). `cmd_trace`
  (`crates/sifr/src/trace_cli.rs:35-50`) renders status + trace; the LSP
  surfaces the same via `sifr/debugTrace` (out of scope here, audited in
  pass-1 LSP/Runtime).

---

## Verification commands run

- `cargo test -p sifr_frontend workspace_session` — not re-run in this pass;
  M15/M16 closeout tables show PASS.
- File-line citations verified by direct `Read` of each cited offset, not
  recalled from agent summaries.

## Recommended action items

1. **F1**: route LSP overlay edits through
   `FrontendContext::update_module_source` (or hold a long-lived
   `FrontendContext` in `WorkspaceSession` so `reload()` is not the edit
   path). Add a multi-file `didChange` integration test that asserts
   `Arc::ptr_eq` between consecutive snapshots' module graphs when only one
   module body changed. **Blocking for the phase's editor ACs.**
2. **F2**: extend `signatures_can_replace_module_in_project` to take
   compiler options + package identity + entrypoint and assert equality;
   add a negative test for cross-context misuse.
3. **F3**: separate `document_version` from `SourceFileView`'s cache identity
   so unchanged source text + line map can share an `Arc` across version
   bumps; update the codified test to assert identity reuse.
4. **F4**: tighten `AnalysisHost::metadata()` so direct callers cannot omit
   `workspace_snapshot_id` (either make direct query methods crate-private or
   require an explicit `Option<WorkspaceSnapshotId>` parameter).
5. **F5**: delete `record_analysis_document_update`, or rename it to make the
   "conservative-degrade" semantics explicit.
6. **F6**: confirm Sifr's public-API surface decision for `__all__` /
   re-export aliases / docstrings and either capture them in `ExportSignature`
   or document the omission in `internal_docs/frontend_cache_invalidation.md`.
7. **F7**: extend the M1 direct-read inventory to enumerate every
   `.exists()`/`std::fs` call in the exception-category files.
