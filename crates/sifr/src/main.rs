//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
use clap::{Parser, Subcommand};
use sifr_driver::{build, build_project, check, compile, run_tests, CompileResult};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser)]
#[command(
    name = "sifr",
    version,
    about = "The Sifr programming language compiler"
)]
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
    /// Run tests in a directory
    Test {
        /// Directory containing test files (default: current directory)
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompilationMode {
    SingleFile,
    Project,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { file, output } => cmd_build(&file, &output),
        Commands::Run { file } => cmd_run(&file),
        Commands::Check { file } => cmd_check(&file),
        Commands::Emit { file } => cmd_emit(&file),
        Commands::Test { dir } => cmd_test(&dir),
    }
}

fn resolve_compilation_mode(file: &Path) -> CompilationMode {
    let is_project_entry = file.file_stem().is_some_and(|stem| stem == "main")
        && if let Some(parent) = file.parent() {
            if let Ok(entries) = std::fs::read_dir(parent) {
                entries
                    .flatten()
                    .filter(|e| e.path().extension().is_some_and(|ext| ext == "sifr"))
                    .filter(|e| e.path() != file)
                    .count()
                    > 0
            } else {
                false
            }
        } else {
            false
        };

    if is_project_entry {
        CompilationMode::Project
    } else {
        CompilationMode::SingleFile
    }
}

fn read_source(file: &Path) -> String {
    match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(e) => {
            let _ = writeln!(
                io::stderr(),
                "error: could not read file '{}': {e}",
                file.display()
            );
            process::exit(1);
        }
    }
}

fn cmd_build(file: &Path, output: &Path) {
    let result = match resolve_compilation_mode(file) {
        CompilationMode::Project => build_project(file, output),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build(&source, output)
        }
    };

    match result {
        Ok(binary_path) => {
            let _ = writeln!(
                io::stderr(),
                "compiled successfully: {}",
                binary_path.display()
            );
        }
        Err(errors) => {
            for error in &errors {
                let _ = writeln!(io::stderr(), "{error}");
            }
            process::exit(1);
        }
    }
}

fn cmd_run(file: &Path) {
    let temp_dir = std::env::temp_dir().join("sifr_run");
    let result = match resolve_compilation_mode(file) {
        CompilationMode::Project => build_project(file, &temp_dir),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build(&source, &temp_dir)
        }
    };

    match result {
        Ok(binary_path) => {
            let output = std::process::Command::new(&binary_path)
                .output()
                .unwrap_or_else(|e| {
                    let _ = writeln!(io::stderr(), "error: could not run binary: {e}");
                    process::exit(1);
                });

            // Forward stdout and stderr
            std::io::stdout().write_all(&output.stdout).ok();
            std::io::stderr().write_all(&output.stderr).ok();

            if !output.status.success() {
                process::exit(output.status.code().unwrap_or(1));
            }
        }
        Err(errors) => {
            for error in &errors {
                let _ = writeln!(io::stderr(), "{error}");
            }
            process::exit(1);
        }
    }
}

fn cmd_check(file: &Path) {
    let source = read_source(file);
    let errors = check(&source);

    if errors.is_empty() {
        let _ = writeln!(io::stderr(), "no errors found");
    } else {
        for error in &errors {
            let _ = writeln!(io::stderr(), "{error}");
        }
        process::exit(1);
    }
}

fn cmd_test(dir: &Path) {
    match run_tests(dir) {
        Ok(success) => {
            if !success {
                process::exit(1);
            }
        }
        Err(errors) => {
            for error in &errors {
                let _ = writeln!(io::stderr(), "{error}");
            }
            process::exit(1);
        }
    }
}

fn cmd_emit(file: &Path) {
    let source = read_source(file);

    match compile(&source) {
        CompileResult::Success { rust_source } => {
            let _ = write!(io::stdout(), "{rust_source}");
        }
        CompileResult::Errors { errors } => {
            for error in &errors {
                let _ = writeln!(io::stderr(), "{error}");
            }
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mktemp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "sifr_cli_mode_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_main_with_siblings() {
        let dir = mktemp_dir("project");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(&main, "def main():\n    pass\n").expect("main file should be written");
        std::fs::write(&helper, "def helper() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::Project);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_non_main_entry() {
        let dir = mktemp_dir("single");
        let app = dir.join("app.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(&app, "def main():\n    pass\n").expect("app file should be written");
        std::fs::write(&helper, "def helper() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&app), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }
}
