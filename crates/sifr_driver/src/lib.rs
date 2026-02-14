//! Sifr Compiler Driver
//!
//! Orchestrates the full compilation pipeline:
//! parse -> type-check/HIR -> codegen -> build

use sifr_python_parser::parse_module;
use sifr_hir::lower_module;
use sifr_codegen::{generate_rust, generate_project};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of compilation.
#[derive(Debug)]
pub enum CompileResult {
    /// Compilation succeeded, contains generated Rust source.
    Success {
        rust_source: String,
    },
    /// Compilation failed with errors.
    Errors {
        errors: Vec<CompileError>,
    },
}

/// A compilation error with location info.
#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub phase: CompilePhase,
}

#[derive(Debug, Clone)]
pub enum CompilePhase {
    Parse,
    TypeCheck,
    Codegen,
    Build,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let phase = match self.phase {
            CompilePhase::Parse => "parse error",
            CompilePhase::TypeCheck => "type error",
            CompilePhase::Codegen => "codegen error",
            CompilePhase::Build => "build error",
        };
        write!(f, "{}: {}", phase, self.message)
    }
}

/// Compile Sifr source code to Rust source code.
pub fn compile(source: &str) -> CompileResult {
    // Phase 1: Parse
    let parsed = match parse_module(source) {
        Ok(parsed) => {
            if !parsed.is_valid() {
                let errors: Vec<CompileError> = parsed
                    .errors()
                    .iter()
                    .map(|e| CompileError {
                        message: format!("{}", e),
                        phase: CompilePhase::Parse,
                    })
                    .collect();
                return CompileResult::Errors { errors };
            }
            parsed
        }
        Err(e) => {
            return CompileResult::Errors {
                errors: vec![CompileError {
                    message: format!("failed to parse: {}", e),
                    phase: CompilePhase::Parse,
                }],
            };
        }
    };

    // Phase 2: Lower to HIR (type checking + name resolution)
    let lowering_result = match lower_module(parsed.suite()) {
        Ok(result) => result,
        Err(errors) => {
            let compile_errors: Vec<CompileError> = errors
                .into_iter()
                .map(|e| CompileError {
                    message: e.message,
                    phase: CompilePhase::TypeCheck,
                })
                .collect();
            return CompileResult::Errors {
                errors: compile_errors,
            };
        }
    };

    // Print reveal_type diagnostics to stderr
    for diag in &lowering_result.reveal_types {
        eprintln!("{}", diag);
    }

    // Phase 3: Generate Rust code
    let rust_source = generate_rust(&lowering_result.module);

    CompileResult::Success { rust_source }
}

/// Type-check only (no code generation).
pub fn check(source: &str) -> Vec<CompileError> {
    match compile(source) {
        CompileResult::Success { .. } => vec![],
        CompileResult::Errors { errors } => errors,
    }
}

/// Compile and build a native binary.
pub fn build(source: &str, output_dir: &Path) -> Result<PathBuf, Vec<CompileError>> {
    let rust_source = match compile(source) {
        CompileResult::Success { rust_source } => rust_source,
        CompileResult::Errors { errors } => return Err(errors),
    };

    // Create a temporary Rust project
    let project_dir = output_dir.join("sifr_output");
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| {
        vec![CompileError {
            message: format!("failed to create output directory: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write Cargo.toml
    let (cargo_toml, _) = generate_project(
        &sifr_hir::HirModule { functions: vec![], classes: vec![] },
        "sifr_output",
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write Cargo.toml: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Write main.rs
    std::fs::write(src_dir.join("main.rs"), &rust_source).map_err(|e| {
        vec![CompileError {
            message: format!("failed to write main.rs: {}", e),
            phase: CompilePhase::Build,
        }]
    })?;

    // Run cargo build
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&project_dir)
        .output()
        .map_err(|e| {
            vec![CompileError {
                message: format!("failed to run cargo build: {}", e),
                phase: CompilePhase::Build,
            }]
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(vec![CompileError {
            message: format!("cargo build failed:\n{}", stderr),
            phase: CompilePhase::Build,
        }]);
    }

    // Return path to the built binary
    let binary_name = if cfg!(target_os = "windows") {
        "sifr_output.exe"
    } else {
        "sifr_output"
    };
    let binary_path = project_dir
        .join("target")
        .join("release")
        .join(binary_name);

    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_hello_world() {
        let source = r#"
def main():
    print("Hello, World!")
"#;
        match compile(source) {
            CompileResult::Success { rust_source } => {
                assert!(rust_source.contains("fn main()"));
                assert!(rust_source.contains("println!"));
                assert!(rust_source.contains("Hello, World!"));
            }
            CompileResult::Errors { errors } => {
                panic!("compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_compile_factorial() {
        let source = r#"
def factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)

def main():
    x: int = factorial(5)
    print(x)
"#;
        match compile(source) {
            CompileResult::Success { rust_source } => {
                assert!(rust_source.contains("fn factorial(n: i64) -> i64"));
                assert!(rust_source.contains("fn main()"));
            }
            CompileResult::Errors { errors } => {
                panic!("compilation failed: {:?}", errors);
            }
        }
    }

    #[test]
    fn test_type_mismatch_error() {
        let source = r#"
def main():
    x: int = "hello"
"#;
        let errors = check(source);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.message.contains("type mismatch")));
    }

    #[test]
    fn test_check_valid_program() {
        let source = r#"
def main():
    x: int = 42
    print(x)
"#;
        let errors = check(source);
        assert!(errors.is_empty());
    }
}
