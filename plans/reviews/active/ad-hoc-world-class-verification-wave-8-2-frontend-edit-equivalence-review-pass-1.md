I have enough context. Let me consolidate my findings.

## Review: Wave 8.2 Frontend Edit-Equivalence

### Blocking findings
**None.** The slice is correctly scoped and the contract is sound.

### What I verified

**Cache-assisted vs. clean comparison is real.** Both tests build a single mutating `FrontendContext`, push it through edits via `update_module_source` (which exercises the actual cache-invalidation paths in `graph_cache_and_queries.rs:498-591`, including drop-in replacement vs. graph-rebuild branches), and compare its outputs to a freshly-loaded context built from the same end-state source. The "clean" side is genuinely fresh — `single_file_context`/`load_project` build new contexts each call (helpers at `query_diagnostics_equivalence_tests.rs:124-135, 152-164`).

**All four named scenarios are covered.**
- Single-file (`query_diagnostics_equivalence_tests.rs:9-42`): success-preserving (lines 22-27), diagnostic-introducing (29-34), diagnostic recovery (36-41).
- Project (`query_diagnostics_equivalence_tests.rs:44-122`): success-preserving helper edit (74-79), diagnostic-introducing api edit (86-91), diagnostic recovery (97-102), project-graph dependency change with explicit edge comparison (111-121).

**Graph evidence is canonical, not temp-path noise.** `edge_summary` (lines 192-211) maps `ModuleId → file stem` via `module_stem` before comparison, then sorts. Diagnostic `spans[].file` is also temp-path-free: `render_span` (`sifr_diagnostics/src/render/mod.rs:259`) uses `source_map.display_path` which is wired to `module_name` (`graph_cache_and_queries.rs:730`), and `module_name` is derived as the file *stem* in `load_project_with_provider` (lines 446-452). So both sides use stems like `"main"`/`"api"`/`"helper"` regardless of the temp parent directory.

**Project IDs line up across contexts.** `load_project_with_provider` sorts files and pins the entrypoint at index 0 (lines 420-423), so `ModuleId` assignment is deterministic from the source set alone. The test uses `module_by_stem` rather than hardcoded IDs, which is robust either way.

**Performance contract runs it.** `check_frontend_cache_rules.py:41-50` adds `query_diagnostics_equivalence_tests` to the existing invalidation/determinism trio.

**Guardrails respected.** Test file is 246 lines (< 900 cap). Module is `#[cfg(test)]`-gated in `lib.rs:22-23`, scoped to the `sifr_frontend` crate it tests, and mirrors the style of the sibling `query_diagnostics_m10_tests.rs`.

### Non-blocking observations (no action required)
- The equivalence test asserts output equality but not that cache reuse *occurred*; a regression that silently disabled the cache would still pass. That gap is covered by `source_update_invalidates_cached_queries` and `single_file_queries_are_cached_and_deterministic`, which the contract already runs alongside — so coverage is complete in aggregate, just split across modules.
- `temp_project_dir` doesn't clean up after itself, matching the existing pattern in `query_diagnostics_m10_tests.rs:211` and `query_diagnostics.rs:728`. Not new tech debt.
- The project assertion message at line 90 says "public API diagnostic edit" rather than the more literal "diagnostic-introducing edit" used in the single-file test. Minor inconsistency, not worth a re-spin.

### Verdict
No blockers, no required follow-ups. No further review round needed before PR.
