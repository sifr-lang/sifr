//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
//!   sifr fmt [--check] <path> Format Sifr source files
//!   sifr lint <path>          Run suppressible policy diagnostics
//!   sifr lsp --stdio          Run the native Language Server Protocol server
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used, dead_code))]

mod cli_model_and_entrypoint;
pub(crate) use cli_model_and_entrypoint::main;
mod check_and_package_commands;
mod diagnostic_rendering_and_run;
mod formatter_cli;
mod formatter_config;
mod workspace_run_selection;

#[cfg(test)]
mod diagnostics_and_packages_tests;
#[cfg(test)]
mod mode_resolution_tests;
