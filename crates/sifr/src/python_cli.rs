use super::check_and_package_commands::{
    declaration_python_requirements, load_package_graph_context, package_python_runtime,
};
use super::cli_model_and_entrypoint::{
    diagnostic_with_code, package_diagnostic, DiagnosticFormat, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG,
    EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::{
    current_session_package_id, package_session_for_cwd, render_diagnostics,
};
use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use sifr_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};
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
    /// Validate Python interop without changing the package or environment
    Check(PythonInspectArgs),
    /// Diagnose Python interop and print non-applying patch suggestions
    Doctor(PythonInspectArgs),
    /// Create or recheck executable Python interop certifications
    Certify(PythonCertifyArgs),
}

#[derive(Args)]
struct PythonInspectArgs {
    /// Emit the deterministic report as JSON
    #[arg(long)]
    json: bool,
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
    target: String,
    kind: sifr_package::ArrowCertifiedKind,
    producer_module: String,
    producer_type: String,
    distributions: Vec<sifr_package::ArrowCertifiedDistribution>,
    schema_mode: sifr_package::ArrowCertifiedSchemaMode,
    identity_method: sifr_package::ArrowCertifiedIdentityMethod,
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
        PythonCommands::Check(args) => cmd_python_inspect(&args, false, diagnostic_format),
        PythonCommands::Doctor(args) => cmd_python_inspect(&args, true, diagnostic_format),
        PythonCommands::Certify(args) => cmd_certify(args, diagnostic_format),
    }
}

#[derive(Serialize)]
struct PythonInspectionReport {
    schema_version: u32,
    status: &'static str,
    package: String,
    application: bool,
    graph_digest: String,
    source_digest: String,
    lock: &'static str,
    trust: &'static str,
    environment: PythonEnvironmentReport,
    required_imports: Vec<String>,
    declarations: Vec<PythonDeclarationReport>,
    targets: Vec<PythonTargetReport>,
    bridge_packages: usize,
    requires_async_loop: bool,
}

#[derive(Serialize)]
struct PythonEnvironmentReport {
    status: &'static str,
    digest: Option<String>,
}

#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct PythonDeclarationReport {
    module: Option<String>,
    name: String,
    target: Option<String>,
    kind: String,
}

#[derive(Serialize)]
struct PythonTargetReport {
    target: String,
    status: &'static str,
}

#[derive(Serialize)]
struct PythonDoctorReport {
    #[serde(flatten)]
    inspection: PythonInspectionReport,
    suggestions: Vec<PythonDoctorSuggestion>,
}

#[derive(Serialize)]
struct PythonDoctorSuggestion {
    file: &'static str,
    reason: &'static str,
    patch: String,
}

struct PythonReadOnlyContext {
    package_name: String,
    package_id: sifr_package::SifrPackageId,
    graph: sifr_package::SifrPackageGraph,
    source_map: sifr_package::PackageSourceMap,
    runtime: Option<sifr_driver::PackagePythonRuntime>,
    required_imports: Vec<String>,
    application: bool,
    graph_digest: String,
    source_digest: String,
    entrypoints: Vec<PathBuf>,
}

fn cmd_python_inspect(
    args: &PythonInspectArgs,
    doctor: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let context = match python_read_only_context(diagnostic_format) {
        Ok(context) => context,
        Err(code) => return code,
    };
    let inspection = match run_python_read_only_plan(&context) {
        Ok(report) => report,
        Err(diagnostics) => return render_diagnostics(&diagnostics, diagnostic_format),
    };
    if doctor {
        render_python_doctor(inspection, args.json)
    } else {
        render_python_check(&inspection, args.json)
    }
}

