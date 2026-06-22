Inspection complete. Here are the actionable findings, ordered by severity.

## 1. Blocking — `session.rs` now exceeds the 900-line file-size cap

`crates/sifr_lsp/src/session.rs` is **967 lines** (was 882 before this patch). The file-size guardrail confirms the failure:

```
file-size guardrails: FAIL
- crates/sifr_lsp/src/session.rs: 967 lines (limit 900, category rust)
```

Per `AGENTS.md`, "Hand-maintained first-party source files must stay under 900 lines" and "Run the file-size guardrail before considering work complete." `scripts/run_all_tests.sh` invokes this check, so the PR will fail the mandatory local-validation gate.

The reported validation list doesn't include `scripts/check_file_size_guardrails.py` or `scripts/run_all_tests.sh`, which is why this slipped through. The patch added three sizable tests (~+85 lines) to a file that was already close to the cap. The fix is to split the tests by responsibility (e.g., move the three project-ownership tests into a sibling test module file such as `session/tests/project_ownership.rs` or `session_tests.rs`, since `Session::analysis` is module-private but a `#[cfg(test)] pub(crate) fn` accessor already exists for the new `has_standalone_document` helper and the rest of the test API is `pub(crate)`).

## 2. Minor — `LspProjectAnalysis::update_document` still returns `Result<(), ()>`

`analysis_workspace.rs:334-351` keeps the untyped failure return even though the patch introduced `ProjectDocumentFailure` specifically to distinguish `HostUnavailable` (drop the project) from `DocumentUnavailable` (keep the project, drop the URI).

It happens to compose correctly today because the caller in `LspAnalysisWorkspace::update_document` recovers via `project.open_document(document)` (which IS typed) — so a missing host still bubbles up as `HostUnavailable` on the second call. But this leaves a latent footgun: a future change that adds a non-recovery path after `project.update_document` would silently treat both failure modes the same. Convert it to return `Result<(), ProjectDocumentFailure>` for symmetry with `open_document`.

## 3. Minor — `update_document` recovery silently re-runs the same overlay on parse-fail

In `LspAnalysisWorkspace::update_document` (lines 73-83), when `project.update_document` fails because `host.update_document` returned diagnostics (parse error on a mapped file), the patch then calls `project.open_document` which calls `host.upsert_overlay_document` with the identical text — almost certainly producing the same diagnostics and discarding the originals on the way through `update_document`. The end state (diagnostics stored, file unmapped) is correct, but it does redundant analyzer work and obscures intent. Consider preserving the typed failure from `update_document` so the caller can either store the original diagnostics directly or skip the redundant upsert when the failure is a parse error rather than a mapping miss.

## 4. Minor — `check_lsp_split_brain.py` self-tests are lexically brittle

The guardrail recognizes only two routing shapes literally: `match self.projects.get_mut(&root)` and `if let Some(project) = self.projects.get_mut(&root)`. Any future refactor — renaming the binding, extracting a helper like `self.project_mut(root)`, switching to `.entry(root).or_insert_with(…)` — emits `"unverifiable project ownership routing"` rather than passing. That's acceptable as a deliberate backstop, but worth documenting (a short comment in the script naming the two accepted shapes and what to update if either changes) so the next maintainer doesn't read a false failure as evidence that fallback returned.

No correctness bugs found in the Rust ownership change itself: removal of `LspDocumentAnalysis::open(document)` from project-routed paths is complete, `refresh_projects` no longer plants a fallback, `file_maps_for_document`/`with_document` consistently return `"analysis is unavailable"` for unmapped project URIs, and the typed `ProjectDocumentFailure` correctly partitions remove-project vs keep-project recovery. The three new session tests assert source state through both `with_document_analysis` and `file_maps_for_uri(...).source_for(file)`, which actually exercises the host's per-file source — not just the document store.
