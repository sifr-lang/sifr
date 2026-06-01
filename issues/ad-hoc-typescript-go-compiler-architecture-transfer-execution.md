# Ad Hoc Phase Execution: TypeScript-Go Compiler Architecture Transfer

Status: planned on 2026-05-27

Phase contract: `issues/ad-hoc-typescript-go-compiler-architecture-transfer.md`

## Checklist

- [x] Phase plan drafted
- [x] Phase plan reviewed and approved for implementation
- [x] M0 source and position foundation completed
- [x] M1 architecture contract and guardrails completed
- [x] M2 source provider and overlay store completed
- [x] M3 workspace session data model completed
- [x] M4 analysis snapshot migration completed
- [ ] M5 LSP persistent session integration completed
- [ ] M6 event compaction and dirty scope completed
- [ ] M7 module signatures and dependency invalidation completed
- [ ] M8 first-class flow graph completed
- [ ] M9 fingerprints and cache keys completed
- [ ] M10 snapshot reuse and structural replacement completed
- [ ] M11 LSP scheduler queues completed
- [ ] M12 per-request editor latency budgets completed
- [ ] M13 LSP cancellation, progress, and watchdog completed
- [ ] M14 bucketed indexes and safe parallel lanes completed
- [ ] M15 project residency, watchers, and build info completed
- [ ] M16 trace and status surfaces completed
- [ ] M17 editor corpus and snapshot handles completed
- [ ] Full local validation recorded
- [ ] Final production-readiness review approved

## Planning Lock Addendum

This phase locks the TypeScript-Go-derived architecture transfer before implementation starts. Changing the adopted concept matrix, milestone order, or public/private API boundary requires a reviewed planning update.

### Required Implementation Work

| ID | Work item | Required closeout |
| --- | --- | --- |
| W-0 | Prerequisite discovery found blockers that must be closed before snapshot/session implementation: source-map stubs, split source text/line-map authority, direct filesystem reads, revision-only snapshots, request-local LSP analysis, shallow scheduler/cancellation, and aggregate-only LSP latency evidence. | M0/M1 complete source-map correctness, shared conversion authority, direct-read inventory, actual-vs-target docs, minimum snapshot state shape, and reviewed implementation guardrails. |
| W-1 | Analysis state is not yet owned by a coherent workspace/session snapshot layer. | M3 introduces `WorkspaceSession`; M4 migrates analysis queries onto immutable `WorkspaceSnapshot` handles. |
| W-2 | Open editor buffers, disk files, package files, and failed module lookups are not modeled by a single tracked VFS boundary. | M2 routes frontend/project reads through a typed source-provider abstraction with overlays and tracked dependencies. |
| W-3 | Parse and HIR reuse is not yet a ref-counted snapshot-scoped service. | M9 locks content/options/package-aware cache keys; M10 adds lifetime-safe cache entries. |
| W-4 | Source edits currently trigger broad invalidation instead of dependency-sensitive invalidation. | M7 adds signatures, reverse dependencies, and local-vs-public invalidation rules. |
| W-5 | Dirty work is not classified precisely enough to choose safe reuse. | M6 implements `DirtyScope` with no-op, one-module, reverse-dependency, graph-structure, config/project, and workspace invalidation. |
| W-6 | There is no structural module replacement predicate for stable private edits. | M10 adds `can_replace_module_in_project` and preserves project state only when module signatures and structural inputs remain stable. |
| W-7 | Immutable snapshots would be too expensive without copy-on-write project state. | M10 adds copy-on-write maps for module graph, source maps, diagnostics, indexes, config entries, and package metadata. |
| W-8 | LSP requests can observe detached request-local analysis rather than a captured workspace snapshot. | M5 threads snapshot identity through analysis-backed LSP requests and rejects stale results. |
| W-9 | The scheduler has lanes but not real priority, cancellation, debounce, progress, or stale-result behavior. | M11 adds priority queues and debounce; M13 adds cancellation tokens, delayed progress, watchdog, and deterministic publication. |
| W-10 | Expensive analysis work is not yet parallelized behind deterministic and identity-safe boundaries. | M14 adds approved worker lanes, analyzer affinity where safe, and deterministic diagnostic ordering. |
| W-11 | Completion/import indexes are not bucketed by workspace, package, and stdlib scope. | M14 adds independently dirty/readiness-tracked index buckets. |
| W-12 | Narrowing and ownership facts do not yet share a first-class flow graph substrate. | M8 introduces explicit flow nodes/edges and migrates Option/None, mutation, and ownership invalidation facts onto graph-backed queries. |
| W-13 | Trace output and status/debug surfaces are not strong enough for debugging snapshot/cache/editor behavior after source maps are fixed. | M16 completes deterministic compiler-service tracing and status/debug output. |
| W-14 | Editor regression coverage lacks a marker-rich multi-file corpus for stale snapshots and protocol queries. | M17 adds fourslash-inspired fixtures and internal snapshot-scoped handle checks. |
| W-15 | Project/config/watch retention is not explicit enough for long-lived sessions. | M15 adds project residency, config reverse retention, reference-counted watcher registries, and seen-file-derived watch globs. |
| W-16 | CLI/build mode has no persistent incremental metadata design. | M15 adds `.sifrbuildinfo` or equivalent non-authoritative build metadata with verification against current fingerprints. |
| W-17 | Raw file/editor events can cause redundant invalidation work. | M6/M15 compact open/change/save/close/watch events and define event-storm degradation. |
| W-18 | LSP operational robustness lacks a parent-process watchdog and shared internal API sessions. | M13 adds `--parent-pid` watchdog support; M17 prepares internal API sessions without exposing a public compiler API. |
| W-19 | Editor latency budgets exist in Phase 35/36 planning, but enforced LSP coverage is still aggregate. | M12 keeps Phase 35 frontend budgets, keeps aggregate LSP as smoke coverage, and adds enforced per-request LSP budget cases. |
| W-20 | Package ambiguous imports can currently fail as package/source-map construction diagnostics before canonical source import diagnostics are possible. | M2 preserves package import ambiguity as queryable source-map state when the package map is otherwise valid; M17 adds runtime fixtures proving `SIFR-IMPORT-0005` with package context. |

