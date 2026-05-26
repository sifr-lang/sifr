use crate::{diagnostic, LintOptions};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn collect_sifr_files(
    path: &Path,
    options: &LintOptions,
) -> Result<Vec<PathBuf>, Vec<RenderedDiagnostic>> {
    collect_sifr_files_for_targets(&[path.to_path_buf()], options)
}

pub fn collect_sifr_files_for_targets(
    paths: &[PathBuf],
    options: &LintOptions,
) -> Result<Vec<PathBuf>, Vec<RenderedDiagnostic>> {
    let targets = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.to_vec()
    };
    let include = globset("include", &options.include)?;
    let exclude_patterns = options.exclude.clone();
    let exclude = globset("exclude", &exclude_patterns)?;
    let mut files = BTreeSet::new();
    for target in targets {
        if target.is_file() {
            if should_include_explicit_file(&target, &include, &exclude, options) {
                files.insert(target);
            }
        } else if target.is_dir() {
            collect_directory_files(&target, &include, &exclude, options, &mut files)?;
        } else {
            return Err(vec![diagnostic(
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
                format!("lint target does not exist: {}", target.display()),
                [("path", target.display().to_string())],
                Vec::new(),
                None,
            )]);
        }
    }
    Ok(files.into_iter().collect())
}

fn collect_directory_files(
    root: &Path,
    include: &GlobSet,
    exclude: &GlobSet,
    options: &LintOptions,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let mut builder = WalkBuilder::new(root);
    builder
        .standard_filters(options.respect_gitignore)
        .hidden(false)
        .git_ignore(options.respect_gitignore)
        .git_global(options.respect_gitignore)
        .git_exclude(options.respect_gitignore)
        .parents(options.respect_gitignore);
    if options.respect_gitignore {
        builder
            .add_custom_ignore_filename(".gitignore")
            .add_custom_ignore_filename(".ignore");
    }
    for entry in builder.build() {
        let entry = entry.map_err(|err| {
            vec![diagnostic(
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
                format!("could not read lint target under {}: {err}", root.display()),
                [("path", root.display().to_string())],
                Vec::new(),
                None,
            )]
        })?;
        let path = entry.path();
        if path.is_dir() || is_default_excluded_dir(path) {
            continue;
        }
        if path_matches(root, path, include) && !path_matches(root, path, exclude) {
            files.insert(path.to_path_buf());
        }
    }
    Ok(())
}

fn should_include_explicit_file(
    path: &Path,
    include: &GlobSet,
    exclude: &GlobSet,
    options: &LintOptions,
) -> bool {
    if options.force_exclude && path_matches(Path::new("."), path, exclude) {
        return false;
    }
    is_sifr_file(path) || path_matches(Path::new("."), path, include)
}

fn path_matches(root: &Path, path: &Path, set: &GlobSet) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    set.is_match(relative) || set.is_match(path)
}

fn globset(kind: &str, patterns: &[String]) -> Result<GlobSet, Vec<RenderedDiagnostic>> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern).map_err(|err| {
            vec![diagnostic(
                DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
                format!("invalid lint {kind} glob {pattern:?}: {err}"),
                [("pattern", pattern.clone())],
                Vec::new(),
                None,
            )]
        })?);
    }
    builder.build().map_err(|err| {
        vec![diagnostic(
            DiagnosticCode::WORKSPACE_INVALID_SOURCE_ROOT,
            format!("invalid lint {kind} glob set: {err}"),
            [("kind", kind.to_string())],
            Vec::new(),
            None,
        )]
    })
}

fn is_sifr_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "sifr")
}

fn is_default_excluded_dir(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        matches!(
            name.to_string_lossy().as_ref(),
            ".git" | "target" | ".venv" | "venv" | "node_modules" | "sifr_output"
        )
    })
}
