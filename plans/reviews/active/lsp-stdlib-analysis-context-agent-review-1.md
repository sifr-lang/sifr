Review complete — analyzed the diff across the 11 changed files plus the new `host/stdlib_tests.rs` and traced the data flow through reload, signature-change invalidation, and overlay paths.

## Findings, by severity

**Blocking: none.** The architecture is sound:
- `WorkspaceSession` owns `base_external_defs` (sifr_frontend/src/workspace_session.rs:216) and clones it into `FrontendContext` on every reload (workspace_session.rs:345-360).
- `FrontendContext` carries its own `base_external_defs` and, on signature-affecting edits, resets `external_defs = base_external_defs.clone()` then re-extends from lowered modules (graph_cache_and_queries.rs:561-562, 890-896). Stdlib defs survive both project reloads and partial invalidations.
- All four AnalysisHost entry points pass `sifr_driver::stdlib_external_defs()` (host/implementation.rs:40,48 and host/overlay_updates.rs:14,32) — no LSP path bypasses the seed.
- Crate dependency graph is unchanged: `sifr_analysis` already depends on `sifr_driver`, so the new `pub use stdlib::external_defs as stdlib_external_defs` (sifr_driver/src/lib.rs:34) is a pure re-export with no layering change.
- `compile_stdlib()` is `OnceLock`-cached, so `stdlib_external_defs()` is one compile + one ExternalDefs clone per host open.

**Non-blocking suggestions:**

1. **TYPE-0002 proxy is fragile.** `host/stdlib_tests.rs:38-41` and `lsp_protocol_smoke.py:350-353` use the presence of `SIFR-TYPE-0002` as positive evidence that the import resolved. That works only because `sifr.random.randint` returns `Result[int, ValueError]` (lib/sifr/random.sifr:349) and the sample assigns it to `int`. If that signature is ever loosened to return bare `int`, the test will fail not because of a regression but because the deliberate mismatch evaporated. Consider asserting "no IMPORT/NAME diagnostic at the import or call site" plus a positive symbol-resolution probe (e.g. hover/definition on `randint`) rather than depending on a TYPE diagnostic landing.

2. **Double clone per reload.** `WorkspaceSession::reload` clones `base_external_defs` (workspace_session.rs:348), then `FrontendContext::load_project_with_provider_and_external_defs` clones the value again into its own `base_external_defs` (graph_cache_and_queries.rs:498-499). For ExternalDefs containing the full stdlib BTreeMaps this is two O(n) copies per reload. Wrapping `base_external_defs` in `Arc<ExternalDefs>` (with copy-on-write at the signature-invalidation point) would halve allocation cost per edit cycle without changing semantics.

3. **Error surface change.** Previously `AnalysisHost::open_*` could succeed when stdlib was broken; now they propagate any stdlib compile failure (host/implementation.rs:42, 50). Matches CLI and is arguably more correct, but is a behavior change worth a sentence in `internal_docs/lsp_server.md` ("LSP will surface stdlib compile errors at host open").

4. **No project-mode Rust unit test.** `stdlib_tests.rs` exercises only `open_single_file`. The project path is covered transitively by the Python smoke, but a `cfg(test)` fixture using a tempdir + ProjectRoot would catch project-specific regressions (e.g., wiring `with_external_defs` into `project_with_external_defs`) without requiring the LSP harness. Cheap to add.

5. **Verification doesn't replicate the scribbles repro path.** The three covered contexts (standalone, `sifr.toml` workspace, Cargo-backed package) are functionally equivalent to the original bug, but the explicit scribbles `SIFR_LSP_COMMAND` check is recorded only as ad-hoc validation, not as a committed test. The standalone tempdir case is the right surrogate; just noting that the literal repro is not in the suite.

6. **`stdlib/mod.rs` now mixes wiring with logic.** `external_defs()` (sifr_driver/src/stdlib/mod.rs:16-18) lives in mod.rs alongside `mod`/`pub use` lines. Style-only — moving the helper next to `compile_stdlib` in `bootstrap.rs` and re-exporting through mod.rs would keep mod.rs as wiring.

7. **Manifest schema in smoke uses forbidden production keys.** `write_cargo_backed_sifr_package` emits `[exports]` and `[[bin]]` (lsp_protocol_smoke.py:386-391), which `check_package_manager_guardrails.py:329-332` rejects for production manifests. This is allowed for a synthetic test fixture, and validation passed — but if the package-management guardrail ever expands to scan verification fixtures, this would trip. Worth a one-line comment in the smoke noting the fixture is intentionally non-production.
