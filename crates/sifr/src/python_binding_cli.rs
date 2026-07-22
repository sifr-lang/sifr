use crate::cli_model_and_entrypoint::{
    diagnostic_with_code, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC,
};
use crate::diagnostic_rendering_and_run::render_diagnostics;
use crate::python_runtime_context::package_python_authoring_context;
use clap::Args;
use serde::Serialize;
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const BINDING_PROBE: &str = include_str!("python_binding_probe.py");

#[derive(Args)]
pub(crate) struct PythonBindArgs {
    /// Python module containing the selected symbols
    module: Option<String>,
    /// Comma-separated symbols to author; whole-module generation is forbidden
    #[arg(long, value_delimiter = ',')]
    symbols: Vec<String>,
    /// Package-local checked-in Sifr output path
    #[arg(long)]
    output: Option<PathBuf>,
    /// Package-local .pyi override, highest precedence (repeatable)
    #[arg(long = "override")]
    overrides: Vec<PathBuf>,
    /// Installed stub-only distribution to consult (repeatable)
    #[arg(long = "stub-package")]
    stub_packages: Vec<String>,
    /// Package-local external .pyi source (repeatable)
    #[arg(long = "external-stub")]
    external_stubs: Vec<PathBuf>,
    /// Re-resolve and compare every recorded binding without writing
    #[arg(long)]
    check: bool,
}

#[derive(Serialize)]
struct ProbeConfig<'a> {
    module: &'a str,
    symbols: &'a [String],
    overrides: &'a [String],
    stub_packages: &'a [String],
    external_stubs: &'a [String],
}

pub(crate) fn cmd_bind(args: &PythonBindArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    if args.check {
        if args.module.is_some()
            || !args.symbols.is_empty()
            || args.output.is_some()
            || !args.overrides.is_empty()
            || !args.stub_packages.is_empty()
            || !args.external_stubs.is_empty()
        {
            return fail(
                "`sifr python bind --check` does not accept generation arguments",
                diagnostic_format,
                EXIT_USAGE_OR_CONFIG,
            );
        }
        return check_bindings(diagnostic_format);
    }
    let Some(ref module) = args.module else {
        return fail(
            "`sifr python bind` requires a module and `--symbols`",
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        );
    };
    if args.symbols.is_empty() {
        return fail(
            "`sifr python bind` requires at least one explicit `--symbols` entry",
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        );
    }
    generate_binding(module, args, diagnostic_format)
}

