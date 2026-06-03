I've reviewed the actual fixes against pass 2's blockers. The single-root regression that pass 2 reproduced is fixed, but the new `refresh_projects` still has a reachable hole in a multi-root flow.

## NOT SATISFIED

### Blocker — `refresh_projects` still destroys the fallback when `open_uris` already matches

`crates/sifr_lsp/src/analysis_workspace.rs:101-130`:

```rust
for document in documents.documents() {
    if let Some(root) = workspace_root_for(document.path()) {
        self.documents.remove(document.uri());        // unconditionally clobbers fallback
        grouped.entry(root).or_default().push(document);
    }
}
self.projects.retain(|root, _| grouped.contains_key(root));
for (root, documents) in grouped {
    let open_uris = open_uris(&documents);
    if self
        .projects
        .get(&root)
        .is_some_and(|project| project.open_uris == open_uris)
    {
        continue;                                     // does NOT re-insert what the first loop removed
    }
    let analysis = LspProjectAnalysis::open(root.clone(), &documents);
    for document in &documents {
        if analysis.files_by_uri.contains_key(document.uri()) {
            self.documents.remove(document.uri());
        } else {
            let fallback = LspDocumentAnalysis::open(document);
            self.documents.insert(document.uri().to_string(), fallback);
        }
    }
    self.projects.insert(root, analysis);
}
```

After the first refresh that re-opens a project containing an unmapped orphan, `LspProjectAnalysis::open` sets `open_uris = open_uris(documents)` for *all* passed documents — including the orphan whose `files_by_uri` entry was never created. So `project.open_uris` becomes `{main, orphan}` while `files_by_uri = {main}` and `self.documents = {orphan: fallback}`. Any subsequent `refresh_projects` whose grouped documents for that root are still `{main, orphan}` (i.e. nothing changed in this root, but something else fired a refresh) hits the `continue` branch — *after* the first loop already removed `self.documents[orphan_uri]` — and the fallback is gone.

Reachable normal-editor sequence (not covered by the new test):

1. Open `proj1/src/main.sifr` → triggers refresh; `project.open_uris = {main}`.
2. Open `proj1/src/orphan.sifr` (same package, not imported) → fallback inserted; `project.open_uris` unchanged.
3. Open `proj2/src/main.sifr` (different `sifr.toml` root) → `analysis.open_document` returns `true`, refresh fires. proj1 is re-opened with `[main, orphan]`; fallback re-inserted; `proj1.project.open_uris = {main, orphan}`.
4. Close `proj2/src/main.sifr` → `close_document` always refreshes. First loop removes `self.documents[orphan_uri]`. Second loop: `open_uris_new == project.open_uris == {main, orphan}` → `continue`. Fallback is not re-inserted.
5. Query `proj1/src/orphan.sifr` → `analysis is unavailable for .../orphan.sifr`.

`unmapped_project_file_fallback_survives_project_refresh` (`crates/sifr_lsp/src/session.rs:471-541`) only covers the close-entrypoint-then-reopen path, which works because `project.open_uris` is `{main}` at the time of close and the `continue` branch isn't hit. Multi-root + close-of-other-root reaches it and the test doesn't.

**Fix:** make the two loops consistent. The minimal change is to stop clobbering `self.documents` in the first loop and only remove project-managed URIs in the second loop. Equivalently, in the `continue` branch, re-insert a fallback for every `document.uri()` whose `project.files_by_uri` does not contain it. Add a regression test using the 5-step sequence above (two roots, close the second, then query the orphan in the first).

---

### Other blockers — fixed

**Blocker 2 (smoke gate silent skip):** `scripts/run_all_tests.sh:179-182` now runs `git submodule update --init verification/sifr-large-lsp-verification`, the subrepo `generate_corpus.py check` drift gate, and `--mode smoke --require-submodule`. On a clean clone the submodule init fails loudly under `set -e`, and `--require-submodule` turns a missing manifest into FAIL rather than SKIP. ✓

**Blocker 3 (verifier workload):** `verification/tooling/lsp_large_session.py` full mode now has `diagnostic_requests=True`, `storm_burst_size=10`, periodic `textDocument/diagnostic` requests, `edit_shared_api_text` that actually mutates the exported parameter and references, `edit_private_body_text` that mutates the return expression, and storm bursts of real `didChange` notifications before the request mix. The three edit categories are now behaviorally distinct in what they invalidate. Push diagnostics are still off, but pull diagnostics on a 1206-file corpus + storm bursts + signature changes is a meaningful workload for the scheduler/snapshot/invalidation paths. ✓

---

### Should-fix before merge (not standalone blockers, but tracked from pass 2)

- **paths_match still has no doc comment** (`crates/sifr_analysis/src/host/overlay_updates.rs:80-88`). Tests at 90-118 cover both branches, but the invariant ("source-map paths may be relative to project root; uniqueness is guaranteed by one-host-per-project") isn't stated anywhere. The helper silently relies on this.
- **Full mode has no cadence.** `verification/sifr_large_lsp_verification.md` documents it as a manual command. No nightly schedule, no `RUN_HARDENING` hook. Either schedule it or label the section "manual qualification" so the absence is intentional.
- **Verification docs / review files still untracked.** `git status` shows `verification/sifr_large_lsp_verification.md`, `verification/tooling/lsp_large_session.py`, `reviews/large-lsp-long-session-implementation-review-pass-2.md`, `reviews/large-lsp-long-session-implementation-review-pass-3.md`, and `issues/ad-hoc-large-lsp-long-session-verification.md` all untracked. They must be `git add`ed before the commit lands or the PR ships without them.
- **Thresholds are still ~10–100× looser than observed** (peak 256 MiB vs 18.6, p95 1000 ms vs 9.112, slope 64 MiB/min vs −24.95). These were "Important" not "Blocker" in pass 2 and were partially tightened; flagging that they still won't catch a 2–3× regression.

---

### Verdict

**NOT SATISFIED.** One concrete blocker: `refresh_projects`'s `continue`-on-matching-`open_uris` branch leaves the fallback removed by the first loop and unrestored, so multi-root editor flows still destroy unmapped-file fallbacks. The currently landed regression test doesn't cover that path. Fix the loop to be consistent (don't clobber `self.documents` up front, or re-insert in the `continue` branch) and add a multi-root regression test.
