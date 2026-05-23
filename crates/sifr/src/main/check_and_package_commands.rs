fn redacted_args(args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| sifr_package::cargo::errors::redact_cargo_stderr(arg))
        .collect()
}

fn bounded_excerpt(text: &str) -> String {
    const MAX_LINES: usize = 12;
    const MAX_BYTES: usize = 4096;
    let mut excerpt = text.lines().take(MAX_LINES).collect::<Vec<_>>().join("\n");
    if excerpt.len() > MAX_BYTES {
        excerpt.truncate(MAX_BYTES);
    }
    excerpt
}

fn cmd_check(
    file: Option<&Path>,
    message_format: Option<&str>,
    selection: &sifr_package::CargoPackageSelection,
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
    let mut plan = match session.plan_check(
        file,
        &sifr_package::CargoFeatureSelection::default(),
        selection,
    ) {
        Ok(plan) => plan,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    if let Some(mut cargo) = plan.cargo.take() {
        if let Some(format) = message_format {
            cargo.extend_forwarded_args(&["--message-format".to_string(), format.to_string()]);
        }
        return execute_cargo_plan(&cargo, lock_mode, diagnostic_format);
    }
    if let Some(sifr_package::ResolvedRunTarget::File(path)) = plan.run_target {
        if !session.manifest_less_mode {
            return cmd_check_package_file(&path, &session, lock_mode, diagnostic_format);
        }
        return cmd_check_file(&path, diagnostic_format);
    }
    EXIT_SUCCESS
}

fn cmd_check_file(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let errors = match run_with_panic_boundary(
        "internal compiler panic during check command execution",
        || check_entrypoint(file),
    ) {
        Ok(errors) => errors,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    if errors.is_empty() {
        match diagnostic_format {
            DiagnosticFormat::Human => {
                let _ = writeln!(io::stderr(), "no errors found");
            }
            DiagnosticFormat::Json => {
                let _ = writeln!(io::stdout(), "[]");
            }
            DiagnosticFormat::Compact => {
                let _ = writeln!(
                    io::stderr(),
                    "summary: 0 error(s), 0 warning(s), 0 note(s), 0 help item(s)"
                );
            }
        }
        EXIT_SUCCESS
    } else {
        render_diagnostics(&errors, diagnostic_format)
    }
}

fn cmd_check_package_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let context = match package_compiler_context(session, lock_mode, diagnostic_format) {
        Ok(Some(context)) => context,
        Ok(None) => return cmd_check_file(file, diagnostic_format),
        Err(exit_code) => return exit_code,
    };
    let entrypoint = PackageEntrypoint {
        main_file: file.to_path_buf(),
        package_id: context.package_id,
        graph: context.graph,
        source_map: context.source_map,
    };
    let errors = match run_with_panic_boundary(
        "internal compiler panic during package check command execution",
        || check_package_project(&entrypoint),
    ) {
        Ok(errors) => errors,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    if errors.is_empty() {
        emit_success_message(diagnostic_format, "no errors found");
        EXIT_SUCCESS
    } else {
        render_diagnostics(&errors, diagnostic_format)
    }
}

fn package_compiler_context(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<PackageCompilerContext>, i32> {
    let Some(context) = load_package_graph_context(session, lock_mode, diagnostic_format)? else {
        return Ok(None);
    };
    let Some(package_id) = current_session_package_id(session, &context.graph) else {
        return Ok(None);
    };
    Ok(Some(PackageCompilerContext {
        graph: context.graph,
        source_map: context.source_map,
        package_id,
    }))
}

fn load_package_graph_context(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<PackageGraphContext>, i32> {
    if session.manifest_less_mode {
        return Ok(None);
    }
    let metadata_plan =
        sifr_package::CargoCommandPlan::metadata(session.workspace_root.clone(), lock_mode);
    let output = match std::process::Command::new(&metadata_plan.program)
        .args(&metadata_plan.args)
        .current_dir(&metadata_plan.current_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            let diagnostic =
                cargo_failure_diagnostic(&metadata_plan, lock_mode, None, &error.to_string());
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
            &metadata_plan,
            lock_mode,
            output.status.code(),
            &bounded_excerpt(excerpt),
        );
        render_diagnostics(&[diagnostic], diagnostic_format);
        return Err(EXIT_USER_DIAGNOSTIC);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let metadata = match sifr_package::parse_metadata_json(&stdout) {
        Ok(metadata) => metadata,
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return Err(EXIT_USAGE_OR_CONFIG);
        }
    };
    let normalized = metadata.clone().normalize();
    let graph = match sifr_package::derive_package_graph(metadata) {
        Ok(graph) => graph,
        Err(errors) => {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return Err(EXIT_USER_DIAGNOSTIC);
        }
    };
    let source_map = match sifr_package::PackageSourceMap::build(&graph) {
        Ok(source_map) => source_map,
        Err(errors) => {
            let diagnostics = errors
                .into_iter()
                .map(package_diagnostic)
                .collect::<Vec<_>>();
            render_diagnostics(&diagnostics, diagnostic_format);
            return Err(EXIT_USER_DIAGNOSTIC);
        }
    };
    Ok(Some(PackageGraphContext {
        metadata: normalized,
        graph,
        source_map,
    }))
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    let left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let right = right.canonicalize().unwrap_or_else(|_| right.to_path_buf());
    left == right
}

fn cmd_fmt(path: &Path, check: bool, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during fmt command execution",
        || fmt_entrypoint(path, check),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(changed) => {
            if check {
                if changed.is_empty() {
                    emit_success_message(diagnostic_format, "format check passed");
                    EXIT_SUCCESS
                } else {
                    render_diagnostics(&changed, diagnostic_format)
                }
            } else {
                emit_success_message(diagnostic_format, "formatted Sifr source files");
                EXIT_SUCCESS
            }
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_lint(path: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during lint command execution",
        || lint_entrypoint(path),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(diagnostics) if diagnostics.is_empty() => {
            emit_success_message(diagnostic_format, "no lint diagnostics found");
            EXIT_SUCCESS
        }
        Ok(diagnostics) => render_diagnostics(&diagnostics, diagnostic_format),
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn emit_success_message(diagnostic_format: DiagnosticFormat, message: &str) {
    match diagnostic_format {
        DiagnosticFormat::Human => {
            let _ = writeln!(io::stderr(), "{message}");
        }
        DiagnosticFormat::Json => {
            let _ = writeln!(io::stdout(), "[]");
        }
        DiagnosticFormat::Compact => {
            let _ = writeln!(
                io::stderr(),
                "summary: 0 error(s), 0 warning(s), 0 note(s), 0 help item(s)"
            );
        }
    }
}

fn cmd_test(dir: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let run_result = match run_with_panic_boundary(
        "internal compiler panic during test command execution",
        || run_tests(dir),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match run_result {
        Ok(success) => {
            if success {
                EXIT_SUCCESS
            } else {
                EXIT_USER_DIAGNOSTIC
            }
        }
        Err(errors) => render_diagnostics(&errors, diagnostic_format),
    }
}

fn cmd_emit(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
    let compile_result = match run_with_panic_boundary(
        "internal compiler panic during emit command execution",
        || emit_entrypoint(file),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };
    match compile_result {
        CompileResult::Success { rust_source } => {
            let _ = write!(io::stdout(), "{rust_source}");
            EXIT_SUCCESS
        }
        CompileResult::Errors { errors } => render_diagnostics(&errors, diagnostic_format),
    }
}

fn compile_entrypoint(file: &Path, output: &Path) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    match resolve_compilation_mode(file)? {
        CompilationMode::Project => build_project(file, output),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build(&source, output)
        }
    }
}

fn build_run_artifact(file: &Path) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    match resolve_compilation_mode(file)? {
        CompilationMode::Project => build_cached_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build_cached_single_file(&source, file)
        }
    }
}

fn check_entrypoint(file: &Path) -> Vec<RenderedDiagnostic> {
    match resolve_compilation_mode(file) {
        Err(errors) => errors,
        Ok(CompilationMode::Project) => check_project(file),
        Ok(CompilationMode::SingleFile) => {
            let source = read_source(file);
            check_single_file(&source, file)
        }
    }
}

fn emit_entrypoint(file: &Path) -> CompileResult {
    let mode = match resolve_compilation_mode(file) {
        Ok(mode) => mode,
        Err(errors) => return CompileResult::Errors { errors },
    };
    match mode {
        CompilationMode::Project => emit_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            compile(&source)
        }
    }
}

fn fmt_entrypoint(
    path: &Path,
    check: bool,
) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let files = sifr_format::collect_sifr_files(path)?;
    let mut diagnostics = Vec::new();
    for file in files {
        if check {
            diagnostics.extend(sifr_format::check_path(&file)?);
        } else {
            let _formatted = sifr_format::format_path(&file, false)?;
        }
    }
    Ok(diagnostics)
}

fn lint_entrypoint(path: &Path) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let options = sifr_lint::LintOptions {
        explicit_target: path.is_file(),
        ..sifr_lint::LintOptions::default()
    };
    sifr_lint::lint_path(path, &options).map(|result| result.diagnostics)
}