fn python_read_only_context(
    diagnostic_format: DiagnosticFormat,
) -> Result<PythonReadOnlyContext, i32> {
    let lock_mode = sifr_package::CargoLockMode::Frozen;
    let session = package_session_for_cwd(lock_mode).map_err(|error| {
        render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
        EXIT_USAGE_OR_CONFIG
    })?;
    if session.manifest_less_mode {
        return Err(fail(
            "`sifr python check` and `sifr python doctor` require a Sifr package",
            diagnostic_format,
            EXIT_USAGE_OR_CONFIG,
        ));
    }
    let application_entrypoints = session.runnable_app_paths().map_err(|error| {
        render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
        EXIT_USER_DIAGNOSTIC
    })?;
    let application = !application_entrypoints.is_empty();
    let graph_context = load_package_graph_context(&session, lock_mode, diagnostic_format)?
        .ok_or(EXIT_USAGE_OR_CONFIG)?;
    let package_id =
        current_session_package_id(&session, &graph_context.graph).ok_or(EXIT_USAGE_OR_CONFIG)?;
    let package = graph_context
        .graph
        .packages
        .get(&package_id)
        .ok_or(EXIT_USAGE_OR_CONFIG)?;
    let mut requirements = declaration_python_requirements(&graph_context.source_map, None);
    let bridge_graph = sifr_package::resolve_python_bridge_graph(&graph_context.graph, &package_id)
        .map_err(|errors| {
            render_diagnostics(
                &errors
                    .into_iter()
                    .map(package_diagnostic)
                    .collect::<Vec<_>>(),
                diagnostic_format,
            );
            EXIT_USER_DIAGNOSTIC
        })?;
    requirements.extend(bridge_graph.requirements);
    requirements.sort();
    requirements.dedup();
    let required_imports = requirements
        .iter()
        .map(|requirement| requirement.root.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let runtime = if application || package.manifest.python.selects_environment() {
        package_python_runtime(
            &graph_context.graph,
            &package_id,
            &requirements,
            diagnostic_format,
        )?
    } else {
        None
    };
    let entrypoints = python_inspection_entrypoints(
        &graph_context.source_map,
        &package_id,
        application_entrypoints,
    );
    let source_digest = sifr_package::digest_package_source_snapshot(&graph_context.source_map)
        .map_err(|error| {
            fail(
                format!("could not read the Python inspection source snapshot: {error}"),
                diagnostic_format,
                EXIT_USAGE_OR_CONFIG,
            )
        })?
        .hex;
    Ok(PythonReadOnlyContext {
        package_name: package.sifr_name.0.clone(),
        package_id,
        graph_digest: sifr_package::digest_package_graph(&graph_context.graph).hex,
        source_digest,
        graph: graph_context.graph,
        source_map: graph_context.source_map,
        runtime,
        required_imports,
        application,
        entrypoints,
    })
}

fn python_inspection_entrypoints(
    source_map: &sifr_package::PackageSourceMap,
    package_id: &sifr_package::SifrPackageId,
    application_entrypoints: Vec<PathBuf>,
) -> Vec<PathBuf> {
    if !application_entrypoints.is_empty() {
        return application_entrypoints;
    }
    source_map
        .modules
        .values()
        .filter(|module| &module.package_id == package_id)
        .map(|module| module.file_path.clone())
        .collect()
}

fn run_python_read_only_plan(
    context: &PythonReadOnlyContext,
) -> Result<PythonInspectionReport, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    let mut declarations = BTreeSet::new();
    let mut targets = BTreeMap::new();
    let mut bridge_packages = 0;
    let mut requires_async_loop = false;
    for entrypoint in &context.entrypoints {
        let report = sifr_driver::check_package_python_interop(&sifr_driver::PackageEntrypoint {
            main_file: entrypoint.clone(),
            package_id: context.package_id.clone(),
            graph: context.graph.clone(),
            source_map: context.source_map.clone(),
            python_runtime: context.runtime.clone(),
        })?;
        declarations.extend(report.declarations.into_iter().map(|declaration| {
            PythonDeclarationReport {
                module: declaration.module_name,
                name: declaration.function_name,
                target: declaration.target,
                kind: declaration.kind,
            }
        }));
        for probe in report.target_probes {
            targets.insert(probe.target, target_status_name(probe.status));
        }
        bridge_packages = bridge_packages.max(report.bridge_package_count);
        requires_async_loop |= report.requires_async_loop;
    }
    let environment = context.runtime.as_ref().map_or_else(
        || PythonEnvironmentReport {
            status: if declarations.is_empty() {
                "not-required"
            } else {
                "deferred"
            },
            digest: None,
        },
        |runtime| PythonEnvironmentReport {
            status: "resolved",
            digest: Some(runtime.environment_digest().to_string()),
        },
    );
    let trust = if context.runtime.is_some() {
        "verified"
    } else if context.required_imports.is_empty() {
        "not-required"
    } else {
        "deferred-to-final-application"
    };
    Ok(PythonInspectionReport {
        schema_version: 1,
        status: "ok",
        package: context.package_name.clone(),
        application: context.application,
        graph_digest: context.graph_digest.clone(),
        source_digest: context.source_digest.clone(),
        lock: "verified-frozen-read-only",
        trust,
        environment,
        required_imports: context.required_imports.clone(),
        declarations: declarations.into_iter().collect(),
        targets: targets
            .into_iter()
            .map(|(target, status)| PythonTargetReport { target, status })
            .collect(),
        bridge_packages,
        requires_async_loop,
    })
}

const fn target_status_name(status: sifr_driver::PythonTargetCheckStatus) -> &'static str {
    match status {
        sifr_driver::PythonTargetCheckStatus::Deferred => "deferred",
        sifr_driver::PythonTargetCheckStatus::Verified => "verified",
        sifr_driver::PythonTargetCheckStatus::RuntimeChecked => "runtime-checked",
    }
}

fn render_python_check(report: &PythonInspectionReport, json: bool) -> i32 {
    if json {
        return write_json(report);
    }
    render_inspection_human(report, "Python check");
    EXIT_SUCCESS
}

fn render_python_doctor(report: PythonInspectionReport, json: bool) -> i32 {
    let suggestions = doctor_suggestions(&report);
    if json {
        return write_json(&PythonDoctorReport {
            inspection: report,
            suggestions,
        });
    }
    render_inspection_human(&report, "Python doctor");
    if suggestions.is_empty() {
        let _ = writeln!(io::stdout(), "suggestions: none");
    } else {
        let _ = writeln!(io::stdout(), "suggestions:");
        for suggestion in suggestions {
            let _ = writeln!(io::stdout(), "  {}: {}", suggestion.file, suggestion.reason);
            for line in suggestion.patch.lines() {
                let _ = writeln!(io::stdout(), "    {line}");
            }
        }
    }
    EXIT_SUCCESS
}

