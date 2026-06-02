I've reviewed the pass 3 blocker fix in detail. Let me verify my trace of the fix matches the pass 3 reproduction scenario:

**Pass 3 blocker (refresh_projects destroys multi-root fallback):**

The pre-fix code in the initial grouping loop unconditionally called `self.documents.remove(document.uri())` for any URI with a workspace root. Then the second loop's `continue` branch (matching `open_uris`) didn't re-insert anything, leaving fallbacks lost.

The current fix (`crates/sifr_lsp/src/analysis_workspace.rs:101-129`):
- Removed the unconditional `self.documents.remove` from the grouping loop (lines 102-107 only collect now).
- Only the rebuild branch touches `self.documents`, and it does so symmetrically: `files_by_uri.contains_key` → remove; otherwise insert fallback.
- The `continue` fast-path now leaves `self.documents` untouched, so orphan fallbacks survive an unrelated-root rebuild.

**Trace of the pass 3 5-step scenario, post-fix:**
1. Open `first_root/main` → refresh; `project.open_uris = {main}`; main moved into project.
2. Open `first_root/orphan` → `project.open_document` calls `upsert_overlay`+`document_file_for_path`; the latter fails (orphan not in source map after `reload`) → returns `Err` before `open_uris.insert` at line 318, so `project.open_uris` stays `{main}`; fallback inserted into `self.documents`.
3. Open `second_root/main` → triggers refresh. For `first_root`, `open_uris_new = {main, orphan}` ≠ `project.open_uris = {main}` → **rebuild**, files_by_uri = {main}, orphan re-inserted as fallback. `project.open_uris` becomes `{main, orphan}`.
4. Close `second_root/main` → refresh. For `first_root`, `open_uris_new = {main, orphan} == project.open_uris` → **continue**. Critically, the first loop no longer removed the orphan fallback, so it survives. ✓
5. Query orphan → fallback found.

**Test `unmapped_project_file_fallback_survives_unrelated_root_refresh`** (session.rs:543-620) replays exactly that sequence and passes.

**Other pass 3 items:**
- `paths_match` invariant comment (lines 84-86) and three-branch tests are present.
- Full mode cadence remains manual (pass 3 labeled this should-fix, not blocker).
- Untracked files still present (`verification/tooling/lsp_large_session.py`, `verification/sifr_large_lsp_verification.md`, review/issue docs). Pass 3 labeled this should-fix; flagging that `scripts/run_all_tests.sh` now invokes `lsp_large_session.py`, so the PR must include that file or the script breaks on landing.

**Validation:**
- `cargo test -q -p sifr_lsp` → 27 pass.
- `cargo clippy -q -p sifr_analysis -p sifr_lsp -- -D warnings` → clean.

## SATISFIED

The pass 3 blocker is fixed: `refresh_projects` no longer destroys unmapped-file fallbacks when an unrelated-root close-refresh hits the matching-`open_uris` fast path. The new regression test reproduces the exact 5-step multi-root sequence pass 3 described.

Reminder before pushing the PR (carried over from pass 3, not a code defect): `git add` `verification/tooling/lsp_large_session.py` and `verification/sifr_large_lsp_verification.md` — the script change in `scripts/run_all_tests.sh` already references the tool.
