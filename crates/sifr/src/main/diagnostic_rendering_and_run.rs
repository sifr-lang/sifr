fn render_compact_diagnostics(diagnostics: &[RenderedDiagnostic]) -> String {
    let mut grouped: BTreeMap<(u8, String, bool, String), Vec<&RenderedDiagnostic>> =
        BTreeMap::new();
    for diagnostic in diagnostics {
        let is_summary_group = diagnostic.message.starts_with("... +")
            && diagnostic.message.ends_with("more similar diagnostics");
        let key = (
            severity_rank(diagnostic.severity),
            diagnostic.code.clone(),
            is_summary_group,
            diagnostic.message.clone(),
        );
        grouped.entry(key).or_default().push(diagnostic);
    }

    let mut output = String::new();
    output.push_str(&compact_severity_summary(diagnostics));
    output.push('\n');

    for ((_severity_rank, code, _is_summary_group, message), group) in grouped {
        let severity = group[0].severity;
        let _ = writeln!(
            output,
            "{} [{code}] {message} (x{})",
            severity_label(severity),
            group.len()
        );

        let mut locations: BTreeSet<String> = BTreeSet::new();
        for diagnostic in &group {
            if let Some(span) = diagnostic.spans.iter().find(|span| span.is_primary) {
                locations.insert(compact_location_label(span));
            }
        }

        let rendered_locations = locations
            .iter()
            .take(MAX_COMPACT_REPRESENTATIVE_LOCATIONS)
            .collect::<Vec<_>>();
        for location in rendered_locations {
            let _ = writeln!(output, "  at {location}");
        }
        if locations.len() > MAX_COMPACT_REPRESENTATIVE_LOCATIONS {
            let _ = writeln!(
                output,
                "  ... +{} more",
                locations.len() - MAX_COMPACT_REPRESENTATIVE_LOCATIONS
            );
        }

        if let Some(help) = group
            .iter()
            .find_map(|diagnostic| diagnostic.help.as_deref())
        {
            let _ = writeln!(output, "  help: {help}");
        }
        if let Some(url) = group
            .iter()
            .find_map(|diagnostic| (!diagnostic.url.is_empty()).then_some(diagnostic.url.as_str()))
        {
            let _ = writeln!(output, "  url: {url}");
        }
    }

    output
}

fn canonical_diagnostic_stream(errors: &[RenderedDiagnostic]) -> Vec<RenderedDiagnostic> {
    apply_diagnostic_recovery_limits(errors)
}

fn render_diagnostic_stream(
    diagnostics: &[RenderedDiagnostic],
    format: DiagnosticFormat,
) -> Result<String, serde_json::Error> {
    let mut output = String::new();
    match format {
        DiagnosticFormat::Human => {
            for diagnostic in diagnostics {
                let label = human_label(diagnostic);
                let _ = writeln!(output, "{label}: {message}", message = diagnostic.message);
                for child in &diagnostic.children {
                    let child_label = match child.severity {
                        ChildSeverity::Note => "note",
                        ChildSeverity::Help => "help",
                    };
                    let _ = writeln!(output, "{child_label}: {}", child.message);
                }
            }
        }
        DiagnosticFormat::Json => {
            let json = serde_json::to_string_pretty(diagnostics)?;
            let _ = writeln!(output, "{json}");
        }
        DiagnosticFormat::Compact => {
            let _ = write!(output, "{}", render_compact_diagnostics(diagnostics));
        }
    }
    Ok(output)
}

fn render_diagnostic_output(
    errors: &[RenderedDiagnostic],
    format: DiagnosticFormat,
) -> Result<String, serde_json::Error> {
    let diagnostics = canonical_diagnostic_stream(errors);
    render_diagnostic_stream(&diagnostics, format)
}

