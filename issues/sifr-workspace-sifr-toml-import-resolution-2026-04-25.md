# Sifr Workspace Resolution Via `sifr.toml`

Status: open
Owner: tbd
Created: 2026-04-25
Related: `internal_docs/leetcode_fixture_helper_convention.md`, `issues/leetcode-ws6-silent-fallback-remediation-2026-04-25.md`

## Purpose

Give the Sifr compiler a project-root concept so user code can share modules across directories without renaming entries to `main.sifr` or duplicating files. The trigger is a native `sifr.toml`: when the compiler walks up from the entry file and finds one, the directory becomes a workspace root and absolute-style imports resolve from there in addition to the entry's parent directory.

This unlocks shared `audits/leetcode/helpers/` modules without touching the global `sifr.*` stdlib registry, and is the foundation for any future user library or monorepo layout.

## Problem Definition

Today the compiler resolves user imports in exactly one way:

- `crates/sifr/src/main.rs::resolve_compilation_mode` enters project mode only when the entry file's stem is `main` AND the entry contains at least one local sibling import.
- `crates/sifr/src/main.rs::has_local_project_imports` checks each `from x import ...` for a sibling `x.sifr` next to the entry file.
- `crates/sifr_driver/src/project/discovery.rs::module_source_path` resolves every dependency module relative to the entry's parent directory.
- `crates/sifr_driver/src/stdlib/registry.rs` and `crates/sifr_driver/src/stdlib/bootstrap.rs` intercept any `sifr.*` / `_sifr.*` import via an embedded `STDLIB_FILES` table. This is the only namespace that can be imported from anywhere on the filesystem.

Consequences:

- A non-`main.sifr` entry, such as every LeetCode fixture under `audits/leetcode/`, is forced into single-file mode and cannot import any user module.
- Even when an entry is `main.sifr`, only direct siblings resolve. There is no concept of a project root, source roots, or absolute user imports.
- The only way to share a module today is to add it to the `sifr.*` stdlib registry, which compiles it into the production binary and conflates fixture/library helpers with language stdlib (a category error).

Concrete blockers this creates:

- `internal_docs/leetcode_fixture_helper_convention.md` accepted self-contained inline boilerplate as a workaround. Roughly 20+ LeetCode fixtures duplicate `ListNode` / `TreeNode` definitions and value/next accessor helpers because no shared import path is available.
- The trie helper pressure exposed the same boundary: a fixture-oriented helper could not live behind a workspace-rooted import path, and promoting it into the language stdlib was rejected. Without workspace imports, such helpers must remain inline or duplicated.
- Any future user library (e.g., a logging facade for examples, a math helper crate for demos) has the same dead end.

## Goals

- Define a workspace root as the nearest ancestor containing `sifr.toml`.
- Resolve user imports first relative to the entry's parent directory (current behavior, backward compatible), then relative to one or more declared source roots inside the workspace.
- Keep the embedded `sifr.*` / `_sifr.*` stdlib registry as the highest-priority resolver. Stdlib lookups never touch the filesystem.
- Enter project mode automatically when a workspace root is found, regardless of entry filename.
- Produce clear diagnostics for malformed `sifr.toml`, ambiguous module names, and unresolved imports.

## Non-Goals

- No package directories or `__init__.sifr`. Flat module files only in this slice.
- No dependency manager, registry, lockfile, or external package fetching.
- No new relative-import semantics. The existing `level <= 1` rule stays.
- No change to the `sifr.*` resolver order or its registry contents.
- No multi-workspace or nested-workspace semantics. The nearest `sifr.toml` wins; further ancestors are ignored.
- No re-exports, namespace packages, or wildcard imports.
- No build script, plugin, or pre-processor hooks in `sifr.toml`.
- No `sifr test` workspace-awareness in this slice; build/run/check/emit are the supported workspace-aware commands, and test-command parity is deferred.

## Suggested Solution

### Workspace Discovery

Add `crates/sifr_driver/src/workspace/` (new module) with:

