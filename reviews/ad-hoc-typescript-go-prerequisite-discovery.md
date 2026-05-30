# TypeScript-Go Architecture Transfer Prerequisite Discovery

Date: 2026-05-29

Scope: review the current Sifr implementation before starting the ad-hoc TypeScript-Go compiler-service architecture phase, decide what is genuinely left, and identify prerequisites that would make later milestones unsound if skipped.

## Decisions To Review

1. Complete source maps are a hard prerequisite, not late debuggability work.
2. Sifr needs one shared source text / line-map / position-conversion authority across frontend, diagnostics, and LSP.
3. A typed source-provider/VFS boundary must exist before workspace snapshots can be correct.
4. LSP must stop rebuilding a per-document single-file `AnalysisHost` before scheduler and stale-result work can be meaningful.
5. The current `AnalysisSnapshot` is only a revision token; real immutable snapshot contents must be designed before M2.
6. `DirtyScope` and `ModuleSignature` must be designed before structural module replacement or copy-on-write reuse is allowed.
7. Scheduler/cancellation work needs captured snapshot identity and cancellation tokens before any concurrent request/background work.
8. Editor/watch events need compaction before asynchronous scheduling or fine-grained watcher invalidation.
9. Protocol-level performance gates must split per request family; the current aggregate LSP benchmark is only smoke coverage.
10. Symbol/index work needs source-map-backed ranges and bucketed indexes before scalable completion/import work is claimed.
11. Planning docs must distinguish current implementation from target architecture because several Phase 36 docs describe intended layers that are only partially implemented.

## Current Sifr Evidence

Source maps:

- `crates/sifr_frontend/src/graph_cache_and_queries.rs:76` defines frontend `SourceText` as only a string wrapper.
- `crates/sifr_frontend/src/graph_cache_and_queries.rs:219` defines `SourceMapView` with only files and revision.
- `crates/sifr_frontend/src/graph_cache_and_queries.rs:227` and `:237` return `None` from `text_position_to_span` and `span_to_text_range`.
- `crates/sifr_frontend/src/bin/frontend_query_bench.rs:255` times `interactive.source_map_lookup` but ignores the stub result.
- `crates/sifr_syntax/src/lib.rs:87` has a separate `SourceText` with UTF-8 byte offsets only.
- `crates/sifr_diagnostics/src/source_map/mod.rs:55` has a separate diagnostic source map with text and line starts.
- `crates/sifr_lsp/src/conversion.rs:45` and `:69` use local syntax source text for LSP range conversion instead of frontend source maps.
- `crates/sifr_lsp/src/conversion.rs:391` converts diagnostics from rendered 1-based line/column fields.
- `crates/sifr_lsp/src/capabilities.rs:28` advertises UTF-8 position encoding.

Project source reads:

- `crates/sifr_frontend/src/graph_cache_and_queries.rs:423` loads projects through direct `std::fs::read_to_string` and `std::fs::read_dir`.
- `crates/sifr_driver/src/project/discovery.rs:248` reads project directories directly.
- `crates/sifr_driver/src/project/discovery.rs:416` and `:483` read project/package module source directly.
- `crates/sifr_lint/src/engine.rs:134` reads lint target files directly.
- `crates/sifr_format/src/lib.rs:197` reads target directories directly, and `:445` reads source files directly.
- `crates/sifr_package/src/manifest/sifr.rs:55` reads package manifests directly.
- `crates/sifr_package/src/ops/session_targets.rs:34` reads target directories directly.
- CLI and test support have additional direct reads that need classification as source-provider migration candidates or intentionally out of scope.

LSP/session/snapshot:

- `crates/sifr_lsp/src/document_store.rs:37` stores a per-document `DocumentAnalysis`.
- `crates/sifr_lsp/src/document_store.rs:91`, `:105`, and `:130` rebuild on full change, incremental change, and save.
- `crates/sifr_lsp/src/document_store.rs:258` constructs `FrontendMode::SingleFile` and calls `AnalysisHost::open_single_file`.
- `crates/sifr_analysis/src/host/implementation.rs:56` has an `update_document` seam, but LSP is not using it as a long-lived workspace service.
- `crates/sifr_analysis/src/snapshot.rs:75` stores only `AnalysisRevision` in `AnalysisSnapshot`.