fn render_diagnostics(errors: &[RenderedDiagnostic], format: DiagnosticFormat) -> i32 {
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

fn cmd_build(file: &Path, output: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during build command execution",
        || compile_entrypoint(file, output),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(binary_path) => {
            let _ = writeln!(
                io::stderr(),
                "compiled successfully: {}",
                binary_path.display()
            );
            EXIT_SUCCESS
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_run(
    target: Option<&str>,
    bin: Option<&str>,
    script: Option<&str>,
    app_args: &[String],
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let current_dir = match std::env::current_dir() {
        Ok(current_dir) => current_dir,
        Err(error) => {
            let diagnostic = diagnostic_with_code(
                format!("could not read current directory: {error}"),
                DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED,
            );
            render_diagnostics(&[diagnostic], diagnostic_format);
            return EXIT_USAGE_OR_CONFIG;
        }
    };
    let session =
        match sifr_package::PackageSession::discover(sifr_package::PackageSessionOptions {
            current_dir,
            lock_mode,
        }) {
            Ok(session) => session,
            Err(error) => {
                render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
                return EXIT_USAGE_OR_CONFIG;
            }
        };
    let plan = session.plan_run(&sifr_package::PackageRunRequest {
        target_or_path: target.map(str::to_string),
        bin: bin.map(str::to_string),
        script: script.map(str::to_string),
        app_args: app_args.to_vec(),
        script_depth: 0,
    });
    let plan = match plan {
        Ok(plan) => plan,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if let Some(origin) = plan.script_origin {
        return cmd_script(&origin, diagnostic_format);
    }
    if let Some(
        sifr_package::ResolvedRunTarget::File(path)
        | sifr_package::ResolvedRunTarget::App { path, .. },
    ) = plan.run_target
    {
        if !session.manifest_less_mode {
            return cmd_run_package_file(&path, &session, lock_mode, app_args, diagnostic_format);
        }
        return cmd_run_file(&path, app_args, diagnostic_format);
    }
    if let Some(cargo) = plan.cargo {
        return execute_cargo_plan(&cargo, lock_mode, diagnostic_format);
    }
    EXIT_SUCCESS
}

fn cmd_script(origin: &sifr_package::ScriptOrigin, diagnostic_format: DiagnosticFormat) -> i32 {
    match origin.command.as_str() {
        "run" => cmd_run(
            origin.args.first().map(String::as_str),
            None,
            None,
            &[],
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "check" => cmd_check(
            origin
                .args
                .first()
                .filter(|arg| !arg.starts_with('-'))
                .map(Path::new),
            None,
            &sifr_package::CargoPackageSelection::default(),
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "fetch" => cmd_fetch(sifr_package::CargoLockMode::Normal, diagnostic_format),
        "tree" => cmd_tree(
            sifr_package::CargoLockMode::Normal,
            &origin.args,
            diagnostic_format,
        ),
        "package" => cmd_package(
            &sifr_package::CargoPackageSelection::default(),
            &sifr_package::CargoPackageArchiveOptions::default(),
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "publish" => cmd_publish(
            &sifr_package::CargoPackageSelection::default(),
            &sifr_package::CargoPublishOptions {
                dry_run: origin.args.iter().any(|arg| arg == "--dry-run"),
                ..sifr_package::CargoPublishOptions::default()
            },
            sifr_package::CargoLockMode::Normal,
            diagnostic_format,
        ),
        "vendor" => cmd_vendor(
            origin
                .args
                .first()
                .map(Path::new)
                .unwrap_or_else(|| Path::new("vendor")),
            &sifr_package::CargoVendorOptions::default(),
            sifr_package::CargoLockMode::Normal,
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

fn cmd_run_file(file: &Path, app_args: &[String], diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during run command compilation",
        || build_run_artifact(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(artifact) => {
            let _ = writeln!(io::stderr(), "{}", artifact.cache_status_line());
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

fn cmd_run_package_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    app_args: &[String],
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let context = match package_compiler_context(session, lock_mode, diagnostic_format) {
        Ok(Some(context)) => context,
        Ok(None) => return cmd_run_file(file, app_args, diagnostic_format),
        Err(exit_code) => return exit_code,
    };
    let entrypoint = PackageEntrypoint {
        main_file: file.to_path_buf(),
        package_id: context.package_id,
        graph: context.graph,
        source_map: context.source_map,
    };
    let result = match run_with_panic_boundary(
        "internal compiler panic during package run command compilation",
        || build_cached_package_project(&entrypoint),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(artifact) => run_binary_artifact(&artifact, app_args),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn run_binary_artifact(artifact: &CachedBinaryArtifact, app_args: &[String]) -> i32 {
    let _ = writeln!(io::stderr(), "{}", artifact.cache_status_line());
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

fn cmd_fetch(lock_mode: sifr_package::CargoLockMode, diagnostic_format: DiagnosticFormat) -> i32 {
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
    execute_cargo_plan(&cargo, lock_mode, diagnostic_format)
}

fn cmd_tree(
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

fn cmd_package(
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

fn cmd_publish(
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

fn cmd_vendor(
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

fn package_session_for_cwd(
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

fn run_package_release_preflight(
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

fn selected_release_package_ids(
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

fn current_session_package_id(
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

fn cargo_package_list_entries(
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

fn render_package_diagnostics(
    diagnostics: Vec<sifr_package::PackageDiagnostic>,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let diagnostics = diagnostics
        .into_iter()
        .map(package_diagnostic)
        .collect::<Vec<_>>();
    render_diagnostics(&diagnostics, diagnostic_format)
}

fn execute_cargo_plan(
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

fn cargo_failure_diagnostic(
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

