//! Explicit tool selection pinned before any generated project changes working directory.
use sifr_identity::{IdentityEncoder, NativeBuildId};
use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
    process::Command,
};

const NATIVE_ENV: &[&str] = &[
    "PATH",
    "LIBRARY_PATH",
    "CPATH",
    "PKG_CONFIG",
    "RUSTFLAGS",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_BUILD_TARGET",
    "CARGO_BUILD_RUSTFLAGS",
    "RUSTDOCFLAGS",
    "CC",
    "CXX",
    "AR",
    "CFLAGS",
    "CXXFLAGS",
    "LDFLAGS",
    "PKG_CONFIG_PATH",
    "PKG_CONFIG_SYSROOT_DIR",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "CARGO_HOME",
    "CARGO_TARGET_DIR",
    "PYO3_PYTHON",
    "PYO3_CONFIG_FILE",
    "MACOSX_DEPLOYMENT_TARGET",
    "SDKROOT",
];

#[derive(Clone)]
pub struct NativeToolchain {
    invocation_root: PathBuf,
    cargo: PathBuf,
    rustc: PathBuf,
    selection: Option<String>,
    cargo_version: String,
    rustc_version: String,
    host: String,
    target: String,
    environment: BTreeMap<String, Option<OsString>>,
    configuration: Vec<(PathBuf, Vec<u8>)>,
    configuration_candidates: Vec<PathBuf>,
    execution_environment: Vec<(OsString, OsString)>,
    identity: String,
}
impl fmt::Debug for NativeToolchain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeToolchain")
            .field("identity", &self.identity)
            .field("host", &self.host)
            .field("target", &self.target)
            .field("environment_keys", &self.environment.keys())
            .field("configuration_count", &self.configuration.len())
            .finish_non_exhaustive()
    }
}

fn path_executable(candidate: PathBuf) -> Option<PathBuf> {
    if candidate.is_file() {
        return Some(candidate);
    }
    // Windows tools on PATH are selected by a bare name, while their files
    // have an .exe suffix. Keep the original path for rustup proxy dispatch.
    #[cfg(windows)]
    if candidate.extension().is_none() {
        let executable = candidate.with_extension("exe");
        if executable.is_file() {
            return Some(executable);
        }
    }
    None
}

