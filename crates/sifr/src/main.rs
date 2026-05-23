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
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

include!("main/cli_model_and_entrypoint.rs");
include!("main/diagnostic_rendering_and_run.rs");
include!("main/check_and_package_commands.rs");

#[cfg(test)]
mod tests {
    include!("main/mode_resolution_tests.rs");
    include!("main/diagnostics_and_packages_tests.rs");
}
