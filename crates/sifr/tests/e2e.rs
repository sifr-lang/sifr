//! End-to-end tests for the Sifr compiler.
//!
//! Legacy contract
//! 1. Discovery is lexicographic by fixture path.
//! 2. Expectation annotations preserve declaration order.
//! 3. Failure aggregation reports all failures and passed/failed counts.
//! 4. Pass/fail exit semantics panic on any failure in `test_e2e_pass`.
//!
//! Throughput runner contract
//! - Contract is controlled via `SIFR_E2E_RUNNER_MODE` and legacy booleans.
//! - `SIFR_E2E_SIFR_JOBS`: bounded parallel compile workers.
//! - `SIFR_E2E_RUST_JOBS`: bounded parallel build workers.
//! - `SIFR_E2E_RUN_JOBS`: bounded parallel run workers.
//! - `SIFR_E2E_CARGO_BUILD_JOBS`: cargo jobs per generated group build.
//! - `SIFR_E2E_DISABLE_CACHE=1` disables cache reuse.
#![allow(clippy::expect_used, clippy::unwrap_used)]

include!("e2e/harness_model.rs");
include!("e2e/fixture_compilation.rs");
include!("e2e/batch_execution.rs");
include!("e2e/e2e_entrypoints.rs");
include!("e2e/harness_contract_tests.rs");
