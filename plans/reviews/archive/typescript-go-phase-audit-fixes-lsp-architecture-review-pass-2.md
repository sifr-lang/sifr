I have enough information to write the review. Let me finalize.

---

## Review verdict: **NOT SATISFIED**

The diff makes real progress on most of the prior findings (project-mode shared host, workspace stress coverage, dirty-scope/reuse on didChange, AC-18 source-file identity, workspace-symbol dedup, watchdog idle thread), but introduces a critical regression in URI mapping that breaks cross-file LSP semantics for a large class of projects.

### Blocker

**1. Split-brain `documents` + `projects` causes FileId collision in `uri_map`/`source_map`.**
`LspAnalysisWorkspace` now keeps two analysis worlds for every project-owned URI: a per-document `LspDocumentAnalysis` (always allocates `FileId(0)`) *and* the new `LspProjectAnalysis`. Query routing in `with_document` correctly prefers the project, but `uri_map`/`source_map` *merge* both stores, with `documents` extending **after** projects.

- `crates/sifr_lsp/src/analysis_workspace.rs:34-37` — `open_document` unconditionally inserts an `LspDocumentAnalysis` for every URI, even when it lives under `sifr.toml`.
- `crates/sifr_lsp/src/analysis_workspace.rs:39-56` — `update_document` keeps maintaining the per-document analysis alongside the project one.
- `crates/sifr_lsp/src/analysis_workspace.rs:110-127` — `uri_map`: project entries seed the map, then `documents.iter()` (BTreeMap, alphabetical by URI) all stamp `(0, <uri>)` on top, so `FileId(0)` ends up bound to the **alphabetically last open URI**, regardless of whether that URI is actually the project entrypoint.
- `crates/sifr_lsp/src/analysis_workspace.rs:129-149` — `source_map` has the same shape and the same hazard.

Concrete failure case: project with `main.sifr` (entrypoint, `FileId(0)`) + `utils.sifr` (`FileId(1)`). The single-file overlays for both URIs each carry `FileId(0)`. BTreeMap iteration of `documents` extends `main_uri→0` then `utils_uri→0`, so `uri_map = {0: utils_uri, 1: utils_uri}`. Project-context symbols/references/rename results in `main.sifr` (FileId 0) are then attributed to `utils_uri` by `requests/symbols.rs:57-61`, `requests/navigation.rs:91-93,114-124`. Rename returns edits applied to the wrong file; references/definition return the wrong locations.

The new `verification/tooling/lsp_protocol_stress.py:135-181` happens to dodge this because `helper.sifr` < `main.sifr` alphabetically and `main` is also the entrypoint — the documents extend "lands" `main_uri` on `FileId(0)`, which coincidentally matches the project's mapping. Swap names to `main.sifr` + `utils.sifr` and the test would fail.

**Fix needed (any of):**
- In `open_document`/`update_document`, skip the `self.documents` upsert when `workspace_root_for(document.path())` resolves and the project owns the URI (and remove the corresponding `self.documents` entry from `refresh_projects` for now-project-owned URIs).
- Or, in `uri_map`/`source_map`, filter out `documents` entries whose URI is already present in any `projects[_].files_by_uri`.
- Add a regression test with `main.sifr` (entrypoint) + an alphabetically-later sibling (e.g. `utils.sifr`) that exercises `workspace/symbol`, `textDocument/references`, and `textDocument/rename` and asserts URIs are attributed to the correct files.

### Other issues (not strictly blocking, but should be addressed)

**2. AC-8 parallel-readers test is weaker than its name.** `crates/sifr_analysis/src/host/tests.rs:740-783` spawns 8 threads that each construct an *independent* `AnalysisHost`. That only verifies `load_project_with_provider`'s file-ordering is deterministic across separate instances — it never has multiple readers contending on a shared host/snapshot. The original concern (deterministic ordering when several concurrent queries snapshot the same host) is not covered. Suggest adding a test that shares one `AnalysisHost`+ snapshot across threads.

