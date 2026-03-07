//! Sifr Compiler CLI
//!
//! Usage:
//!   sifr build <file.sifr>    Compile to native binary
//!   sifr run <file.sifr>      Compile and run
//!   sifr check <file.sifr>    Type-check only
//!   sifr emit <file.sifr>     Show generated Rust code
use clap::{Parser, Subcommand, ValueEnum};
use sifr_driver::{
    apply_diagnostic_recovery_limits, build, build_project, check, check_project, compile,
    compile_errors_to_diagnostics, run_tests, CompileError, CompileResult,
};
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
    /// Diagnostic output format
    #[arg(long, value_enum, default_value_t = DiagnosticFormat::Human)]
    diagnostic_format: DiagnosticFormat,

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum DiagnosticFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompilationMode {
    SingleFile,
    Project,
}

fn main() {
    let cli = Cli::parse();
    let diagnostic_format = cli.diagnostic_format;

    match cli.command {
        Commands::Build { file, output } => cmd_build(&file, &output, diagnostic_format),
        Commands::Run { file } => cmd_run(&file, diagnostic_format),
        Commands::Check { file } => cmd_check(&file, diagnostic_format),
        Commands::Emit { file } => cmd_emit(&file, diagnostic_format),
        Commands::Test { dir } => cmd_test(&dir, diagnostic_format),
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
        if import_from.level > 1 {
            return false;
        }
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

struct InvocationWorkspace {
    path: PathBuf,
}

impl InvocationWorkspace {
    fn create(prefix: &str) -> io::Result<Self> {
        let base_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let root = std::env::temp_dir();
        for attempt in 0..8u8 {
            let unique = if attempt == 0 {
                format!("{}_{}_{}", prefix, process::id(), base_nanos)
            } else {
                format!("{}_{}_{}_{}", prefix, process::id(), base_nanos, attempt)
            };
            let path = root.join(unique);
            match std::fs::create_dir(&path) {
                Ok(_) => return Ok(Self { path }),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "failed to allocate unique workspace for prefix '{}'",
                prefix
            ),
        ))
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for InvocationWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn render_compile_errors(errors: &[CompileError], format: DiagnosticFormat) {
    let diagnostics = apply_diagnostic_recovery_limits(&compile_errors_to_diagnostics(errors));
    match format {
        DiagnosticFormat::Human => {
            for diagnostic in diagnostics {
                let label = if diagnostic.code.starts_with("SIFR-PARSE-") {
                    "parse error"
                } else if diagnostic.code.starts_with("SIFR-TYPE-") {
                    "type error"
                } else if diagnostic.code.starts_with("SIFR-CODEGEN-") {
                    "codegen error"
                } else if diagnostic.code.starts_with("SIFR-BUILD-") {
                    "build error"
                } else {
                    match diagnostic.severity {
                        sifr_driver::Severity::Error => "error",
                        sifr_driver::Severity::Warning => "warning",
                        sifr_driver::Severity::Note => "note",
                        sifr_driver::Severity::Help => "help",
                    }
                };
                let _ = writeln!(
                    io::stderr(),
                    "{label}: {message}",
                    message = diagnostic.message
                );
            }
        }
        DiagnosticFormat::Json => match serde_json::to_string_pretty(&diagnostics) {
            Ok(json) => {
                let _ = writeln!(io::stderr(), "{json}");
            }
            Err(e) => {
                let _ = writeln!(
                    io::stderr(),
                    "build error: failed to serialize diagnostics as json: {e}"
                );
                process::exit(2);
            }
        },
    }
}

fn cmd_build(file: &Path, output: &Path, diagnostic_format: DiagnosticFormat) {
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
            render_compile_errors(&errors, diagnostic_format);
            process::exit(1);
        }
    }
}

fn cmd_run(file: &Path, diagnostic_format: DiagnosticFormat) {
    let workspace = match InvocationWorkspace::create("sifr_run") {
        Ok(workspace) => workspace,
        Err(e) => {
            let _ = writeln!(io::stderr(), "error: could not create run workspace: {e}");
            process::exit(1);
        }
    };
    let result = compile_entrypoint(file, workspace.path());

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
            render_compile_errors(&errors, diagnostic_format);
            process::exit(1);
        }
    }
}

fn cmd_check(file: &Path, diagnostic_format: DiagnosticFormat) {
    let errors = check_entrypoint(file);

    if errors.is_empty() {
        match diagnostic_format {
            DiagnosticFormat::Human => {
                let _ = writeln!(io::stderr(), "no errors found");
            }
            DiagnosticFormat::Json => {
                let _ = writeln!(io::stdout(), "[]");
            }
        }
    } else {
        render_compile_errors(&errors, diagnostic_format);
        process::exit(1);
    }
}

fn cmd_test(dir: &Path, diagnostic_format: DiagnosticFormat) {
    match run_tests(dir) {
        Ok(success) => {
            if !success {
                process::exit(1);
            }
        }
        Err(errors) => {
            render_compile_errors(&errors, diagnostic_format);
            process::exit(1);
        }
    }
}

