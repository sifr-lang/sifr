use super::cli_model_and_entrypoint::{
    diagnostic_with_code, package_diagnostic, read_source, resolve_compilation_mode,
    run_with_panic_boundary, CompilationMode, DiagnosticFormat, PackageCompilerContext,
    EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::{
    current_session_package_id, execute_cargo_plan, package_session_for_cwd, render_diagnostics,
};
use super::formatter_cli::FmtArgs;
use super::package_graph_context::load_package_graph_context_for_entrypoint;
use super::python_runtime_context::{package_python_runtime, package_python_runtime_for_check};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_driver::{
    build_cached_project, build_cached_single_file, build_package_project_report,
    build_project_report, build_single_file_report, check_package_project, check_project,
    check_single_file, compile, emit_project, run_tests, BuildReport, CachedBinaryArtifact,
    CompileResult, PackageEntrypoint,
};
use sifr_format::config::{effective_format_config, EffectiveFormatConfig, FormatConfigOverrides};
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use sifr_python_ast::{Expr, Stmt};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash as _, Hasher as _};
use std::io::{self, IsTerminal as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
pub(super) fn cmd_check(
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
    if session.manifest_less_mode && lock_mode != sifr_package::CargoLockMode::Normal {
        return render_diagnostics(
            &[super::cli_lock_modes::lock_mode_requires_package(
                "check", lock_mode,
            )],
            diagnostic_format,
        );
    }
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

pub(super) fn cmd_check_file(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
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
                let _ = writeln!(io::stderr(), "0 errors, 0 warnings, 0 notes");
            }
        }
        EXIT_SUCCESS
    } else {
        render_diagnostics(&errors, diagnostic_format)
    }
}

