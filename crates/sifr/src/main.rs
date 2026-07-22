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
mod cli_model_and_entrypoint;
pub(crate) use cli_model_and_entrypoint::main;
mod build_output;
mod check_and_package_commands;
mod diagnostic_rendering_and_run;
mod explain_cli;
mod formatter_cli;
mod lint_cli;
mod package_python_certifications;
mod python_binding_cli;
mod python_dlpack_certification_cli;
mod python_runtime_context;
mod self_update_cli;
mod self_update_metadata;
mod self_update_receipt;
mod self_update_runner;
mod sysroot_cli;
mod trace_cli;
mod workspace_run_selection;

#[cfg(test)]
mod bridge_cli_tests;
#[cfg(test)]
mod diagnostics_and_packages_tests;
#[cfg(test)]
mod mode_resolution_tests;
mod python_cli;