fn generate_binding(
    module: &str,
    args: &PythonBindArgs,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let import_root = module.split('.').next().unwrap_or(module).to_string();
    let context = match package_python_authoring_context(
        sifr_package::CargoLockMode::Normal,
        &[import_root],
        diagnostic_format,
    ) {
        Ok(context) => context,
        Err(code) => return code,
    };
    let symbols = match normalized_symbols(&args.symbols) {
        Ok(symbols) => symbols,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USAGE_OR_CONFIG),
    };
    let overrides = match package_typing_sources(&context.package_root, &args.overrides) {
        Ok(sources) => sources,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USAGE_OR_CONFIG),
    };
    let external_stubs = match package_typing_sources(&context.package_root, &args.external_stubs) {
        Ok(sources) => sources,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USAGE_OR_CONFIG),
    };
    let mut stub_packages = args.stub_packages.clone();
    stub_packages.sort();
    stub_packages.dedup();
    if stub_packages
        .iter()
        .any(|package| package.trim().is_empty())
    {
        return fail(
            "Python stub-package names must be non-empty",
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        );
    }
    let output_relative = args.output.clone().unwrap_or_else(|| {
        PathBuf::from("src").join(format!("{}_python.sifr", module.replace('.', "_")))
    });
    let output =
        match sifr_package::safe_python_binding_output(&context.package_root, &output_relative) {
            Ok(output) => output,
            Err(reason) => return fail(reason, diagnostic_format, EXIT_USAGE_OR_CONFIG),
        };
    let probe = match run_probe(
        &context,
        module,
        &symbols,
        &overrides,
        &stub_packages,
        &external_stubs,
    ) {
        Ok(probe) => probe,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    };
    let scaffold = match sifr_driver::render_python_binding_scaffold(module, &symbols, &probe) {
        Ok(scaffold) => scaffold,
        Err(errors) => return fail(errors.join("\n"), diagnostic_format, EXIT_USER_DIAGNOSTIC),
    };
    let artifact_path = context
        .package_root
        .join(sifr_package::PYTHON_BINDINGS_FILE);
    let mut artifact = if artifact_path.is_file() {
        match sifr_package::load_python_bindings_for_update(&context.package_root, module) {
            Ok(mut artifact) => {
                artifact.environment_digest =
                    context.runtime.authoring_environment_digest().to_string();
                artifact
            }
            Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
        }
    } else {
        sifr_package::PythonBindingArtifact {
            schema_version: sifr_package::PYTHON_BINDING_SCHEMA_VERSION,
            environment_digest: context.runtime.authoring_environment_digest().to_string(),
            bindings: Vec::new(),
        }
    };
    if let Some(parent) = output.parent() {
        if let Err(error) = std::fs::create_dir_all(parent) {
            return fail(
                format!("could not create Python binding output directory: {error}"),
                diagnostic_format,
                EXIT_USAGE_OR_CONFIG,
            );
        }
    }
    if let Err(error) = std::fs::write(&output, scaffold.source.as_bytes()) {
        return fail(
            format!(
                "could not write Python binding '{}': {error}",
                output.display()
            ),
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        );
    }
    artifact.bindings.push(sifr_package::PythonBinding {
        module: module.to_string(),
        symbols,
        output: output_relative.to_string_lossy().replace('\\', "/"),
        soabi: probe.soabi,
        distribution: probe.distribution,
        overrides: relative_strings(&context.package_root, &overrides),
        stub_packages,
        external_stubs: relative_strings(&context.package_root, &external_stubs),
        sources: scaffold.sources,
        source_fingerprint: scaffold.source_fingerprint,
        generated_digest: sifr_package::python_binding_generated_digest(scaffold.source.as_bytes()),
    });
    artifact
        .bindings
        .sort_by(|left, right| left.module.cmp(&right.module));
    match sifr_package::write_python_bindings(&context.package_root, &artifact) {
        Ok(path) => {
            let _ = writeln!(
                io::stdout(),
                "authored {} Python symbol(s) from '{module}' in {} (metadata: {})",
                artifact
                    .bindings
                    .iter()
                    .find(|binding| binding.module == module)
                    .map_or(0, |binding| binding.symbols.len()),
                output.display(),
                path.display()
            );
            EXIT_SUCCESS
        }
        Err(reason) => fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    }
}

fn check_bindings(diagnostic_format: DiagnosticFormat) -> i32 {
    let context = match package_python_authoring_context(
        sifr_package::CargoLockMode::Frozen,
        &[],
        diagnostic_format,
    ) {
        Ok(context) => context,
        Err(code) => return code,
    };
    let artifact = match sifr_package::load_python_bindings(
        &context.package_root,
        context.runtime.authoring_environment_digest(),
    ) {
        Ok(artifact) => artifact,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    };
    for binding in &artifact.bindings {
        let overrides = absolute_sources(&context.package_root, &binding.overrides);
        let external_stubs = absolute_sources(&context.package_root, &binding.external_stubs);
        let probe = match run_probe(
            &context,
            &binding.module,
            &binding.symbols,
            &overrides,
            &binding.stub_packages,
            &external_stubs,
        ) {
            Ok(probe) => probe,
            Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
        };
        let scaffold = match sifr_driver::render_python_binding_scaffold(
            &binding.module,
            &binding.symbols,
            &probe,
        ) {
            Ok(scaffold) => scaffold,
            Err(errors) => return fail(errors.join("\n"), diagnostic_format, EXIT_USER_DIAGNOSTIC),
        };
        if probe.soabi != binding.soabi
            || probe.distribution != binding.distribution
            || scaffold.sources != binding.sources
            || scaffold.source_fingerprint != binding.source_fingerprint
            || sifr_package::python_binding_generated_digest(scaffold.source.as_bytes())
                != binding.generated_digest
        {
            return fail(
                format!(
                    "Python binding source or environment drifted for module '{}'",
                    binding.module
                ),
                diagnostic_format,
                EXIT_USER_DIAGNOSTIC,
            );
        }
    }
    let symbol_count = artifact
        .bindings
        .iter()
        .map(|binding| binding.symbols.len())
        .sum::<usize>();
    let _ = writeln!(
        io::stdout(),
        "Python bindings: ok ({} module(s), {symbol_count} symbol(s))",
        artifact.bindings.len()
    );
    EXIT_SUCCESS
}

