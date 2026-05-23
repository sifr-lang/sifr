//! Sifr Code Generation: translates typed HIR into Rust source code.
#![allow(dead_code)]
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

include!("lib_modules_and_codegen.rs");
include!("lib_runtime_needs.rs");
include!("lib_project_codegen.rs");
include!("lib_emitter_state.rs");