Invalidation/cache:

- `crates/sifr_frontend/src/graph_cache_and_queries.rs:520` updates a module source.
- On any text change, `crates/sifr_frontend/src/graph_cache_and_queries.rs:545` clears lowered HIR, diagnostics, and analysis for all modules and bumps graph revision.
- `internal_docs/frontend_cache_invalidation.md:18` documents the current broad invalidation behavior.
- `rg -n "DirtyScope|ModuleSignature|can_replace_module_in_project|ExportSignature|CowProjectState|WorkspaceSession|WorkspaceSnapshot" crates internal_docs` currently finds no implementation outside planning text.

Scheduler/cancellation/events:

- `crates/sifr_lsp/src/scheduler.rs:1` only defines lane labels.
- `crates/sifr_lsp/src/request_queue.rs:5` only tracks pending request IDs.
- `crates/sifr_lsp/src/server.rs:85` handles each request synchronously.
- `crates/sifr_lsp/src/session.rs:43` handles cancellation by removing a pending ID only.
- `crates/sifr_lsp/src/notifications/mod.rs:54` handles watched-file changes by republishing diagnostics directly.
- `crates/sifr_lsp/src/notifications/mod.rs:91` applies each didChange event immediately and republishes diagnostics.
- M1-M3 should remain serialized unless snapshot identity and cancellation tokens are moved earlier than M4.

Performance gates:

- `verification/performance/manifest.json:61` through `:65` includes Phase 35 interactive frontend-query cases.
- `verification/performance/manifest.json:67` has one aggregate `lsp-query-001-request-families` case.
- `verification/performance/lsp_query_bench.py:52` only recognizes `lsp.request_families`.
- `verification/tooling/check_phase36_closeout.py:131` validates that the aggregate case exists, not that per-request cases exist.
- `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md:534` lists planned per-request protocol budgets.

Symbol/index readiness:

- `crates/sifr_analysis/src/symbols.rs:89` returns document symbols with `range: None`; precise symbol/navigation ranges depend on the M0 source-map foundation.
- `crates/sifr_analysis/src/symbols.rs:103` serves workspace symbols from one whole-project index.

Docs versus implementation:

- `internal_docs/lsp_server.md:31` through `:39` describes target layers including line indexes, cancellation tokens, request scheduling, and snapshot capture, but current code only implements a subset.

## TypeScript-Go Evidence Used As Design Input

- `typescript-go/internal/compiler/program.go:284` has `Program.UpdateProgram` for one changed file, gated by `canReplaceFileInProgram`.
- `typescript-go/internal/project/snapshot.go:233` clones snapshots from compacted changes and overlays.
- `typescript-go/internal/project/compilerhost.go:94` acquires parsed source files through a parse cache keyed by parse options, hash, and script kind.
- `typescript-go/internal/project/watch.go:32` implements a reference-counted watcher registry.
- `typescript-go/internal/lsp/server.go:1002` negotiates UTF-16/UTF-8 position encoding and starts a parent-process watchdog when available.

## Proposed Contract Changes

- Add M0 before M1 for prerequisite closure.
- Move source-map completion into M0.
- Keep M7 for trace/debuggability and post-M0 source-position coverage extensions.
- Add W-0 to the execution tracker for discovered blockers.
- Require Claude review before implementation approval.

## Review Questions For Claude

1. Are any prerequisite blockers missing from the decisions above?
2. Are any items incorrectly labeled prerequisites when they should remain later milestone work?
3. Is source-map completion correctly classified as a hard prerequisite?
4. Is the dependency order M0 -> M1 -> M2 -> M3 -> M4 defensible?
5. Are there source-code facts here that contradict the proposed phase contract?
