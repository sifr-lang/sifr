use super::cli_model_and_entrypoint::{
    diagnostic_with_code, run_with_panic_boundary, DiagnosticFormat,
    EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::render_diagnostic_output;
use clap::{Args, ValueEnum};
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use sifr_lint::{EffectiveLintConfig, LintConfigOverrides, PerFileIgnore, UnsafeFixPolicy};
use std::collections::BTreeMap;
use std::io::{self, Read as _, Write as _};
use std::path::{Path, PathBuf};

#[derive(Args, Clone, Debug)]
pub(crate) struct LintArgs {
    /// Filename context for linting stdin
    #[arg(long)]
    pub(crate) stdin_filename: Option<PathBuf>,

    /// Select Sifr policy rules or categories
    #[arg(long, value_delimiter = ',')]
    pub(crate) select: Vec<String>,

    /// Extend selected Sifr policy rules or categories
    #[arg(long, value_delimiter = ',')]
    pub(crate) extend_select: Vec<String>,

    /// Ignore Sifr policy rules or categories
    #[arg(long, value_delimiter = ',')]
    pub(crate) ignore: Vec<String>,

    /// Ignore `# sifr: ignore[...]` suppression comments for this run
    #[arg(long)]
    pub(crate) ignore_suppressions: bool,

    /// Ignore rules for files matching a glob, as GLOB:RULE[,RULE]
    #[arg(long)]
    pub(crate) per_file_ignores: Vec<String>,

    /// Extend per-file ignores, as GLOB:RULE[,RULE]
    #[arg(long)]
    pub(crate) extend_per_file_ignores: Vec<String>,

    /// Lint output format
    #[arg(long, value_enum)]
    pub(crate) output_format: Option<LintOutputFormat>,

    /// Write lint output to a file
    #[arg(long)]
    pub(crate) output_file: Option<PathBuf>,

    /// Print discovered files without linting
    #[arg(long, conflicts_with = "show_settings")]
    pub(crate) show_files: bool,

    /// Print resolved lint settings without linting
    #[arg(long, conflicts_with = "statistics")]
    pub(crate) show_settings: bool,

    /// Print diagnostic counts by Sifr policy rule
    #[arg(long, conflicts_with_all = ["show_files", "show_settings", "diff"])]
    pub(crate) statistics: bool,

    /// Limit fix application to selected Sifr policy rules or categories
    #[arg(long, value_delimiter = ',')]
    pub(crate) fixable: Vec<String>,

    /// Extend selected fixable Sifr policy rules or categories
    #[arg(long, value_delimiter = ',')]
    pub(crate) extend_fixable: Vec<String>,

    /// Exclude selected Sifr policy rules or categories from fixes
    #[arg(long, value_delimiter = ',')]
    pub(crate) unfixable: Vec<String>,

    /// Extend selected unfixable Sifr policy rules or categories
    #[arg(long, value_delimiter = ',')]
    pub(crate) extend_unfixable: Vec<String>,

    /// Apply safe Sifr policy fixes to files
    #[arg(long, conflicts_with = "diff")]
    pub(crate) fix: bool,

    /// Apply fixes and suppress remaining diagnostic output
    #[arg(long, conflicts_with = "diff")]
    pub(crate) fix_only: bool,

    /// Print a patch for safe Sifr policy fixes without writing files
    #[arg(long, conflicts_with_all = ["fix", "fix_only", "statistics"])]
    pub(crate) diff: bool,

    /// Enable unsafe Sifr policy fixes
    #[arg(long, conflicts_with = "no_unsafe_fixes")]
    pub(crate) unsafe_fixes: bool,

    /// Disable unsafe Sifr policy fixes
    #[arg(long)]
    pub(crate) no_unsafe_fixes: bool,

    /// Print a deterministic summary of available fixes
    #[arg(long)]
    pub(crate) show_fixes: bool,

    /// Enable preview policy rules
    #[arg(long, conflicts_with = "no_preview")]
    pub(crate) preview: bool,

    /// Disable preview policy rules
    #[arg(long)]
    pub(crate) no_preview: bool,

    /// Exclude paths matching a lint glob
    #[arg(long, value_delimiter = ',')]
    pub(crate) exclude: Vec<String>,

    /// Extend configured lint excludes
    #[arg(long, value_delimiter = ',')]
    pub(crate) extend_exclude: Vec<String>,

    /// Respect VCS ignore files
    #[arg(long, conflicts_with = "no_respect_gitignore")]
    pub(crate) respect_gitignore: bool,

    /// Do not respect VCS ignore files
    #[arg(long)]
    pub(crate) no_respect_gitignore: bool,

    /// Apply excludes to explicit file targets
    #[arg(long, conflicts_with = "no_force_exclude")]
    pub(crate) force_exclude: bool,

    /// Do not apply excludes to explicit file targets
    #[arg(long)]
    pub(crate) no_force_exclude: bool,

    /// Exit successfully even when lint diagnostics remain
    #[arg(long, conflicts_with = "exit_non_zero_on_fix")]
    pub(crate) exit_zero: bool,

    /// Exit with diagnostics status when fixes were applied
    #[arg(long)]
    pub(crate) exit_non_zero_on_fix: bool,

    /// Input .sifr files or directories; defaults to current directory
    #[arg(value_name = "FILES")]
    pub(crate) paths: Vec<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(crate) enum LintOutputFormat {
    Concise,
    Full,
    Json,
}

impl LintOutputFormat {
    fn diagnostic_format(self) -> DiagnosticFormat {
        match self {
            Self::Concise => DiagnosticFormat::Compact,
            Self::Full => DiagnosticFormat::Human,
            Self::Json => DiagnosticFormat::Json,
        }
    }
}

pub(super) fn cmd_lint(
    args: &LintArgs,
    config_inputs: &[String],
    isolated: bool,
    diagnostic_format: DiagnosticFormat,
) -> i32 {
    let lint_format = args
        .output_format
        .map_or(diagnostic_format, LintOutputFormat::diagnostic_format);
    let result = match run_with_panic_boundary(
        "internal compiler panic during lint command execution",
        || lint_entrypoint(args, config_inputs, isolated),
    ) {
        Ok(result) => result,
        Err(internal) => return render_lint_diagnostics(&[*internal], lint_format, args, false),
    };

    match result {
        Ok(LintCommandResult::Diagnostics(diagnostics)) => {
            if diagnostics.is_empty() {
                return emit_success(lint_format, args);
            }
            let exit = render_lint_diagnostics(&diagnostics, lint_format, args, false);
            if args.exit_zero {
                EXIT_SUCCESS
            } else {
                exit
            }
        }
        Ok(LintCommandResult::Text(output)) => write_lint_output(&output, args, true),
        Ok(LintCommandResult::Statistics(diagnostics)) => {
            let write_exit = write_lint_output(&render_statistics(&diagnostics), args, true);
            if write_exit != EXIT_SUCCESS || diagnostics.is_empty() || args.exit_zero {
                write_exit
            } else {
                EXIT_USER_DIAGNOSTIC
            }
        }
        Ok(LintCommandResult::Fixes(result)) => render_fix_result(&result, lint_format, args),
        Err(errors) => render_lint_diagnostics(&errors, lint_format, args, true),
    }
}

enum LintCommandResult {
    Diagnostics(Vec<RenderedDiagnostic>),
    Statistics(Vec<RenderedDiagnostic>),
    Fixes(FixCommandResult),
    Text(String),
}

struct FixCommandResult {
    diagnostics: Vec<RenderedDiagnostic>,
    summary: String,
    diff: String,
    applied_count: usize,
}

fn lint_entrypoint(
    args: &LintArgs,
    config_inputs: &[String],
    isolated: bool,
) -> Result<LintCommandResult, Vec<RenderedDiagnostic>> {
    let overrides = lint_cli_overrides(args)?;
    let start_dir = lint_start_dir(args);
    let config = sifr_lint::effective_lint_config(&start_dir, config_inputs, isolated, &overrides)?;
    if args.show_settings {
        return Ok(LintCommandResult::Text(render_settings(&config)));
    }
    let targets = lint_targets(args);
    if args.show_files {
        let files = sifr_lint::collect_sifr_files_for_targets(&targets, &config.options)?;
        let output = files
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        return Ok(LintCommandResult::Text(format!("{output}\n")));
    }
    if fix_mode_requested(args) {
        return run_fix_command(args, &config.options).map(LintCommandResult::Fixes);
    }
    if reads_stdin(args) {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source).map_err(|err| {
            vec![diagnostic_with_code(
                format!("could not read lint source from stdin: {err}"),
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            )]
        })?;
        let file = args.stdin_filename.as_deref();
        let diagnostics = sifr_lint::lint_source(&source, file, &config.options).diagnostics;
        if args.statistics {
            return Ok(LintCommandResult::Statistics(diagnostics));
        }
        return Ok(LintCommandResult::Diagnostics(diagnostics));
    }
    let diagnostics = sifr_lint::lint_paths(&targets, &config.options)?.diagnostics;
    if args.statistics {
        return Ok(LintCommandResult::Statistics(diagnostics));
    }
    Ok(LintCommandResult::Diagnostics(diagnostics))
}

fn fix_mode_requested(args: &LintArgs) -> bool {
    args.fix || args.fix_only || args.diff || args.show_fixes
}

fn run_fix_command(
    args: &LintArgs,
    options: &sifr_lint::LintOptions,
) -> Result<FixCommandResult, Vec<RenderedDiagnostic>> {
    if reads_stdin(args) {
        let mut source = String::new();
        io::stdin().read_to_string(&mut source).map_err(|err| {
            vec![diagnostic_with_code(
                format!("could not read lint source from stdin: {err}"),
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            )]
        })?;
        let fixed = sifr_lint::fix_source(&source, args.stdin_filename.as_deref(), options);
        let diff = render_source_diff(
            args.stdin_filename
                .as_deref()
                .unwrap_or_else(|| Path::new("<stdin>")),
            &source,
            &fixed.fixed_source,
        );
        let diagnostics = fix_result_diagnostics(&fixed, args);
        return Ok(FixCommandResult {
            diagnostics,
            summary: render_fix_summary(&fixed.applied_fixes),
            diff,
            applied_count: fixed.applied_fixes.len(),
        });
    }

    let files = sifr_lint::collect_sifr_files_for_targets(&lint_targets(args), options)?;
    let mut diagnostics = Vec::new();
    let mut summary_counts = BTreeMap::<String, usize>::new();
    let mut diff = String::new();
    let mut applied_count = 0usize;

    for file in files {
        let source = std::fs::read_to_string(&file).map_err(|err| {
            vec![diagnostic_with_code(
                format!("could not read lint file {}: {err}", file.display()),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
        let fixed = sifr_lint::fix_source(&source, Some(&file), options);
        for fix in &fixed.applied_fixes {
            *summary_counts.entry(fix.rule_id.clone()).or_default() += 1;
        }
        applied_count = applied_count.saturating_add(fixed.applied_fixes.len());
        if args.diff {
            diff.push_str(&render_source_diff(&file, &source, &fixed.fixed_source));
        } else if (args.fix || args.fix_only) && fixed.fixed_source != source {
            std::fs::write(&file, &fixed.fixed_source).map_err(|err| {
                vec![diagnostic_with_code(
                    format!("could not write fixed lint file {}: {err}", file.display()),
                    DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
                )]
            })?;
        }
        diagnostics.extend(fix_result_diagnostics(&fixed, args));
    }

    Ok(FixCommandResult {
        diagnostics,
        summary: render_fix_counts(&summary_counts),
        diff,
        applied_count,
    })
}

fn fix_result_diagnostics(
    fixed: &sifr_lint::FixedSource,
    args: &LintArgs,
) -> Vec<RenderedDiagnostic> {
    if args.fix_only {
        Vec::new()
    } else if args.fix || args.diff {
        fixed.remaining_diagnostics.clone()
    } else {
        fixed.diagnostics.clone()
    }
}

fn render_fix_result(result: &FixCommandResult, format: DiagnosticFormat, args: &LintArgs) -> i32 {
    if args.diff {
        let write_exit = write_lint_output(&result.diff, args, true);
        return if write_exit != EXIT_SUCCESS || result.applied_count == 0 {
            write_exit
        } else {
            EXIT_USER_DIAGNOSTIC
        };
    }
    if args.show_fixes || args.fix_only {
        let write_exit = write_lint_output(&result.summary, args, true);
        if write_exit != EXIT_SUCCESS {
            return write_exit;
        }
    }
    if !result.diagnostics.is_empty() {
        return render_lint_diagnostics(&result.diagnostics, format, args, false);
    }
    if args.exit_non_zero_on_fix && result.applied_count > 0 {
        EXIT_USER_DIAGNOSTIC
    } else {
        EXIT_SUCCESS
    }
}

fn render_source_diff(path: &Path, before: &str, after: &str) -> String {
    if before == after {
        return String::new();
    }
    let mut output = format!("--- {}\n+++ {}\n@@\n", path.display(), path.display());
    for line in before.split_inclusive('\n') {
        output.push('-');
        output.push_str(line);
        if !line.ends_with('\n') {
            output.push('\n');
        }
    }
    for line in after.split_inclusive('\n') {
        output.push('+');
        output.push_str(line);
        if !line.ends_with('\n') {
            output.push('\n');
        }
    }
    output
}

fn render_fix_summary(fixes: &[sifr_lint::LintFix]) -> String {
    let mut counts = BTreeMap::<String, usize>::new();
    for fix in fixes {
        *counts.entry(fix.rule_id.clone()).or_default() += 1;
    }
    render_fix_counts(&counts)
}

fn render_fix_counts(counts: &BTreeMap<String, usize>) -> String {
    use std::fmt::Write as _;

    if counts.is_empty() {
        return "0 fixes\n".to_string();
    }
    let mut output = String::new();
    for (rule, count) in counts {
        let _ = writeln!(output, "{count} {rule}");
    }
    output
}

fn lint_cli_overrides(args: &LintArgs) -> Result<LintConfigOverrides, Vec<RenderedDiagnostic>> {
    Ok(LintConfigOverrides {
        select: (!args.select.is_empty()).then(|| args.select.clone()),
        extend_select: args.extend_select.clone(),
        ignore: args.ignore.clone(),
        per_file_ignores: parse_per_file_ignores(&args.per_file_ignores)?,
        extend_per_file_ignores: parse_per_file_ignores(&args.extend_per_file_ignores)?,
        exclude: args.exclude.clone(),
        extend_exclude: args.extend_exclude.clone(),
        respect_gitignore: flag_override(args.respect_gitignore, args.no_respect_gitignore),
        force_exclude: flag_override(args.force_exclude, args.no_force_exclude),
        preview: flag_override(args.preview, args.no_preview),
        ignore_suppressions: args.ignore_suppressions.then_some(true),
        fixable: args.fixable.clone(),
        extend_fixable: args.extend_fixable.clone(),
        unfixable: args.unfixable.clone(),
        extend_unfixable: args.extend_unfixable.clone(),
        unsafe_fixes: unsafe_fix_policy_override(args.unsafe_fixes, args.no_unsafe_fixes),
    })
}

fn parse_per_file_ignores(
    values: &[String],
) -> Result<Vec<PerFileIgnore>, Vec<RenderedDiagnostic>> {
    let mut ignores = Vec::new();
    for value in values {
        let Some((pattern, rules)) = value.split_once(':') else {
            return Err(vec![diagnostic_with_code(
                format!("per-file ignore must use GLOB:RULE[,RULE], got {value:?}"),
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            )]);
        };
        let rules = rules
            .split(',')
            .map(str::trim)
            .filter(|rule| !rule.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if pattern.trim().is_empty() || rules.is_empty() {
            return Err(vec![diagnostic_with_code(
                format!("per-file ignore must include a glob and at least one rule: {value:?}"),
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            )]);
        }
        ignores.push(PerFileIgnore {
            pattern: pattern.trim().to_string(),
            rules,
        });
    }
    Ok(ignores)
}

fn lint_targets(args: &LintArgs) -> Vec<PathBuf> {
    if reads_stdin(args) {
        return Vec::new();
    }
    if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    }
}

fn reads_stdin(args: &LintArgs) -> bool {
    args.paths.iter().any(|path| path == Path::new("-"))
}

fn lint_start_dir(args: &LintArgs) -> PathBuf {
    if let Some(path) = args.stdin_filename.as_deref().and_then(Path::parent) {
        return path.to_path_buf();
    }
    args.paths
        .iter()
        .find(|path| path != &Path::new("-"))
        .and_then(|path| {
            if path.is_dir() {
                Some(path.as_path())
            } else {
                path.parent()
            }
        })
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

fn render_settings(config: &EffectiveLintConfig) -> String {
    let options = &config.options;
    format!(
        "config = {}\npreview = {}\nselect = {:?}\nextend_select = {:?}\nignore = {:?}\ninclude = {:?}\nexclude = {:?}\nrespect_gitignore = {}\nforce_exclude = {}\nignore_suppressions = {}\nper_file_ignores = {}\nfixable = {:?}\nextend_fixable = {:?}\nunfixable = {:?}\nextend_unfixable = {:?}\nunsafe_fixes = {:?}\n",
        config
            .config_path
            .as_ref()
            .map_or_else(|| "<none>".to_string(), |path| path.display().to_string()),
        options.preview,
        options.select,
        options.extend_select,
        options.ignore,
        options.include,
        options.exclude,
        options.respect_gitignore,
        options.force_exclude,
        options.ignore_suppressions,
        options.per_file_ignores.len(),
        options.fixable,
        options.extend_fixable,
        options.unfixable,
        options.extend_unfixable,
        options.unsafe_fixes,
    )
}

fn render_statistics(diagnostics: &[RenderedDiagnostic]) -> String {
    use std::fmt::Write as _;

    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for diagnostic in diagnostics {
        if let Some(DiagnosticArg::String(rule)) = diagnostic.args.get("rule") {
            *counts.entry(rule.clone()).or_default() += 1;
        }
    }
    if counts.is_empty() {
        return "0 diagnostics\n".to_string();
    }
    let mut output = String::new();
    for (rule, count) in counts {
        let _ = writeln!(output, "{count} {rule}");
    }
    output
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

fn unsafe_fix_policy_override(enable: bool, disable: bool) -> Option<UnsafeFixPolicy> {
    if disable {
        Some(UnsafeFixPolicy::Disabled)
    } else if enable {
        Some(UnsafeFixPolicy::Enabled)
    } else {
        None
    }
}

fn render_lint_diagnostics(
    diagnostics: &[RenderedDiagnostic],
    format: DiagnosticFormat,
    args: &LintArgs,
    usage_error: bool,
) -> i32 {
    let output = match render_diagnostic_output(diagnostics, format) {
        Ok(output) => output,
        Err(error) => {
            let _ = writeln!(
                io::stderr(),
                "lint error: failed to serialize diagnostics as json: {error}"
            );
            return EXIT_INTERNAL_COMPILER_FAILURE;
        }
    };
    let write_exit = write_lint_output(&output, args, false);
    if write_exit != EXIT_SUCCESS {
        return write_exit;
    }
    if usage_error {
        EXIT_USAGE_OR_CONFIG
    } else {
        EXIT_USER_DIAGNOSTIC
    }
}

fn write_lint_output(output: &str, args: &LintArgs, stdout: bool) -> i32 {
    if let Some(path) = &args.output_file {
        return match std::fs::write(path, output) {
            Ok(()) => EXIT_SUCCESS,
            Err(error) => {
                let _ = writeln!(
                    io::stderr(),
                    "lint error: could not write output file {}: {error}",
                    path.display()
                );
                EXIT_USAGE_OR_CONFIG
            }
        };
    }
    if stdout {
        let _ = write!(io::stdout(), "{output}");
    } else {
        let _ = write!(io::stderr(), "{output}");
    }
    EXIT_SUCCESS
}

fn emit_success(format: DiagnosticFormat, args: &LintArgs) -> i32 {
    let output = match format {
        DiagnosticFormat::Human => "no lint diagnostics found\n".to_string(),
        DiagnosticFormat::Json => "[]\n".to_string(),
        DiagnosticFormat::Compact => {
            "summary: 0 error(s), 0 warning(s), 0 note(s), 0 help item(s)\n".to_string()
        }
    };
    write_lint_output(&output, args, false)
}
