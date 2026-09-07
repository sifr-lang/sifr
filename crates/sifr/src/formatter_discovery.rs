//! Interpret the working directory's gitignore at its own path boundary.

use super::check_and_package_commands::formatter_cli_diagnostic;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::SourceProvider;
use std::path::{Component, Path, PathBuf};

pub(super) struct FormatterGitignore {
    root: PathBuf,
    matcher: Gitignore,
}

impl FormatterGitignore {
    pub(super) fn load(
        cwd: &Path,
        enabled: bool,
        provider: &mut impl SourceProvider,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let root = normalize_path(cwd);
        let path = root.join(".gitignore");
        let mut builder = GitignoreBuilder::new(&root);
        if enabled && provider.is_file(&path) {
            let source = provider.read_file(&path).map_err(|error| {
                vec![formatter_cli_diagnostic(format!(
                    "could not read .gitignore for formatter discovery: {error}"
                ))]
            })?;
            for (index, line) in source.as_str().lines().enumerate() {
                let line = if index == 0 {
                    line.trim_start_matches('\u{feff}')
                } else {
                    line
                };
                builder
                    .add_line(Some(path.clone()), line)
                    .map_err(|error| {
                        vec![formatter_cli_diagnostic(format!(
                            "invalid formatter gitignore {}:{}: {error}",
                            path.display(),
                            index + 1
                        ))]
                    })?;
            }
        }
        let matcher = builder.build().map_err(|error| {
            vec![formatter_cli_diagnostic(format!(
                "could not compile formatter gitignore {}: {error}",
                path.display()
            ))]
        })?;
        Ok(Self { root, matcher })
    }

    pub(super) fn is_ignored(
        &self,
        path: &Path,
        provider: &mut impl SourceProvider,
    ) -> Result<bool, Vec<RenderedDiagnostic>> {
        if self.matcher.is_empty() {
            return Ok(false);
        }
        let absolute = self.root.join(path);
        let Some(parent) = absolute.parent() else {
            return Ok(false);
        };
        let Some(name) = absolute.file_name() else {
            return Ok(false);
        };
        // Resolve directory aliases (e.g. macOS /var -> /private/var), but
        // preserve a file symlink's name for gitignore matching.
        let parent = provider.canonicalize(parent).map_err(|error| {
            vec![formatter_cli_diagnostic(format!(
                "could not resolve formatter discovery path {}: {error}",
                parent.display()
            ))]
        })?;
        Ok(self.matches_resolved_path(&parent.join(name)))
    }

    fn matches_resolved_path(&self, path: &Path) -> bool {
        let absolute = normalize_path(&self.root.join(path));
        // A working-directory gitignore has no authority over an outside target.
        let Ok(relative) = absolute.strip_prefix(&self.root) else {
            return false;
        };
        self.matcher
            .matched_path_or_any_parents(relative, false)
            .is_ignore()
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_frontend::DiskSourceProvider;

    fn matcher(root: &Path, rules: &str, enabled: bool) -> FormatterGitignore {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join(".gitignore"), rules).unwrap();
        FormatterGitignore::load(root, enabled, &mut DiskSourceProvider::new()).unwrap()
    }

    #[test]
    fn formatter_discovery_root_ignores_do_not_match_host_ancestors() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("tmp/checkout");
        let ignore = matcher(&root, "/tmp/\n*.generated.sifr\n", true);
        for path in [
            root.join("project/main.sifr"),
            PathBuf::from("project/main.sifr"),
            PathBuf::from("./project/../project/main.sifr"),
        ] {
            assert!(!ignore.matches_resolved_path(&path), "{}", path.display());
        }
        assert!(ignore.matches_resolved_path(&root.join("tmp/main.sifr")));
        assert!(ignore.matches_resolved_path(Path::new("project/a.generated.sifr")));
        assert!(!ignore.matches_resolved_path(&dir.path().join("outside/a.generated.sifr")));
        assert!(!ignore.matches_resolved_path(Path::new("../outside/a.generated.sifr")));
    }

    #[test]
    fn formatter_discovery_uses_globs_components_and_negation() {
        let dir = tempfile::tempdir().unwrap();
        let ignore = matcher(dir.path(), "build/\n*.sifr\n!keep.sifr\n", true);
        assert!(ignore.matches_resolved_path(Path::new("nested/build/main.sifr")));
        assert!(ignore.matches_resolved_path(Path::new("nested/main.sifr")));
        assert!(!ignore.matches_resolved_path(Path::new("nested/keep.sifr")));
        assert!(!ignore.matches_resolved_path(Path::new("rebuild/readme.txt")));
    }

    #[test]
    fn formatter_discovery_disabled_ignores_select_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let ignore = matcher(dir.path(), "*\n", false);
        assert!(!ignore.matches_resolved_path(&dir.path().join("main.sifr")));
    }
}
