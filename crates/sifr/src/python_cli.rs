use super::check_and_package_commands::load_package_graph_context;
use super::cli_model_and_entrypoint::{
    diagnostic_with_code, package_diagnostic, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::{
    current_session_package_id, package_session_for_cwd, render_diagnostics,
};
use clap::{Args, Subcommand};
use serde::Deserialize;
use sifr_diagnostics::DiagnosticCode;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Args)]
pub(crate) struct PythonArgs {
    #[command(subcommand)]
    command: PythonCommands,
}

#[derive(Subcommand)]
enum PythonCommands {
    /// Create or recheck executable Python interop certifications
    Certify(PythonCertifyArgs),
}

#[derive(Args)]
struct PythonCertifyArgs {
    /// Re-run every recorded fixture without modifying the artifact
    #[arg(long)]
    check: bool,
    #[command(subcommand)]
    command: Option<PythonCertifyCommands>,
}

#[derive(Subcommand)]
enum PythonCertifyCommands {
    /// Certify one Arrow C Data Interface producer
    Arrow {
        /// Exact dotted declaration target
        target: String,
        /// Package-local executable Python fixture
        #[arg(long)]
        fixture: PathBuf,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ArrowFixtureEvidence {
    producer_module: String,
    producer_type: String,
    distributions: Vec<sifr_package::ArrowCertifiedDistribution>,
    schema_mode: sifr_package::ArrowCertifiedSchemaMode,
    pointer_identity_verified: bool,
    exact_release_count: u64,
    copy_performed: bool,
}

struct CertificationContext {
    package_root: PathBuf,
    interpreter: PathBuf,
    environment_digest: String,
}

pub(crate) fn cmd_python(args: PythonArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    match args.command {
        PythonCommands::Certify(args) => cmd_certify(args, diagnostic_format),
    }
}

fn cmd_certify(args: PythonCertifyArgs, diagnostic_format: DiagnosticFormat) -> i32 {
    if args.check == args.command.is_some() {
        return fail(
            "use either `sifr python certify --check` or `sifr python certify arrow TARGET --fixture PATH`",
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        );
    }
    let context = match certification_context(diagnostic_format) {
        Ok(context) => context,
        Err(code) => return code,
    };
    if args.check {
        return check_certifications(&context, diagnostic_format);
    }
    let Some(PythonCertifyCommands::Arrow { target, fixture }) = args.command else {
        return EXIT_USAGE_OR_CONFIG;
    };
    certify_arrow(&context, &target, &fixture, diagnostic_format)
}

fn certification_context(diagnostic_format: DiagnosticFormat) -> Result<CertificationContext, i32> {
    let session =
        package_session_for_cwd(sifr_package::CargoLockMode::Normal).map_err(|error| {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            EXIT_USAGE_OR_CONFIG
        })?;
    if session.manifest_less_mode {
        return Err(fail(
            "Python certification requires a Sifr package",
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        ));
    }
    let graph_context = load_package_graph_context(
        &session,
        sifr_package::CargoLockMode::Normal,
        diagnostic_format,
    )?
    .ok_or(EXIT_USAGE_OR_CONFIG)?;
    let package_id =
        current_session_package_id(&session, &graph_context.graph).ok_or(EXIT_USAGE_OR_CONFIG)?;
    let package = graph_context
        .graph
        .packages
        .get(&package_id)
        .ok_or(EXIT_USAGE_OR_CONFIG)?;
    let resolved = sifr_package::resolve_python_environment(&graph_context.graph, &package_id)
        .map_err(|errors| {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            EXIT_USER_DIAGNOSTIC
        })?
        .ok_or_else(|| {
            fail(
                "Python certification requires a root-selected Python environment",
                diagnostic_format,
                EXIT_USER_DIAGNOSTIC,
            )
        })?;
    let request = sifr_package::PythonEnvironmentProbeRequest::from(&resolved);
    let probe = sifr_package::probe_python_environment(&request).map_err(|error| {
        render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
        EXIT_USER_DIAGNOSTIC
    })?;
    let environment_digest = sifr_package::digest_python_environment_probe(&request, &probe).hex;
    Ok(CertificationContext {
        package_root: package.package_root.clone(),
        interpreter: request.interpreter,
        environment_digest,
    })
}

fn certify_arrow(
    context: &CertificationContext,
    target: &str,
    fixture: &Path,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let fixture = if fixture.is_absolute() {
        fixture.to_path_buf()
    } else {
        context.package_root.join(fixture)
    };
    let relative = match fixture.strip_prefix(&context.package_root) {
        Ok(path) if !path.as_os_str().is_empty() => path,
        _ => {
            return fail(
                "Arrow certification fixture must stay inside the package",
                diagnostic_format,
                EXIT_USER_DIAGNOSTIC,
            );
        }
    };
    let evidence = match run_fixture(context, &fixture, target) {
        Ok(evidence) => evidence,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    };
    if let Err(reason) = validate_evidence(context, target, &evidence) {
        return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC);
    }
    let fixture_digest = match sifr_package::arrow_fixture_digest(&fixture) {
        Ok(digest) => digest,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    };
    let certification = sifr_package::ArrowCertification {
        target: target.to_string(),
        fixture: relative.to_string_lossy().replace('\\', "/"),
        fixture_digest,
        producer_module: evidence.producer_module,
        producer_type: evidence.producer_type,
        distributions: evidence.distributions,
        schema_mode: evidence.schema_mode,
        pointer_identity_verified: evidence.pointer_identity_verified,
        exact_release_count: evidence.exact_release_count,
        copy_performed: evidence.copy_performed,
    };
    let artifact_path = context
        .package_root
        .join(sifr_package::PYTHON_CERTIFICATIONS_FILE);
    let mut artifact = if artifact_path.is_file() {
        match sifr_package::load_python_certifications_for_update(
            &context.package_root,
            &context.environment_digest,
            target,
        ) {
            Ok(artifact) => artifact,
            Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
        }
    } else {
        sifr_package::PythonCertificationArtifact {
            schema_version: sifr_package::ARROW_CERTIFICATION_SCHEMA_VERSION,
            environment_digest: context.environment_digest.clone(),
            arrow: Vec::new(),
        }
    };
    artifact.arrow.retain(|existing| existing.target != target);
    artifact.arrow.push(certification);
    artifact
        .arrow
        .sort_by(|left, right| left.target.cmp(&right.target));
    match sifr_package::write_python_certifications(&context.package_root, &artifact) {
        Ok(path) => {
            let _ = writeln!(
                io::stdout(),
                "certified Arrow target '{target}' in {}",
                path.display()
            );
            EXIT_SUCCESS
        }
        Err(reason) => fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    }
}

fn check_certifications(
    context: &CertificationContext,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let artifact = match sifr_package::load_python_certifications(
        &context.package_root,
        &context.environment_digest,
    ) {
        Ok(artifact) => artifact,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
    };
    for certification in &artifact.arrow {
        let fixture =
            match sifr_package::arrow_fixture_path(&context.package_root, &certification.fixture) {
                Ok(path) => path,
                Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
            };
        let evidence = match run_fixture(context, &fixture, &certification.target) {
            Ok(evidence) => evidence,
            Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
        };
        if let Err(reason) = validate_evidence(context, &certification.target, &evidence) {
            return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC);
        }
        if evidence.producer_module != certification.producer_module
            || evidence.producer_type != certification.producer_type
            || evidence.distributions != certification.distributions
            || evidence.schema_mode != certification.schema_mode
            || evidence.pointer_identity_verified != certification.pointer_identity_verified
            || evidence.exact_release_count != certification.exact_release_count
            || evidence.copy_performed != certification.copy_performed
        {
            return fail(
                format!(
                    "Arrow certification evidence changed for '{}'",
                    certification.target
                ),
                diagnostic_format,
                EXIT_USER_DIAGNOSTIC,
            );
        }
    }
    let _ = writeln!(
        io::stdout(),
        "Python certifications: ok ({} Arrow target(s))",
        artifact.arrow.len()
    );
    EXIT_SUCCESS
}

