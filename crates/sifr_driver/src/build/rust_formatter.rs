use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code};
use sifr_diagnostics::DiagnosticCode;
use std::collections::HashSet;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Stdio};

#[cfg(windows)]
const EMPTY_RUSTFMT_CONFIG: &str = "NUL";
#[cfg(not(windows))]
const EMPTY_RUSTFMT_CONFIG: &str = "/dev/null";

pub(crate) fn format_generated_rust(
    source: &str,
    label: &str,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let executable = std::env::var_os("RUSTFMT").unwrap_or_else(|| OsString::from("rustfmt"));
    let canonicalize = |source: &str| {
        sifr_codegen::canonicalize_generated_rust_source(source).map_err(|message| {
            vec![diagnostic_with_code(
                format!("failed to canonicalize generated {label}: {message}"),
                DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
            )]
        })
    };
    let canonical = canonicalize(source)?;
    let formatted =
        format_generated_rust_with(&executable, &canonical, label).map_err(|message| {
            vec![diagnostic_with_code(
                message,
                DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
            )]
        })?;
    let final_canonical = sifr_codegen::finalize_formatted_generated_rust_source(&formatted)
        .map_err(|message| {
            vec![diagnostic_with_code(
                format!("failed to finalize formatted generated {label}: {message}"),
                DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
            )]
        })?;
    format_generated_rust_with(&executable, &final_canonical, label).map_err(|message| {
        vec![diagnostic_with_code(
            message,
            DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
        )]
    })
}

pub(crate) fn discover_project_const_functions<'source>(
    sources: impl IntoIterator<Item = &'source str>,
) -> Result<HashSet<String>, Vec<RenderedDiagnostic>> {
    sifr_codegen::discover_project_const_function_names(sources).map_err(|message| {
        vec![diagnostic_with_code(
            format!("failed to inspect generated project APIs: {message}"),
            DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
        )]
    })
}

pub(crate) fn format_generated_rust_with_project_consts(
    source: &str,
    label: &str,
    project_const_functions: &HashSet<String>,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let finalized = sifr_codegen::finalize_formatted_generated_rust_source_with_project_consts(
        source,
        project_const_functions,
    )
    .map_err(|message| {
        vec![diagnostic_with_code(
            format!("failed to finalize generated project APIs in {label}: {message}"),
            DiagnosticCode::BUILD_RUSTC_OR_CARGO_FAILURE,
        )]
    })?;
    format_generated_rust(&finalized, label)
}

fn format_generated_rust_with(
    executable: &std::ffi::OsStr,
    source: &str,
    label: &str,
) -> Result<String, String> {
    let mut child = Command::new(executable)
        .args([
            "--edition",
            "2024",
            "--emit",
            "stdout",
            "--config-path",
            EMPTY_RUSTFMT_CONFIG,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start rustfmt for generated {label}: {error}"))?;
    let Some(mut stdin) = child.stdin.take() else {
        return Err(format!(
            "failed to open rustfmt input for generated {label}"
        ));
    };

    let (write_result, output_result) = std::thread::scope(|scope| {
        let writer = scope.spawn(move || stdin.write_all(source.as_bytes()));
        let output = child.wait_with_output();
        (writer.join(), output)
    });
    let write_result =
        write_result.map_err(|_| format!("rustfmt input writer failed for generated {label}"))?;
    write_result
        .map_err(|error| format!("failed to write generated {label} to rustfmt: {error}"))?;
    let output = output_result
        .map_err(|error| format!("failed to read rustfmt output for generated {label}: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "rustfmt rejected generated {label} (status {}): {}",
            output.status,
            stderr.trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| {
        format!("rustfmt returned non-UTF-8 output for generated {label}: {error}")
    })
}

#[cfg(test)]
mod tests {
    use super::{format_generated_rust, format_generated_rust_with};

    #[test]
    fn generated_rust_uses_the_canonical_toolchain_layout() {
        let formatted = format_generated_rust("fn value()->i64{let x=1;x}\n", "test.rs")
            .unwrap_or_else(|errors| panic!("formatting must succeed: {errors:?}"));

        assert_eq!(formatted, "const fn value() -> i64 {\n    1\n}\n");
    }

    #[test]
    fn missing_rustfmt_is_a_structured_formatting_failure() {
        let missing = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("__missing_sifr_rustfmt_for_test__");

        let error = format_generated_rust_with(missing.as_os_str(), "fn main() {}", "test.rs")
            .expect_err("a nonexistent rustfmt executable must fail closed");

        assert!(
            error.starts_with("failed to start rustfmt for generated test.rs:"),
            "unexpected formatter error: {error}"
        );
    }

    #[test]
    fn assembled_source_is_canonicalized_before_layout() {
        let formatted =
            format_generated_rust("fn __generated(_value: i64) -> i64 { _value }", "test.rs")
                .unwrap_or_else(|errors| panic!("formatting must succeed: {errors:?}"));

        let parsed = syn::parse_file(&formatted).expect("formatted Rust should parse");
        let syn::Item::Fn(function) = &parsed.items[0] else {
            panic!("expected generated function");
        };
        assert_eq!(function.sig.ident, "sifr_generated_generated");
        let syn::FnArg::Typed(parameter) = &function.sig.inputs[0] else {
            panic!("expected typed generated parameter");
        };
        let syn::Pat::Ident(parameter) = parameter.pat.as_ref() else {
            panic!("expected generated identifier parameter");
        };
        assert_eq!(parameter.ident, "sifr_generated_value");
    }
}
