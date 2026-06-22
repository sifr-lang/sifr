I've inspected the patched files, the new unit tests, the static check (including its self-test), and the protocol-stress checks. Findings below, ordered by severity.

## Findings

### 1. (medium) Static check's `if let` boundary is brittle and self-test does not cover the shape the production code now uses

`check_lsp_split_brain.py:80-95` chooses a boundary by string position:

```python
elif project_if_let != -1:
    project_arm_start = project_if_let
    project_arm_end = method_text.find("} else {", project_if_let)
```

The actual production code in `analysis_workspace.rs:62-85` (`update_document`) has a *nested* `} else {` inside the no-project branch (the inner `if let Some(analysis) = self.documents.get_mut(&uri) { … } else { … }`). Today that nested `} else {` appears *after* the outer one in source order, so the boundary lands on the right delimiter — but the invariant the check relies on is "no closer `} else {` appears before the one closing `if let Some(root)`." Any future reformatting or refactor that inserts a `} else {` inside the project arm (e.g., a nested guard) will silently shrink `project_arm` and a fallback `LspDocumentAnalysis::open(document)` placed *after* that inner else but still inside the project arm would slip past the check.

Compounding this, `run_self_test` at `check_lsp_split_brain.py:111-162` only seeds the `match self.projects.get_mut(&root) { … None => true }` shape. The actual code uses `if let Some(project) = self.projects.get_mut(&root)`, so the dominant branch of the detector has no regression-detection self-test. A future edit that breaks the `if let` branch (e.g., removes the `project_if_let` lookup, swaps the boundary string) would not be caught by `--self-test`.

Two cheap hardenings:
- Seed a second self-test fixture using the `if let Some(project) = self.projects.get_mut(&root)` shape with a fallback inside the arm, asserting it is flagged with `"standalone analysis from a project-owned path"`.
- Replace the heuristic boundary with a structural one: scan every `LspDocumentAnalysis::open(document)` site in the file, and assert each appears within an `else { … }` branch of `if let Some(_) = workspace_root_for(...)`. That makes the invariant explicit instead of inferring it from `} else {` positions.

### 2. (low/medium) Workspace tears down the entire project on a single-document overlay failure

`LspAnalysisWorkspace::open_document` / `update_document` (`analysis_workspace.rs:45-85`) now treat any `Err(())` from `LspProjectAnalysis::{open_document,update_document}` as cause to `self.projects.remove(&root)` and force a full `refresh_projects` rebuild. But `LspProjectAnalysis::open_document` (`analysis_workspace.rs:290-323`) returns `Err(())` for three distinct reasons:

1. `host` is `None` (project never opened).
2. `upsert_overlay_document` failed for *this one* document — but the method has already stored `load_diagnostics` for the failed URI, removed it from `files_by_uri`, and added it to `open_uris`.
3. `document_file_for_path` failed after a successful overlay upsert.

For (1), tearing down and rebuilding is the recovery hook (the fix you intended for first-round finding #6). For (2) and (3), the project's other documents are healthy and the per-document diagnostics the project just stored are immediately discarded when the project is removed — `refresh_projects` rebuilds from overlays, losing the granular load diagnostics. If `AnalysisHost::open_project_with_overlays` is intolerant of per-overlay parse errors (i.e., it returns `Err` when *any* overlay fails to load), every transient syntax error during typing will collapse the entire project to `host: None` until the user fixes it — a regression in editing UX driven by a fallback removal that was intended only to enforce single ownership.

Recommendation: distinguish "host is None" failure from "per-document overlay failure" at the workspace boundary. Add a typed error variant or two separate methods on `LspProjectAnalysis` so the workspace only `remove`s the project when the host itself is gone; leave the project intact when only the per-document upsert path failed (state is already recorded inside the project). At minimum, document the rebuild cost in `analysis_workspace.rs` or `internal_docs/architecture.md` if the current intent is "rebuild on every overlay error."

### 3. (low) New Rust unit test does not exercise `file_maps_for_uri`

`project_save_without_version_keeps_project_owner_current` (`session.rs:455-511`) covers `with_document_analysis` (which routes through `with_document` → `LspProjectAnalysis::with_host`) and `has_standalone_document`, but never calls `session.file_maps_for_uri(&uri)` against the shortened source. `file_maps_for_uri` is the other path that consumed stale standalone text in the original symptom (semantic-token ranges resolved against the wrong source). Adding `let maps = session.file_maps_for_uri(&uri).expect(...); assert_eq!(maps.source_for(file)?, shortened)` after the v3 didChange would close the second arm of the bug class at the unit level.

### 4. (low) Self-test assertion only verifies one of the two seeded violations

The seed in `check_lsp_split_brain.py:113-159` contains both a project-arm fallback *and* a `refresh_projects` fallback, but the assertion at line 161 only checks for `"standalone analysis from a project-owned path"`. A regression that disables the `refresh_projects`-specific detector at lines 67-72 would still pass `--self-test`. Add a second assertion: `assert any("refresh_projects creates standalone project fallback" in item for item in found)` (and a separate seed/assertion for the missing-`self.documents.remove(...)` check at line 71-72).

### 5. (low) Stress test broadens range checks but only on two query types

`assert_ranges_within_source` / `assert_semantic_tokens_within_source` at `lsp_protocol_stress.py:263-299` directly close the original "range outside the document" bug class for `foldingRange` and `semanticTokens/full`. They do not exercise any other position-sensitive query (`hover`, `documentSymbol`, `references`, `rename`) against the shortened v3 text. A future regression that re-introduces stale standalone analysis only through a path that bypasses file-maps (e.g., a query that goes straight to `with_document_analysis` and returns positions computed from snapshot text) would be invisible. Cheapest broadening: a `documentSymbol` request against `shortened` asserting every reported range falls within `line_lengths(shortened)`.

## Notes that did not become findings

- `LspAnalysisWorkspace::with_document` (`analysis_workspace.rs:184-203`) falls back to `self.documents` when the project doesn't yet contain the URI. That is only reachable when a document was opened before `sifr.toml` existed and a query lands in the window before the next document event triggers `refresh_projects`. Out of scope for "project-rooted documents," but worth noting if a future task touches manifest-watcher behavior.
- `LspProjectAnalysis::open_document` storing `diagnostics.clone()` for *all* documents in the constructor's `Err` branch (`analysis_workspace.rs:281-285`) is unchanged by this patch and not a split-brain concern.
- The `fallback_calls != 2` count in `project_fallback_violations` correctly pins the two legitimate no-project branches and would fail open if any third site is added; this works as intended.

## Bottom line

No remaining split-brain *route* for project-rooted documents in steady state: `open_document`, `update_document`, and `refresh_projects` all wipe `self.documents[uri]` before/while routing to a project, and `with_document` prefers the project when present. The actionable items above are about (a) the verification harness being narrower than the invariant it claims to enforce (#1, #4, #5), (b) one missing unit-level assertion (#3), and (c) one behavioral concern introduced by removing the fallback (#2). #1 and #2 are worth addressing before merge; the rest are quality-of-detection improvements.