**3. `refresh_projects` does a full project rebuild on every open/close.** `analysis_workspace.rs:62-82` and `LspProjectAnalysis::open` go through `AnalysisHost::open_project_with_overlays` → `session.reload()` → `FrontendContext::load_project_with_provider`, i.e. a fresh project load each time `open_uris` changes. didChange (`update_document`) does use the M10 dirty-scope/reuse path, but the open/close lifecycle bypasses it. Consider an `add_overlay`/`remove_overlay`-style entry that keeps the project context alive across open/close.

**4. Misleading test name.** `crates/sifr_frontend/src/graph_cache_and_queries/reuse.rs:554` is still named `document_version_only_update_recaches_source_file_view` even though the assertions (`Arc::ptr_eq`, `source file should be reused for the same source text`) now affirm *reuse*. Rename to `…reuses_source_file_view`.

**5. Stale `document_version` baked into the cached `SourceFileView`.** With the AC-18 fix (`graph_cache_and_queries.rs:502-510`, `reuse.rs:294-309`) the source-file Arc is reused on a version-only update, but the `SourceFileView.document_version` inside the cache is the *old* value because `cached_source_file_view` is not refreshed. No current consumer reads it (everyone routes through `document_version_for_file`), so this is latent — but it is a footgun. Either drop `document_version` from `SourceFileView` or have `update_module_source` patch the cached view in-place.

**6. Watchdog idle thread has no test.** `crates/sifr_lsp/src/watchdog.rs:37-47` adds an unbounded 500ms-poll thread that calls `kill -0` and `std::process::exit(0)` on the first error, but no unit test covers it (existing tests still only call `check()` directly). Add at least one test that spawns the thread, kills a child `sleep` process, and observes the watchdog disposition (or refactor `check` to be exit-policy-agnostic so the loop is testable in isolation).

**7. `workspace_symbol` dedup is correct but wasteful.** `requests/symbols.rs:31-52` iterates every open URI and re-runs `workspace_symbols` on the project host once per URI, then drops duplicates via `(file, name, kind, container)`. For an N-file project that's N×full workspace scans. Consider iterating projects once + remaining single-file documents once.

**8. `project_entrypoint` heuristic flips between opens.** `analysis_workspace.rs:382-393` picks the file named `main` if any of the currently-open documents has that stem, otherwise the first open document. So opening `helper.sifr` alone makes it the entrypoint, then opening `main.sifr` rebuilds the project with main as the new entrypoint — and FileIds reshuffle. Combined with finding #1 that's another correctness foot-gun (cached FileIds in pending diagnostic jobs become stale). At minimum, prefer "any `<root>/main.sifr` on disk" over "any open document named main".

**9. Driver `cwd` test mutates global state.** `crates/sifr_driver/src/workspace/tests.rs:1-26` uses a `OnceLock<Mutex<()>>` to serialize *this one* `chdir` test, but the rest of the test suite is unaware of the lock. If anyone else adds a `chdir`-touching test without using the same guard, races will return. Either move the guard to a crate-level `tests/common` and force all `chdir` tests through it, or stop relying on cwd entirely by passing the path through the provider.

### What is satisfied
- Project-mode shared host under `sifr.toml` exists and `with_document` routes there (`analysis_workspace.rs:151-170`).
- didChange now uses `host.update_document` → `update_module_source` (M10 dirty-scope/reuse path), not full reload (`overlay_updates.rs:34-49`, `host/implementation.rs:63-116`).
- Workspace-shaped perf fixture (`verification/performance/query_projects/lsp_workspace/`) wired into cold-start/workspace-diagnostics/references/rename manifest entries (`verification/performance/manifest.json:68,71,72`).
- AC-18 source-file identity preserved on version-only updates (`graph_cache_and_queries.rs:502-510`, `reuse.rs:294-309`); test in `reuse.rs:554-606`.
- Watchdog idle timer exists (`watchdog.rs:37-47`); message-loop check kept.
- Driver normalizes empty-path root to `"."` (`workspace/mod.rs:40-49`) and `package_discovery` no longer joins paths with `,` (`package_discovery.rs:220`).

**Bottom line:** Fix #1 before merging — it actively breaks references/rename/workspace-symbol for any project whose entrypoint URI sorts before another open sibling (a very common shape). #2–#9 should follow but don't independently block.