### Locked Concept Decisions

| Area | Locked decision |
| --- | --- |
| Prerequisite closure | Complete M0 before implementing session/snapshot work that depends on source maps, overlay source ownership, dirty scopes, or scheduler semantics. |
| Source maps | Treat complete source maps as a hard prerequisite, not late debuggability work. |
| TypeScript-Go snapshots | Adopt the coherent snapshot model, not TypeScript semantics. |
| VFS and overlays | Adopt a Sifr-owned tracked file-system boundary. |
| Parse/HIR caches | Adopt ref-counted, content/options-keyed reuse. |
| Invalidation | Replace broad invalidation with dependency and export-signature invalidation. |
| Dirty scopes | Model dirty work explicitly before choosing reuse or invalidation. |
| Structural replacement | Clone project state only when one-module edits preserve project structure and public interface. |
| Copy-on-write snapshots | Reuse unchanged maps/indexes by identity during snapshot finalization. |
| Event compaction | Compact editor/watch events before dirty-scope classification. |
| Checker pools | Adapt only behind deterministic identity-safe worker boundaries. |
| Bucketed indexes | Split symbol/import indexes by workspace, package, and stdlib buckets with readiness states. |
| Project residency | Retain only projects/configs/watchers needed by open files, references, or explicit API sessions. |
| Build metadata | Persistent build info is a cold-start accelerator, never correctness authority. |
| Flow graph | Adopt first-class flow nodes for Sifr narrowing and ownership. |
| Tracing | Adopt deterministic compiler-service tracing. |
| Status/debug | Expose local project/cache/index/memory state for debugging without telemetry export. |
| Editor budgets | Enforce protocol-level editor latency per request family, not only through `perf.lsp.request_families`. |
| Package diagnostics | Source import ambiguity owns `SIFR-IMPORT-0005`; package construction owns only package-invalidity diagnostics. |
| Compiler API handles | Prepare internally only; public API exposure is future scope. |

## Review Log

