# TypeScript-Go Architecture Transfer: Editor Corpus And Snapshot Handles

editor-corpus snapshot-handle surface locks representative editor query behavior with marker-based multi-file
fixtures and prepares internal snapshot-scoped handles for future compiler API
work.

## Editor Corpus

`verification/areas/developer_tooling/editor_query_corpus/multi_file` contains checked-in
`.sifr` fixtures with `# @marker` directives. The analysis test harness strips
marker lines into a temporary project, then drives hover, completion,
definition, references, rename, diagnostics, semantic tokens, formatting, code
actions, and stale-snapshot rejection through `AnalysisHost`.

The corpus is intentionally analysis-owned. LSP request tests can reuse the
same query semantics through `AnalysisSnapshot`, but the fixture rules does
not let protocol adapters reimplement semantic lookup.

## Snapshot Handles

`crates/sifr_analysis/src/handles.rs` defines internal-only handles for
symbols, types, signatures, diagnostics, and source spans. Each handle stores
the originating `WorkspaceSnapshotId` plus graph/source `AnalysisRevision`.
Resolving a handle against a different snapshot returns `StaleSnapshot`.

These handles are not exported from `sifr_analysis`; they prepare the compiler
API shape without exposing a public compiler API in this workstream.

## Package Diagnostics

Runtime package fixtures now prove the boundary between source import
ambiguity and fatal package-map errors:

- `package_ambiguous_import_canonical` emits `SIFR-IMPORT-0005` with package
  source-map context and no companion `SIFR-PACKAGE-*`.
- `package_fatal_source_map_no_import_ambiguity` emits `SIFR-PACKAGE-0713`
  and no companion `SIFR-IMPORT-*`.

`verification/areas/developer_tooling/check_diagnostic_source_canonicalization_rules.py`
enforces both non-duplication directions.
