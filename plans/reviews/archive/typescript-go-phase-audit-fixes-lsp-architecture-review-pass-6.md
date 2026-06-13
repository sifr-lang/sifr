SATISFIED.

**`project_entrypoint` change analysis** (crates/sifr_lsp/src/analysis_workspace.rs:557–566):

The new ordering `[root.join("src/main.sifr"), root.join("main.sifr")]` is a strict, ordered preference list — `is_file()` short‑circuits the loop. Effect on existing fixtures:

- `verification/tooling/lsp_protocol_stress.py:138` (`run_project_cross_file_queries`) writes only `root/main.sifr`. With the new probe, `root/src/main.sifr` does not exist, so the loop falls through to `root/main.sifr` — same resolved entrypoint as before.
- `verification/tooling/lsp_protocol_stress.py:184` (`run_multi_project_workspace_symbols`) writes `alpha/main.sifr` and `beta/main.sifr` (no `src/` layout). Same fallthrough; both projects resolve to their root `main.sifr`.

The added `src/main.sifr` preference is also consistent with the package layout supported by `crates/sifr_driver/src/workspace/tests.rs` (e.g. lines 118, 173, 194, 237, 259), so an LSP session opening a package using the `src/` layout will now pick the canonical entrypoint instead of an arbitrary "main"-stem document or the first overlay. No behavior change for the root‑layout stress fixtures.

**Other checks on the broader diff:** the changes around `LspProjectAnalysis` (open / update_document / workspace_symbols / file_maps) and the `with_document` routing into the project host all remain consistent with the previously approved pass — no regressions introduced.

**Blockers remaining for merge:** none.
