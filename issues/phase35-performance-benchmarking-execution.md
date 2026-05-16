# Phase 35 Performance Benchmarking And Budgets Execution

status: in_progress

## Milestone Checklist

- [x] m35.4a syntax/frontend ownership crates created.
- [x] m35.4a driver parser diagnostics delegated to `sifr_syntax`.
- [x] m35.4a frontend cache smoke contract added.
- [x] m35.4a split-brain guardrail added with seeded negative self-test.
- [x] m35.4a Ruff fork fixture revalidation contract added.
- [x] m35.1 baseline benchmark suite.
- [x] m35.2 budget and waiver policy.
- [ ] m35.3 enforcement integration for performance budgets.
- [ ] m35.4b full CLI adoption and query regression lock.

## Validation Evidence

- `cargo check -p sifr_driver -p sifr_syntax -p sifr_frontend` -> PASS.
- `cargo clippy -p sifr_syntax -p sifr_frontend -- -D warnings` -> PASS.
- `cargo test -p sifr_syntax` -> PASS.
- `cargo test -p sifr_frontend` -> PASS.
- `python3 verification/performance/check_ruff_fork_update_contract.py` -> PASS.
- `python3 verification/performance/check_split_brain_guardrail.py` -> PASS.
- `python3 verification/performance/check_split_brain_guardrail.py --self-test` -> PASS.
- `python3 verification/performance/check_frontend_cache_contract.py` -> PASS.
- `reviews/phase35-m35-4a-review-pass-1.md` -> NOT SATISFIED; blockers B1/B2 fixed.
- `reviews/phase35-m35-4a-review-pass-2.md` -> SATISFIED for m35.4a.
- `scripts/run_all_tests.sh --profile quick` -> PASS for m35.1 (`wall_time=944.94s`) and m35.2 (`wall_time=748.90s`); wall-time advisories recorded in `target/validation_lane_reports/quick.latest.json`, report signature `f808284595f17a99`.
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

## Open Migration Targets

- Route all CLI frontend flows through `sifr_frontend` in m35.4b.
- Replace remaining raw parser use in CLI mode detection and stdlib bootstrap with `sifr_syntax`.
- Replace the temporary split-brain allowlist entries with the final strict approved-boundary policy.
