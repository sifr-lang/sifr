**SATISFIED.**

The pass-4 non-blocking gap (open-file-only file maps) is closed without introducing borrow, stale-source, URI, or API regressions, and the pass-2 / pass-3 / pass-4 fixes remain intact.

### How the pass-4 open-file-only gap closes

- `AnalysisHost::path_for_file` / `source_text_for_file` (`crates/sifr_analysis/src/host/implementation.rs:143-154`) delegate to `FrontendContext::path_for_file` / `source_text_for_file` (`crates/sifr_frontend/src/graph_cache_and_queries.rs:605-616`), which read `self.modules[index].source` and `.path` directly — no `SourceFileView` indirection, so no stale-source risk after the `text_changed` gating in `graph_cache_and_queries.rs:505-510`.
- `LspProjectAnalysis::uri_by_file` (`analysis_workspace.rs:299-316`) layers `files_by_uri` (client URIs for open documents) over `host.files()` (synthesized `file_uri_for_path` for project files the editor hasn't opened), so identifiers loaded transitively (e.g., `utils.sifr`) now get URIs.
- `LspProjectAnalysis::file_maps` (`analysis_workspace.rs:281-299`) builds `source_by_file` over every `host.files()` entry, so `conversion::location` / `workspace_edit` / `code_action` lookups resolve cross-file targets.
- `LspProjectAnalysis::workspace_symbols` (`analysis_workspace.rs:318-345`) uses the same `Self::uri_by_file` so symbols in unopened project files surface with the right URI — directly satisfies the cross-file stress assertion at `verification/tooling/lsp_protocol_stress.py:147-152` (only `main.sifr` opened, helper definition expected to come back tagged with `file_uri(utils)`).

### Pass-2 blocker — still fixed

Project URIs are kept out of `self.documents`:
- `analysis_workspace.rs:44-48,53-65,79-94` — open/update/refresh paths drop project URIs from `self.documents`.
- `analysis_workspace.rs:174-188` — `with_document` routes project URIs to `LspProjectAnalysis::with_host` before falling back to `self.documents`.

### Pass-3 / pass-4 multi-project FileId collision — still fixed

- `LspWorkspaceSymbol { symbol, uri }` (`analysis_workspace.rs:36-39`) keeps URI scoped to the originating project, and `Session::workspace_symbols` (`session.rs:155-167`) iterates each project's host separately so `FileId(0)` from project A and project B never share a URI.
- Dedup key in `requests/symbols.rs:35-42` includes `uri`, so identical FileId tuples from sibling projects are preserved (`alpha_entry` / `beta_entry` stress at `verification/tooling/lsp_protocol_stress.py:184-225`).

### No new borrow / stale / URI / API regressions

- **Borrow:** `Self::uri_by_file(&self.files_by_uri, host)` returns an owned `BTreeMap`, computed *before* `host.snapshot()` (`analysis_workspace.rs:322-329`), so no overlapping `&` / `&mut` on host.
- **Stale source:** `source_text_for_file` reads `module.source`, which is overwritten in `update_module_after_change` (`graph_cache_and_queries.rs:501`) before the cache-gate decides whether to bump `source_revision`. The removal of `SourceFileView::document_version` is benign: all reads now go through `FrontendContext::document_version_for_file`, which reads `modules[index].document_version` (always updated at line 503).
- **URI:** Open files keep the client-provided URI; unopened files derive via `Url::from_file_path` on the canonical absolute path returned by `path_for_file`. The stress test compares against Python's `file_uri(...)` built from the same absolute path, so they match.
- **API:** `path_for_file`, `source_text_for_file`, `open_project_with_overlays`, `LspFileMaps`, `LspWorkspaceSymbol` are all additive. Removed `SourceFileView::document_version` is internal to `sifr_frontend`; the only call sites updated in this diff are `reuse.rs:233,268` and the test at `reuse.rs:553-587`.

### Non-blocking items worth flagging

1. **`verification/performance/query_projects/lsp_workspace/` is still untracked** (`manifest.json:68,71,72` reference it). Pass-4 already called this a must-do-before-merge; it's still not staged. Run `git add verification/performance/query_projects/lsp_workspace/` before this lands or CI's perf gate will fail.
2. **`LspProjectAnalysis::file_maps` clones every source string on every navigation/code-action/rename call** (`analysis_workspace.rs:281-299`). Cost is O(project files × source bytes) per request — acceptable for now but worth a follow-up (e.g., return `Arc<str>` or a lazy lookup) before larger projects land.
3. **`LspProjectAnalysis::workspace_symbols` still returns `Option<Vec<…>>`** where `None` and `Some(empty)` are caller-equivalent (`analysis_workspace.rs:318`) — pass-4 #2 unaddressed, harmless.
4. **`refresh_projects` still does a full `open_project_with_overlays` whenever `open_uris` changes** (`analysis_workspace.rs:79-99`) — pass-3 #3 / pass-4 #4 unaddressed, perf-only.
5. **`project_entrypoint` heuristic ignores `sifr.toml`** (`analysis_workspace.rs:556-573`). Projects whose manifest declares a non-`main.sifr` entrypoint will be opened with the wrong entrypoint — fine for the current stress fixtures, but a known shape gap for real `sifr.toml` configurations.

Bottom line: the pass-4 open-file-only file-map gap is genuinely closed, pass-2/3/4 fixes hold, and the cross-file stress (`utils.sifr` not opened) now exercises the new path end-to-end. The only must-do before merge is `git add verification/performance/query_projects/lsp_workspace/`.
