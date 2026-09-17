//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
//!   sifr fmt [OPTIONS] [FILES]...
//!                              Format Sifr source files
//!   sifr lint [OPTIONS] [FILES]...
//!                              Run suppressible policy diagnostics
//!   sifr lsp --stdio          Run the native Language Server Protocol server
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, dead_code))]

mod bridge_cli;
mod cache_cli;
mod cargo_diagnostics;
mod cli_lock_modes;
mod cli_model_and_entrypoint;
mod native_execution;
pub(crate) use cli_model_and_entrypoint::main;
mod build_output;
mod check_and_package_commands;
mod command_args;
mod deferred_cli_args;
mod diagnostic_rendering_and_run;
#[cfg(test)]
mod diagnostic_test_sink;
#[cfg(test)]
mod eager_cli_contract_tests;
mod explain_cli;
mod formatter_cli;
mod formatter_discovery;
mod host_tool_cli;
mod host_tool_sandbox;
mod lint_cli;
mod metadata_cli;
mod package_graph_context;
mod package_python_certifications;
mod package_session_cli;
mod python_binding_cli;
mod python_dlpack_certification_cli;
mod python_runtime_context;
mod self_update_cli;
mod self_update_metadata;
mod self_update_metadata_source;
mod self_update_receipt;
mod self_update_runner;
mod sysroot_cli;
mod trace_cli;
mod workspace_run_selection;

#[cfg(test)]
mod bridge_cli_tests;
#[cfg(test)]
mod cargo_lock_mode_certification_tests;
#[cfg(test)]
mod diagnostics_and_packages_tests;
#[cfg(test)]
mod mode_resolution_tests;
mod python_cli;

mod compiled_identity;
#[doc(hidden)]
pub use compiled_identity::compiled_input_tokens;

fn compiler_identity() -> sifr_identity::CompilerIdentity {
    if cfg!(test) {
        return sifr_identity::CompilerIdentity::for_test(compiled_input_tokens(), "sifr-unit");
    }
    // A malformed build-generated ID is a compiler build invariant.
    match sifr_identity::CompilerIdentity::product(env!("SIFR_COMPILER_BUILD_ID")) {
        Ok(identity) => identity,
        Err(error) => panic!("{error}"),
    }
}

fn compiler_context() -> sifr_driver::CompilerContext {
    sifr_driver::CompilerContext::new(compiler_identity())
}