fn cmd_emit(file: &Path, diagnostic_format: DiagnosticFormat) {
    let source = read_source(file);

    match compile(&source) {
        CompileResult::Success { rust_source } => {
            let _ = write!(io::stdout(), "{rust_source}");
        }
        CompileResult::Errors { errors } => {
            render_compile_errors(&errors, diagnostic_format);
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

fn check_entrypoint(file: &Path) -> Vec<CompileError> {
    match resolve_compilation_mode(file) {
        CompilationMode::Project => check_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            check(&source)
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
    fn test_invocation_workspace_create_returns_unique_paths() {
        let first = InvocationWorkspace::create("sifr_run_workspace")
            .expect("first workspace should exist");
        let second = InvocationWorkspace::create("sifr_run_workspace")
            .expect("second workspace should exist");
        assert_ne!(first.path(), second.path());
        assert!(first.path().exists());
        assert!(second.path().exists());
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
    fn test_resolve_compilation_mode_single_file_for_regular_import_with_local_module() {
        let dir = mktemp_dir("regular_import_local_module");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(&main, "import helper\n\ndef main():\n    print(\"ok\")\n")
            .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 1\n")
            .expect("helper file should be written");

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
    fn test_resolve_compilation_mode_single_file_for_multi_level_relative_import() {
        let dir = mktemp_dir("relative_import_multi_level");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from ..helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 1\n")
            .expect("helper file should be written");

        assert_eq!(resolve_compilation_mode(&main), CompilationMode::SingleFile);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_bare_relative_import() {
        let dir = mktemp_dir("relative_import_bare");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from . import value\n\ndef main():\n    print(value)\n",
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

    #[test]
    fn test_compile_entrypoint_error_consistency_for_import_statement() {
        let dir = mktemp_dir("entrypoint_import_statement");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(&main, "import helper\n\ndef main():\n    print(\"ok\")\n")
            .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 1\n")
            .expect("helper file should be written");

        let run_out = mktemp_dir("run_path_import_statement");
        let build_out = mktemp_dir("build_path_import_statement");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported import statement 'import helper'")));

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_bare_relative_import() {
        let dir = mktemp_dir("entrypoint_bare_relative");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from . import helper\n\ndef main():\n    print(helper)\n",
        )
        .expect("main file should be written");

        let run_out = mktemp_dir("run_path_bare_relative");
        let build_out = mktemp_dir("build_path_bare_relative");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported bare relative import")));

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_compile_entrypoint_error_consistency_for_multi_level_relative_import() {
        let dir = mktemp_dir("entrypoint_multi_level_relative");
        let main = dir.join("main.sifr");
        std::fs::write(
            &main,
            "from ..helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");

        let run_out = mktemp_dir("run_path_multi_level_relative");
        let build_out = mktemp_dir("build_path_multi_level_relative");
        let run_err = compile_entrypoint(&main, &run_out).expect_err("run compile should fail");
        let build_err =
            compile_entrypoint(&main, &build_out).expect_err("build compile should fail");
        let run_messages: Vec<String> = run_err.iter().map(ToString::to_string).collect();
        let build_messages: Vec<String> = build_err.iter().map(ToString::to_string).collect();
        assert_eq!(run_messages, build_messages);
        assert!(run_messages
            .iter()
            .any(|m| m.contains("unsupported relative import level 2")));

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_project_mode_resolves_local_imports() {
        let dir = mktemp_dir("check_entrypoint_project_imports");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return 42\n")
            .expect("helper file should be written");

        let errors = check_entrypoint(&main);
        assert!(
            errors.is_empty(),
            "project-aware check should succeed for valid local imports: {errors:?}"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_check_entrypoint_project_mode_error_parity_with_compile_entrypoint() {
        let dir = mktemp_dir("check_entrypoint_error_parity");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return \"bad\"\n")
            .expect("helper file should be written");

        let check_errors = check_entrypoint(&main);
        let build_out = mktemp_dir("check_entrypoint_build_out");
        let build_errors = compile_entrypoint(&main, &build_out)
            .err()
            .expect("build path should fail for helper type mismatch");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.to_string()).collect();
        let build_messages: Vec<String> = build_errors.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(check_messages, build_messages);
        assert!(check_messages
            .iter()
            .any(|m| m.contains("[helper] return type mismatch")));

        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_frontend_error_messages_match_across_check_build_and_run_paths() {
        let dir = mktemp_dir("frontend_error_mode_parity");
        let main = dir.join("main.sifr");
        let helper = dir.join("helper.sifr");
        std::fs::write(
            &main,
            "from helper import value\n\ndef main():\n    print(value())\n",
        )
        .expect("main file should be written");
        std::fs::write(&helper, "def value() -> int:\n    return \"bad\"\n")
            .expect("helper file should be written");

        let check_errors = check_entrypoint(&main);
        let run_out = mktemp_dir("frontend_parity_run_out");
        let build_out = mktemp_dir("frontend_parity_build_out");
        let run_errors = compile_entrypoint(&main, &run_out)
            .err()
            .expect("run path should fail on helper type error");
        let build_errors = compile_entrypoint(&main, &build_out)
            .err()
            .expect("build path should fail on helper type error");

        let check_messages: Vec<String> = check_errors.into_iter().map(|e| e.to_string()).collect();
        let run_messages: Vec<String> = run_errors.into_iter().map(|e| e.to_string()).collect();
        let build_messages: Vec<String> = build_errors.into_iter().map(|e| e.to_string()).collect();
        assert_eq!(check_messages, run_messages);
        assert_eq!(run_messages, build_messages);

        let _ = std::fs::remove_dir_all(run_out);
        let _ = std::fs::remove_dir_all(build_out);
        let _ = std::fs::remove_dir_all(dir);
    }
}
