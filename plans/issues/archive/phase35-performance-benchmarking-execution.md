# Phase 35 Performance Benchmarking And Budgets Execution

status: completed

## Milestone Checklist

- [x] m35.4a syntax/frontend ownership crates created.
- [x] m35.4a driver parser diagnostics delegated to `sifr_syntax`.
- [x] m35.4a frontend cache smoke contract added.
- [x] m35.4a split-brain guardrail added with seeded negative self-test.
- [x] m35.4a Ruff fork fixture revalidation contract added.
- [x] m35.1 baseline benchmark suite.
- [x] m35.2 budget and waiver policy.
- [x] m35.3 enforcement integration for performance budgets.
- [x] m35.4b full CLI adoption and query regression lock.

## Validation Evidence

- `cargo check -p sifr_driver -p sifr_syntax -p sifr_frontend` -> PASS.
- `cargo clippy -p sifr_syntax -p sifr_frontend -- -D warnings` -> PASS.
- `cargo test -p sifr_syntax` -> PASS.
- `cargo test -p sifr_frontend` -> PASS.
- `python3 verification/performance/check_ruff_fork_update_rules.py` -> PASS.
- `python3 verification/performance/check_split_brain_guardrail.py` -> PASS.
- `python3 verification/performance/check_split_brain_guardrail.py --self-test` -> PASS.
- `python3 verification/performance/check_frontend_cache_rules.py` -> PASS.
- `reviews/phase35-m35-4a-review-pass-1.md` -> NOT SATISFIED; blockers B1/B2 fixed.
- `reviews/phase35-m35-4a-review-pass-2.md` -> SATISFIED for m35.4a.
- `scripts/run_all_tests.sh --profile quick` -> PASS for m35.1 (`wall_time=944.94s`), m35.2 (`wall_time=748.90s`), m35.3 (`wall_time=564.73s`), and m35.4b (`wall_time=1236.88s`); wall-time advisories recorded in `target/validation_lane_reports/quick.latest.json`, report signature `f808284595f17a99`.
- `verification/performance/manifest.json` -> 45 benchmark cases across the required Phase 35 groups.
- `verification/performance/baselines.json` -> captured with all 45 manifest cases; maximum coefficient of variation `0.091581`.
- `python3 verification/performance/run_benchmarks.py --validate-only` -> PASS.
- `python3 verification/performance/run_benchmarks.py --self-test` -> PASS; malformed manifest, missing input, timeout result, missing metric, and high-variance result seeds fail with expected diagnostics.
- `python3 verification/performance/run_benchmarks.py --sample-scale smoke` -> PASS across all 45 cases; evidence `target/performance/evidence/bench-1778969000-18081.json`.
- `python3 verification/performance/run_benchmarks.py --capture-baseline` -> PASS; wrote `verification/performance/baselines.json` and evidence `target/performance/evidence/bench-1778968427-69591.json`.
- `cargo fmt --check` -> PASS.
- `cargo clippy -p sifr_frontend -- -D warnings` -> PASS.
- `cargo test -p sifr_frontend` -> PASS.
- `reviews/phase35-m35-1-review-pass-1.md` -> SATISFIED for m35.1.
- `verification/performance/budgets.json` -> derived from all 45 m35.1 baseline results.
- `verification/performance/waivers.json` -> active waiver registry initialized empty.
- `internal_docs/performance_budgets.md` -> budget derivation, waiver policy, and local commands documented.
- `python3 verification/performance/check_budgets.py` -> PASS against checked-in baselines.
- `python3 verification/performance/check_budgets.py --self-test` -> PASS; seeded median, p95, RSS, timeout, missing-result, unknown-id, malformed-result, expired-waiver, malformed-waiver, and correctness-waiver failures are rejected.
- `python3 -m py_compile verification/performance/check_budgets.py verification/performance/run_benchmarks.py` -> PASS.
- `reviews/phase35-m35-2-review-pass-1.md` -> SATISFIED for m35.2.
- `scripts/run_all_tests.sh` -> adds "Performance Budget Checks" after Phase 35 frontend/syntax guardrails.
- Quick lane performance checks -> manifest validation, benchmark self-test, checked-in baseline budget gate, budget self-test, and a two-case frontend-query smoke.
- PR/nightly/release performance checks -> same schema/negative checks plus a reviewed seven-case representative benchmark subset with budget comparison against `target/performance/<profile>.budget.latest.json`.
- `python3 verification/performance/run_benchmarks.py --case check-single-file-001-arithmetic --case check-project-004-project-graph --case build-single-file-001-break-continue --case build-project-001-additional-modules --case incremental-local-loop-001-unchanged-file-update --case interactive-tooling-foundation-002-warm-diagnostics-query --case phase27-non-regression-002-json-diagnostic-schema --json-out target/performance/test.pr.subset.budget.json && python3 verification/performance/check_budgets.py --results target/performance/test.pr.subset.budget.json --allow-subset` -> PASS.
- `bash -n scripts/run_all_tests.sh` -> PASS.
- `reviews/phase35-m35-3-review-pass-1.md` -> SATISFIED for m35.3.
- m35.4b removed the duplicate driver frontend parser/lowering/export shims and routes driver/CLI frontend flows through `sifr_frontend`/`sifr_syntax`.
- m35.4b added `sifr_syntax::parse_module_suite` so CLI/project compiler paths keep the syntax ownership boundary without paying token/trivia collection cost when only the AST suite is needed.
- m35.4b fixed command benchmark RSS measurement to use per-command `/usr/bin/time` output where available, avoiding cumulative `RUSAGE_CHILDREN` contamination from earlier validation subprocesses.
- `verification/performance/check_split_brain_guardrail.py` -> PASS with no driver/CLI migration allowlist.
- `cargo check -p sifr_frontend -p sifr_driver -p sifr` -> PASS.
- `cargo clippy -p sifr_frontend -p sifr_driver -p sifr -- -D warnings` -> PASS.
- `cargo fmt --check` -> PASS.
- `cargo test -p sifr_frontend` -> PASS.
- `cargo test -p sifr_driver project_build_check -- --skip cached_project_binary` -> PASS.
- `python3 verification/performance/check_frontend_cache_rules.py` -> PASS.
- `python3 verification/performance/run_benchmarks.py --validate-only` -> PASS.
- `python3 verification/performance/run_benchmarks.py --self-test` -> PASS.
- `python3 verification/performance/check_budgets.py` -> PASS.
- `python3 verification/performance/check_budgets.py --self-test` -> PASS.
- CLI positive smoke: `target/debug/sifr check crates/sifr/tests/verification/project/multi_module_run/main.sifr` -> PASS with `no errors found`.
- CLI emit smoke: `target/debug/sifr emit crates/sifr/tests/verification/project/multi_module_run/main.sifr >/tmp/sifr_emit_smoke.rs` -> PASS.
- `reviews/phase35-m35-4b-review-pass-1.md` -> SATISFIED for m35.4b.
- `reviews/phase35-m35-4b-review-pass-2.md` -> SATISFIED for post-review AST-only parser and benchmark RSS measurement deltas.
- `scripts/run_all_tests.sh --profile pr` -> PASS for m35.4b (`wall_time=2662.52s`); warm wall-time and group-skew advisories recorded in `target/validation_lane_reports/pr.latest.json`, report signature `6cd36071cf629b47`.
- m35.4b merged in PR [#2127](https://github.com/sifr-lang/sifr/pull/2127) at `40ecd64ab7c80df01e60c7e79c9b847cad02c489`.
- `reviews/phase35-full-closure-review-pass-1.md` -> SATISFIED for full Phase 35 closure; no blockers.

## Open Migration Targets

- None for Phase 35 after m35.4b; Phase 36 must build tooling on `sifr_frontend` without adding semantics-bearing adapter paths.
