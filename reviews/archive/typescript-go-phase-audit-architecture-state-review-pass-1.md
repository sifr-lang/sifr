Audit complete. Summary of what I found:

**Verdict: CHANGES RECOMMENDED**. The frontend ships every *named* surface the phase locked in, but one architectural commitment is silently disabled in the LSP path.

**Findings** (written to `reviews/typescript-go-phase-audit-architecture-state-review-pass-2.md`):

- **F1 (BLOCKING, AC-1/5/6/17/18)**: LSP `didChange` calls `host.upsert_overlay_document` → `session.reload()` → `FrontendContext::load_project_with_provider`, which constructs a **fresh** `FrontendContext` with empty `reuse_caches`, zeroed revisions, and no `module_graph_cache`/`source_map_cache`. The M10 incremental path lives on `FrontendContext::update_module_source` and is reached by `AnalysisHost::update_document` — which the LSP never calls. The M10 reuse tests pass because they invoke `update_module_source` directly; they don't exercise the editor topology the ACs target. Already flagged from a different angle as F1 in the LSP/Runtime pass-3 review.

- **F2 (MAJOR, AC-17)**: `signatures_can_replace_module_in_project` (reuse.rs:141–149) only compares imports+exports, omitting compiler options, package identity, and entrypoint that the locked decision requires.

- **F3 (MAJOR, AC-18)**: `update_module_source` invalidates `source_map_cache` and bumps `source_revision` unconditionally — even for `DirtyScope::None + DocumentVersionOnly` transitions. The codified test asserts `!Arc::ptr_eq` on source maps, contradicting AC-18.

- **F4 (MINOR, AC-7)**: `AnalysisHost::metadata()` defaults `workspace_snapshot_id` to `None`; the snapshot-routed path correctly overrides it, but the direct-host entry points are reachable without snapshot wrapping.

- **F5 (MINOR)**: `WorkspaceSession::record_analysis_document_update` is dead code that would clobber dirty-scope precision if used.

- **F6 (MINOR)**: `ExportSignature` doesn't capture `__all__`, module docstrings, or re-export aliases.

- **F7 (MINOR, AC-3)**: Direct-read inventory in the M1 guardrail is incomplete for `projection.rs` and `build/workspace.rs`.

**Satisfied**: AC-2, AC-3, AC-4, AC-7 (LSP surface), AC-16, AC-21, AC-22, AC-23, AC-25 are met as audited.
