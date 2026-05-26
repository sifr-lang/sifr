use super::cli_model_and_entrypoint::diagnostic_with_code;
use super::formatter_cli::FmtArgs;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct EffectiveFmtConfig {
    pub(crate) format_options: sifr_format::FormatOptions,
    pub(crate) exclude: Vec<String>,
    pub(crate) respect_gitignore: bool,
    pub(crate) force_exclude: bool,
    pub(crate) no_cache: bool,
    pub(crate) cache_dir: PathBuf,
}

impl Default for EffectiveFmtConfig {
    fn default() -> Self {
        Self {
            format_options: sifr_format::FormatOptions::default(),
            exclude: Vec::new(),
            respect_gitignore: true,
            force_exclude: false,
            no_cache: false,
            cache_dir: PathBuf::from(".sifr_cache/formatter"),
        }
    }
}

pub(crate) fn effective_fmt_config(
    args: &FmtArgs,
    config_inputs: &[String],
    isolated: bool,
) -> Result<EffectiveFmtConfig, Vec<RenderedDiagnostic>> {
    let mut config = EffectiveFmtConfig::default();
    if !isolated {
        if let Some(path) = discover_sifr_toml()? {
            apply_config_file(&mut config, &path, &mut BTreeSet::new())?;
        }
        for input in config_inputs {
            if let Some((key, value)) = input.split_once('=') {
                apply_config_override(&mut config, key, value)?;
            } else {
                apply_config_file(&mut config, Path::new(input), &mut BTreeSet::new())?;
            }
        }
    }
    apply_cli_overrides(&mut config, args);
    Ok(config)
}

fn discover_sifr_toml() -> Result<Option<PathBuf>, Vec<RenderedDiagnostic>> {
    let cwd = std::env::current_dir().map_err(|err| {
        vec![fmt_diagnostic(format!(
            "could not read current directory: {err}"
        ))]
    })?;
    for dir in cwd.ancestors() {
        let candidate = dir.join("sifr.toml");
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

fn apply_config_file(
    config: &mut EffectiveFmtConfig,
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let canonical = path.canonicalize().map_err(|err| {
        vec![fmt_diagnostic(format!(
            "could not resolve formatter config {}: {err}",
            path.display()
        ))]
    })?;
    if !seen.insert(canonical.clone()) {
        return Err(vec![fmt_diagnostic(format!(
            "formatter config extend cycle includes {}",
            path.display()
        ))]);
    }
    let source = fs::read_to_string(&canonical).map_err(|err| {
        vec![fmt_diagnostic(format!(
            "could not read formatter config {}: {err}",
            canonical.display()
        ))]
    })?;
    let value = toml::from_str::<toml::Value>(&source).map_err(|err| {
        vec![fmt_diagnostic(format!(
            "could not parse formatter config {}: {err}",
            canonical.display()
        ))]
    })?;
    if let Some(extend) = value.get("extend").and_then(toml::Value::as_str) {
        let parent = canonical.parent().unwrap_or_else(|| Path::new("."));
        apply_config_file(config, &parent.join(extend), seen)?;
    }
    if let Some(format) = value.get("format") {
        apply_format_table(config, format)?;
    }
    Ok(())
}

fn apply_format_table(
    config: &mut EffectiveFmtConfig,
    value: &toml::Value,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(table) = value.as_table() else {
        return Err(vec![fmt_diagnostic(
            "formatter config [format] must be a table",
        )]);
    };
    for (key, value) in table {
        match key.as_str() {
            "line-length" | "line_length" => {
                config.format_options.line_length = as_u16(key, value)?;
            }
            "preview" => {
                config.format_options.preview = as_bool(key, value)?;
            }
            "exclude" => {
                config.exclude = as_string_list(key, value)?;
            }
            "respect-gitignore" | "respect_gitignore" => {
                config.respect_gitignore = as_bool(key, value)?;
            }
            "force-exclude" | "force_exclude" => {
                config.force_exclude = as_bool(key, value)?;
            }
            "no-cache" | "no_cache" => {
                config.no_cache = as_bool(key, value)?;
            }
            "cache-dir" | "cache_dir" => {
                config.cache_dir = PathBuf::from(as_string(key, value)?);
            }
            "target-version" | "target_version" | "extension" => {
                return Err(vec![fmt_diagnostic(format!(
                    "unsupported Python formatter option in Sifr config: {key}"
                ))]);
            }
            _ => {
                return Err(vec![fmt_diagnostic(format!(
                    "unknown formatter config key: {key}"
                ))]);
            }
        }
    }
    Ok(())
}

fn apply_config_override(
    config: &mut EffectiveFmtConfig,
    key: &str,
    raw_value: &str,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let value = raw_value
        .parse::<toml::Value>()
        .unwrap_or_else(|_| toml::Value::String(raw_value.trim_matches('"').to_string()));
    let mut table = toml::map::Map::new();
    table.insert(key.to_string(), value);
    apply_format_table(config, &toml::Value::Table(table))
}

fn apply_cli_overrides(config: &mut EffectiveFmtConfig, args: &FmtArgs) {
    if let Some(line_length) = args.line_length {
        config.format_options.line_length = line_length;
    }
    if args.preview {
        config.format_options.preview = true;
    }
    if args.no_preview {
        config.format_options.preview = false;
    }
    if !args.exclude.is_empty() {
        config.exclude.extend(args.exclude.clone());
    }
    if args.respect_gitignore {
        config.respect_gitignore = true;
    }
    if args.no_respect_gitignore {
        config.respect_gitignore = false;
    }
    if args.force_exclude {
        config.force_exclude = true;
    }
    if args.no_force_exclude {
        config.force_exclude = false;
    }
    if args.no_cache {
        config.no_cache = true;
    }
    if let Some(cache_dir) = &args.cache_dir {
        config.cache_dir = cache_dir.clone();
    }
}

fn as_u16(key: &str, value: &toml::Value) -> Result<u16, Vec<RenderedDiagnostic>> {
    let Some(raw) = value.as_integer() else {
        return Err(vec![fmt_diagnostic(format!("{key} must be an integer"))]);
    };
    u16::try_from(raw).map_err(|_| vec![fmt_diagnostic(format!("{key} is out of range"))])
}

fn as_bool(key: &str, value: &toml::Value) -> Result<bool, Vec<RenderedDiagnostic>> {
    value
        .as_bool()
        .ok_or_else(|| vec![fmt_diagnostic(format!("{key} must be a boolean"))])
}

fn as_string(key: &str, value: &toml::Value) -> Result<String, Vec<RenderedDiagnostic>> {
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| vec![fmt_diagnostic(format!("{key} must be a string"))])
}

fn as_string_list(key: &str, value: &toml::Value) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let Some(values) = value.as_array() else {
        return Err(vec![fmt_diagnostic(format!("{key} must be an array"))]);
    };
    values
        .iter()
        .map(|value| as_string(key, value))
        .collect::<Result<Vec<_>, _>>()
}

fn fmt_diagnostic(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic_with_code(message, DiagnosticCode::FMT_FORMATTING_DRIFT)
}
