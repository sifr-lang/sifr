use super::build_output::{render_build_success, BuildOutputOptions};
use super::check_and_package_commands::{
    bounded_excerpt, build_run_artifact, cmd_check, compile_entrypoint_report,
    compile_package_entrypoint_report, load_package_graph_context, package_compiler_context,
    paths_equal, redacted_args,
};
use super::cli_model_and_entrypoint::{
    diagnostic_exit_code, diagnostic_with_code, package_diagnostic, run_with_panic_boundary,
    DiagnosticFormat, PackageGraphContext, EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS,
    EXIT_USAGE_OR_CONFIG, EXIT_USER_DIAGNOSTIC, SIFR_BUILD_VERSION,
};
use super::workspace_run_selection::resolve_run_session;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_driver::{
    apply_diagnostic_recovery_limits, build_cached_package_project, CachedBinaryArtifact,
    PackageEntrypoint,
};
use std::collections::BTreeSet;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process;

pub(super) fn canonical_diagnostic_stream(
    errors: &[RenderedDiagnostic],
) -> Vec<RenderedDiagnostic> {
    apply_diagnostic_recovery_limits(errors)
}

pub(super) fn render_diagnostic_stream(
    diagnostics: &[RenderedDiagnostic],
    format: DiagnosticFormat,
) -> Result<String, serde_json::Error> {
    let mut output = String::new();
    match format {
        DiagnosticFormat::Human => {
            output.push_str(&sifr_diagnostics::render_human_diagnostics(diagnostics));
        }
        DiagnosticFormat::Json => {
            output.push_str(&sifr_diagnostics::render_json_diagnostics(diagnostics)?);
        }
        DiagnosticFormat::Compact => {
            output.push_str(&sifr_diagnostics::render_compact_diagnostics(diagnostics));
        }
    }
    Ok(output)
}

pub(super) fn render_diagnostic_output(
    errors: &[RenderedDiagnostic],
    format: DiagnosticFormat,
) -> Result<String, serde_json::Error> {
    let diagnostics = canonical_diagnostic_stream(errors);
    render_diagnostic_stream(&diagnostics, format)
}

pub(crate) fn render_diagnostics(errors: &[RenderedDiagnostic], format: DiagnosticFormat) -> i32 {
    match render_diagnostic_output(errors, format) {
        Ok(output) => {
            let _ = write!(io::stderr(), "{output}");
        }
        Err(e) => {
            let _ = writeln!(
                io::stderr(),
                "build error: failed to serialize diagnostics as json: {e}"
            );
            return EXIT_INTERNAL_COMPILER_FAILURE;
        }
    }
    diagnostic_exit_code(errors)
}

