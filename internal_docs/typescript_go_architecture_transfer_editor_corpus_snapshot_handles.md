# TypeScript-Go Architecture Transfer: Editor Corpus And Snapshot Handles

Status: merged via [#2265](https://github.com/sifr-lang/sifr/pull/2265)

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

## Validation

- `cargo test -p sifr_analysis marker_editor_corpus_covers_multifile_queries_and_stale_snapshots -- --nocapture` -> PASS
- `cargo test -p sifr_analysis snapshot_handles_are_internal_and_reject_wrong_snapshot_resolution -- --nocapture` -> PASS
- `cargo test -p sifr_analysis` -> PASS, 23 tests
- `python3 verification/areas/developer_tooling/check_diagnostic_source_canonicalization_rules.py` -> PASS
- `python3 verification/areas/developer_tooling/check_diagnostic_source_canonicalization_rules.py --self-test` -> PASS
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py` -> PASS
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test` -> PASS
- `cargo test -p sifr -- --skip test_e2e_pass` -> PASS, 57 unit tests and 33 non-pass e2e tests
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_analysis -p sifr -- -D warnings` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- `git diff --check` -> PASS
- Claude reviewer pass 1 -> SATISFIED with residual recommendations
- Claude reviewer pass 2 -> SATISFIED
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time 279.93s, advisory: group skew is high