fn executable(name: &str, cwd: &Path) -> Result<PathBuf, String> {
    let value = Path::new(name);
    let candidate = if value.components().count() > 1 {
        if value.is_absolute() {
            value.to_path_buf()
        } else {
            cwd.join(value)
        }
    } else {
        env::split_paths(&env::var_os("PATH").unwrap_or_default())
            .find_map(|directory| path_executable(directory.join(value)))
            .ok_or_else(|| "selected native executable is unavailable".to_string())?
    };
    if !candidate.is_file() {
        return Err("selected native executable is unavailable".into());
    }
    // Do not canonicalize a rustup proxy symlink: argv[0] is part of its dispatch.
    Ok(if candidate.is_absolute() {
        candidate
    } else {
        cwd.join(candidate)
    })
}
fn output(command: &mut Command) -> Result<String, String> {
    let result = command.output().map_err(|error| {
        format!(
            "selected native tool could not be executed ({:?})",
            error.kind()
        )
    })?;
    if !result.status.success() {
        return Err("selected native tool returned failure".into());
    }
    String::from_utf8(result.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|_| "selected native tool returned invalid UTF-8".into())
}
type CargoConfiguration = Vec<(PathBuf, Vec<u8>)>;
fn cargo_configuration(cwd: &Path) -> Result<(CargoConfiguration, Vec<PathBuf>), String> {
    let mut configuration = Vec::new();
    let mut candidates = Vec::new();
    for directory in cwd.ancestors() {
        for name in ["config", "config.toml"] {
            let path = directory.join(".cargo").join(name);
            candidates.push(path.clone());
            if path.is_file() {
                configuration.push((
                    path.clone(),
                    std::fs::read(path).map_err(|_| "cannot read selected Cargo configuration")?,
                ));
            }
        }
    }
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")));
    if let Some(home) = cargo_home {
        for name in ["config", "config.toml"] {
            let path = home.join(name);
            if !candidates.contains(&path) {
                candidates.push(path.clone());
            }
            if path.is_file() && !configuration.iter().any(|(seen, _)| seen == &path) {
                configuration.push((
                    path.clone(),
                    std::fs::read(path)
                        .map_err(|_| "cannot read selected user Cargo configuration")?,
                ));
            }
        }
    }
    Ok((configuration, candidates))
}
impl NativeToolchain {
    pub fn resolve_at(cwd: &Path) -> Result<Self, String> {
        let canonical_root = cwd
            .canonicalize()
            .map_err(|_| "native invocation root is unavailable")?;
        let cwd = canonical_root.as_path();
        let requested = env::var("SIFR_RUST_TOOLCHAIN")
            .ok()
            .or_else(|| env::var("RUSTUP_TOOLCHAIN").ok());
        let cargo_override = env::var("SIFR_CARGO").ok();
        let rustc_override = env::var("SIFR_RUSTC").ok();
        let paired_override = cargo_override.is_some() && rustc_override.is_some();
        let (cargo, mut rustc, selection) = match (cargo_override, rustc_override) {
            (Some(cargo), Some(rustc)) => (
                executable(&cargo, cwd)?,
                executable(&rustc, cwd)?,
                requested,
            ),
            (None, None) => {
                let cargo = executable("cargo", cwd)?;
                let rustc = executable("rustc", cwd)?;
                // A rustup installation must resolve in the intentional caller workspace,
                // before temporary roots could select a different toolchain.
                if let Ok(rustup) = executable("rustup", cwd) {
                    let selection = match requested {
                        Some(value) => value,
                        None => output(
                            Command::new(&rustup)
                                .args(["show", "active-toolchain"])
                                .current_dir(cwd),
                        )?
                        .split_whitespace()
                        .next()
                        .ok_or("no active Rust toolchain")?
                        .to_owned(),
                    };
                    let selected_cargo = output(
                        Command::new(&rustup)
                            .args(["which", "--toolchain", &selection, "cargo"])
                            .current_dir(cwd),
                    )?;
                    let selected_rustc = output(
                        Command::new(&rustup)
                            .args(["which", "--toolchain", &selection, "rustc"])
                            .current_dir(cwd),
                    )?;
                    (
                        PathBuf::from(selected_cargo),
                        PathBuf::from(selected_rustc),
                        Some(selection),
                    )
                } else if requested.is_some() {
                    return Err("an explicit Rust toolchain requires rustup or paired SIFR_CARGO/SIFR_RUSTC executables".into());
                } else {
                    (cargo, rustc, None)
                }
            }
            _ => return Err("SIFR_CARGO and SIFR_RUSTC must be supplied together".into()),
        };
        if !paired_override {
            let (configuration, _) = cargo_configuration(cwd)?;
            let mut configured_rustc = None;
            for (path, bytes) in configuration.iter().rev() {
                let text =
                    std::str::from_utf8(bytes).map_err(|_| "Cargo configuration is not UTF-8")?;
                let config: toml::Value =
                    toml::from_str(text).map_err(|_| "invalid Cargo configuration")?;
                if let Some(program) = config
                    .get("build")
                    .and_then(|build| build.get("rustc"))
                    .and_then(toml::Value::as_str)
                {
                    // Cargo resolves config-relative paths against the parent of .cargo.
                    let root = path.parent().and_then(Path::parent).unwrap_or(cwd);
                    configured_rustc = Some(executable(program, root)?);
                }
            }
            if let Some(program) = env::var("RUSTC")
                .ok()
                .or_else(|| env::var("CARGO_BUILD_RUSTC").ok())
            {
                configured_rustc = Some(executable(&program, cwd)?);
            }
            if let Some(program) = configured_rustc {
                rustc = program;
            }
        }
        Self::from_executables(cwd, cargo, rustc, selection)
    }
    pub fn from_executables(
        cwd: &Path,
        cargo: PathBuf,
        rustc: PathBuf,
        selection: Option<String>,
    ) -> Result<Self, String> {
        if !cargo.is_absolute() || !rustc.is_absolute() {
            return Err("native executable paths must be absolute".into());
        }
        let version = |program: &Path| {
            let mut command = Command::new(program);
            command.arg("-vV").current_dir(cwd);
            if let Some(selection) = &selection {
                command.env("RUSTUP_TOOLCHAIN", selection);
            }
            output(&mut command)
        };
        let cargo_version = version(&cargo)?;
        let rustc_version = version(&rustc)?;
        let host = rustc_version
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .ok_or("selected rustc did not report its host")?
            .to_owned();
        let mut environment: BTreeMap<_, _> = NATIVE_ENV
            .iter()
            .map(|key| ((*key).to_owned(), env::var_os(key)))
            .collect();
        // Cargo supports target/profile-specific settings; retain only build-affecting
        // names, never registry tokens/passwords or the whole process environment.
        for (key, value) in env::vars_os() {
            if let Some(name) = key.to_str() {
                if name.starts_with("CARGO_TARGET_") || name.starts_with("CARGO_PROFILE_") {
                    environment.insert(name.to_owned(), Some(value));
                }
            }
        }
        let (configuration, configuration_candidates) = cargo_configuration(cwd)?;
        let mut target = host.clone();
        for (_, bytes) in configuration.iter().rev() {
            let text =
                std::str::from_utf8(bytes).map_err(|_| "Cargo configuration is not UTF-8")?;
            let config: toml::Value =
                toml::from_str(text).map_err(|_| "invalid Cargo configuration")?;
            if let Some(value) = config.get("build").and_then(|build| build.get("target")) {
                value
                    .as_str()
                    .ok_or("multiple Cargo targets are not supported for a single native artifact")?
                    .clone_into(&mut target);
            }
        }
        if let Some(value) = env::var_os("CARGO_BUILD_TARGET") {
            target = value
                .into_string()
                .map_err(|_| "Cargo target is not UTF-8")?;
        }
        let mut hash = IdentityEncoder::new("resolved-native-toolchain-v1");
        for (name, value) in [
            ("cargo", cargo_version.as_str()),
            ("rustc", rustc_version.as_str()),
            ("host", host.as_str()),
        ] {
            hash.field(name, value.as_bytes());
        }
        for (name, path) in [("cargo-executable", &cargo), ("rustc-executable", &rustc)] {
            let digest = crate::sha256_file(path)
                .map_err(|_| "cannot identify selected native executable")?;
            hash.field(name, digest.as_bytes());
        }
        hash.field(
            "selection",
            selection.as_deref().unwrap_or("<direct>").as_bytes(),
        );
        hash.field("cargo-path", cargo.as_os_str().as_encoded_bytes());
        hash.field("rustc-path", rustc.as_os_str().as_encoded_bytes());
        for (name, value) in &environment {
            hash.field("environment-name", name.as_bytes());
            hash.field("environment-present", &[u8::from(value.is_some())]);
            if let Some(value) = value {
                hash.field("environment-value", value.as_encoded_bytes());
            }
        }
        for (path, bytes) in &configuration {
            hash.field("config-path", path.as_os_str().as_encoded_bytes());
            hash.field("config-bytes", bytes);
        }
        let identity = hash.finish();
        Ok(Self {
            invocation_root: cwd.to_path_buf(),
            cargo,
            rustc,
            selection,
            cargo_version,
            rustc_version,
            host,
            target,
            environment,
            configuration,
            configuration_candidates,
            execution_environment: env::vars_os().collect(),
            identity,
        })
    }
    #[must_use]
    pub fn with_declared_environment(mut self, names: impl IntoIterator<Item = String>) -> Self {
        let names: std::collections::BTreeSet<_> = names.into_iter().collect();
        let mut hash = IdentityEncoder::new("declared-native-environment-v2");
        hash.field("toolchain", self.identity.as_bytes());
        for name in names {
            let value = self
                .execution_environment
                .iter()
                .find(|(key, _)| key == name.as_str())
                .map(|(_, value)| value.clone());
            hash.field("environment-name", name.as_bytes());
            hash.field("environment-present", &[u8::from(value.is_some())]);
            if let Some(value) = &value {
                hash.field("environment-value", value.as_encoded_bytes());
            }
            self.environment.insert(name, value);
        }
        self.identity = hash.finish();
        self
    }
    #[cfg(test)]
    pub(crate) fn with_test_execution_environment(
        mut self,
        name: &str,
        value: Option<&str>,
    ) -> Self {
        self.execution_environment.retain(|(key, _)| key != name);
        if let Some(value) = value {
            self.execution_environment.push((name.into(), value.into()));
        }
        self
    }
    pub fn validate_configuration(&self) -> Result<(), String> {
        for path in &self.configuration_candidates {
            if path.is_file()
                != self
                    .configuration
                    .iter()
                    .any(|(existing, _)| existing == path)
            {
                return Err(
                    "selected Cargo configuration inventory changed; resolve a new native context"
                        .into(),
                );
            }
        }
        for (path, expected) in &self.configuration {
            let actual = std::fs::read(path)
                .map_err(|_| "selected Cargo configuration became unavailable")?;
            if &actual != expected {
                return Err(
                    "selected Cargo configuration changed; resolve a new native context".into(),
                );
            }
        }
        Ok(())
    }
    /// Check captured effective configuration before selecting an application profile.
    pub fn validate_application_profile(
        &self,
        profile: &str,
        manifest: &Path,
    ) -> Result<(), String> {
        fn check(value: &toml::Value, profile: &str) -> Result<(), String> {
            // Rustflags have higher priority than profile settings. Reject flags
            // that disable the same language/runtime boundary in any target scope.
            fn flags(value: &toml::Value) -> Result<(), String> {
                if let toml::Value::Table(table) = value {
                    for (key, value) in table {
                        if key == "rustflags" {
                            crate::native_profile::validate_flags(&value.to_string())?;
                        } else {
                            flags(value)?;
                        }
                    }
                }
                Ok(())
            }
            if let Some(table) = value.get("profile").and_then(|v| v.get(profile)) {
                fn boundary(value: &toml::Value) -> Result<(), String> {
                    if value
                        .get("panic")
                        .and_then(toml::Value::as_str)
                        .is_some_and(|v| v != "unwind")
                        || value.get("overflow-checks").and_then(toml::Value::as_bool)
                            == Some(false)
                    {
                        return Err(
                            "application profile requires panic=unwind and overflow-checks=true"
                                .into(),
                        );
                    }
                    if let Some(table) = value.as_table() {
                        for child in table.values() {
                            boundary(child)?;
                        }
                    }
                    Ok(())
                }
                boundary(table)?;
            }
            flags(value)
        }
        for (_, bytes) in &self.configuration {
            let value: toml::Table = String::from_utf8_lossy(bytes)
                .parse()
                .map_err(|_| "invalid Cargo profile configuration")?;
            check(&toml::Value::Table(value), profile)?;
        }
        // Generated applications declare their own workspace. Their profile
        // authority is this manifest, never a manifest above the caller's CWD.
        let document: toml::Table = std::fs::read_to_string(manifest)
            .map_err(|_| "cannot read generated application manifest")?
            .parse()
            .map_err(|_| "invalid generated application manifest")?;
        if !document.get("workspace").is_some_and(toml::Value::is_table)
            || document
                .get("package")
                .and_then(|value| value.get("workspace"))
                .is_some()
        {
            return Err("generated application must own its Cargo workspace".into());
        }
        check(&toml::Value::Table(document), profile)?;
        let prefix = format!("CARGO_PROFILE_{}_", profile.to_uppercase());
        for (name, value) in &self.environment {
            if let Some(value) = value {
                let value = value.to_string_lossy();
                if (name == &format!("{prefix}PANIC") && value != "unwind")
                    || (name == &format!("{prefix}OVERFLOW_CHECKS") && value != "true")
                {
                    return Err("application profile environment invalidates required unwind/overflow boundary".into());
                }
                if name.ends_with("RUSTFLAGS") {
                    crate::native_profile::validate_flags(&value)?;
                }
            }
        }
        Ok(())
    }
    pub fn cargo_command(&self) -> Result<Command, String> {
        self.validate_configuration()?;
        let mut command = Command::new(&self.cargo);
        command.current_dir(&self.invocation_root);
        command
            .env_clear()
            .envs(self.execution_environment.iter().cloned());
        // Cargo loads these validated configurations from the pinned root.
        // Passing them again with --config would concatenate array settings twice.
        command.env("RUSTC", &self.rustc);
        if let Some(selection) = &self.selection {
            command.env("RUSTUP_TOOLCHAIN", selection);
        }
        for (key, value) in &self.environment {
            if let Some(value) = value {
                command.env(key, value);
            } else {
                command.env_remove(key);
            }
        }
        Ok(command)
    }
    pub fn cargo_path(&self) -> &Path {
        &self.cargo
    }
    pub fn rustc_path(&self) -> &Path {
        &self.rustc
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn cargo_version(&self) -> &str {
        &self.cargo_version
    }
    pub fn rustc_version(&self) -> &str {
        &self.rustc_version
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn host(&self) -> &str {
        &self.host
    }
}

/// An invocation binds the resolved tools to the actual graph and operation inputs.
/// Values here are digests, never secret-bearing environment or credential strings.
#[derive(Clone, Debug)]
pub struct NativeBuildContext {
    pub toolchain: NativeToolchain,
    pub target: String,
    pub profile: String,
    pub flags_id: String,
    pub features_id: String,
    pub resolution_id: String,
    pub python_loader_id: Option<String>,
    pub trust_policy_id: String,
    pub destination: PathBuf,
}
impl NativeBuildContext {
    pub fn identity(&self) -> NativeBuildId {
        NativeBuildId::from_records([
            ("toolchain", self.toolchain.identity().as_bytes()),
            ("target", self.target.as_bytes()),
            ("profile", self.profile.as_bytes()),
            ("flags", self.flags_id.as_bytes()),
            ("features", self.features_id.as_bytes()),
            ("resolution", self.resolution_id.as_bytes()),
            (
                "python",
                self.python_loader_id.as_deref().unwrap_or("").as_bytes(),
            ),
            ("trust", self.trust_policy_id.as_bytes()),
            (
                "destination",
                self.destination.as_os_str().as_encoded_bytes(),
            ),
        ])
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use super::{NativeToolchain, executable, path_executable};
    use std::{env, fs};

    #[test]
    fn windows_path_lookup_resolves_exe_and_keeps_explicit_paths_exact() {
        let root = env::temp_dir().join(format!(
            "sifr-native-executable-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir(&root).unwrap();
        let bare = root.join("fixture-cargo");
        let executable_path = bare.with_extension("exe");
        fs::write(&executable_path, b"fixture").unwrap();

        assert_eq!(path_executable(bare.clone()), Some(executable_path));
        assert!(
            executable(bare.to_str().unwrap(), &root).is_err(),
            "an explicit path must not acquire an extension"
        );

        let cwd = env::current_dir().unwrap();
        let selected = NativeToolchain::resolve_at(&cwd).unwrap();
        assert!(selected.cargo_path().is_file());
        assert!(selected.rustc_path().is_file());
        fs::remove_dir_all(root).unwrap();
    }
}
