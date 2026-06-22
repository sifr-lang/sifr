I've traced the patch end-to-end against the architecture goal. Below are findings ordered by severity.

## Findings

### 1. Stress test asserts shape but not range validity — medium

`verification/areas/developer_tooling/lsp_protocol_stress.py:242-247` checks only that `foldingRange` returns a list and `semanticTokens/full` returns a dict with a `"data"` key. The original bug surfaced as "semantic token end/start is outside the document" / "range outside the document" — i.e. ill-formed *values* within an otherwise well-shaped response. A regression that re-introduces a stale-text owner could still return `{"data": [...]}` whose deltas exceed the new (shorter) text, and the test would pass.

Strengthen by either:
- Decoding the returned tokens (5-tuple deltas) and asserting every position is within the `shortened` text's line/char bounds; or
- Asserting `foldingRange` entries' `endLine` ≤ last line of `shortened`.

Either check directly closes the bug class instead of relying on transport errors.

### 2. Static split-brain check is narrow — medium

`verification/areas/developer_tooling/check_lsp_split_brain.py:56-77` only inspects two methods (`open_document`, `update_document`) and only when they contain the literal `match self.projects.get_mut(&root)` and the literal `None => true` sentinel. Gaps:

- A future refactor like `if let Some(project) = self.projects.get_mut(&root)` silently makes `project_match == -1` and the check returns no failures (line 67 — `continue` with no warning). The "unverifiable" branch only fires when `match` is found but the `None => true` arm is missing.
- `refresh_projects` (analysis_workspace.rs:91-114) is not inspected. The patch removed the fallback branch there, but a future regression that adds `LspDocumentAnalysis::open(document)` inside that loop would not be caught.
- Any new method that mutates `self.documents` from a project-routed path is not inspected.

Suggested tightening: scan the *whole* `impl LspAnalysisWorkspace` block for any `LspDocumentAnalysis::open(document)` call sites and only allow them inside the `else { /* no workspace root */ }` branches that the static check can identify. Or add a positive assertion that `refresh_projects` always calls `self.documents.remove(...)` after `LspProjectAnalysis::open`.

### 3. No focused Rust unit test for the didSave same-version path — low

The semantic-equivalent of the bug repro lives only in the protocol stress test. A `cargo test -p sifr_lsp` test that:

1. opens a project document at v1,
2. `change_compacted(uri, Some(2), ...)`,
3. `save_document(uri, Some(text_at_v2))` (note: `DocumentStore::save` does *not* bump version and returns `true` even when text is unchanged — document_store.rs:107-115),
4. asserts `session.analysis` has no entry in its `documents` map for that URI and that `file_maps_for_uri`/`with_document_analysis` succeed against the project owner,

would deterministically exercise the new fallback re-open without requiring the LSP subprocess. This is the cheapest way to lock down the fix.

### 4. `update_document` allows a pre-existing standalone to keep owning a project-rooted URI — low

analysis_workspace.rs:62-65 short-circuits to the standalone if one already exists, before consulting the project. This is only reachable in the transition state where a document was opened before `sifr.toml` existed and the project has not yet been built. In that state no project owns the URI, so it isn't split-brain. But it is brittle: if any future code path inserts a standalone for a project-rooted URI (the static check above will not always catch it), this branch will silently route around the project and serve stale text indefinitely until the next `open`/`close` triggers `refresh_projects`.

Two cheap hardenings:
- Restructure to *prefer* the project when `workspace_root_for` returned `Some(root)` and only fall back to standalone for `else`; drop the standalone branch in the project arm.
- Or, at least, `self.documents.remove(&uri)` upfront in the project-rooted branch (mirroring `open_document` at line 47) so a leftover standalone is never reused for a project-rooted URI.

### 5. `LspProjectAnalysis::open_document` wipes prior load diagnostics on file-lookup failure — low

analysis_workspace.rs:302-310: when `host.document_file_for_path` returns `Err`, the code calls `self.load_diagnostics.remove(document.uri())`. The comment in the task says "load diagnostics are stored where available". On the upsert-failure branch (line 297) we *insert* diagnostics; on this branch we *delete* whatever was there. If a previous open had stashed real load diagnostics for this URI and a later open got past `upsert_overlay_document` but failed the file-id lookup, the user loses those diagnostics and gets nothing. Either store the (non-existent) diagnostics consistently or leave the prior set in place — the current asymmetry is easy to misread.

### 6. Recovery for projects that opened with `host: None` is implicit — low (pre-existing, surfaced by this patch)

`LspProjectAnalysis::open_document` returns `Err` without touching `open_uris` when `host` is `None` (analysis_workspace.rs:286-288). The workspace's `open_document` returns `true` in that case, triggering `refresh_projects`, which rebuilds because `open_uris` now differs — that's the recovery hook. But for `update_document` against the same broken project, the first failure path also returns `true` and goes through `refresh_projects`, which on the *next* round will see `open_uris == open_uris` and skip the rebuild forever (the rebuild during the previous round already synced the set). A document edited in a project whose host failed to open will never get analysis again in this session, even if the failure was transient (e.g., an invalid file that the user then fixes via overlay edits). Removing the fallback is the architectural intent, but the silent "broken forever" UX is worth either documenting (in `internal_docs/architecture.md`) or addressing with a content-hash-based rebuild trigger.

## Bottom line

No remaining split-brain route in the project-owned path: `open_document`/`update_document` no longer materialize a standalone for a project-rooted URI; the project's own `open_document` removes `files_by_uri` and stores `load_diagnostics` on overlay failure so stale ranges can't be served; `refresh_projects` unconditionally drops standalone entries for grouped URIs. The remaining items above are about *defending* that invariant (#1, #2, #3, #4) and about edge-case behavior after project-level failure (#5, #6), not about a still-open split-brain hole.