- `find_workspace_root(entry: &Path) -> Option<WorkspaceRoot>`: walk parents of `entry` until the filesystem root, return the first directory containing a parseable `sifr.toml`. Stop at the first match (nearest workspace wins).
- `WorkspaceRoot { dir: PathBuf, config: SifrWorkspaceConfig }`.
- `SifrWorkspaceConfig { source_roots: Vec<PathBuf>, package_name: Option<String> }`.

`sifr.toml` shape:

```toml
[package]
# Optional human-readable workspace/package name. No functional effect in this slice.
name = "sifr-monorepo"
version = "0.0.0"
edition = "2026"

[source]
# Optional list of directories, relative to the workspace root, from which
# absolute-style user imports resolve. Default: ["."].
# Order matters: earlier entries take precedence on ambiguity.
roots = ["audits", "lib", "examples"]
```

Validation rules:

- `package.name`, when present, must be a string. Otherwise: parse error.
- `source.roots`, when present, must be a list of strings. Each entry must be a relative path that does not escape the workspace root via `..`. Each entry must resolve to an existing directory at discovery time. Missing or non-directory entries are a hard error.
- Unknown top-level tables and unknown nested keys are accepted and ignored in this slice so reserved Cargo-inspired tables remain forward-compatible.
- A parseable empty `sifr.toml` is valid and behaves as if `source.roots = ["."]`.

### Resolver Order

For an import `from <module_name> import ...` where `<module_name>` is not `typing` / `enum` / `sifr.*` / `_sifr.*`:

1. Stdlib registry (`crates/sifr_driver/src/stdlib/registry.rs`). Unchanged.
2. Entry-sibling: `<entry.parent>/<dotted_to_path(module_name)>.sifr`. Backward compatible.
3. Workspace sources, in declaration order: `<workspace_root>/<source>/<dotted_to_path(module_name)>.sifr` for each source in `[source].roots`.

Ambiguity rule: if the same module name resolves under more than one workspace source, fail with a diagnostic that lists every match. Entry-sibling resolution always wins over workspace resolution and never participates in ambiguity reporting (this preserves the current behavior of fixtures that already work).

`dotted_to_path` rule: dots in the module name become path separators. `helpers.list_node` -> `helpers/list_node.sifr`. No package directories, no `__init__.sifr`.

### Compilation Mode

Update `crates/sifr/src/main.rs::resolve_compilation_mode`:

- If `find_workspace_root(entry)` returns a workspace, enter `Project` mode unconditionally.
- Otherwise keep the current `main.sifr` + local-imports heuristic for backward compatibility.

The `main.sifr` heuristic stays for users who do not yet have a `sifr.toml`.

### Module Discovery

Update `crates/sifr_driver/src/project/discovery.rs`:

- Replace `module_source_path(project_dir, module_name)` with a `ModuleResolver` that holds an ordered list of search roots:
  1. entry parent directory
  2. each `<workspace_root>/<source>` from the workspace config (when present)
- `ModuleResolver::resolve(module_name) -> Result<ResolvedModule, ResolutionError>`
- `ResolutionError` carries the module name and the list of paths that were tried, so diagnostics can render each tried location.
- `parse_import_closure_modules` consumes a `ModuleResolver` instead of a single `project_dir`.

### Frontend / Driver Wiring

- `crates/sifr_driver/src/build/entrypoint.rs`: extend `RootedEntrypoint` and `RootedEntrypointPlan` with a workspace-aware constructor. The plan owns both the entry path and the optional `WorkspaceRoot`.
- `crates/sifr_driver/src/build/api.rs`: `build_project`, `check_project`, etc. accept the workspace silently (constructed inside the driver from the entry path).
- Cache key (`crates/sifr_driver/src/build/materialize.rs`): include workspace-resolved module paths so a cache hit cannot serve stale results when a workspace source changes content.

### Diagnostics

Add user-facing errors with stable wording and tested phrasing:

- Workspace discovery
  - "could not parse sifr.toml at `<path>`: <reason>"
  - "[source].roots entry `<value>` is not a directory under the workspace root"
  - "[source].roots entry `<value>` escapes the workspace root via `..`"
