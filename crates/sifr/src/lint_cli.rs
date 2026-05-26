use super::cli_model_and_entrypoint::{
    diagnostic_with_code, run_with_panic_boundary, DiagnosticFormat,
    EXIT_INTERNAL_COMPILER_FAILURE, EXIT_SUCCESS, EXIT_USAGE_OR_CONFIG, EXIT_USER_DIAGNOSTIC,
};
use super::diagnostic_rendering_and_run::render_diagnostic_output;
use clap::{Args, ValueEnum};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_lint::{EffectiveLintConfig, LintConfigOverrides, PerFileIgnore};
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
    #[arg(long)]
    pub(crate) show_settings: bool,

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
    #[arg(long)]
    pub(crate) exit_zero: bool,

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
        Err(errors) => render_lint_diagnostics(&errors, lint_format, args, true),
    }
}

enum LintCommandResult {
    Diagnostics(Vec<RenderedDiagnostic>),
    Text(String),
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
        return Ok(LintCommandResult::Diagnostics(diagnostics));
    }
    let diagnostics = sifr_lint::lint_paths(&targets, &config.options)?.diagnostics;
    Ok(LintCommandResult::Diagnostics(diagnostics))
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
        "config = {}\npreview = {}\nselect = {:?}\nextend_select = {:?}\nignore = {:?}\ninclude = {:?}\nexclude = {:?}\nrespect_gitignore = {}\nforce_exclude = {}\nper_file_ignores = {}\n",
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
        options.per_file_ignores.len(),
    )
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
