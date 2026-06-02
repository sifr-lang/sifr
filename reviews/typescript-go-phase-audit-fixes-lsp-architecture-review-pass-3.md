## Review verdict: **SATISFIED**

The diff resolves the pass-2 blocker and addresses each of the prior findings 5–8 cleanly. The remaining items are non-blocking polish.

### Blocker (URI/FileId split-brain) — fixed

`LspAnalysisWorkspace` now keeps project-owned URIs entirely out of `self.documents`:

- `analysis_workspace.rs:34-41` — `open_document` early-returns (and removes any stale entry) when `workspace_root_for` finds a `sifr.toml`.
- `analysis_workspace.rs:43-62` — `update_document` routes a project-owned URI to the project's `update_document` and never falls back into `self.documents`.
- `analysis_workspace.rs:68-89` — `refresh_projects` removes project-owned URIs from `self.documents` defensively and keeps the project map keyed by root.
- `analysis_workspace.rs:117-137` and `:140-166` — `uri_map`/`source_map` first seed from projects, then `documents.iter()` is filtered through `project_owned_uris()` so the same URI cannot land twice. The same filter applies in `workspace_symbols` at `:178-185`.

The new regression test in `verification/tooling/lsp_protocol_stress.py:138-182` uses the exact `main.sifr` (entrypoint) + alphabetically-later `utils.sifr` shape called out in pass-2, and asserts `workspace/symbol`, `textDocument/references`, and `textDocument/rename` all attribute results to the correct files — which would have failed under the old code.

### Other pass-2 findings

- **#5 stale `document_version` in `SourceFileView`** — removed entirely. `source_maps.rs:90-97` drops the field; `graph_cache_and_queries.rs:502-510` only invalidates the cached view when text changes; `reuse.rs:549-590` test is renamed `…reuses_source_file_view` and asserts `Arc::ptr_eq` on the source map.
- **#6 watchdog idle-thread test** — added at `watchdog.rs:104-121` via a testable `spawn_thread_with` seam that lets the test inject a channel callback instead of `std::process::exit`. `server.rs:32-34` wires `spawn_exit_thread` for the production path (sound because `ParentWatchdog` is `Copy`).
- **#7 `workspace_symbol` N×scan** — `requests/symbols.rs:23-52` calls `session.workspace_symbols` once; each project is scanned exactly once in `analysis_workspace.rs:168-187`.
- **#8 project entrypoint stability** — `analysis_workspace.rs:470-486` now prefers `<root>/main.sifr` on disk before falling back to open documents.
- **#2 AC-8 shared readers** — `crates/sifr_analysis/src/host/tests.rs:740-792` now shares one `AnalysisHost` (Arc<Mutex>) and one cloned snapshot across 8 threads. The Mutex serializes them so it's not genuinely concurrent execution, but it does exercise the shared-host invariant pass-2 asked for.

### Minor remaining items (non-blocking)

- `refresh_projects` still triggers a full project rebuild on every open/close (only didChange takes the M10 dirty-scope path). Pass-2 finding #3 was not addressed.
- `CWD_TEST_LOCK` at `crates/sifr_driver/src/workspace/tests.rs:4` is still local to that file; any other `chdir` test must opt in to be safe (pass-2 #9 not addressed).
- `project_entrypoint` still falls back to "first open document" when no `<root>/main.sifr` exists and nothing is named main; cached FileIds in pending diagnostic jobs could still reshuffle in that edge case.
- Latent (pre-existing, not pass-2 scoped): `uri_map`/`source_map` use `FileId.as_u32()` as a global key while FileIds are per-host, so multi-project or project+scratch coexistence can still collide on FileId(0). Not exercised by the new regression test.

**Bottom line:** the cross-file FileId/URI fix is correct and adequately tested, and findings 5–8 are properly closed. Ship it; track the three minor items above as follow-ups.