- Module resolution
  - "could not resolve import '<module>'; tried entry-relative `<path>` and workspace-relative `<path1>`, `<path2>`, ..."
  - "module '<module>' is ambiguous in workspace `<root>`: matches `<path1>` and `<path2>`; reorder `[source].roots` or rename one module to disambiguate"

A user must always be able to read the diagnostic and know exactly where the compiler looked.

Canonical diagnostic codes and documentation URLs are owned by `issues/ad-hoc-sifr-workspace-sifr-toml-import-resolution-2026-04-25.md` WS3.

### Backward Compatibility

- No `sifr.toml` anywhere in the parent chain: behavior is identical to today.
- A malformed `sifr.toml` is a hard diagnostic; a parseable empty `sifr.toml` is a valid workspace with default roots.
- An existing `main.sifr` project that does not declare a workspace: continues to work via the local-imports heuristic.
- Existing LeetCode fixtures that compile in single-file mode keep compiling in single-file mode unless and until a workspace `sifr.toml` is added.

### Test Plan

Unit tests in `crates/sifr_driver/src/workspace/tests.rs`:

- discovers the nearest `sifr.toml`
- defaults empty `sifr.toml` to `source.roots = ["."]`
- rejects malformed sifr.toml with a precise diagnostic
- rejects sources that escape the workspace root
- rejects sources that point at a non-directory
- defaults `source.roots` to `["."]` when omitted
- prefers the nearest workspace when multiple ancestors qualify

Unit tests in `crates/sifr_driver/src/project/discovery.rs`:

- entry-sibling resolution still wins over workspace resolution
- workspace sources resolve in declaration order
- ambiguity across workspace sources produces a diagnostic listing every match
- unresolved import diagnostic lists every tried path

Verification-suite tests under `crates/sifr/tests/verification/project/` and `verification/suites/manifest.json`:

- `pass/workspace_rooted_user_import.sifr` plus a workspace fixture tree:
  ```
  sifr.toml
  audits/leetcode/0021_merge_two_sorted_lists.sifr
  helpers/list_node.sifr
  ```
  fixture imports `from helpers.list_node import ListNode` and runs.
  This synthetic fixture intentionally places `helpers/list_node.sifr` at the workspace root; the real LeetCode pilot places the helper under `audits/leetcode/helpers/list_node.sifr`.
- `fail/workspace_rooted_ambiguous_import.sifr`: same module name resolvable under two source roots, expects diagnostic.
- `fail/workspace_malformed_sifr.toml.sifr`: malformed TOML, expects parse-error diagnostic.

Snapshot/insta coverage: stable diagnostic text for every new error.

LeetCode pilot:

- Add `audits/leetcode/helpers/list_node.sifr` with the canonical `ListNode` plus shared accessors.
- Add `sifr.toml` at the repo root with `[package]` metadata and `[source] roots = ["audits/leetcode", "."]`.
- Migrate one fixture, suggested `0021_merge_two_sorted_lists.sifr`, to import `from helpers.list_node import ListNode`.
- Confirm the pair scan moves expected lines out of the fixture and into the helper file.

Required gates:

- `scripts/run_all_tests.sh --profile quick`
- `scripts/run_all_tests.sh` before merge
- `cargo fmt --check`
- `cargo clippy --workspace -- -D warnings`
- `python3 scripts/check_hir_maintainability_guardrails.py`
- Full LeetCode corpus rerun with no new `CHECK_ERROR`, `RUN_ERROR`, or `TIMEOUT`.

## Implementation Steps

