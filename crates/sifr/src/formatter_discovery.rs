//! Interpret the working directory's gitignore at its own path boundary.

use super::check_and_package_commands::formatter_cli_diagnostic;
use ignore::gitignore::GitignoreBuilder;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::SourceProvider;
use std::path::{Component, Path, PathBuf};

pub(super) struct FormatterGitignore {
    root: PathBuf,
    rules: Vec<Rule>,
}

struct Rule {
    source: String,
    mandatory_literal: Option<String>,
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
        let mut rules = Vec::new();
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
                if !line.is_empty() && !line.starts_with('#') {
                    rules.push(Rule {
                        source: line.to_string(),
                        mandatory_literal: mandatory_literal(line),
                    });
                }
            }
        }
        Ok(Self { root, rules })
    }

    pub(super) fn is_ignored(
        &self,
        path: &Path,
        provider: &mut impl SourceProvider,
    ) -> Result<bool, Vec<RenderedDiagnostic>> {
        if self.rules.is_empty() {
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
        self.matches_resolved_path(&parent.join(name))
    }

    fn matches_resolved_path(&self, path: &Path) -> Result<bool, Vec<RenderedDiagnostic>> {
        let absolute = normalize_path(&self.root.join(path));
        // A working-directory gitignore has no authority over an outside target.
        let Ok(relative) = absolute.strip_prefix(&self.root) else {
            return Ok(false);
        };
        // Skip automaton construction only when a mandatory literal cannot
        // occur in this path or its parents. The gitignore engine still owns
        // all possible matches and their ordering.
        let mut builder = GitignoreBuilder::new(&self.root);
        let mut relevant = false;
        for rule in &self.rules {
            if could_match(rule, relative) {
                relevant = true;
                builder.add_line(None, &rule.source).map_err(ignore_error)?;
            }
        }
        if !relevant {
            return Ok(false);
        }
        Ok(builder
            .build()
            .map_err(ignore_error)?
            .matched_path_or_any_parents(relative, false)
            .is_ignore())
    }
}

fn ignore_error(error: ignore::Error) -> Vec<RenderedDiagnostic> {
    vec![formatter_cli_diagnostic(format!(
        "could not compile formatter gitignore: {error}"
    ))]
}

// Compute this necessary condition once, rather than rescanning every rule
// for each selected source. Escaped or compound syntax stays with the engine.
fn mandatory_literal(rule: &str) -> Option<String> {
    if rule.contains(['\\', '[', ']', '{', '}']) {
        return None;
    }
    let rule = rule.trim_end();
    let rule = rule.strip_prefix('!').unwrap_or(rule);
    rule.split(['*', '?', '/'])
        .max_by_key(|literal| literal.len())
        .map(str::to_string)
}

fn could_match(rule: &Rule, path: &Path) -> bool {
    match (&rule.mandatory_literal, path.to_str()) {
        (Some(literal), Some(path)) => path.contains(literal),
        _ => true,
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
            assert!(
                !ignore.matches_resolved_path(&path).unwrap(),
                "{}",
                path.display()
            );
        }
        assert!(
            ignore
                .matches_resolved_path(&root.join("tmp/main.sifr"))
                .unwrap()
        );
        assert!(
            ignore
                .matches_resolved_path(Path::new("project/a.generated.sifr"))
                .unwrap()
        );
        assert!(
            !ignore
                .matches_resolved_path(&dir.path().join("outside/a.generated.sifr"))
                .unwrap()
        );
        assert!(
            !ignore
                .matches_resolved_path(Path::new("../outside/a.generated.sifr"))
                .unwrap()
        );
    }

    #[test]
    fn formatter_discovery_uses_globs_components_and_negation() {
        let dir = tempfile::tempdir().unwrap();
        let ignore = matcher(dir.path(), "build/\n*.sifr\n!keep.sifr\n", true);
        assert!(
            ignore
                .matches_resolved_path(Path::new("nested/build/main.sifr"))
                .unwrap()
        );
        assert!(
            ignore
                .matches_resolved_path(Path::new("nested/main.sifr"))
                .unwrap()
        );
        assert!(
            !ignore
                .matches_resolved_path(Path::new("nested/keep.sifr"))
                .unwrap()
        );
        assert!(
            !ignore
                .matches_resolved_path(Path::new("rebuild/readme.txt"))
                .unwrap()
        );
    }

    #[test]
    fn formatter_discovery_disabled_ignores_select_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let ignore = matcher(dir.path(), "*\n", false);
        assert!(
            !ignore
                .matches_resolved_path(&dir.path().join("main.sifr"))
                .unwrap()
        );
    }

    #[test]
    fn formatter_discovery_malformed_rules_keep_source_line_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        for rule in ["{a,b", "[z-a]", "\\"] {
            std::fs::write(
                dir.path().join(".gitignore"),
                format!("# rules\n\n{rule}\n"),
            )
            .unwrap();
            let result = FormatterGitignore::load(dir.path(), true, &mut DiskSourceProvider::new());
            let Err(diagnostics) = result else {
                panic!("malformed rule {rule:?} must retain its diagnostic");
            };
            assert_eq!(diagnostics.len(), 1);
            assert_eq!(diagnostics[0].code, "SIFR-FMT-0001");
            assert!(
                diagnostics[0]
                    .message
                    .contains("invalid formatter gitignore")
            );
            assert!(diagnostics[0].message.contains(".gitignore:3:"));
        }
    }

    #[test]
    fn formatter_discovery_disabled_ignores_do_not_parse_malformed_rules() {
        let dir = tempfile::tempdir().unwrap();
        for rule in ["{a,b", "[z-a]", "\\"] {
            let ignore = matcher(dir.path(), &format!("*.sifr\n{rule}\n"), false);
            assert!(
                !ignore
                    .matches_resolved_path(Path::new("nested/main.sifr"))
                    .unwrap()
            );
        }
    }

    #[test]
    fn formatter_discovery_literal_filter_agrees_with_complete_engine() {
        let dir = tempfile::tempdir().unwrap();
        let rules = "/tmp/\n*.sifr\n!keep.sifr\nfoo/**/bar\n[ab].txt\n\\#name\n{one,two}.rs\né?*.log\nspace\\ \n";
        let ignore = matcher(dir.path(), rules, true);
        let mut full = GitignoreBuilder::new(dir.path());
        for rule in rules.lines() {
            full.add_line(None, rule).unwrap();
        }
        let full = full.build().unwrap();
        for name in [
            "tmp/a.sifr",
            "keep.sifr",
            "other/a.sifr",
            "foo/x/bar/baz",
            "a.txt",
            "#name",
            "one.rs",
            "unknown.txt",
            "other/foo/x/bar/a.txt",
            "éx.log",
            "space ",
            "nested/space ",
        ] {
            let path = Path::new(name);
            assert_eq!(
                ignore.matches_resolved_path(path).unwrap(),
                full.matched_path_or_any_parents(path, false).is_ignore(),
                "{name}"
            );
        }
    }
}