fn run_probe(
    context: &crate::python_runtime_context::PythonAuthoringContext,
    module: &str,
    symbols: &[String],
    overrides: &[String],
    stub_packages: &[String],
    external_stubs: &[String],
) -> Result<sifr_driver::PythonBindingProbeReport, String> {
    let config = serde_json::to_string(&ProbeConfig {
        module,
        symbols,
        overrides,
        stub_packages,
        external_stubs,
    })
    .map_err(|error| format!("could not serialize Python binding probe request: {error}"))?;
    let output = Command::new(context.runtime.interpreter())
        .args(["-I", "-c", BINDING_PROBE, &config])
        .current_dir(&context.package_root)
        .output()
        .map_err(|error| format!("could not execute Python binding probe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Python binding probe failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Python binding probe returned invalid JSON: {error}"))
}

fn normalized_symbols(symbols: &[String]) -> Result<Vec<String>, String> {
    let mut normalized = symbols
        .iter()
        .map(|symbol| symbol.trim().to_string())
        .collect::<Vec<_>>();
    if normalized.iter().any(String::is_empty) {
        return Err("Python binding symbols must be non-empty".to_string());
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn package_typing_sources(package_root: &Path, sources: &[PathBuf]) -> Result<Vec<String>, String> {
    sources
        .iter()
        .map(|source| {
            let candidate = if source.is_absolute() {
                source.clone()
            } else {
                package_root.join(source)
            };
            let metadata = std::fs::symlink_metadata(&candidate).map_err(|error| {
                format!(
                    "could not inspect typing source '{}': {error}",
                    candidate.display()
                )
            })?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(format!(
                    "Python typing source '{}' must be a regular package file",
                    candidate.display()
                ));
            }
            let canonical_root = package_root
                .canonicalize()
                .map_err(|error| format!("could not resolve package root: {error}"))?;
            let canonical = candidate.canonicalize().map_err(|error| {
                format!(
                    "could not resolve typing source '{}': {error}",
                    candidate.display()
                )
            })?;
            let relative = canonical.strip_prefix(&canonical_root).map_err(|_| {
                format!(
                    "Python typing source '{}' must stay inside the package",
                    candidate.display()
                )
            })?;
            if relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            {
                return Err("Python typing source must have a normal package path".to_string());
            }
            Ok(canonical.to_string_lossy().into_owned())
        })
        .collect()
}

fn relative_strings(package_root: &Path, absolute: &[String]) -> Vec<String> {
    let canonical_root = package_root
        .canonicalize()
        .unwrap_or_else(|_| package_root.to_path_buf());
    absolute
        .iter()
        .filter_map(|path| {
            Path::new(path)
                .strip_prefix(&canonical_root)
                .ok()
                .map(Path::to_path_buf)
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        })
        .collect()
}

fn absolute_sources(package_root: &Path, relative: &[String]) -> Vec<String> {
    relative
        .iter()
        .map(|path| package_root.join(path).to_string_lossy().into_owned())
        .collect()
}

fn fail(reason: impl Into<String>, diagnostic_format: DiagnosticFormat, code: i32) -> i32 {
    render_diagnostics(
        &[diagnostic_with_code(
            reason,
            sifr_diagnostics::DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE,
        )],
        diagnostic_format,
    );
    code
}
