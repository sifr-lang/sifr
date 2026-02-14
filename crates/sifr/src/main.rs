//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code

use clap::{Parser, Subcommand};
use sifr_driver::{compile, check, build, build_project, CompileResult};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "sifr", version, about = "The Sifr programming language compiler")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .sifr file to a native binary
    Build {
        /// Input .sifr file
        file: PathBuf,
        /// Output directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },
    /// Compile and run a .sifr file
    Run {
        /// Input .sifr file
        file: PathBuf,
    },
    /// Type-check a .sifr file without compiling
    Check {
        /// Input .sifr file
        file: PathBuf,
    },
    /// Show the generated Rust source code
    Emit {
        /// Input .sifr file
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file, output } => cmd_build(&file, &output),
        Commands::Run { file } => cmd_run(&file),
        Commands::Check { file } => cmd_check(&file),
        Commands::Emit { file } => cmd_emit(&file),
    }
}

fn read_source(file: &PathBuf) -> String {
    match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("error: could not read file '{}': {}", file.display(), e);
            process::exit(1);
        }
    }
}

fn cmd_build(file: &PathBuf, output: &PathBuf) {
    let source = read_source(file);

    match build(&source, output) {
        Ok(binary_path) => {
            eprintln!("compiled successfully: {}", binary_path.display());
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error);
            }
            process::exit(1);
        }
    }
}

fn cmd_run(file: &PathBuf) {
    let temp_dir = std::env::temp_dir().join("sifr_run");

    // Check if this is a multi-file project:
    // The file must be named main.sifr AND there must be other .sifr files in the same directory
    let is_multi_file = file.file_stem().map_or(false, |stem| stem == "main")
        && if let Some(parent) = file.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                entries.flatten()
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "sifr"))
                    .filter(|e| e.path() != *file)
                    .count() > 0
            } else {
                false
            }
        } else {
            false
        };

    let result = if is_multi_file {
        build_project(file, &temp_dir)
    } else {
        let source = read_source(file);
        build(&source, &temp_dir)
    };

    match result {
        Ok(binary_path) => {
            let output = std::process::Command::new(&binary_path)
                .output()
                .unwrap_or_else(|e| {
                    eprintln!("error: could not run binary: {}", e);
                    process::exit(1);
                });

            // Forward stdout and stderr
            use std::io::Write;
            std::io::stdout().write_all(&output.stdout).ok();
            std::io::stderr().write_all(&output.stderr).ok();

            if !output.status.success() {
                process::exit(output.status.code().unwrap_or(1));
            }
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("{}", error);
            }
            process::exit(1);
        }
    }
}

fn cmd_check(file: &PathBuf) {
    let source = read_source(file);
    let errors = check(&source);

    if errors.is_empty() {
        eprintln!("no errors found");
    } else {
        for error in &errors {
            eprintln!("{}", error);
        }
        process::exit(1);
    }
}

fn cmd_emit(file: &PathBuf) {
    let source = read_source(file);

    match compile(&source) {
        CompileResult::Success { rust_source } => {
            print!("{}", rust_source);
        }
        CompileResult::Errors { errors } => {
            for error in &errors {
                eprintln!("{}", error);
            }
            process::exit(1);
        }
    }
}