- `2026-05-27`: Initial phase plan drafted from local TypeScript-Go codebase review and current Sifr frontend/LSP/cache architecture.
- `2026-05-27`: Reviewer addendum incorporated. The phase now explicitly includes structural module replacement, precise dirty scopes, copy-on-write snapshots, event compaction, project residency, config/watch registries, bucketed symbol/import indexes, non-authoritative build metadata, parent-process watchdog support, delayed progress, and status/debug reporting.
- `2026-05-27`: Performance-budget addendum incorporated. The phase now explicitly requires splitting the current aggregate LSP request-family benchmark into enforced per-feature editor latency budgets while retaining Phase 35 frontend-query budgets as foundation coverage.
- `2026-05-29`: Full prerequisite discovery completed and recorded in `reviews/ad-hoc-typescript-go-prerequisite-discovery.md`. The phase now has M0 prerequisite closure, treating source-map completion, shared conversion authority, direct-read inventory, actual-vs-target docs, minimum snapshot shape, and serialized pre-scheduler execution as implementation readiness blockers.
- `2026-05-29`: Claude review recorded in `reviews/ad-hoc-typescript-go-prerequisite-discovery-claude-review.md`. Feedback incorporated by adding explicit no-match evidence for dirty-scope/snapshot types, direct filesystem read file/line examples, stronger scheduler ordering, conditional event-compaction wording, and symbol-range dependency on M0 source maps.
- `2026-05-29`: Claude architecture-decision review rounds recorded in `reviews/ad-hoc-typescript-go-architecture-decisions-claude-review.md`. Feedback incorporated by locking `sifr_source`, `DirtyReason`, dirty-scope priority, overlay transfer, cancellation token shape, flow graph shape, cache fingerprints, and source migration sequence.
- `2026-05-29`: Claude milestone-slicing review rounds recorded in `reviews/ad-hoc-typescript-go-milestone-slicing-claude-review.md`. Feedback incorporated by splitting session/snapshot/LSP work, scheduler/cancellation work, cache-key/reuse work, adding explicit dependencies, moving flow graph earlier, and clarifying trace/status as a normalization milestone.
- `2026-05-30`: Package ambiguous-import source-map consideration incorporated. The phase now locks the boundary between fatal package/source-map diagnostics and canonical import-site `SIFR-IMPORT-0005` diagnostics.
- `2026-05-30`: Claude package source-map review recorded in `reviews/ad-hoc-typescript-go-package-ambiguous-import-source-map-review.md`. Feedback incorporated by adding explicit package import-resolution state coverage, candidate-retention proof, end-to-end diagnostic fixtures, and non-duplication checks.
- `2026-06-01`: Claude implementation-start review recorded in `reviews/typescript-go-architecture-transfer-plan-review-pass-1.md`. M0 approved as the first implementation PR after tightening its public API, migration-site, test-gate, out-of-scope, and dependency-direction guardrail language.
- `2026-06-01`: M4 implementation review pass 1 recorded in `reviews/typescript-go-m4-analysis-snapshot-review-pass-1.md`; reviewer approved M4 for PR after confirming captured workspace snapshots, snapshot query metadata, LSP snapshot routing, stale-result checks, conservative dirty-scope state, and serialized execution.

## Validation Log

