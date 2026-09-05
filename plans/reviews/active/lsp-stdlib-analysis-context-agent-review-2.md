## Review: LSP stdlib analysis-context fix

### Blocking findings

**None.** The wiring is consistent, the base defs are preserved correctly across reloads, and the regression tests cover both single-file and project modes plus three LSP layouts (loose file, `sifr.toml` workspace, `sifr.toml` + `Cargo.toml`). I traced:

- All four `AnalysisHost` entry points (`open_project`, `open_single_file`, `open_project_with_overlays`, `open_single_file_overlay`) now route through the `_with_external_defs` constructors — `implementation.rs:39-53`, `overlay_updates.rs:10-40`.
- `WorkspaceSession` stores the caller-provided `base_external_defs` and the only assignment to `self.external_defs` after construction is the reset at `graph_cache_and_queries.rs:561` (`= self.base_external_defs.clone()`), so reloads and graph-structure invalidations keep stdlib defs.
- `FrontendContext::load_single_file_with_external_defs` / `load_project_with_provider_and_external_defs` seed both `base_external_defs` and `external_defs` with the caller's defs (`graph_cache_and_queries.rs:368-369`, `498-499`), so the very first lowering already sees `sifr.*`.
- `sifr_driver::stdlib_external_defs()` reuses the existing `OnceLock` cache (`cache.rs:8-13` → `bootstrap.rs:19-21`), so failures are propagated deterministically and successes are amortized.
- LSP error handling already treats `AnalysisHost::open_*` failures as load diagnostics (`analysis_workspace.rs:288-297`, `:454-458`), which matches the new doc claim that bootstrap failure surfaces as compiler diagnostics rather than empty context.

### Non-blocking suggestions

1. **Defs cache shape.** `stdlib_external_defs()` (`bootstrap.rs:19-21`) clones the entire cached `StdlibCompiled` (defs + the much larger `StdlibCode`) on every call only to drop `code`. For `AnalysisHost` opens this is a wasted clone; consider memoizing the `defs` separately or returning by `Arc<ExternalDefs>`.

2. **Hot-path reload clones.** `base_external_defs` is cloned at `workspace_session.rs:300, 304, 345, 357` plus the reset path at `graph_cache_and_queries.rs:561`. `ExternalDefs` is ~10 nested `HashMap`s; wrapping it in `Arc<ExternalDefs>` would make session reloads near-free without changing semantics, since the reset path treats it as immutable.

3. **"Cargo-backed" fixture framing.** `write_cargo_backed_sifr_package` (`lsp_protocol_smoke.py:376-416`) adds `Cargo.toml` and `src/lib.rs`, but `workspace_root_for` (`crates/sifr_lsp/src/analysis_workspace.rs:580-590`) only keys off `sifr.toml`. So this fixture and `write_sifr_workspace_manifest` exercise the same LSP code path; the value is the richer manifest (`[exports]`, `[[bin]]`), not the Cargo files. The comment at `lsp_protocol_smoke.py:377-379` could clarify that — otherwise a future reader may assume the LSP looks at `Cargo.toml`.

4. **Trace CLI consistency.** `crates/sifr/src/trace_cli.rs:39,43` still calls `WorkspaceSession::open_project` / `open_single_file` with default empty defs. This is out of the LSP scope of this PR and not a regression, but it's the same shape of bug — a `sifr trace foo.sifr` where `foo.sifr` imports `sifr.random` would carry the old behavior. Worth a follow-up.

5. **Temp-dir leak on failure.** `crates/sifr_analysis/src/host/stdlib_tests.rs:80` only removes the project dir on the success path. A panicking assertion above leaves the temp tree behind. Minor — the same pattern exists in `trace_cli.rs` tests — but a `defer`-style guard would be cleaner.

6. **Doc nit.** `internal_docs/typescript_go_architecture_transfer_workspace_session.md:35-40` says "Default frontend constructors use an empty base for low-level tests" — strictly true, but the default-constructor callers still include `trace_cli.rs` and `WorkspaceSession::project()`/`single_file()` in `workspace_session_tests.rs`. Phrasing it as "callers that don't supply defs (low-level tests, trace CLI) keep the empty base" is more accurate.
