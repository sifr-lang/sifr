//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
use clap::{Parser, Subcommand};
use sifr_driver::{build, build_project, check, compile, run_tests, CompileError, CompileResult};
use sifr_python_ast::Stmt;
use sifr_python_parser::parse_module;
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
    let is_project_entry =
        file.file_stem().is_some_and(|stem| stem == "main") && has_local_project_imports(file);

    if is_project_entry {
        CompilationMode::Project
    } else {
        CompilationMode::SingleFile
    }
}

fn has_local_project_imports(file: &Path) -> bool {
    let Some(parent) = file.parent() else {
        return false;
    };
    let source = match std::fs::read_to_string(file) {
        Ok(source) => source,
        Err(_) => return false,
    };
    let parsed = match parse_module(&source) {
        Ok(parsed) if parsed.is_valid() => parsed,
        _ => return false,
    };

    parsed.suite().iter().any(|stmt| {
        let Stmt::ImportFrom(import_from) = stmt else {
            return false;
        };
        let Some(module) = &import_from.module else {
            return false;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            return false;
        }
        parent.join(format!("{module_name}.sifr")).is_file()
    })
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
    let result = compile_entrypoint(file, output);

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
    let result = compile_entrypoint(file, &temp_dir);

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

fn compile_entrypoint(file: &Path, output: &Path) -> Result<PathBuf, Vec<CompileError>> {
    match resolve_compilation_mode(file) {
        CompilationMode::Project => build_project(file, output),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build(&source, output)
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
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
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

    #[test]
    fn test_resolve_compilation_mode_single_file_for_main_without_local_imports() {
        let dir = mktemp_dir("main_no_imports");
        let main = dir.join("main.sifr");
        let scratch = dir.join("scratch.sifr");
        std::fs::write(&main, "def main():\n    print(\"ok\")\n")
            .expect("main file should be written");
        std::fs::write(&scratch, "def nope(:\n").expect("scratch file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_stdlib_only_imports() {
        let dir = mktemp_dir("main_stdlib_only");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from sifr.math import floor\n\ndef main():\n    print(floor(3.9))\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def helper() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_missing_local_module() {
        let dir = mktemp_dir("missing_local");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_invalid_main_source() {
        let dir = mktemp_dir("invalid_main");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(&main, "def main(:\n").expect("main file should be written");
        std::fs::write(&helper, "def helper() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_typing_import() {
        let dir = mktemp_dir("typing_import");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from typing import List\n\ndef main():\n    values: List[int] = [1]\n    print(values)\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def helper() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_typing_import_with_local_typing_file() {
        let dir = mktemp_dir("typing_import_local_file");
        let main = dir.join("main.sifr");
        let typing_local = dir.join("typing.sifr");
        std::fs::write(
            &main,
            "from typing import List\n\ndef main():\n    values: List[int] = [1]\n    print(values)\n",
        )
        .expect("main file should be written");
        std::fs::write(&typing_local, "def local() -> int:\n    return 1\n")
            .expect("typing file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_enum_import() {
        let dir = mktemp_dir("enum_import");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from enum import Enum\n\ndef main():\n    print(\"ok\")\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def helper() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_enum_import_with_local_enum_file() {
        let dir = mktemp_dir("enum_import_local_file");
        let main = dir.join("main.sifr");
        let enum_local = dir.join("enum.sifr");
        std::fs::write(
            &main,
            "from enum import Enum\n\ndef main():\n    print(\"ok\")\n",
        )
        .expect("main file should be written");
        std::fs::write(&enum_local, "def local() -> int:\n    return 1\n")
            .expect("enum file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_package_init_import() {
        let dir = mktemp_dir("pkg_import");
        let main = dir.join("main.sifr");
        let pkg_dir = dir.join("pkg");
        std::fs::create_dir_all(&pkg_dir).expect("pkg dir should be created");
        std::fs::write(
            &main,
            "from pkg import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(
            pkg_dir.join("__init__.sifr"),
            "def value() -> int:\n    return 1\n",
        )
        .expect("pkg init should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_relative_import_with_sibling() {
        let dir = mktemp_dir("relative_import");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from .helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::Project);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_relative_import_without_sibling() {
        let dir = mktemp_dir("relative_import_missing_sibling");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from .helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_project_mode() {
        let dir = mktemp_dir("entrypoint_consistency");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value(:\n").expect("helper file should be written");

        let run_out = mktemp_dir("run_path");
        let build_out = mktemp_dir("build_path");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }
}