- Validation evidence will be recorded per implementation milestone.
- Planning validation starts with `git diff --check`.
- `2026-05-29`: `git diff --check -- issues/ad-hoc-typescript-go-compiler-architecture-transfer.md issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md reviews/ad-hoc-typescript-go-prerequisite-discovery.md reviews/ad-hoc-typescript-go-prerequisite-discovery-claude-review.md` -> passed for tracked diff context.
- `2026-05-29`: `git diff --no-index --check /dev/null <new planning file>` loop over the four new planning/review files -> passed with no whitespace errors.
- `2026-05-29`: `rg -n "[[:blank:]]+$" issues/ad-hoc-typescript-go-compiler-architecture-transfer.md issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md reviews/ad-hoc-typescript-go-prerequisite-discovery.md reviews/ad-hoc-typescript-go-prerequisite-discovery-claude-review.md` -> no trailing whitespace.
- `2026-05-29`: `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `2026-05-29`: `git diff --check -- issues/ad-hoc-typescript-go-compiler-architecture-transfer.md issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md reviews/ad-hoc-typescript-go-prerequisite-discovery.md reviews/ad-hoc-typescript-go-prerequisite-discovery-claude-review.md reviews/ad-hoc-typescript-go-architecture-decisions-claude-review.md reviews/ad-hoc-typescript-go-milestone-slicing-claude-review.md` -> passed.
- `2026-05-29`: `git diff --no-index --check /dev/null <planning/review file>` loop over the six planning/review files -> passed with no whitespace errors.
- `2026-05-29`: `rg -n "[[:blank:]]+$" issues/ad-hoc-typescript-go-compiler-architecture-transfer.md issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md reviews/ad-hoc-typescript-go-prerequisite-discovery.md reviews/ad-hoc-typescript-go-prerequisite-discovery-claude-review.md reviews/ad-hoc-typescript-go-architecture-decisions-claude-review.md reviews/ad-hoc-typescript-go-milestone-slicing-claude-review.md` -> no trailing whitespace.
- `2026-05-29`: `python3 scripts/check_file_size_guardrails.py` -> PASS after decision and milestone review updates.
- `2026-05-30`: `rg -n "[[:blank:]]+$" issues/ad-hoc-typescript-go-compiler-architecture-transfer.md issues/ad-hoc-typescript-go-compiler-architecture-transfer-execution.md reviews/ad-hoc-typescript-go-package-ambiguous-import-source-map-review.md` -> no trailing whitespace.
- `2026-05-30`: `git diff --no-index --check /dev/null <planning/review file>` loop over the seven TypeScript-Go planning/review files -> passed with no whitespace errors.
- `2026-05-30`: `python3 scripts/check_file_size_guardrails.py` -> PASS after package source-map diagnostic boundary update.
- `2026-06-01`: M0 focused validation in progress on branch `wave_tsgo_m0_source_foundation`.
- `2026-06-01`: `cargo test -p sifr_source` -> PASS.
- `2026-06-01`: `cargo test -p sifr_syntax` -> PASS.
- `2026-06-01`: `cargo test -p sifr_diagnostics && cargo test -p sifr_frontend && cargo test -p sifr_lsp` -> PASS after source-position authority migration.
- `2026-06-01`: `cargo test -p sifr_analysis` -> PASS.
- `2026-06-01`: `cargo test -p sifr -- --skip test_e2e_pass` -> PASS.
- `2026-06-01`: `cargo fmt --check` -> PASS.
- `2026-06-01`: `python3 scripts/check_file_size_guardrails.py` -> PASS.
- `2026-06-01`: `python3 scripts/check_source_crate_dependency_direction.py` -> PASS.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS before M0 implementation review; report `target/validation_lane_reports/quick.latest.json`, wall time 272.93s.
- `2026-06-01`: M0 implementation review pass 1 recorded in `reviews/typescript-go-m0-source-foundation-review-pass-1.md`; reviewer required fixing `cargo clippy --workspace -- -D warnings`.
- `2026-06-01`: `cargo clippy --workspace -- -D warnings` -> PASS after changing `SourceMapView::text_position_to_span` to borrow `TextPosition`.
- `2026-06-01`: `cargo test -p sifr_frontend` -> PASS after clippy follow-up.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS after clippy follow-up; report `target/validation_lane_reports/quick.latest.json`, wall time 232.80s.
- `2026-06-01`: `git diff --check` -> PASS after clippy follow-up.
- `2026-06-01`: `cargo fmt --check` -> PASS after clippy follow-up.
- `2026-06-01`: M0 implementation review pass 2 recorded in `reviews/typescript-go-m0-source-foundation-review-pass-2.md`; reviewer approved M0 for PR.
- `2026-06-01`: M0 merged via PR [#2229](https://github.com/sifr-lang/sifr/pull/2229), merge commit `e548c11310690553d614dc859a86177217a4c958`.
- `2026-06-01`: M1 validation in progress on branch `wave_tsgo_m1_architecture_guardrails`.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS.
- `2026-06-01`: `cargo clippy --workspace -- -D warnings` -> PASS.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS with new M1 guardrail wired into local validation; report `target/validation_lane_reports/quick.latest.json`, wall time 242.39s.
- `2026-06-01`: `git diff --check` -> PASS.
- `2026-06-01`: `cargo fmt --check` -> PASS.
- `2026-06-01`: M1 focused validation in progress on branch `wave_tsgo_m1_architecture_guardrails`.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS.
- `2026-06-01`: `cargo fmt --check` -> PASS for M1.
- `2026-06-01`: `git diff --check` -> PASS for M1.
- `2026-06-01`: `python3 scripts/check_file_size_guardrails.py && python3 scripts/check_source_crate_dependency_direction.py` -> PASS for M1.
- `2026-06-01`: `cargo test -p sifr_frontend && cargo test -p sifr_lsp` -> PASS for M1.
- `2026-06-01`: `cargo test -p sifr_analysis` -> PASS for M1.
- `2026-06-01`: `cargo test -p sifr -- --skip test_e2e_pass` -> PASS for M1.
- `2026-06-01`: M1 implementation review pass 1 recorded in `reviews/typescript-go-architecture-transfer-m1-review-pass-1.md`; reviewer requested direct-read inventory corrections and completeness/drift checking before PR.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS after M1 review remediation.
- `2026-06-01`: M1 implementation review pass 2 recorded in `reviews/typescript-go-architecture-transfer-m1-review-pass-2.md`; reviewer found the probe regex missed `.is_file()`/`.is_dir()` and one package lock-mode probe.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS after M1 review pass 2 remediation.
- `2026-06-01`: M1 implementation review pass 3 recorded in `reviews/typescript-go-architecture-transfer-m1-review-pass-3.md`; reviewer approved M1 for PR.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS for M1 (`target/validation_lane_reports/quick.latest.json`, wall time 282.00s).
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` initially failed after M1 review remediation because the guardrail doc omitted `crates/sifr_driver/src/build/workspace.rs:296`; the inventory and script were corrected.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS after build-metadata inventory correction.
- `2026-06-01`: `git diff --check`, `cargo fmt --check`, and `python3 scripts/check_file_size_guardrails.py` -> PASS after build-metadata inventory correction.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS after M1 review remediation; report `target/validation_lane_reports/quick.latest.json`, wall time 221.14s.
- `2026-06-01`: M1 implementation review pass 2 recorded in `reviews/typescript-go-architecture-transfer-m1-review-pass-2.md`; reviewer requested fixing the direct-read/probe regex and adding `crates/sifr_package/src/cargo/lock_modes.rs:46`.
- `2026-06-01`: M1 implementation review pass 3 recorded in `reviews/typescript-go-architecture-transfer-m1-review-pass-3.md`; reviewer approved M1 for PR after focused validation.
- `2026-06-01`: M1 merged via PR [#2230](https://github.com/sifr-lang/sifr/pull/2230), merge commit `a1a402c01348181684fc028c6fc14fba76d8b0cf`. Duplicate PR [#2231](https://github.com/sifr-lang/sifr/pull/2231) was closed without merge.
- `2026-06-01`: M2 validation in progress on branch `wave_tsgo_m2_source_provider_overlay`.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS for M2 after excluding the intentional `SourceProvider` boundary implementation from the direct-read scan and documenting remaining non-semantic exceptions.
- `2026-06-01`: `cargo fmt --check` -> PASS for M2.
- `2026-06-01`: `python3 scripts/check_file_size_guardrails.py` -> PASS for M2.
- `2026-06-01`: `cargo test -p sifr -- --skip test_e2e_pass` -> PASS for M2.
- `2026-06-01`: `cargo test -p sifr_driver -p sifr_package -p sifr_frontend -p sifr_format -p sifr_lint` -> PASS for M2.
- `2026-06-01`: `cargo clippy --workspace -- -D warnings` -> PASS for M2.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` initially failed for M2 on package-manager file-size guardrails; `imports/source_map.rs`, `manifest/sifr.rs`, and the M2 package tests were split by responsibility.
- `2026-06-01`: `python3 scripts/check_package_manager_guardrails.py` and `cargo test -p sifr_package` -> PASS after M2 package guardrail remediation.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS for M2; report `target/validation_lane_reports/quick.latest.json`, wall time 274.59s.
- `2026-06-01`: `cargo clippy --workspace -- -D warnings` -> PASS after final M2 package guardrail splits.
- `2026-06-01`: M2 implementation review pass 3 recorded in `reviews/typescript-go-m2-source-provider-overlay-review-pass-3.md`; reviewer approved M2 for PR with non-blocking follow-up notes.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` initially failed on package-manager file-size guardrails after M2 expanded `source_map.rs`, `manifest/sifr.rs`, and `milestone_37_3_tests.rs`; source-map discovery/resolution and manifest field parsing were split by responsibility.
- `2026-06-01`: `cargo test -p sifr_package`, `cargo clippy --workspace -- -D warnings`, and `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test` -> PASS after the package-manager guardrail remediation.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS after M2 remediation; report `target/validation_lane_reports/quick.latest.json`, wall time 260.57s.
- `2026-06-01`: M2 final implementation review pass 4 recorded in `reviews/typescript-go-m2-source-provider-overlay-review-pass-4.md`; reviewer approved the current post-remediation tree for PR.
- `2026-06-01`: M2 merged via PR [#2233](https://github.com/sifr-lang/sifr/pull/2233), merge commit `c74d73b59d179156b3a19c5f4176fc5738c49ab4`.
- `2026-06-01`: M3 validation completed on branch `wave_tsgo_m3_workspace_session`.
- `2026-06-01`: `cargo test -p sifr_frontend workspace_session`, `cargo test -p sifr_frontend`, `cargo test -p sifr_analysis`, `cargo test -p sifr_lsp`, and `cargo test -p sifr -- --skip test_e2e_pass` -> PASS for M3.
- `2026-06-01`: `cargo fmt --check`, `git diff --check`, and `cargo clippy --workspace -- -D warnings` -> PASS for M3.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`, `python3 scripts/check_file_size_guardrails.py`, and `python3 scripts/check_package_manager_guardrails.py` -> PASS for M3.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS for M3; report `target/validation_lane_reports/quick.latest.json`, wall time 261.09s.
- `2026-06-01`: M3 implementation review pass 1 recorded in `reviews/typescript-go-m3-workspace-session-review-pass-1.md`; reviewer approved M3 for PR after validation and requested only pre-PR tracker/doc/test cleanup. Cleanup was applied, and `cargo test -p sifr_frontend workspace_session`, `cargo clippy -p sifr_frontend -- -D warnings`, `cargo fmt --check`, and `git diff --check` passed after cleanup.
- `2026-06-01`: M3 merged via PR [#2235](https://github.com/sifr-lang/sifr/pull/2235), merge commit `b5582baf42ad3f4cf37643d9dd6e96d2107dd088`.
- `2026-06-01`: M4 validation in progress on branch `wave_tsgo_m4_analysis_snapshot`.
- `2026-06-01`: `cargo fmt --check` and `git diff --check` -> PASS for M4.
- `2026-06-01`: `cargo test -p sifr_analysis`, `cargo test -p sifr_lsp`, and `cargo test -p sifr_frontend` -> PASS for M4.
- `2026-06-01`: `cargo test -p sifr -- --skip test_e2e_pass` -> PASS for M4.
- `2026-06-01`: `cargo clippy -p sifr_analysis -p sifr_lsp -p sifr_frontend -- -D warnings` and `cargo clippy --workspace -- -D warnings` -> PASS for M4.
- `2026-06-01`: `python3 verification/tooling/check_typescript_go_m1_guardrails.py && python3 verification/tooling/check_typescript_go_m1_guardrails.py --self-test`, `python3 scripts/check_file_size_guardrails.py`, and `python3 scripts/check_package_manager_guardrails.py` -> PASS for M4.
- `2026-06-01`: `scripts/run_all_tests.sh --profile quick` -> PASS for M4; report `target/validation_lane_reports/quick.latest.json`, wall time 234.97s.
- `2026-06-01`: `cargo test -p sifr_analysis`, `cargo clippy -p sifr_analysis -- -D warnings`, `cargo fmt --check`, and `git diff --check` -> PASS after adding full snapshot query metadata coverage.

## PR Log

- `2026-05-30`: Phase planning/review PR [#2206](https://github.com/sifr-lang/sifr/pull/2206) records the architecture transfer contract, execution tracker, and incorporated review artifacts.
- `2026-06-01`: M0 source and position foundation merged in [#2229](https://github.com/sifr-lang/sifr/pull/2229).
- `2026-06-01`: M1 architecture contract and guardrails merged in [#2230](https://github.com/sifr-lang/sifr/pull/2230).
- `2026-06-01`: M2 source provider and overlay store merged in [#2233](https://github.com/sifr-lang/sifr/pull/2233).
- `2026-06-01`: M3 workspace session data model merged in [#2235](https://github.com/sifr-lang/sifr/pull/2235).
- `2026-06-01`: M4 analysis snapshot migration merged in [#2237](https://github.com/sifr-lang/sifr/pull/2237).
- Additional implementation PR links will be recorded per milestone after they are opened and merged.
