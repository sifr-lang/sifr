//! Public emission contracts reuse the production library; private invariants remain unit tests.
#![allow(clippy::expect_used, clippy::unwrap_used)]
use sifr_codegen::*;

#[path = "contracts/lib_codegen_tests.rs"]
mod lib_codegen_tests;
#[path = "contracts/generated_rust_canonicalizer_control_flow_tests.rs"]
mod generated_rust_canonicalizer_control_flow_tests;
