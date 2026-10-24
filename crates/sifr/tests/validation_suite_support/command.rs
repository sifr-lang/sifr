//! Select the outer Cargo build's CLI and isolate child package context.
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

pub(super) fn expand_argv(
    argv: &[String],
    repo_root: &Path,
    tmp_dir: &Path,
    compiler: &Path,
) -> Result<Vec<String>, String> {
    if argv.is_empty() {
        return Err("validation suite command has empty argv".to_owned());
    }
    let mut expanded: Vec<_> = argv
        .iter()
        .map(|value| {
            value
                .replace("<TMP>", &tmp_dir.display().to_string())
                .replace("<REPO>", &repo_root.display().to_string())
        })
        .collect();
    if argv[0] == "<SIFR>" {
        if !compiler.is_file() {
            return Err(format!(
                "Cargo-selected validation compiler is missing: {}",
                compiler.display()
            ));
        }
        expanded[0] = compiler.display().to_string();
    }
    Ok(expanded)
}

/// Package metadata belongs to the parent test, not the CLI or Cargo children.
/// Keep explicit toolchain, target, flags, profile and resource settings intact.
pub(super) fn remove_parent_package_context(
    command: &mut Command,
    keys: impl IntoIterator<Item = OsString>,
) {
    for key in keys {
        if key.to_str().is_some_and(|name| {
            name.starts_with("CARGO_PKG_")
                || matches!(
                    name,
                    "CARGO_MANIFEST_DIR" | "CARGO_MANIFEST_PATH" | "OUT_DIR"
                )
        }) {
            command.env_remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn selected_cli_preserves_argument_boundaries_and_working_path_tokens() {
        let root = tempfile::tempdir().unwrap();
        let compiler = root.path().join("selected compiler.exe");
        std::fs::write(&compiler, b"selected fixture").unwrap();
        let repo = root.path().join("repository with spaces");
        let tmp = root.path().join("temporary files");
        let argv = expand_argv(
            &strings(&[
                "<SIFR>",
                "check",
                "<REPO>/input.sifr",
                "<TMP>/output",
                "literal argument",
            ]),
            &repo,
            &tmp,
            &compiler,
        )
        .unwrap();
        assert_eq!(
            argv,
            vec![
                compiler.display().to_string(),
                "check".into(),
                format!("{}/input.sifr", repo.display()),
                format!("{}/output", tmp.display()),
                "literal argument".into(),
            ]
        );
    }

    #[test]
    fn explicit_cli_selection_fails_when_missing_and_never_rewrites_other_commands() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("absent compiler");
        assert!(
            expand_argv(
                &strings(&["<SIFR>", "check"]),
                root.path(),
                root.path(),
                &missing
            )
            .is_err()
        );
        assert!(expand_argv(&[], root.path(), root.path(), &missing).is_err());
        for argv in [
            strings(&["cargo", "test", "--locked", "-p", "sifr_lowering"]),
            strings(&["cargo", "run", "-p", "sifr", "--", "check"]),
            strings(&["python3", "assertions.py"]),
        ] {
            assert_eq!(
                expand_argv(&argv, root.path(), root.path(), &missing).unwrap(),
                argv
            );
        }
    }

    #[test]
    fn children_do_not_inherit_parent_package_metadata() {
        let context = [
            ("CARGO_MANIFEST_DIR", "/parent/package"),
            ("CARGO_MANIFEST_PATH", "/parent/package/Cargo.toml"),
            ("CARGO_PKG_NAME", "parent"),
            ("CARGO_PKG_VERSION_PRE", ""),
            ("CARGO_PKG_DESCRIPTION", "parent description"),
            ("OUT_DIR", "/parent/build"),
        ];
        let mut command = Command::new("child");
        command.envs(context);
        remove_parent_package_context(&mut command, context.map(|(key, _)| OsString::from(key)));
        assert_eq!(command.get_envs().count(), context.len());
        assert!(command.get_envs().all(|(_, value)| value.is_none()));
    }

    #[test]
    fn children_retain_intentional_build_and_execution_settings() {
        let settings = [
            ("CARGO_TARGET_DIR", "/owned/target"),
            ("CARGO_NET_OFFLINE", "true"),
            ("CARGO_BUILD_JOBS", "2"),
            ("CARGO_PROFILE_DEV_DEBUG", "1"),
            ("CARGO_ENCODED_RUSTFLAGS", "-C\x1fdebuginfo=1"),
            ("RUSTFLAGS", "--cfg retained_configuration"),
            ("RUSTC", "/selected/rustc"),
            ("RUSTC_WRAPPER", "/selected/wrapper"),
            ("RUSTUP_TOOLCHAIN", "selected"),
            ("SIFR_SYSROOT", "/selected/sysroot"),
            ("PATH", "/selected/path"),
        ];
        let mut command = Command::new("child");
        command.envs(settings);
        remove_parent_package_context(&mut command, settings.map(|(key, _)| OsString::from(key)));
        let actual: std::collections::BTreeMap<_, _> = command.get_envs().collect();
        for (key, value) in settings {
            assert_eq!(
                actual.get(std::ffi::OsStr::new(key)),
                Some(&Some(std::ffi::OsStr::new(value)))
            );
        }
    }
}
