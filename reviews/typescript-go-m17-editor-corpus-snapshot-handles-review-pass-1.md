Reviewed the working tree end-to-end. Both runtime contracts behave exactly as M17 requires (verified with live `cargo run sifr check` invocations on both package fixtures).

## SATISFIED

The M17 scope is covered correctly and closeout fit is solid:

**Editor corpus**
- `verification/tooling/editor_query_corpus/multi_file/{main,helper}.sifr` provide a marker-based multi-file fixture.
- `crates/sifr_analysis/src/host/m17_tests.rs:118` exercises every M17 query family (hover, completion, definition, references, rename, semantic_tokens, formatting, code_actions, diagnostics) via `AnalysisHost`, then verifies stale-snapshot rejection through `snapshot.hover` after a `host.update_document` round-trip.

**Snapshot handles are properly internal**
- `crates/sifr_analysis/src/handles.rs:1` declares `#![allow(dead_code)]` and every type/method is `pub(crate)`.
- `crates/sifr_analysis/src/lib.rs:10` only `mod handles;` — no `pub use handles::*;`. No symbol from `handles` leaks via `cargo doc` or downstream crates.
- `ensure_handle_current` checks both `WorkspaceSnapshotId` and `AnalysisRevision { graph, source }`, and `snapshot_handles_are_internal_and_reject_wrong_snapshot_resolution` (`m17_tests.rs:236`) verifies all five handle kinds plus snapshot-level `hover` reject the prior snapshot after an update.

**Package diagnostic non-duplication**
- Ran `sifr check` on both fixtures live:
  - `package_ambiguous_import_canonical` emits only `SIFR-IMPORT-0005` with `candidate_paths`, `written_module_path`, `package_import_origin`, `resolution_scope=package`, and primary span on `helper` in `from helper import value`. No `SIFR-PACKAGE-*`.
  - `package_fatal_source_map_no_import_ambiguity` emits only `SIFR-PACKAGE-0713` with `origin_kind=sifr_manifest`, `manifest_key=__init__.sifr`, `manifest_path`. No `SIFR-IMPORT-*`.
- `check_diagnostic_source_canonicalization_contract.py:347,374` adds `forbidden_prefixes` to both directions, and the M1 guardrails (`check_typescript_go_m1_guardrails.py:524`) pin both fixture names and both forbidden-prefix asserts into the contract file.

**Doc/tracker fit**
- `internal_docs/architecture.md` M17 line added; `internal_docs/frontend_query_architecture.md` M17 note added; `internal_docs/typescript_go_architecture_transfer_m1_guardrails.md` adds the M17 caveat plus future-obligation row; new `internal_docs/typescript_go_architecture_transfer_m17_editor_corpus_snapshot_handles.md` records corpus/handle/package-diagnostic contract.
- `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md` tracker has M17 row plus full local-validation log.

## Residual (non-blocking)

These are low-priority cleanup, not closeout obstacles:

1. **`helper_export` marker is documented but unexercised.** `helper.sifr:2` declares `queries=definition,references,semantic_tokens` but `m17_tests.rs` only resolves `value_binding` and `stale_return`. None of the asserted queries cross into helper symbols, so the "multi-file" aspect is shallow: the corpus contains two files, but every asserted lookup targets `main.sifr` locals. Consider asserting `definition`/`references` on `helper_value` to actually traverse the file boundary.
2. **`SnapshotHandleAnchor::kind` is dead at runtime.** `handles.rs:21` stores `kind`, but `ensure_handle_current` (`handles.rs:154`) only compares `snapshot_id` and `revision`. Since each handle type has its own `resolve_*` entry point, kind never disambiguates anything. Either drop the field or have the resolver assert the expected kind.
3. **Temp-dir cleanup is best-effort.** `m17_tests.rs:232` only runs `remove_dir_all` on the happy path; a panicking assertion leaves `/tmp/sifr_analysis_m17_…` behind. A `defer`-style guard or `tempfile::TempDir` would tidy this.