1. Add a `toml` crate dependency to the workspace `Cargo.toml`. The `toml_edit` and `toml_parser` crates already appear in `Cargo.lock` transitively, so adoption cost is minimal. Prefer the lighter `toml` crate for read-only parsing.
2. Land `crates/sifr_driver/src/workspace/` with `find_workspace_root`, `SifrWorkspaceConfig`, parsing, validation, and unit tests. No resolver wiring yet.
3. Refactor `crates/sifr_driver/src/project/discovery.rs` to take a `ModuleResolver` rather than a `project_dir`. Keep the resolver behavior identical (single search root) so the refactor is no-op.
4. Plumb `WorkspaceRoot` through `crates/sifr_driver/src/build/entrypoint.rs` and `crates/sifr_driver/src/build/api.rs`. Single-file callers continue to pass no workspace; project callers may pass a discovered workspace.
5. Update `crates/sifr/src/main.rs::resolve_compilation_mode` and `has_local_project_imports` to consult the workspace when discovered. Project mode triggers either via the `main.sifr` heuristic or via a workspace.
6. Extend `ModuleResolver` to consult workspace sources in declaration order with the ambiguity rule.
7. Add unit tests for the resolver, e2e tests for both pass and fail flows, and snapshots for every new diagnostic.
8. Add `internal_docs/sifr_workspace_design.md` mirroring this issue's resolved design (one-pager, links back to this issue).
9. Update `internal_docs/architecture.md` to mention workspace discovery in the pipeline overview.
10. LeetCode pilot in a separate PR: add the workspace `sifr.toml`, extract `helpers/list_node.sifr`, migrate `0021_merge_two_sorted_lists.sifr`, regenerate the pair scan and full corpus run.

## Open Questions

- Should `sifr.toml` allow disabling stdlib registry intercept? Default is no; the registry always wins. Decision: out of scope for this slice.
- Should the resolver accept package directories (`helpers/list_node/__init__.sifr`)? Decision: no, deferred.
- Should there be a CLI flag to override the discovered workspace? Useful for tests, harmful for reproducibility. Decision: defer; tests can synthesize a tempdir layout.
- Should additional `sifr.toml` package fields beyond `[package].name` be read at all? Decision: parse and reserve them, but only `[source].roots` has semantic effect in this slice.

## Risks

- Cache invalidation: if cache keys do not include workspace-resolved paths, a content change in a workspace helper can serve a stale binary. Tests must cover this explicitly.
- Diagnostic noise: a malformed native `sifr.toml` in a parent directory breaks Sifr invocations below it. Mitigation: this is acceptable because `sifr.toml` is Sifr-owned configuration, unlike a shared `pyproject.toml`.
- Surprise project mode: adding `sifr.toml` activates project mode. Mitigation: this is explicit because `sifr.toml` is Sifr-owned, and no `sifr.toml` keeps legacy behavior unchanged.

## Exit Criteria

- `sifr.toml` workspace discovery is implemented, tested, and documented.
- Module resolver consults entry-sibling first, then declared workspace sources, with diagnostics for misses and ambiguity.
- Stdlib `sifr.*` resolution is unchanged.
- A LeetCode pilot fixture imports a workspace-relative helper and runs successfully.
- Full LeetCode corpus run remains free of `CHECK_ERROR`, `RUN_ERROR`, and `TIMEOUT`.
- No regression for existing single-file or `main.sifr`-project entries with no `sifr.toml`.
- `scripts/run_all_tests.sh --profile quick` and `scripts/run_all_tests.sh` pass.
- `internal_docs/sifr_workspace_design.md` and `internal_docs/architecture.md` are updated.

## Required Artifacts

- New module: `crates/sifr_driver/src/workspace/`
- Extended: `crates/sifr_driver/src/project/discovery.rs`, `crates/sifr_driver/src/build/entrypoint.rs`, `crates/sifr_driver/src/build/api.rs`, `crates/sifr/src/main.rs`
- New design note: `internal_docs/sifr_workspace_design.md`
- Updated: `internal_docs/architecture.md`
- Pilot artifacts (separate PR): `sifr.toml`, `audits/leetcode/helpers/list_node.sifr`, migrated `audits/leetcode/0021_merge_two_sorted_lists.sifr`, regenerated `verification/leetcode/leetcode_pair_diff_scan_<YYYYMMDD>.json`, regenerated `verification/leetcode/full_corpus_current_results_<YYYYMMDD>_workspace_pilot.json`.
