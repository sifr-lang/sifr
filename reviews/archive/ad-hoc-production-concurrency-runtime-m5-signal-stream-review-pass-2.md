# M5 signal stream shape/lowering review — pass 2

## Pass-1 blocker is closed

The pass-1 blocker was that the Tokio `signal` feature only landed in `crates/sifr_stdlib/src/features.rs` (the generated runtime path) and was missing from three places used by tests and the grouped e2e harness. All three are now fixed and aligned on the same alphabetized features string `["io-util", "macros", "process", "rt", "signal", "sync", "time"]`:

- `crates/sifr/tests/e2e_support/fixture_compilation.rs:481` — `tokio_dependency_spec()` (the helper the grouped harness uses) now includes `"signal"`. This is the spec written into Cargo.toml for every grouped fixture build, so `signal_stream_shape_strsignal` (and any future signal fixture) compiles under the grouped batch path that previously failed.
- `crates/sifr/tests/e2e_support/harness_contract_tests.rs:522` — `test_generate_cargo_toml_required_tokio_uses_runtime_features` pins the same features string, so the harness contract test continues to detect drift between helper and assertion.
- `crates/sifr_codegen/src/lib_codegen_tests/async_runtime_codegen_tests.rs:165` — `test_generate_project_emits_tokio_dependency_when_required` now matches the updated `TOKIO_DEPS` from `features.rs:189`. This test was the unit-level signal that pass-1 caught failing locally; it is now consistent.
- `crates/sifr/tests/e2e/pass/signal_stream_shape_strsignal.sifr:29-31` — the unawaited futures now carry an explicit `# Shape pin only` comment with the rationale that deterministic external signal delivery is a later M5 harness wave. This is exactly the shape-pin TODO the pass-1 smaller finding requested.
- `issues/ad-hoc-production-concurrency-runtime-platform-substrate-execution.md:601-606` — the execution ledger records the blocker remediation and now lists the post-remediation `cargo test` runs for both the unit assertion and the harness assertion, the post-remediation `scripts/run_e2e_pass.sh --profile create-pr` pass (117 passed, 0 failed, `report_signature=ded105ad58090608`), and the authoritative `scripts/run_all_tests.sh --profile create-pr` pass with the same e2e signature and platform golden `pass=6, skip=1`. The advisories on warm wall-time and cache hit rate are noted honestly.

The validation evidence in the prompt matches the ledger: `cargo fmt --check`, the two targeted unit tests, `scripts/run_e2e_pass.sh --profile create-pr` (117 passed), and `scripts/run_all_tests.sh --profile create-pr` (e2e 117 passed, advisories only) all PASS.

## No new blockers in the current diff

The signal-stream wave's surface area (`crates/sifr_codegen/src/intrinsics/registry/signal.rs`, `crates/sifr_stdlib/src/signal.rs`, `lib/sifr/signal.sifr`, the registry wiring at `crates/sifr_codegen/src/intrinsics/registry.rs:701-712`, the registry test at `crates/sifr_codegen/src/intrinsics/registry_core_tests.rs:271-299`, the fixture, traceability/host-matrix docs, and manifest entries) is unchanged from pass-1 except for the documentation comment fix above. Pass-1's smaller findings (raw text emit inside `RustExpr::Ident`, Sifr-side `strsignal` integer match, `ShutdownStream` single-method stub, supported-host umbrella-vs-specific overlap) are still present but were already classified as non-blocking and tracked as M5 follow-ups; nothing about the blocker remediation made them worse.

## PR-scope note (not a blocker for the technical review)

The working tree carries unrelated network/HTTP edits and review artifacts (`issues/ad-hoc-production-network-http-platform-substrate.md`, `issues/ad-hoc-production-network-http-platform-substrate-execution.md`, and two `reviews/ad-hoc-production-network-http-platform-substrate-implementation-readiness-review-pass-*.md` files) that have nothing to do with this signal-stream wave. The same scope-hygiene concern was already flagged in the M4 process handle boundaries pass-3 review. These files must stay out of the signal-stream PR; that is a staging concern, not a code-correctness blocker for the wave under review.

## Answer to the follow-up question

The pass-1 blocker is fully closed: the Tokio `signal` feature is now consistent across the generated runtime path, the grouped e2e harness helper, the harness contract assertion, and the codegen project assertion; the shape-pin fixture documents why the futures are not awaited; and the execution ledger records both the remediation steps and the authoritative `scripts/run_all_tests.sh --profile create-pr` lane passing with the new fixture (`117 passed, 0 failed, report_signature=ded105ad58090608`). No new blockers appear in the current diff.

RESULT: PASS
