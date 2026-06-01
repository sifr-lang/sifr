use crate::{
    lint_config_diagnostic, rule_metadata, EffectiveLintConfig, LintConfigOverrides, LintOptions,
    PerFileIgnore, RuleSeverity, UnsafeFixPolicy, RULES,
};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{DiskSourceProvider, SourceProvider};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn effective_lint_config(
    start_dir: &Path,
    config_inputs: &[String],
    isolated: bool,
    overrides: &LintConfigOverrides,
) -> Result<EffectiveLintConfig, Vec<RenderedDiagnostic>> {
    let mut config = EffectiveLintConfig::default();
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
    apply_overrides(&mut config.options, overrides);
    validate_rule_selectors(&config.options)?;
    Ok(config)
}

fn discover_sifr_toml(start_dir: &Path) -> Result<Option<PathBuf>, Vec<RenderedDiagnostic>> {
    let mut provider = DiskSourceProvider::new();
    discover_sifr_toml_with_provider(start_dir, &mut provider)
}

fn discover_sifr_toml_with_provider(
    start_dir: &Path,
    provider: &mut impl SourceProvider,
) -> Result<Option<PathBuf>, Vec<RenderedDiagnostic>> {
    let base = if start_dir.as_os_str().is_empty() {
        Path::new(".")
    } else {
        start_dir
    };
    let canonical = provider.canonicalize(base).map_err(|err| {
        vec![lint_config_diagnostic(format!(
            "could not resolve lint config start directory {}: {err}",
            base.display()
        ))]
    })?;
    for dir in canonical.ancestors() {
        let candidate = dir.join("sifr.toml");
        if provider.is_file(&candidate) {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

fn apply_config_file(
    config: &mut EffectiveLintConfig,
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let mut provider = DiskSourceProvider::new();
    apply_config_file_with_provider(config, path, seen, &mut provider)
}

fn apply_config_file_with_provider(
    config: &mut EffectiveLintConfig,
    path: &Path,
    seen: &mut BTreeSet<PathBuf>,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let canonical = provider.canonicalize(path).map_err(|err| {
        vec![lint_config_diagnostic(format!(
            "could not resolve lint config {}: {err}",
            path.display()
        ))]
    })?;
    if !seen.insert(canonical.clone()) {
        return Err(vec![lint_config_diagnostic(format!(
            "lint config extend cycle includes {}",
            path.display()
        ))]);
    }
    let source = provider
        .read_file(&canonical)
        .map(|source| source.as_str().to_string())
        .map_err(|err| {
            vec![lint_config_diagnostic(format!(
                "could not read lint config {}: {err}",
                canonical.display()
            ))]
        })?;
    let value = toml::from_str::<toml::Value>(&source).map_err(|err| {
        vec![lint_config_diagnostic(format!(
            "could not parse lint config {}: {err}",
            canonical.display()
        ))]
    })?;
    let parent = canonical.parent().unwrap_or_else(|| Path::new("."));
    apply_extends(config, value.get("extend"), parent, seen, provider)?;
    if let Some(lint) = value.get("lint") {
        apply_extends(config, lint.get("extend"), parent, seen, provider)?;
        apply_lint_table(&mut config.options, lint)?;
        config.config_path = Some(canonical);
    }
    Ok(())
}

fn apply_extends(
    config: &mut EffectiveLintConfig,
    value: Option<&toml::Value>,
    parent: &Path,
    seen: &mut BTreeSet<PathBuf>,
    provider: &mut impl SourceProvider,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(value) = value else {
        return Ok(());
    };
    if let Some(path) = value.as_str() {
        return apply_config_file_with_provider(config, &parent.join(path), seen, provider);
    }
    let Some(paths) = value.as_array() else {
        return Err(vec![lint_config_diagnostic(
            "extend must be a string or array",
        )]);
    };
    for path in paths {
        let path = as_string("extend", path)?;
        apply_config_file_with_provider(config, &parent.join(path), seen, provider)?;
    }
    Ok(())
}

fn apply_lint_table(
    options: &mut LintOptions,
    value: &toml::Value,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(table) = value.as_table() else {
        return Err(vec![lint_config_diagnostic("[lint] must be a table")]);
    };
    for (key, value) in table {
        match key.as_str() {
            "extend" => {}
            "preview" => options.preview = as_bool(key, value)?,
            "select" => options.select = as_string_list(key, value)?,
            "extend-select" | "extend_select" => {
                options.extend_select.extend(as_string_list(key, value)?);
            }
            "ignore" => options.ignore.extend(as_string_list(key, value)?),
            "include" => options.include = as_string_list(key, value)?,
            "extend-include" | "extend_include" => {
                options.include.extend(as_string_list(key, value)?);
            }
            "exclude" => options.exclude = as_string_list(key, value)?,
            "extend-exclude" | "extend_exclude" => {
                options.exclude.extend(as_string_list(key, value)?);
            }
            "respect-gitignore" | "respect_gitignore" => {
                options.respect_gitignore = as_bool(key, value)?;
            }
            "force-exclude" | "force_exclude" => options.force_exclude = as_bool(key, value)?,
            "rules" => apply_rule_table(options, value)?,
            "per-file-ignores" | "per_file_ignores" => apply_per_file_ignores(options, value)?,
            "fixable" => options.fixable = as_string_list(key, value)?,
            "extend-fixable" | "extend_fixable" => {
                options.extend_fixable.extend(as_string_list(key, value)?);
            }
            "unfixable" => options.unfixable = as_string_list(key, value)?,
            "extend-unfixable" | "extend_unfixable" => {
                options.extend_unfixable.extend(as_string_list(key, value)?);
            }
            "unsafe-fixes" | "unsafe_fixes" => options.unsafe_fixes = as_unsafe_fixes(key, value)?,
            "extend-ignore" | "extend_ignore" | "target-version" | "target_version"
            | "extension" | "src" | "namespace-packages" | "namespace_packages" | "builtins"
            | "typing-modules" | "typing_modules" => {
                return Err(vec![lint_config_diagnostic(format!(
                    "unsupported Ruff/Python lint option in Sifr config: {key}"
                ))]);
            }
            _ => {
                return Err(vec![lint_config_diagnostic(format!(
                    "unknown lint config key: {key}"
                ))]);
            }
        }
    }
    Ok(())
}

fn apply_rule_table(
    options: &mut LintOptions,
    value: &toml::Value,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(table) = value.as_table() else {
        return Err(vec![lint_config_diagnostic("[lint.rules] must be a table")]);
    };
    for (rule, level) in table {
        require_known_rule(rule)?;
        options
            .rule_levels
            .insert(rule.clone(), as_rule_severity(rule, level)?);
    }
    Ok(())
}

fn apply_per_file_ignores(
    options: &mut LintOptions,
    value: &toml::Value,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let Some(table) = value.as_table() else {
        return Err(vec![lint_config_diagnostic(
            "[lint.per-file-ignores] must be a table",
        )]);
    };
    for (pattern, rules) in table {
        let rules = as_string_list(pattern, rules)?;
        for rule in &rules {
            require_known_rule(rule)?;
        }
        options.per_file_ignores.push(PerFileIgnore {
            pattern: pattern.clone(),
            rules,
        });
    }
    Ok(())
}

fn apply_config_override(
    config: &mut EffectiveLintConfig,
    key: &str,
    raw_value: &str,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let value = raw_value
        .parse::<toml::Value>()
        .unwrap_or_else(|_| toml::Value::String(raw_value.trim_matches('"').to_string()));
    let mut table = toml::map::Map::new();
    table.insert(key.to_string(), value);
    apply_lint_table(&mut config.options, &toml::Value::Table(table))
}

fn apply_overrides(options: &mut LintOptions, overrides: &LintConfigOverrides) {
    if let Some(select) = &overrides.select {
        options.select.clone_from(select);
    }
    options
        .extend_select
        .extend(overrides.extend_select.clone());
    options.ignore.extend(overrides.ignore.clone());
    options
        .per_file_ignores
        .extend(overrides.per_file_ignores.clone());
    options
        .per_file_ignores
        .extend(overrides.extend_per_file_ignores.clone());
    options.fixable.extend(overrides.fixable.clone());
    options
        .extend_fixable
        .extend(overrides.extend_fixable.clone());
    options.unfixable.extend(overrides.unfixable.clone());
    options
        .extend_unfixable
        .extend(overrides.extend_unfixable.clone());
    options.exclude.extend(overrides.exclude.clone());
    options.exclude.extend(overrides.extend_exclude.clone());
    if let Some(respect_gitignore) = overrides.respect_gitignore {
        options.respect_gitignore = respect_gitignore;
    }
    if let Some(force_exclude) = overrides.force_exclude {
        options.force_exclude = force_exclude;
    }
    if let Some(preview) = overrides.preview {
        options.preview = preview;
    }
    if let Some(ignore_suppressions) = overrides.ignore_suppressions {
        options.ignore_suppressions = ignore_suppressions;
    }
    if let Some(unsafe_fixes) = overrides.unsafe_fixes {
        options.unsafe_fixes = unsafe_fixes;
    }
}

fn validate_rule_selectors(options: &LintOptions) -> Result<(), Vec<RenderedDiagnostic>> {
    for selector in options
        .select
        .iter()
        .chain(options.extend_select.iter())
        .chain(options.ignore.iter())
        .chain(options.fixable.iter())
        .chain(options.extend_fixable.iter())
        .chain(options.unfixable.iter())
        .chain(options.extend_unfixable.iter())
    {
        if selector == "default" || selector == "all" {
            continue;
        }
        if RULES
            .iter()
            .any(|rule| rule.id == selector || rule.category == selector)
        {
            continue;
        }
        return Err(vec![lint_config_diagnostic(format!(
            "unknown Sifr lint rule selector: {selector}"
        ))]);
    }
    Ok(())
}

fn as_bool(key: &str, value: &toml::Value) -> Result<bool, Vec<RenderedDiagnostic>> {
    value
        .as_bool()
        .ok_or_else(|| vec![lint_config_diagnostic(format!("{key} must be a boolean"))])
}

fn as_string(key: &str, value: &toml::Value) -> Result<String, Vec<RenderedDiagnostic>> {
    value
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| vec![lint_config_diagnostic(format!("{key} must be a string"))])
}

fn as_string_list(key: &str, value: &toml::Value) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let Some(values) = value.as_array() else {
        return Err(vec![lint_config_diagnostic(format!(
            "{key} must be an array"
        ))]);
    };
    values
        .iter()
        .map(|value| as_string(key, value))
        .collect::<Result<Vec<_>, _>>()
}

fn as_rule_severity(
    key: &str,
    value: &toml::Value,
) -> Result<RuleSeverity, Vec<RenderedDiagnostic>> {
    match as_string(key, value)?.as_str() {
        "ignore" => Ok(RuleSeverity::Ignore),
        "warn" => Ok(RuleSeverity::Warn),
        "error" => Ok(RuleSeverity::Error),
        _ => Err(vec![lint_config_diagnostic(format!(
            "{key} severity must be one of ignore, warn, or error"
        ))]),
    }
}

fn as_unsafe_fixes(
    key: &str,
    value: &toml::Value,
) -> Result<UnsafeFixPolicy, Vec<RenderedDiagnostic>> {
    match as_string(key, value)?.as_str() {
        "disabled" => Ok(UnsafeFixPolicy::Disabled),
        "hint" => Ok(UnsafeFixPolicy::Hint),
        "enabled" => Ok(UnsafeFixPolicy::Enabled),
        _ => Err(vec![lint_config_diagnostic(format!(
            "{key} must be one of disabled, hint, or enabled"
        ))]),
    }
}

fn require_known_rule(rule: &str) -> Result<(), Vec<RenderedDiagnostic>> {
    if rule_metadata(rule).is_some() {
        Ok(())
    } else {
        Err(vec![lint_config_diagnostic(format!(
            "unknown Sifr lint rule id: {rule}"
        ))])
    }
}
