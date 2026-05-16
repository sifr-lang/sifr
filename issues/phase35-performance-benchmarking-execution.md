# Phase 35 Performance Benchmarking And Budgets Execution

status: in_progress

## Milestone Checklist

- [x] m35.4a syntax/frontend ownership crates created.
- [x] m35.4a driver parser diagnostics delegated to `sifr_syntax`.
- [x] m35.4a frontend cache smoke contract added.
- [x] m35.4a split-brain guardrail added with seeded negative self-test.
- [x] m35.4a Ruff fork fixture revalidation contract added.
- [ ] m35.1 baseline benchmark suite.
- [ ] m35.2 budget and waiver policy.
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
- `scripts/run_all_tests.sh --profile quick` -> PASS; wall-time advisory recorded in `target/validation_lane_reports/quick.latest.json`.

## Open Migration Targets

- Route all CLI frontend flows through `sifr_frontend` in m35.4b.
- Replace remaining raw parser use in CLI mode detection and stdlib bootstrap with `sifr_syntax`.
- Replace the temporary split-brain allowlist entries with the final strict approved-boundary policy.
