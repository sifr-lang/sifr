## Review verdict: **SATISFIED**

The new scoped LSP file-map work closes the latent FileId-collision footgun that pass-3 had explicitly called out as a non-blocking remaining item. The pass-2 split-brain blocker remains fixed, and the new multi-project workspace-symbol stress test directly exercises the FileId(0)-across-projects case.

### Pass-2 blocker — still fixed
`LspAnalysisWorkspace` still keeps project-owned URIs out of `self.documents`:
- `analysis_workspace.rs:44-48` — `open_document` early-returns and drops any stale entry under a `sifr.toml` root.
- `analysis_workspace.rs:53-65` — `update_document` routes project URIs to `LspProjectAnalysis::update_document`.
- `analysis_workspace.rs:78-99` — `refresh_projects` removes project-owned URIs from `self.documents`.
- `analysis_workspace.rs:148-167` — `workspace_symbols` filters `documents` through `project_owned_uris()`.

### Pass-3's latent FileId(0) collision — now closed
The previous global `LspAnalysisWorkspace::uri_map()` / `source_map()` are gone. Per-document scoped `LspFileMaps` is the only path:
- `analysis_workspace.rs:31-34, 127-146, 198-212` — `LspFileMaps` is built from a single host (project *or* scratch), so `FileId(0)` from a project never overlaps `FileId(0)` from a scratch host.
- `LspProjectAnalysis::file_maps` at `analysis_workspace.rs:316-329` and `LspDocumentAnalysis::file_maps` at `analysis_workspace.rs:450-465` each emit only their own host's `(FileId.as_u32(), uri)` pairs.
- `session.rs:151-154` (`file_maps_for_uri`) routes via `file_maps_for_document`, so navigation/rename/code-action/type-hierarchy callers (`navigation.rs:81,110`, `code_action.rs:18,72`, `type_hierarchy.rs:11`) never see a global merged map.

### Workspace-symbol URI tagging
- `analysis_workspace.rs:36-39` — `LspWorkspaceSymbol { symbol, uri }` carries the originating URI alongside each result.
- `analysis_workspace.rs:331-362` and `:467-494` — the project and scratch overloads tag every symbol with the right URI, dropping project symbols whose `FileId` isn't in `files_by_uri` (acceptable for now: only open files were ever surfaced).
- `requests/symbols.rs:28-46` — dedup tuple now includes `uri`, so identical FileId(0) entries from two projects survive deduplication, and the call iterates projects + standalone documents once each.

### Deferred code-action resolve URI data
- `conversion.rs:267-285` — `code_action_data` writes `"uri": request_uri` into the deferred action payload alongside `"file"`.
- `requests/code_action.rs:39-101` — `resolve` now requires `data.uri`, uses it to load the right `file_maps_for_uri`, and the inner `file.as_u32() != data_file` check at `:83-87` keeps catching FileId reshuffles after project rebuilds. The version check at `:73-81` still catches in-place edits.

### Multi-project workspace-symbol regression test
`verification/tooling/lsp_protocol_stress.py:184-225` builds two siblings (`alpha/sifr.toml`+`alpha/main.sifr`, `beta/sifr.toml`+`beta/main.sifr`), opens both `main.sifr` files (each `FileId(0)` in its host), and asserts both `alpha_entry`/`beta_entry` come back with their respective URIs. This would have failed under the previous global `uri_map` since both `(0, ...)` entries would collapse.

The companion cross-file regression at `verification/tooling/lsp_protocol_stress.py:135-181` (the `helper` rename across `utils.sifr`+`main.sifr`) is unchanged from pass-3 and still passes through the new scoped maps because both URIs are open and resolve via `LspProjectAnalysis::file_maps`.

### Non-blocking remaining items

These do not block the merge but are worth tracking:

1. **`LspProjectAnalysis::file_maps` only covers open URIs** (`analysis_workspace.rs:316-329`). A rename whose analysis touches a project file that the editor hasn't opened would fail `workspace_edit` with `unknown file N`. The shape didn't exist pre-project-mode, so it's a new gap rather than a regression — but consider seeding `files_by_uri` from all project files (not just opened ones) and constructing URIs from paths on the fly.
2. **`LspProjectAnalysis::workspace_symbols` returns `Option<Vec<…>>`** (`analysis_workspace.rs:331-362`) where the caller treats `None` and `Some(empty)` the same. Collapse to `Vec`.
3. **`verification/performance/query_projects/lsp_workspace/`** is referenced by `verification/performance/manifest.json:68,71,72` but is still untracked (`git status` shows it under "Untracked files"). It needs to be `git add`-ed before this lands, or the perf gate will fail in CI.
4. **`refresh_projects` still does a full `open_project_with_overlays` on any change to `open_uris`** (pass-3 #3 unaddressed).
5. **`CWD_TEST_LOCK`** at `crates/sifr_driver/src/workspace/tests.rs:4-26` still local; any other crate-level `chdir` test bypasses it (pass-3 #9 unaddressed).
6. **Pre-existing, unrelated**: `requests/type_hierarchy.rs:48-52` `hierarchy()` returns an empty array whether `items` is empty or not — but this code path is unchanged by this diff.

**Bottom line:** Scoped `LspFileMaps`/`LspWorkspaceSymbol` is the right shape, the pass-2 blocker stays fixed, pass-3's latent FileId(0) collision is now genuinely closed, and the multi-project stress test pins it. The only must-do before merge is `git add verification/performance/query_projects/lsp_workspace/`.
