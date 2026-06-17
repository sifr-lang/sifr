# TypeScript-Go Architecture Transfer: Bucketed Indexes And Safe Parallel Lanes

Status: merged in [#2259](https://github.com/sifr-lang/sifr/pull/2259)

bucketed symbol-index surface scales editor symbol/import queries without weakening deterministic compiler
identity.

## Bucketed Index Rules

`sifr_analysis::SymbolIndex` now stores per-module buckets keyed by
`SymbolBucketId`. Buckets carry a `SymbolBucketKind` of workspace, package, or
stdlib, and `SymbolBucketReadiness` reports `SymbolBucketReadinessState`, total
symbol entries, and import-entry counts. Current frontend module graph views do
not carry package or stdlib module identity, so package and stdlib aggregate
buckets are represented explicitly as `Unavailable` rather than being
misclassified from file ids.

`AnalysisHost::update_document` preserves an existing symbol index and refreshes
only `InvalidationReport::invalidated_modules`. Cold queries still build the
full index once for the current graph/source revision. Repeated workspace symbol
queries retain deterministic ordering by file, name, kind, and ordinal.
`AnalysisHost::completion` and `AnalysisHost::workspace_import_symbols` consume
bucketed symbol/import views filtered by available bucket readiness.
`completion_symbols` currently applies the same symbol filtering as
`workspace_symbols` plus bucket readiness; completion-specific ranking still
lives in `sifr_analysis::completion`.

## Worker-Lane Rules

`sifr_analysis::ApprovedWorkerLane` records compiler lanes that may be considered for
future worker execution: parse, source-map creation, independent HIR lower, lint
file rules, formatter checks, and selected diagnostics.

`SingleOwnerCompilerPhase` records compiler stages that remain serialized until a later
capability record proves a stronger ownership model: type identity creation, ownership
mutation, package graph mutation, and codegen state.

bucketed symbol-index surface defines the allowed lane policy only. It does not start background worker
execution; later work must still prove snapshot ownership, cancellation,
and deterministic publication before using these lanes.

## Validation

- `cargo test -p sifr_analysis symbol_index -- --nocapture` -> PASS
- `cargo test -p sifr_analysis worker_lanes -- --nocapture` -> PASS
- `cargo test -p sifr_analysis` -> PASS, 20 tests
- `cargo fmt --check` -> PASS
- `cargo clippy -p sifr_analysis -p sifr_frontend -- -D warnings` -> PASS
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py` -> PASS
- `python3 verification/areas/developer_tooling/check_typescript_go_transfer_guardrails.py --self-test` -> PASS
- `python3 verification/areas/developer_tooling/check_tooling_readiness.py` -> PASS
- `python3 verification/areas/developer_tooling/check_tooling_readiness.py --self-test` -> PASS
- `git diff --check` -> PASS
- `python3 scripts/check_file_size_guardrails.py` -> PASS
- Claude reviewer pass 1 -> CHANGES_REQUESTED
- Claude reviewer pass 2 -> SATISFIED with residual cleanup
- Claude reviewer pass 3 -> SATISFIED
- `scripts/run_all_tests.sh --profile create-pr` -> PASS, report `target/validation_lane_reports/create-pr.latest.json`, wall time 261.03s, advisory: group skew is high