fn run_fixture(
    context: &CertificationContext,
    fixture: &Path,
    target: &str,
) -> Result<ArrowFixtureEvidence, String> {
    let output = Command::new(&context.interpreter)
        .arg("-I")
        .arg(fixture)
        .arg(target)
        .current_dir(&context.package_root)
        .output()
        .map_err(|error| format!("could not execute Arrow certification fixture: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "Arrow certification fixture failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Arrow certification fixture returned invalid JSON: {error}"))
}

fn validate_evidence(
    context: &CertificationContext,
    target: &str,
    evidence: &ArrowFixtureEvidence,
) -> Result<(), String> {
    if target.trim().is_empty()
        || evidence.producer_module.trim().is_empty()
        || evidence.producer_type.trim().is_empty()
    {
        return Err("Arrow certification identities must be non-empty".to_string());
    }
    if evidence.copy_performed || !evidence.pointer_identity_verified {
        return Err(format!(
            "Arrow fixture for '{target}' did not prove a no-copy transfer"
        ));
    }
    if evidence.exact_release_count != 1 {
        return Err(format!(
            "Arrow fixture for '{target}' did not prove exactly one release"
        ));
    }
    if evidence.distributions.is_empty() {
        return Err(format!(
            "Arrow fixture for '{target}' did not report an exact distribution"
        ));
    }
    for distribution in &evidence.distributions {
        let installed = installed_distribution_version(context, &distribution.name)?;
        if installed != distribution.version {
            return Err(format!(
                "Arrow fixture for '{target}' reported distribution '{}=={}', but the selected environment contains '{}=={}'",
                distribution.name, distribution.version, distribution.name, installed
            ));
        }
    }
    Ok(())
}

fn installed_distribution_version(
    context: &CertificationContext,
    distribution: &str,
) -> Result<String, String> {
    let output = Command::new(&context.interpreter)
        .args([
            "-I",
            "-c",
            "import importlib.metadata,sys; print(importlib.metadata.version(sys.argv[1]))",
            distribution,
        ])
        .output()
        .map_err(|error| {
            format!("could not inspect Python distribution '{distribution}': {error}")
        })?;
    if !output.status.success() {
        return Err(format!(
            "Python distribution '{}' is not installed in the selected environment: {}",
            distribution,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn fail(reason: impl Into<String>, diagnostic_format: DiagnosticFormat, code: i32) -> i32 {
    render_diagnostics(
        &[diagnostic_with_code(
            reason,
            DiagnosticCode::PYZC_INVALID_DECLARATION,
        )],
        diagnostic_format,
    );
    code
}