fn render_inspection_human(report: &PythonInspectionReport, title: &str) {
    let _ = writeln!(io::stdout(), "{title}: {}", report.status);
    let _ = writeln!(io::stdout(), "package: {}", report.package);
    let _ = writeln!(
        io::stdout(),
        "snapshot: graph={} source={}",
        report.graph_digest,
        report.source_digest
    );
    let _ = writeln!(io::stdout(), "lock: {}", report.lock);
    let _ = writeln!(io::stdout(), "trust: {}", report.trust);
    let _ = writeln!(io::stdout(), "environment: {}", report.environment.status);
    let _ = writeln!(io::stdout(), "declarations: {}", report.declarations.len());
    let _ = writeln!(io::stdout(), "targets:");
    if report.targets.is_empty() {
        let _ = writeln!(io::stdout(), "  (none)");
    } else {
        for target in &report.targets {
            let _ = writeln!(io::stdout(), "  {}: {}", target.target, target.status);
        }
    }
}

fn doctor_suggestions(report: &PythonInspectionReport) -> Vec<PythonDoctorSuggestion> {
    if report.environment.status != "deferred" {
        return Vec::new();
    }
    let trusted = report
        .required_imports
        .iter()
        .map(|root| serde_json::to_string(root).unwrap_or_else(|_| "\"<import>\"".to_string()))
        .collect::<Vec<_>>()
        .join(", ");
    vec![PythonDoctorSuggestion {
        file: "final-application/sifr.toml",
        reason: "select and trust the Python environment in the final application",
        patch: format!(
            "@@ [python]\n+venv = \".venv\"\n+pyproject = \"pyproject.toml\"\n+lock = \"uv.lock\"\n@@ [trust]\n+python = [{trusted}]"
        ),
    }]
}

fn write_json(value: &impl Serialize) -> i32 {
    match serde_json::to_writer_pretty(io::stdout(), value) {
        Ok(()) => {
            let _ = writeln!(io::stdout());
            EXIT_SUCCESS
        }
        Err(error) => {
            let _ = writeln!(
                io::stderr(),
                "could not serialize Python inspection report: {error}"
            );
            EXIT_USAGE_OR_CONFIG
        }
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
    let mut derived = declaration_python_requirements(&graph_context.source_map, None);
    let bridge_graph = sifr_package::resolve_python_bridge_graph(&graph_context.graph, &package_id)
        .map_err(|errors| {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            EXIT_USER_DIAGNOSTIC
        })?;
    derived.extend(bridge_graph.requirements);
    let resolved = sifr_package::resolve_python_environment_with_requirements(
        &graph_context.graph,
        &package_id,
        &derived,
    )
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
    let (fixture, relative) = match validated_fixture(context, fixture) {
        Ok(value) => value,
        Err(reason) => return fail(reason, diagnostic_format, EXIT_USER_DIAGNOSTIC),
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
        kind: evidence.kind,
        fixture: relative.to_string_lossy().replace('\\', "/"),
        fixture_digest,
        producer_module: evidence.producer_module,
        producer_type: evidence.producer_type,
        distributions: evidence.distributions,
        schema_mode: evidence.schema_mode,
        identity_method: evidence.identity_method,
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

fn validated_fixture(
    context: &CertificationContext,
    fixture: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let candidate = if fixture.is_absolute() {
        fixture.to_path_buf()
    } else {
        context.package_root.join(fixture)
    };
    let relative = candidate
        .strip_prefix(&context.package_root)
        .map_err(|_| "Arrow certification fixture must stay inside the package".to_string())?;
    if relative.as_os_str().is_empty() {
        return Err("Arrow certification fixture must name a package file".to_string());
    }
    let candidate =
        sifr_package::arrow_fixture_path(&context.package_root, &relative.to_string_lossy())?;
    let metadata = std::fs::symlink_metadata(&candidate).map_err(|error| {
        format!(
            "could not inspect fixture '{}': {error}",
            candidate.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "Arrow certification fixture '{}' must be a regular package file",
            candidate.display()
        ));
    }
    let canonical_root = context
        .package_root
        .canonicalize()
        .map_err(|error| format!("could not resolve package root: {error}"))?;
    let canonical_fixture = candidate
        .canonicalize()
        .map_err(|error| format!("could not resolve fixture: {error}"))?;
    if !canonical_fixture.starts_with(canonical_root) {
        return Err("Arrow certification fixture must stay inside the package".to_string());
    }
    Ok((candidate, relative.to_path_buf()))
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
            || evidence.target != certification.target
            || evidence.kind != certification.kind
            || evidence.producer_type != certification.producer_type
            || evidence.distributions != certification.distributions
            || evidence.schema_mode != certification.schema_mode
            || evidence.identity_method != certification.identity_method
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
        || evidence.target != target
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
