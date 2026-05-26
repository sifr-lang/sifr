use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct EffectiveFormatConfig {
    pub format_options: crate::FormatOptions,
    pub exclude: Vec<String>,
    pub respect_gitignore: bool,
    pub force_exclude: bool,
    pub no_cache: bool,
    pub cache_dir: PathBuf,
}

impl Default for EffectiveFormatConfig {
    fn default() -> Self {
        Self {
            format_options: crate::FormatOptions::default(),
            exclude: Vec::new(),
            respect_gitignore: true,
            force_exclude: false,
            no_cache: false,
            cache_dir: PathBuf::from(".sifr_cache/formatter"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FormatConfigOverrides {
    pub line_length: Option<u16>,
    pub preview: Option<bool>,
    pub exclude: Vec<String>,
    pub respect_gitignore: Option<bool>,
    pub force_exclude: Option<bool>,
    pub no_cache: Option<bool>,
    pub cache_dir: Option<PathBuf>,
}

pub fn effective_format_config(
    start_dir: &Path,
    config_inputs: &[String],
    isolated: bool,
    overrides: &FormatConfigOverrides,
) -> Result<EffectiveFormatConfig, Vec<RenderedDiagnostic>> {
    let mut config = EffectiveFormatConfig::default();
    if !isolated {
        if let Some(path) = discover_sifr_toml(start_dir)? {
            apply_config_file(&mut config, &path, &mut BTreeSet::new())?;
        }
    }
    for input in config_inputs {
        if let Some((key, value)) = input.split_once('=') {
            apply_config_override(&mut config, key, value)?;
        } else if !isolated {
            apply_config_file(&mut config, Path::new(input), &mut BTreeSet::new())?;
        }
    }
    apply_overrides(&mut config, overrides);
    Ok(config)
}

pub fn effective_format_options_for_file(
    path: &Path,
) -> Result<crate::FormatOptions, Vec<RenderedDiagnostic>> {
    let start_dir = path.parent().unwrap_or_else(|| Path::new("."));
    effective_format_config(start_dir, &[], false, &FormatConfigOverrides::default())
        .map(|config| config.format_options)
}

fn discover_sifr_toml(start_dir: &Path) -> Result<Option<PathBuf>, Vec<RenderedDiagnostic>> {
    let base = if start_dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        start_dir
    };
    let canonical = base.canonicalize().map_err(|err| {
        vec![fmt_diagnostic(format!(
            "could not resolve formatter config start directory {}: {err}",
            base.display()
        ))]
    })?;
    for dir in canonical.ancestors() {
        let candidate = dir.join("sifr.toml");
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

fn apply_config_file(
    config: &mut EffectiveFormatConfig,
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
    let parent = canonical.parent().unwrap_or_else(|| Path::new("."));
    apply_extends(config, value.get("extend"), parent, seen)?;
    if let Some(format) = value.get("format") {
        apply_extends(config, format.get("extend"), parent, seen)?;
        apply_format_table(config, format)?;
    }
    Ok(())
}

fn apply_extends(
    config: &mut EffectiveFormatConfig,
    value: Option<&toml::Value>,
    parent: &Path,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(value) = value else {
        return Ok(());
    };
    if let Some(path) = value.as_str() {
        return apply_config_file(config, &parent.join(path), seen);
    }
    let Some(paths) = value.as_array() else {
        return Err(vec![fmt_diagnostic("extend must be a string or array")]);
    };
    for path in paths {
        let path = as_string("extend", path)?;
        apply_config_file(config, &parent.join(path), seen)?;
    }
    Ok(())
}

fn apply_format_table(
    config: &mut EffectiveFormatConfig,
    value: &toml::Value,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(table) = value.as_table() else {
        return Err(vec![fmt_diagnostic(
            "formatter config [format] must be a table",
        )]);
    };
    for (key, value) in table {
        match key.as_str() {
            "extend" => {}
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
            "cache" => {
                config.no_cache = !as_bool(key, value)?;
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
    config: &mut EffectiveFormatConfig,
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

fn apply_overrides(config: &mut EffectiveFormatConfig, overrides: &FormatConfigOverrides) {
    if let Some(line_length) = overrides.line_length {
        config.format_options.line_length = line_length;
    }
    if let Some(preview) = overrides.preview {
        config.format_options.preview = preview;
    }
    if !overrides.exclude.is_empty() {
        config.exclude.extend(overrides.exclude.clone());
    }
    if let Some(respect_gitignore) = overrides.respect_gitignore {
        config.respect_gitignore = respect_gitignore;
    }
    if let Some(force_exclude) = overrides.force_exclude {
        config.force_exclude = force_exclude;
    }
    if let Some(no_cache) = overrides.no_cache {
        config.no_cache = no_cache;
    }
    if let Some(cache_dir) = &overrides.cache_dir {
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
    let message = message.into();
    let code = DiagnosticCode::FMT_FORMATTING_DRIFT;
    let mut args = BTreeMap::new();
    args.insert(
        "message".to_string(),
        DiagnosticArg::String(message.clone()),
    );
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: "{message}".to_string(),
        args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}