pub(super) fn cmd_build(
    file: &Path,
    output: &Path,
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    match package_session_for_cwd(sifr_package::CargoLockMode::Normal) {
        Ok(session) if !session.manifest_less_mode => {
            match compile_package_entrypoint_report(
                file,
                output,
                &session,
                sifr_package::CargoLockMode::Normal,
                diagnostic_format,
            ) {
                Ok(Some(report)) => return emit_build_result(&report, quiet, diagnostic_format),
                Ok(None) => {}
                Err(exit_code) => return exit_code,
            }
        }
        Ok(_) => {}
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    }

    let result = match run_with_panic_boundary(
        "internal compiler panic during build command execution",
        || compile_entrypoint_report(file, output),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(report) => emit_build_result(&report, quiet, diagnostic_format),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn emit_build_result(
    report: &sifr_driver::BuildReport,
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostic_exit = emit_report_frontend_diagnostics(report, diagnostic_format);
    if diagnostic_exit != EXIT_SUCCESS {
        return diagnostic_exit;
    }
    emit_build_report(report, quiet, true, diagnostic_format);
    EXIT_SUCCESS
}

fn emit_report_frontend_diagnostics(
    report: &sifr_driver::BuildReport,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    if report.frontend_diagnostics().is_empty() {
        return EXIT_SUCCESS;
    }
    render_diagnostics(report.frontend_diagnostics(), diagnostic_format)
}

fn emit_build_report(
    report: &sifr_driver::BuildReport,
    quiet: bool,
    include_binary: bool,
    diagnostic_format: DiagnosticFormat,
) {
    if diagnostic_format != DiagnosticFormat::Human {
        return;
    }
    let rendered = render_build_success(
        report,
        &BuildOutputOptions {
            version: SIFR_BUILD_VERSION,
            quiet,
            include_binary,
        },
    );
    let _ = write!(io::stderr(), "{rendered}");
}

#[cfg(test)]
pub(super) fn cmd_run(
    target: Option<&str>,
    bin: Option<&str>,
    script: Option<&str>,
    packages: &[String],
    app_args: &[String],
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let options = RunCommandOptions {
        target,
        bin,
        script,
        packages,
        app_args,
        lock_mode,
        quiet: false,
        diagnostic_format,
    };
    cmd_run_with_options(&options)
}

pub(super) struct RunCommandOptions<'a> {
    pub(super) target: Option<&'a str>,
    pub(super) bin: Option<&'a str>,
    pub(super) script: Option<&'a str>,
    pub(super) packages: &'a [String],
    pub(super) app_args: &'a [String],
    pub(super) lock_mode: sifr_package::CargoLockMode,
    pub(super) quiet: bool,
    pub(super) diagnostic_format: DiagnosticFormat,
}

pub(super) fn cmd_run_with_options(options: &RunCommandOptions<'_>) -> i32 {
    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(error) => {
            let diagnostic = diagnostic_with_code(
                format!("could not read current directory: {error}"),
                DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED,
            );
            render_diagnostics(&[diagnostic], options.diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let session =
        match sifr_package::PackageSession::discover(sifr_package::PackageSessionOptions {
            current_dir,
            lock_mode: options.lock_mode,
        }) {
            Ok(session) => session,
            Err(error) => {
                render_diagnostics(&[package_diagnostic(error)], options.diagnostic_format);
                return EXIT_USAGE_OR_CONFIG;
            }
        };
    let session = match resolve_run_session(
        session,
        options.target,
        options.packages,
        options.lock_mode,
        options.diagnostic_format,
    ) {
        Ok(session) => session,
        Err(exit_code) => return exit_code,
    };
    cmd_run_with_session(&RunSessionRequest {
        session: &session,
        target: options.target,
        bin: options.bin,
        script: options.script,
        app_args: options.app_args,
        lock_mode: options.lock_mode,
        quiet: options.quiet,
        diagnostic_format: options.diagnostic_format,
        script_depth: 0,
    })
}

struct RunSessionRequest<'a> {
    session: &'a sifr_package::PackageSession,
    target: Option<&'a str>,
    bin: Option<&'a str>,
    script: Option<&'a str>,
    app_args: &'a [String],
    lock_mode: sifr_package::CargoLockMode,
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
    script_depth: u8,
}

fn cmd_run_with_session(request: &RunSessionRequest<'_>) -> i32 {
    let session = request.session;
    let plan = session.plan_run(&sifr_package::PackageRunRequest {
        target_or_path: request.target.map(str::to_string),
        bin: request.bin.map(str::to_string),
        script: request.script.map(str::to_string),
        app_args: request.app_args.to_vec(),
        script_depth: request.script_depth,
    });
    let plan = match plan {
        Ok(plan) => plan,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], request.diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if let Some(origin) = plan.script_origin {
        return cmd_script(
            &origin,
            Some(session),
            request.lock_mode,
            request.quiet,
            request.diagnostic_format,
        );
    }
    if let Some(
        sifr_package::ResolvedRunTarget::File(path)
        | sifr_package::ResolvedRunTarget::App { path, .. },
    ) = plan.run_target
    {
        if !session.manifest_less_mode {
            return cmd_run_package_file(
                &path,
                session,
                request.lock_mode,
                request.app_args,
                request.quiet,
                request.diagnostic_format,
            );
        }
        return cmd_run_file(
            &path,
            request.app_args,
            request.quiet,
            request.diagnostic_format,
        );
    }
    if let Some(cargo) = plan.cargo {
        return execute_cargo_plan(&cargo, request.lock_mode, request.diagnostic_format);
    }
    EXIT_SUCCESS
}

pub(super) fn cmd_script(
    origin: &sifr_package::ScriptOrigin,
    run_session: Option<&sifr_package::PackageSession>,
    lock_mode: sifr_package::CargoLockMode,
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    match origin.command.as_str() {
        "run" => {
            if let Some(session) = run_session {
                cmd_run_with_session(&RunSessionRequest {
                    session,
                    target: origin.args.first().map(String::as_str),
                    bin: None,
                    script: None,
                    app_args: &[],
                    lock_mode,
                    quiet,
                    diagnostic_format,
                    script_depth: 1,
                })
            } else {
                let options = RunCommandOptions {
                    target: origin.args.first().map(String::as_str),
                    bin: None,
                    script: None,
                    packages: &[],
                    app_args: &[],
                    lock_mode,
                    quiet,
                    diagnostic_format,
                };
                cmd_run_with_options(&options)
            }
        }
        "check" => cmd_check(
            origin
                .args
                .first()
                .filter(|arg| !arg.starts_with('-'))
                .map(Path::new),
            None,
            &sifr_package::CargoPackageSelection::default(),
            lock_mode,
            diagnostic_format,
        ),
        "fetch" => cmd_fetch(lock_mode, diagnostic_format),
        "tree" => cmd_tree(lock_mode, &origin.args, diagnostic_format),
        "package" => cmd_package(
            &sifr_package::CargoPackageSelection::default(),
            &sifr_package::CargoPackageArchiveOptions::default(),
            lock_mode,
            diagnostic_format,
        ),
        "publish" => cmd_publish(
            &sifr_package::CargoPackageSelection::default(),
            &sifr_package::CargoPublishOptions {
                dry_run: origin.args.iter().any(|arg| arg == "--dry-run"),
                ..sifr_package::CargoPublishOptions::default()
            },
            lock_mode,
            diagnostic_format,
        ),
        "vendor" => cmd_vendor(
            origin
                .args
                .first()
                .map(Path::new)
                .unwrap_or_else(|| Path::new("vendor")),
            &sifr_package::CargoVendorOptions::default(),
            lock_mode,
            diagnostic_format,
        ),
        _ => {
            let diagnostic = diagnostic_with_code(
                format!("unsupported package script command '{}'", origin.command),
                DiagnosticCode::PACKAGE_SCRIPT_RECURSION,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            EXIT_USAGE_OR_CONFIG
        }
    }
}

pub(super) fn cmd_run_file(
    file: &Path,
    app_args: &[String],
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during run command compilation",
        || build_run_artifact(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(artifact) => {
            let diagnostic_exit =
                emit_report_frontend_diagnostics(artifact.build_report(), diagnostic_format);
            if diagnostic_exit != EXIT_SUCCESS {
                return diagnostic_exit;
            }
            if !quiet && !artifact.build_report().cache_hit() {
                emit_build_report(artifact.build_report(), false, false, diagnostic_format);
            }
            let output = std::process::Command::new(artifact.binary_path())
                .args(app_args)
                .output()
                .unwrap_or_else(|e| {
                    let _ = writeln!(io::stderr(), "error: could not run binary: {e}");
                    process::exit(EXIT_USAGE_OR_CONFIG);
                });

            // Forward stdout and stderr
            std::io::stdout().write_all(&output.stdout).ok();
            std::io::stderr().write_all(&output.stderr).ok();

            if !output.status.success() {
                return EXIT_USER_DIAGNOSTIC;
            }
            EXIT_SUCCESS
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

pub(super) fn cmd_run_package_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    app_args: &[String],
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let context =
        match package_compiler_context(session, lock_mode, diagnostic_format, Some(file), false) {
            Ok(Some(context)) => context,
            Ok(None) => return cmd_run_file(file, app_args, quiet, diagnostic_format),
            Err(exit_code) => return exit_code,
        };
    let entrypoint = PackageEntrypoint {
        main_file: file.to_path_buf(),
        package_id: context.package_id,
        graph: context.graph,
        source_map: context.source_map,
        python_runtime: context.python_runtime,
    };
    let result = match run_with_panic_boundary(
        "internal compiler panic during package run command compilation",
        || build_cached_package_project(&entrypoint),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(artifact) => run_binary_artifact(&artifact, app_args, quiet, diagnostic_format),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

pub(super) fn run_binary_artifact(
    artifact: &CachedBinaryArtifact,
    app_args: &[String],
    quiet: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostic_exit =
        emit_report_frontend_diagnostics(artifact.build_report(), diagnostic_format);
    if diagnostic_exit != EXIT_SUCCESS {
        return diagnostic_exit;
    }
    if !quiet && !artifact.build_report().cache_hit() {
        emit_build_report(artifact.build_report(), false, false, diagnostic_format);
    }
    let output = std::process::Command::new(artifact.binary_path())
        .args(app_args)
        .output()
        .unwrap_or_else(|e| {
            let _ = writeln!(io::stderr(), "error: could not run binary: {e}");
            process::exit(EXIT_USAGE_OR_CONFIG);
        });

    std::io::stdout().write_all(&output.stdout).ok();
    std::io::stderr().write_all(&output.stderr).ok();

    if output.status.success() {
        EXIT_SUCCESS
    } else {
        EXIT_USER_DIAGNOSTIC
    }
}

pub(super) fn cmd_fetch(
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let Some(cargo) = session.plan_fetch().cargo else {
        return EXIT_SUCCESS;
    };
    let exit = execute_cargo_plan(&cargo, lock_mode, diagnostic_format);
    if exit == EXIT_SUCCESS {
        let _ = writeln!(io::stderr(), "{}", fetch_success_message());
    }
    exit
}

pub(super) const fn fetch_success_message() -> &'static str {
    "fetched package dependencies successfully"
}

pub(super) fn cmd_tree(
    lock_mode: sifr_package::CargoLockMode,
    args: &[String],
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let Some(cargo) = session.plan_tree(args).cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

pub(super) fn cmd_package(
    selection: &sifr_package::CargoPackageSelection,
    options: &sifr_package::CargoPackageArchiveOptions,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if let Err(exit_code) = run_package_release_preflight(
        &session,
        selection,
        options.allow_dirty,
        lock_mode,
        diagnostic_format,
    ) {
        return exit_code;
    }
    let plan = session.plan_package(
        &sifr_package::CargoFeatureSelection::default(),
        selection,
        options,
    );
    let Some(cargo) = plan.cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

pub(super) fn cmd_publish(
    selection: &sifr_package::CargoPackageSelection,
    options: &sifr_package::CargoPublishOptions,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if let Err(exit_code) = run_package_release_preflight(
        &session,
        selection,
        options.allow_dirty,
        lock_mode,
        diagnostic_format,
    ) {
        return exit_code;
    }
    let plan = session.plan_publish(
        &sifr_package::CargoFeatureSelection::default(),
        selection,
        options,
    );
    let Some(cargo) = plan.cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

pub(super) fn cmd_vendor(
    output_dir: &Path,
    options: &sifr_package::CargoVendorOptions,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let session = match package_session_for_cwd(lock_mode) {
        Ok(session) => session,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let plan = session.plan_vendor(output_dir, options);
    let Some(cargo) = plan.cargo else {
        return EXIT_SUCCESS;
    };
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

pub(super) fn package_session_for_cwd(
    lock_mode: sifr_package::CargoLockMode,
) -> Result<sifr_package::PackageSession, sifr_package::PackageDiagnostic> {
    let current_dir = std::env::current_dir().map_err(|error| {
        sifr_package::PackageDiagnostic::cargo_command_failed(
            sifr_package::CargoAction::Metadata,
            format!("could not read current directory: {error}"),
        )
    })?;
    sifr_package::PackageSession::discover(sifr_package::PackageSessionOptions {
        current_dir,
        lock_mode,
    })
}

pub(super) fn run_package_release_preflight(
    session: &sifr_package::PackageSession,
    selection: &sifr_package::CargoPackageSelection,
    allow_dirty: bool,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<(), i32> {
    let Some(context) = load_package_graph_context(session, lock_mode, diagnostic_format)? else {
        return Ok(());
    };
    let package_ids = selected_release_package_ids(&context, session, selection)
        .map_err(|diagnostics| render_package_diagnostics(diagnostics, diagnostic_format))?;
    if let Err(diagnostics) = sifr_package::validate_backend_trust(&context.graph) {
        return Err(render_package_diagnostics(diagnostics, diagnostic_format));
    }
    let mut diagnostics = Vec::new();
    for package_id in package_ids {
        let Some(package) = context.graph.packages.get(&package_id) else {
            continue;
        };
        let entries = cargo_package_list_entries(
            session,
            lock_mode,
            &package.cargo_package_name,
            allow_dirty,
            diagnostic_format,
        )?;
        if let Err(errors) =
            sifr_package::validate_package_archive(package, &context.source_map, &entries)
        {
            diagnostics.extend(errors);
        }
    }
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(render_package_diagnostics(diagnostics, diagnostic_format))
    }
}

pub(super) fn selected_release_package_ids(
    context: &PackageGraphContext,
    session: &sifr_package::PackageSession,
    selection: &sifr_package::CargoPackageSelection,
) -> Result<BTreeSet<sifr_package::SifrPackageId>, Vec<sifr_package::PackageDiagnostic>> {
    let mut selected = BTreeSet::new();
    if selection.workspace {
        selected.extend(
            sifr_package::select_sifr_workspace_members(&context.metadata, &context.graph)?
                .selected_sifr_packages,
        );
    }
    if !selection.packages.is_empty() {
        selected.extend(
            sifr_package::explicit_package_selection(
                &context.metadata,
                &context.graph,
                &selection.packages,
            )?
            .selected_sifr_packages,
        );
    }
    if !selection.workspace && selection.packages.is_empty() {
        selected.extend(current_session_package_id(session, &context.graph));
    }
    for excluded in &selection.excludes {
        selected.retain(|package_id| {
            context
                .graph
                .packages
                .get(package_id)
                .is_none_or(|package| {
                    package.cargo_package_name != *excluded
                        && package.package_id.0 != *excluded
                        && package.sifr_name.0 != *excluded
                })
        });
    }
    Ok(selected)
}

pub(super) fn current_session_package_id(
    session: &sifr_package::PackageSession,
    graph: &sifr_package::SifrPackageGraph,
) -> Option<sifr_package::SifrPackageId> {
    let manifest_path = session.manifest_path.as_ref()?;
    graph
        .packages
        .values()
        .find(|package| paths_equal(&package.sifr_manifest, manifest_path))
        .map(|package| package.package_id.clone())
}

pub(super) fn cargo_package_list_entries(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    cargo_package_name: &str,
    allow_dirty: bool,
    diagnostic_format: DiagnosticFormat,
) -> Result<Vec<sifr_package::PackageArchiveEntry>, i32> {
    let selection = sifr_package::CargoPackageSelection {
        workspace: false,
        packages: vec![cargo_package_name.to_string()],
        excludes: Vec::new(),
    };
    let options = sifr_package::CargoPackageArchiveOptions {
        list: true,
        allow_dirty,
        ..sifr_package::CargoPackageArchiveOptions::default()
    };
    let plan = sifr_package::CargoCommandPlan::package_with_options(
        session.workspace_root.clone(),
        lock_mode,
        &sifr_package::CargoFeatureSelection::default(),
        &selection,
        &options,
    );
    let output = match std::process::Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let diagnostic = cargo_failure_diagnostic(&plan, lock_mode, None, &error.to_string());
            render_diagnostics(&[diagnostic], diagnostic_format);
            return Err(EXIT_USAGE_OR_CONFIG);
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let excerpt = if stderr.trim().is_empty() {
            stdout.as_ref()
        } else {
            stderr.as_ref()
        };
        let diagnostic = cargo_failure_diagnostic(
            &plan,
            lock_mode,
            output.status.code(),
            &bounded_excerpt(excerpt),
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| sifr_package::PackageArchiveEntry {
            relative_path: PathBuf::from(line),
        })
        .collect())
}

pub(super) fn render_package_diagnostics(
    diagnostics: Vec<sifr_package::PackageDiagnostic>,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostics = diagnostics
        .into_iter()
        .map(package_diagnostic)
        .collect::<Vec<_>>();
    render_diagnostics(&diagnostics, diagnostic_format)
}

pub(super) fn execute_cargo_plan(
    plan: &sifr_package::CargoCommandPlan,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let output = match std::process::Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let diagnostic = cargo_failure_diagnostic(plan, lock_mode, None, &error.to_string());
            render_diagnostics(&[diagnostic], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    if output.status.success() {
        let _ = io::stdout().write_all(&output.stdout);
        let _ = io::stderr().write_all(&output.stderr);
        return EXIT_SUCCESS;
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let excerpt = if stderr.trim().is_empty() {
        stdout.as_ref()
    } else {
        stderr.as_ref()
    };
    let diagnostic = cargo_failure_diagnostic(
        plan,
        lock_mode,
        output.status.code(),
        &bounded_excerpt(excerpt),
    );
    render_diagnostics(&[diagnostic], diagnostic_format);
    EXIT_USER_DIAGNOSTIC
}

pub(super) fn cargo_failure_diagnostic(
    plan: &sifr_package::CargoCommandPlan,
    lock_mode: sifr_package::CargoLockMode,
    exit_status: Option<i32>,
    excerpt: &str,
) -> RenderedDiagnostic {
    let stderr_redacted = sifr_package::cargo::errors::redact_cargo_stderr(excerpt);
    let package = sifr_package::map_cargo_failure(plan.action, &stderr_redacted);
    let mut diagnostic = package_diagnostic(package);
    diagnostic.args.insert(
        "action".to_string(),
        DiagnosticArg::String(plan.action.as_str().to_string()),
    );
    diagnostic.args.insert(
        "current_dir".to_string(),
        DiagnosticArg::String(plan.current_dir.display().to_string()),
    );
    diagnostic.args.insert(
        "args_redacted".to_string(),
        DiagnosticArg::String(redacted_args(&plan.args).join(" ")),
    );
    diagnostic.args.insert(
        "lock_mode".to_string(),
        DiagnosticArg::String(format!("{lock_mode:?}").to_ascii_lowercase()),
    );
    diagnostic.args.insert(
        "network_mode".to_string(),
        DiagnosticArg::String(if lock_mode.is_network_disallowed() {
            "offline".to_string()
        } else {
            "online".to_string()
        }),
    );
    diagnostic.args.insert(
        "stderr_redacted".to_string(),
        DiagnosticArg::String(stderr_redacted),
    );
    if let Some(status) = exit_status {
        diagnostic.args.insert(
            "exit_status".to_string(),
            DiagnosticArg::String(status.to_string()),
        );
    }
    diagnostic
}