pub(super) fn cmd_check_package_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let allow_python_deferral = match session.runnable_app_paths() {
        Ok(paths) => paths.is_empty(),
        Err(error) => {
            render_diagnostics(&[package_diagnostic(error)], diagnostic_format);
            return EXIT_USER_DIAGNOSTIC;
        }
    };
    let entrypoint = match package_entrypoint_for_file(
        file,
        session,
        lock_mode,
        diagnostic_format,
        allow_python_deferral,
    ) {
        Ok(Some(entrypoint)) => entrypoint,
        Ok(None) => return cmd_check_file(file, diagnostic_format),
        Err(exit_code) => return exit_code,
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

pub(super) fn package_entrypoint_for_file(
    file: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
    allow_python_deferral: bool,
) -> Result<Option<PackageEntrypoint>, i32> {
    let Some(context) = package_compiler_context(
        session,
        lock_mode,
        diagnostic_format,
        Some(file),
        allow_python_deferral,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(PackageEntrypoint {
        main_file: file.to_path_buf(),
        package_id: context.package_id,
        graph: context.graph,
        source_map: context.source_map,
        python_runtime: context.python_runtime,
        lock_mode,
    }))
}

pub(super) fn package_compiler_context(
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
    entry_file: Option<&Path>,
    allow_python_deferral: bool,
) -> Result<Option<PackageCompilerContext>, i32> {
    let Some(context) =
        load_package_graph_context_for_entrypoint(session, lock_mode, diagnostic_format)?
    else {
        return Ok(None);
    };
    let Some(package_id) = current_session_package_id(session, &context.graph) else {
        return Ok(None);
    };
    let mut derived_python_requirements = declaration_python_requirements(
        &context.source_map,
        entry_file.map(|file| (file, &package_id)),
    );
    let bridge_graph = match sifr_package::resolve_python_bridge_graph(&context.graph, &package_id)
    {
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
    derived_python_requirements.extend(bridge_graph.requirements);
    let python_runtime = if allow_python_deferral {
        package_python_runtime_for_check(
            &context.graph,
            &package_id,
            &derived_python_requirements,
            true,
            diagnostic_format,
        )?
        .runtime
    } else {
        package_python_runtime(
            &context.graph,
            &package_id,
            &derived_python_requirements,
            diagnostic_format,
        )?
    };
    Ok(Some(PackageCompilerContext {
        graph: context.graph,
        source_map: context.source_map,
        package_id,
        python_runtime,
    }))
}

pub(super) fn declaration_python_requirements(
    source_map: &sifr_package::PackageSourceMap,
    entry_file: Option<(&Path, &sifr_package::SifrPackageId)>,
) -> Vec<sifr_package::PythonRequirementContribution> {
    let mut contributions = Vec::new();
    let mut provider = DiskSourceProvider::new();
    for module in source_map.modules.values() {
        let Ok(source) = provider.read_file(&module.file_path) else {
            continue;
        };
        let context = module.file_path.to_string_lossy();
        let Ok(parsed) = sifr_syntax::parse_module(source.as_str(), Some(&context)) else {
            continue;
        };
        for statement in parsed.suite() {
            collect_statement_python_requirements(
                statement,
                &module.package_id,
                &module.file_path,
                &mut contributions,
            );
        }
    }
    if let Some((file, package_id)) = entry_file {
        if let Ok(source) = provider.read_file(file) {
            let context = file.to_string_lossy();
            if let Ok(parsed) = sifr_syntax::parse_module(source.as_str(), Some(&context)) {
                for statement in parsed.suite() {
                    collect_statement_python_requirements(
                        statement,
                        package_id,
                        file,
                        &mut contributions,
                    );
                }
            }
        }
    }
    contributions.sort();
    contributions.dedup();
    contributions
}

fn collect_statement_python_requirements(
    statement: &Stmt,
    package_id: &sifr_package::SifrPackageId,
    file_path: &Path,
    out: &mut Vec<sifr_package::PythonRequirementContribution>,
) {
    match statement {
        Stmt::FunctionDef(function) => {
            for decorator in &function.decorator_list {
                let Expr::Call(call) = &decorator.expression else {
                    continue;
                };
                if dotted_root(&call.func).as_deref() != Some("python") {
                    continue;
                }
                if matches!(call.func.as_ref(), Expr::Attribute(attribute) if attribute.attr.as_str() == "callback")
                {
                    continue;
                }
                let Some(target) = call.arguments.args.first() else {
                    continue;
                };
                let Some(root) = dotted_root(target) else {
                    continue;
                };
                if matches!(root.as_str(), "bridge" | "Self") {
                    continue;
                }
                out.push(sifr_package::PythonRequirementContribution {
                    root,
                    package_id: package_id.clone(),
                    kind: sifr_package::PythonRequirementKind::Declaration,
                    source: file_path.display().to_string(),
                });
            }
        }
        Stmt::ClassDef(class) => {
            for nested in &class.body {
                collect_statement_python_requirements(nested, package_id, file_path, out);
            }
        }
        _ => {}
    }
}

fn dotted_root(expression: &Expr) -> Option<String> {
    match expression {
        Expr::Name(name) => Some(name.id.to_string()),
        Expr::Attribute(attribute) => dotted_root(&attribute.value),
        _ => None,
    }
}

pub(super) fn cmd_fmt(
    args: &FmtArgs,
    config_inputs: &[String],
    isolated: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let result = match run_with_panic_boundary(
        "internal compiler panic during fmt command execution",
        || fmt_entrypoint(args, config_inputs, isolated),
    ) {
        Ok(result) => result,
        Err(internal) => return render_diagnostics(&[*internal], diagnostic_format),
    };

    match result {
        Ok(changed) => {
            if args.check {
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

pub(super) fn emit_success_message(diagnostic_format: DiagnosticFormat, message: &str) {
    match diagnostic_format {
        DiagnosticFormat::Human => {
            let _ = writeln!(io::stderr(), "{message}");
        }
        DiagnosticFormat::Json => {
            let _ = writeln!(io::stdout(), "[]");
        }
        DiagnosticFormat::Compact => {
            let _ = writeln!(io::stderr(), "0 errors, 0 warnings, 0 notes");
        }
    }
}

pub(super) fn cmd_test(dir: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
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

pub(super) fn cmd_emit(file: &Path, diagnostic_format: DiagnosticFormat) -> i32 {
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

#[cfg(test)]
pub(super) fn compile_entrypoint(
    file: &Path,
    output: &Path,
) -> Result<PathBuf, Vec<RenderedDiagnostic>> {
    compile_entrypoint_report(file, output).map(|report| report.binary_path().to_path_buf())
}

pub(super) fn compile_entrypoint_report(
    file: &Path,
    output: &Path,
) -> Result<BuildReport, Vec<RenderedDiagnostic>> {
    match resolve_compilation_mode(file)? {
        CompilationMode::Project => build_project_report(file, output),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build_single_file_report(&source, file, output)
        }
    }
}

pub(super) fn compile_package_entrypoint_report(
    file: &Path,
    output: &Path,
    session: &sifr_package::PackageSession,
    lock_mode: sifr_package::CargoLockMode,
    diagnostic_format: DiagnosticFormat,
) -> Result<Option<BuildReport>, i32> {
    let Some(entrypoint) =
        package_entrypoint_for_file(file, session, lock_mode, diagnostic_format, false)?
    else {
        return Ok(None);
    };
    match run_with_panic_boundary(
        "internal compiler panic during package build command execution",
        || build_package_project_report(&entrypoint, output),
    ) {
        Ok(Ok(report)) => Ok(Some(report)),
        Ok(Err(errors)) => Err(render_diagnostics(&errors, diagnostic_format)),
        Err(internal) => Err(render_diagnostics(&[*internal], diagnostic_format)),
    }
}

pub(super) fn build_run_artifact(
    file: &Path,
) -> Result<CachedBinaryArtifact, Vec<RenderedDiagnostic>> {
    match resolve_compilation_mode(file)? {
        CompilationMode::Project => build_cached_project(file),
        CompilationMode::SingleFile => {
            let source = read_source(file);
            build_cached_single_file(&source, file)
        }
    }
}

pub(super) fn check_entrypoint(file: &Path) -> Vec<RenderedDiagnostic> {
    match resolve_compilation_mode(file) {
        Err(errors) => errors,
        Ok(CompilationMode::Project) => check_project(file),
        Ok(CompilationMode::SingleFile) => {
            let source = read_source(file);
            check_single_file(&source, file)
        }
    }
}

pub(super) fn emit_entrypoint(file: &Path) -> CompileResult {
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

pub(super) fn fmt_entrypoint(
    args: &FmtArgs,
    config_inputs: &[String],
    isolated: bool,
) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let cwd = std::env::current_dir().map_err(|err| {
        vec![formatter_cli_diagnostic(format!(
            "could not read current directory: {err}"
        ))]
    })?;
    let config =
        effective_format_config(&cwd, config_inputs, isolated, &format_cli_overrides(args))?;
    let options = config.format_options;
    if args.stdin_filename.is_some() || (args.paths.is_empty() && !io::stdin().is_terminal()) {
        return fmt_stdin(args, options);
    }

    let targets = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };
    let mut diagnostics = Vec::new();
    let mut provider = DiskSourceProvider::new();
    for target in targets {
        let explicit_target = provider.is_file(&target);
        let files = select_formatter_files(&target, &config, explicit_target)?;
        for file in files {
            if args.check {
                diagnostics.extend(sifr_format::check_path_with_options(&file, options)?);
            } else if args.diff {
                let source = read_formatter_source(&file)?;
                let formatted = format_source_or_range(&source, &file, args, options)?;
                if formatted != source {
                    write_unified_diff(&file, &source, &formatted);
                    diagnostics.push(formatting_drift_for_path(&source, &file));
                }
            } else if args.range.is_some() {
                let source = read_formatter_source(&file)?;
                let formatted = format_source_or_range(&source, &file, args, options)?;
                if formatted != source {
                    fs::write(&file, formatted).map_err(|err| {
                        vec![formatter_cli_diagnostic(format!(
                            "could not write file {}: {err}",
                            file.display()
                        ))]
                    })?;
                }
            } else {
                if try_formatter_cache_hit(&file, options, &config)? {
                    continue;
                }
                let _formatted = sifr_format::format_path_with_options(&file, false, options)?;
                write_formatter_cache_entry(&file, options, &config)?;
            }
        }
    }
    Ok(diagnostics)
}

fn fmt_stdin(
    args: &FmtArgs,
    options: sifr_format::FormatOptions,
) -> Result<Vec<RenderedDiagnostic>, Vec<RenderedDiagnostic>> {
    let mut source = String::new();
    io::stdin().read_to_string(&mut source).map_err(|err| {
        vec![formatter_cli_diagnostic(format!(
            "could not read formatter stdin: {err}"
        ))]
    })?;
    let file = args
        .stdin_filename
        .as_deref()
        .unwrap_or(Path::new("<stdin>"));
    let formatted = format_source_or_range(&source, file, args, options)?;
    if args.check {
        if formatted == source {
            return Ok(Vec::new());
        }
        return Ok(vec![formatting_drift_for_path(&source, file)]);
    }
    if args.diff {
        write_unified_diff(file, &source, &formatted);
    } else {
        let _ = write!(io::stdout(), "{formatted}");
    }
    Ok(Vec::new())
}

fn format_source_or_range(
    source: &str,
    file: &Path,
    args: &FmtArgs,
    options: sifr_format::FormatOptions,
) -> Result<String, Vec<RenderedDiagnostic>> {
    if let Some(range) = &args.range {
        let range = parse_byte_range(range)?;
        let edits = sifr_format::format_range(source, range, Some(file), options)?;
        let mut formatted = source.to_string();
        for edit in edits.into_iter().rev() {
            let start = usize::from(edit.range.start());
            let end = usize::from(edit.range.end());
            formatted.replace_range(start..end, &edit.replacement);
        }
        Ok(formatted)
    } else {
        sifr_format::format_source(source, Some(file), options).map(|result| result.formatted)
    }
}

fn format_cli_overrides(args: &FmtArgs) -> FormatConfigOverrides {
    FormatConfigOverrides {
        line_length: args.line_length,
        preview: flag_override(args.preview, args.no_preview),
        exclude: args.exclude.clone(),
        respect_gitignore: flag_override(args.respect_gitignore, args.no_respect_gitignore),
        force_exclude: flag_override(args.force_exclude, args.no_force_exclude),
        no_cache: args.no_cache.then_some(true),
        cache_dir: args.cache_dir.clone(),
    }
}

fn flag_override(enable: bool, disable: bool) -> Option<bool> {
    if disable {
        Some(false)
    } else if enable {
        Some(true)
    } else {
        None
    }
}

fn select_formatter_files(
    target: &Path,
    config: &EffectiveFormatConfig,
    explicit_target: bool,
) -> Result<Vec<PathBuf>, Vec<RenderedDiagnostic>> {
    let files = sifr_format::collect_sifr_files(target)?;
    let mut selected = Vec::new();
    let ignore_patterns = if config.respect_gitignore {
        read_gitignore_patterns()?
    } else {
        Vec::new()
    };
    for file in files {
        let excluded =
            pattern_matches(&file, &config.exclude) || pattern_matches(&file, &ignore_patterns);
        if excluded && (!explicit_target || config.force_exclude) {
            continue;
        }
        selected.push(file);
    }
    Ok(selected)
}

fn read_gitignore_patterns() -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let path = Path::new(".gitignore");
    let mut provider = DiskSourceProvider::new();
    if !provider.is_file(path) {
        return Ok(Vec::new());
    }
    let source = provider.read_file(path).map_err(|err| {
        vec![formatter_cli_diagnostic(format!(
            "could not read .gitignore for formatter discovery: {err}"
        ))]
    })?;
    Ok(source
        .as_str()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect())
}

fn pattern_matches(path: &Path, patterns: &[String]) -> bool {
    let path = path.to_string_lossy();
    patterns.iter().any(|pattern| {
        let pattern = pattern.trim().trim_matches('*').trim_matches('/');
        !pattern.is_empty() && path.contains(pattern)
    })
}

fn try_formatter_cache_hit(
    path: &Path,
    options: sifr_format::FormatOptions,
    config: &EffectiveFormatConfig,
) -> Result<bool, Vec<RenderedDiagnostic>> {
    if config.no_cache {
        return Ok(false);
    }
    let source = read_formatter_source(path)?;
    let key = formatter_cache_key(path, &source, options);
    Ok(config.cache_dir.join(key).is_file())
}

fn write_formatter_cache_entry(
    path: &Path,
    options: sifr_format::FormatOptions,
    config: &EffectiveFormatConfig,
) -> Result<(), Vec<RenderedDiagnostic>> {
    if config.no_cache {
        return Ok(());
    }
    let source = read_formatter_source(path)?;
    fs::create_dir_all(&config.cache_dir).map_err(|err| {
        vec![formatter_cli_diagnostic(format!(
            "could not create formatter cache {}: {err}",
            config.cache_dir.display()
        ))]
    })?;
    let key = formatter_cache_key(path, &source, options);
    fs::write(config.cache_dir.join(key), b"ok").map_err(|err| {
        vec![formatter_cli_diagnostic(format!(
            "could not write formatter cache {}: {err}",
            config.cache_dir.display()
        ))]
    })
}

fn read_formatter_source(path: &Path) -> Result<String, Vec<RenderedDiagnostic>> {
    DiskSourceProvider::new()
        .read_file(path)
        .map(|source| source.as_str().to_string())
        .map_err(|err| {
            vec![formatter_cli_diagnostic(format!(
                "could not read file {}: {err}",
                path.display()
            ))]
        })
}

fn formatter_cache_key(path: &Path, source: &str, options: sifr_format::FormatOptions) -> String {
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    source.hash(&mut hasher);
    options.final_newline.hash(&mut hasher);
    options.line_length.hash(&mut hasher);
    options.preview.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn parse_byte_range(raw: &str) -> Result<TextRange, Vec<RenderedDiagnostic>> {
    let Some((start, end)) = raw.split_once(':') else {
        return Err(vec![formatter_cli_diagnostic(
            "formatter range must use START:END byte offsets",
        )]);
    };
    let start = parse_text_size(start)?;
    let end = parse_text_size(end)?;
    if start > end {
        return Err(vec![formatter_cli_diagnostic(
            "formatter range start must be before range end",
        )]);
    }
    Ok(TextRange::new(start, end))
}

fn parse_text_size(raw: &str) -> Result<TextSize, Vec<RenderedDiagnostic>> {
    let value = raw.parse::<u32>().map_err(|_| {
        vec![formatter_cli_diagnostic(
            "formatter range offsets must be unsigned integers",
        )]
    })?;
    Ok(TextSize::new(value))
}

fn write_unified_diff(path: &Path, before: &str, after: &str) {
    let _ = writeln!(io::stdout(), "--- {}", path.display());
    let _ = writeln!(io::stdout(), "+++ {}", path.display());
    for line in before.lines() {
        let _ = writeln!(io::stdout(), "-{line}");
    }
    for line in after.lines() {
        let _ = writeln!(io::stdout(), "+{line}");
    }
}

fn formatting_drift_for_path(source: &str, path: &Path) -> RenderedDiagnostic {
    match sifr_format::check_source(source, Some(path), sifr_format::FormatOptions::default()) {
        Ok(check) if !check.diagnostics.is_empty() => check.diagnostics[0].clone(),
        _ => formatter_cli_diagnostic(format!(
            "source is not formatted with sifr fmt: {}",
            path.display()
        )),
    }
}

fn formatter_cli_diagnostic(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic_with_code(message, DiagnosticCode::FMT_FORMATTING_DRIFT)
}
