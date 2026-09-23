use super::rust_interop::PackageRustInteropContext;
use super::rust_interop_digest::{digest_file, digest_path_checked, normalized_path_string};
use super::sysroot_interop::SysrootRustInteropTrust;
use sifr_codegen::{RustBridgeSourceDigest, RustInteropCargoInputs};
use sifr_identity::IdentityEncoder;
use sifr_package::{TrustPolicy, digest_package_graph, digest_package_source_snapshot};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn bridge_source_digests(
    context: &PackageRustInteropContext,
    package: &sifr_package::SifrPackageMetadata,
) -> Result<Vec<RustBridgeSourceDigest>, String> {
    let mut digests = package
        .manifest
        .rust
        .bridges
        .iter()
        .map(|bridge_root| {
            Ok(RustBridgeSourceDigest {
                package_id: context.package_id.0.clone(),
                bridge_root: normalized_path_string(bridge_root),
                digest: digest_path_checked(&package.package_root.join(bridge_root)).map_err(
                    |error| {
                        format!(
                            "unreadable Rust bridge source {}: {error}",
                            bridge_root.display()
                        )
                    },
                )?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    digests.sort_by(|left, right| {
        (&left.package_id, &left.bridge_root).cmp(&(&right.package_id, &right.bridge_root))
    });
    Ok(digests)
}

pub(super) fn cargo_inputs(
    resolution: &super::cargo_resolution::CargoResolutionPolicy,
    context: &PackageRustInteropContext,
    package: &sifr_package::SifrPackageMetadata,
) -> Result<RustInteropCargoInputs, String> {
    if let Some(trust) = context
        .sysroot_trust
        .as_ref()
        .filter(|trust| trust.package_id == package.package_id)
    {
        return sysroot_cargo_inputs(resolution, trust, &package.manifest.trust);
    }
    let graph_digest = digest_package_graph(&context.graph);
    let source_map_digest = digest_package_source_snapshot(&context.source_map)
        .map_err(|error| format!("unreadable package source snapshot: {error}"))?;
    let trust_policy_digest = trust_policy_digest(&package.manifest.trust);
    let mut declared_build_env = package.manifest.trust.build_env.clone();
    declared_build_env.sort();
    Ok(RustInteropCargoInputs {
        package_id: context.package_id.0.clone(),
        cargo_metadata_digest: None,
        sqlx_offline_metadata_digest: None,
        package_graph_digest: Some(graph_digest.hex),
        package_source_map_digest: Some(source_map_digest.hex),
        cargo_lock_digest: cargo_lock_digest(&package.package_root),
        target_triple: std::env::var("SIFR_TARGET").ok().or_else(|| {
            resolution
                .native_toolchain
                .as_ref()
                .ok()
                .map(|tools| tools.target().to_owned())
        }),
        target_features: target_features(),
        cargo_profile: resolution.application_profile.cargo_name().to_string(),
        panic_strategy: std::env::var("SIFR_RUST_PANIC_STRATEGY").ok(),
        profile_codegen_settings: profile_codegen_settings(
            &package.package_root,
            resolution.application_profile.cargo_name(),
        ),
        cargo_version: resolution
            .native_toolchain
            .as_ref()
            .ok()
            .map(|tools| tools.cargo_version().to_owned()),
        rustc_version: resolution
            .native_toolchain
            .as_ref()
            .ok()
            .map(|tools| tools.rustc_version().to_owned()),
        trust_policy_digest,
        declared_build_env,
    })
}

pub(super) fn combined_cargo_inputs(
    mut primary: RustInteropCargoInputs,
    secondary: RustInteropCargoInputs,
) -> RustInteropCargoInputs {
    let digest = combined_cargo_inputs_digest(&primary, &secondary);
    primary.package_id = format!("{}+{}", primary.package_id, secondary.package_id);
    primary.cargo_metadata_digest = Some(digest.clone());
    primary.sqlx_offline_metadata_digest = combine_optional_digest(
        primary.sqlx_offline_metadata_digest.take(),
        secondary.sqlx_offline_metadata_digest,
    );
    primary.package_graph_digest = Some(digest);
    primary.package_source_map_digest = combine_optional_digest(
        primary.package_source_map_digest.take(),
        secondary.package_source_map_digest,
    );
    primary.cargo_lock_digest = combine_optional_digest(
        primary.cargo_lock_digest.take(),
        secondary.cargo_lock_digest,
    );
    primary.trust_policy_digest = combined_digest(&[
        primary.trust_policy_digest.as_str(),
        secondary.trust_policy_digest.as_str(),
    ]);
    primary
        .declared_build_env
        .extend(secondary.declared_build_env);
    primary.declared_build_env.sort();
    primary.declared_build_env.dedup();
    primary
}

fn sysroot_cargo_inputs(
    resolution: &super::cargo_resolution::CargoResolutionPolicy,
    trust: &SysrootRustInteropTrust,
    policy: &TrustPolicy,
) -> Result<RustInteropCargoInputs, String> {
    let trust_policy_digest = trust_policy_digest(policy);
    let mut declared_build_env = policy.build_env.clone();
    declared_build_env.sort();
    Ok(RustInteropCargoInputs {
        package_id: format!("sifr-sysroot-stdlib@{}", trust.toolchain_id),
        cargo_metadata_digest: Some(sysroot_metadata_digest(trust)?),
        sqlx_offline_metadata_digest: None,
        package_graph_digest: Some(trust.sysroot_content_sha256.clone()),
        package_source_map_digest: Some(
            digest_path_checked(&trust.stdlib_private_sources)
                .map_err(|error| format!("unreadable sysroot private sources: {error}"))?,
        ),
        cargo_lock_digest: digest_file(&trust.cargo_lock),
        target_triple: std::env::var("SIFR_TARGET").ok().or_else(|| {
            resolution
                .native_toolchain
                .as_ref()
                .ok()
                .map(|tools| tools.target().to_owned())
        }),
        target_features: target_features(),
        cargo_profile: resolution.application_profile.cargo_name().to_string(),
        panic_strategy: std::env::var("SIFR_RUST_PANIC_STRATEGY").ok(),
        profile_codegen_settings: profile_codegen_settings(
            &trust.sysroot_root,
            resolution.application_profile.cargo_name(),
        ),
        cargo_version: resolution
            .native_toolchain
            .as_ref()
            .ok()
            .map(|tools| tools.cargo_version().to_owned()),
        rustc_version: resolution
            .native_toolchain
            .as_ref()
            .ok()
            .map(|tools| tools.rustc_version().to_owned()),
        trust_policy_digest,
        declared_build_env,
    })
}

pub(super) fn generated_bridge_module_path(module_name: Option<&str>) -> Vec<String> {
    let mut path = vec!["__sifr_bridge".to_string()];
    match module_name {
        Some(module_name) => path.extend(module_name.split('.').map(str::to_string)),
        None => path.push("__sifr_binary_entry".to_string()),
    }
    path
}

pub(super) fn first_generated_bridge_import(source_root: &Path) -> Option<PathBuf> {
    let mut pending = vec![source_root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let Ok(source) = fs::read_to_string(&path) else {
                continue;
            };
            if imports_generated_bridge_namespace(&source) {
                return Some(path);
            }
        }
    }
    None
}

#[derive(Default)]
pub(super) struct GeneratedBridgeImportCache {
    imports_by_source_root: HashMap<PathBuf, Option<PathBuf>>,
}

impl GeneratedBridgeImportCache {
    pub(super) fn inspect(&mut self, source_root: &Path) -> Option<PathBuf> {
        self.inspect_with(source_root, first_generated_bridge_import)
    }

    fn inspect_with(
        &mut self,
        source_root: &Path,
        inspect: impl FnOnce(&Path) -> Option<PathBuf>,
    ) -> Option<PathBuf> {
        self.imports_by_source_root
            .entry(source_root.to_path_buf())
            .or_insert_with(|| inspect(source_root))
            .clone()
    }
}

fn imports_generated_bridge_namespace(source: &str) -> bool {
    let tokens = rust_namespace_tokens(source);
    tokens.windows(3).any(|window| {
        matches!(
            window,
            [RustToken::Ident(first), RustToken::ColonColon, RustToken::Ident(second)]
                if first == "crate" && second == "__sifr_bridge"
        )
    }) || tokens.windows(2).any(|window| {
        matches!(
            window,
            [RustToken::Ident(first), RustToken::ColonColon] if first == "__sifr_bridge"
        )
    })
}

#[derive(Debug, PartialEq, Eq)]
enum RustToken {
    Ident(String),
    ColonColon,
}

fn rust_namespace_tokens(source: &str) -> Vec<RustToken> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            'r' if matches!(chars.peek(), Some('"' | '#')) => {
                if !skip_raw_string(&mut chars) {
                    tokens.push(RustToken::Ident("r".to_string()));
                }
            }
            '/' if chars.peek() == Some(&'/') => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut previous = '\0';
                for next in chars.by_ref() {
                    if previous == '*' && next == '/' {
                        break;
                    }
                    previous = next;
                }
            }
            '"' => skip_quoted(&mut chars),
            ':' if chars.peek() == Some(&':') => {
                chars.next();
                tokens.push(RustToken::ColonColon);
            }
            '_' | 'a'..='z' | 'A'..='Z' => {
                let mut ident = String::from(ch);
                while let Some(next) = chars.peek() {
                    if *next == '_' || next.is_ascii_alphanumeric() {
                        ident.push(*next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(RustToken::Ident(ident));
            }
            _ => {}
        }
    }
    tokens
}

fn skip_quoted(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    let mut escaped = false;
    for next in chars.by_ref() {
        if escaped {
            escaped = false;
            continue;
        }
        if next == '\\' {
            escaped = true;
            continue;
        }
        if next == '"' {
            break;
        }
    }
}

fn skip_raw_string(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    let mut hashes = 0_usize;
    while chars.peek() == Some(&'#') {
        hashes += 1;
        chars.next();
    }
    if chars.peek() != Some(&'"') {
        return false;
    }
    chars.next();
    let mut closing_quote = false;
    let mut trailing_hashes = 0_usize;
    for next in chars.by_ref() {
        if next == '"' {
            if hashes == 0 {
                break;
            }
            closing_quote = true;
            trailing_hashes = 0;
            continue;
        }
        if closing_quote && next == '#' {
            trailing_hashes += 1;
            if trailing_hashes == hashes {
                break;
            }
            continue;
        }
        closing_quote = false;
        trailing_hashes = 0;
    }
    true
}

fn cargo_lock_digest(package_root: &Path) -> Option<String> {
    nearest_ancestor_file(package_root, "Cargo.lock").and_then(|path| digest_file(&path))
}

fn combined_cargo_inputs_digest(
    primary: &RustInteropCargoInputs,
    secondary: &RustInteropCargoInputs,
) -> String {
    let mut identity = IdentityEncoder::new("combined-rust-cargo-inputs-v2");
    encode_cargo_inputs(&mut identity, "primary", primary);
    encode_cargo_inputs(&mut identity, "secondary", secondary);
    identity.finish()
}

fn encode_cargo_inputs(identity: &mut IdentityEncoder, side: &str, input: &RustInteropCargoInputs) {
    identity.field("side", side.as_bytes());
    identity.field("package-id", input.package_id.as_bytes());
    for (name, value) in [
        ("metadata", &input.cargo_metadata_digest),
        ("sqlx", &input.sqlx_offline_metadata_digest),
        ("graph", &input.package_graph_digest),
        ("source-map", &input.package_source_map_digest),
        ("lock", &input.cargo_lock_digest),
        ("target", &input.target_triple),
        ("panic", &input.panic_strategy),
        ("cargo-version", &input.cargo_version),
        ("rustc-version", &input.rustc_version),
    ] {
        identity.field(name, &[u8::from(value.is_some())]);
        if let Some(value) = value {
            identity.field(name, value.as_bytes());
        }
    }
    identity.field("profile", input.cargo_profile.as_bytes());
    identity.field("trust", input.trust_policy_digest.as_bytes());
    encode_values(identity, "feature", &input.target_features);
    identity.field(
        "setting-count",
        &(input.profile_codegen_settings.len() as u64).to_be_bytes(),
    );
    for (name, value) in &input.profile_codegen_settings {
        identity.field("setting-name", name.as_bytes());
        identity.field("setting-value", value.as_bytes());
    }
    encode_values(identity, "build-env", &input.declared_build_env);
}

fn combine_optional_digest(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(left), Some(right)) => Some(combined_digest(&[left.as_str(), right.as_str()])),
    }
}

fn combined_digest(parts: &[&str]) -> String {
    let mut identity = IdentityEncoder::new("combined-rust-input-v2");
    identity.field("count", &(parts.len() as u64).to_be_bytes());
    for part in parts {
        identity.field("part", part.as_bytes());
    }
    identity.finish()
}

fn sysroot_metadata_digest(trust: &SysrootRustInteropTrust) -> Result<String, String> {
    let mut identity = IdentityEncoder::new("sysroot-rust-metadata-v2");
    identity.field(
        "root",
        normalized_path_string(&trust.sysroot_root).as_bytes(),
    );
    identity.field("package-id", trust.package_id.0.as_bytes());
    identity.field("toolchain", trust.toolchain_id.as_bytes());
    identity.field("content", trust.sysroot_content_sha256.as_bytes());
    identity.field(
        "vendor-path",
        normalized_path_string(&trust.vendor_dir).as_bytes(),
    );
    identity.field(
        "lock-path",
        normalized_path_string(&trust.cargo_lock).as_bytes(),
    );
    for (name, path) in [
        ("private-sources", &trust.stdlib_private_sources),
        ("stdlib-crate", &trust.stdlib_crate),
        ("runtime-crate", &trust.runtime_crate),
    ] {
        identity.field("tree-path", normalized_path_string(path).as_bytes());
        identity.field(
            name,
            digest_path_checked(path)
                .map_err(|error| format!("unreadable sysroot {name}: {error}"))?
                .as_bytes(),
        );
    }
    let lock = digest_file(&trust.cargo_lock).ok_or_else(|| {
        format!(
            "unreadable sysroot Cargo lock: {}",
            trust.cargo_lock.display()
        )
    })?;
    identity.field("cargo-lock", lock.as_bytes());
    Ok(identity.finish())
}

fn nearest_ancestor_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join(file_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn profile_codegen_settings(package_root: &Path, profile: &str) -> Vec<(String, String)> {
    let mut settings = Vec::new();
    for cargo_toml in ancestor_cargo_tomls(package_root) {
        let Ok(source) = fs::read_to_string(&cargo_toml) else {
            continue;
        };
        let Ok(table) = source.parse::<toml::Table>() else {
            continue;
        };
        let Some(profile_table) = table
            .get("profile")
            .and_then(toml::Value::as_table)
            .and_then(|profiles| profiles.get(profile))
            .and_then(toml::Value::as_table)
        else {
            continue;
        };
        for key in [
            "opt-level",
            "lto",
            "codegen-units",
            "panic",
            "debug",
            "strip",
        ] {
            if let Some(value) = profile_table.get(key) {
                settings.push((
                    format!("{}:{key}", normalized_path_string(&cargo_toml)),
                    value.to_string(),
                ));
            }
        }
    }
    settings.sort();
    settings
}

fn ancestor_cargo_tomls(package_root: &Path) -> Vec<PathBuf> {
    package_root
        .ancestors()
        .map(|ancestor| ancestor.join("Cargo.toml"))
        .filter(|candidate| candidate.is_file())
        .collect()
}

fn target_features() -> Vec<String> {
    let mut features = Vec::new();
    if let Ok(flags) = std::env::var("RUSTFLAGS") {
        features.push(format!("RUSTFLAGS={flags}"));
    }
    if let Ok(flags) = std::env::var("CARGO_ENCODED_RUSTFLAGS") {
        features.push(format!("CARGO_ENCODED_RUSTFLAGS={flags}"));
    }
    features.sort();
    features
}

fn encode_values(identity: &mut IdentityEncoder, name: &str, values: &[String]) {
    identity.field(name, &(values.len() as u64).to_be_bytes());
    for value in values {
        identity.field(name, value.as_bytes());
    }
}

fn trust_policy_digest(trust: &TrustPolicy) -> String {
    let mut identity = IdentityEncoder::new("rust-trust-policy-v2");
    for (name, values) in [
        ("security-capabilities", &trust.security_capabilities),
        ("native", &trust.native),
        ("build-scripts", &trust.build_scripts),
        ("proc-macros", &trust.proc_macros),
        ("python", &trust.python),
        ("python-native", &trust.python_native),
        ("rust-build-scripts", &trust.rust_build_scripts),
        ("rust-proc-macros", &trust.rust_proc_macros),
        ("native-links", &trust.native_links),
        ("unsafe-rust-bridges", &trust.unsafe_rust_bridges),
        ("build-env", &trust.build_env),
        ("rust-no-panic", &trust.rust_no_panic),
        ("rust-panic-abort", &trust.rust_panic_abort),
    ] {
        let mut values = values.clone();
        values.sort();
        encode_values(&mut identity, name, &values);
    }
    identity.finish()
}

#[cfg(test)]
mod tests {
    use super::{
        GeneratedBridgeImportCache, combined_cargo_inputs, generated_bridge_module_path,
        imports_generated_bridge_namespace,
    };
    use sifr_codegen::RustInteropCargoInputs;

    #[test]
    fn generated_bridge_module_path_keeps_binary_entry_distinct_from_main_module() {
        assert_ne!(
            generated_bridge_module_path(None),
            generated_bridge_module_path(Some("main"))
        );
        assert_eq!(
            generated_bridge_module_path(None),
            [
                "__sifr_bridge".to_string(),
                "__sifr_binary_entry".to_string()
            ]
        );
    }

    #[test]
    fn generated_bridge_import_scanner_ignores_comments_strings_and_related_names() {
        assert!(!imports_generated_bridge_namespace(
            "// crate::__sifr_bridge::Token\nconst NOTE: &str = \"__sifr_bridge::Token\";\nconst RAW: &str = r#\"crate::__sifr_bridge::Token\"#;\nfn __sifr_bridge_compat() {}\n"
        ));
        assert!(imports_generated_bridge_namespace(
            "use crate :: __sifr_bridge :: app :: TokenBridge;\n"
        ));
        assert!(imports_generated_bridge_namespace(
            "use __sifr_bridge :: app :: TokenBridge;\n"
        ));
    }

    #[test]
    fn generated_bridge_import_scanner_caches_each_backend_source_root() {
        let mut cache = GeneratedBridgeImportCache::default();
        let clean_root = std::path::Path::new("/backend/clean/src");
        let importing_root = std::path::Path::new("/backend/importing/src");
        let imported_path = importing_root.join("lib.rs");
        let mut clean_inspections = 0;
        let mut importing_inspections = 0;

        assert_eq!(
            cache.inspect_with(clean_root, |_| {
                clean_inspections += 1;
                None
            }),
            None
        );
        assert_eq!(
            cache.inspect_with(clean_root, |_| {
                clean_inspections += 1;
                Some(clean_root.join("changed.rs"))
            }),
            None
        );
        assert_eq!(
            cache.inspect_with(importing_root, |_| {
                importing_inspections += 1;
                Some(imported_path.clone())
            }),
            Some(imported_path.clone())
        );
        assert_eq!(
            cache.inspect_with(importing_root, |_| {
                importing_inspections += 1;
                None
            }),
            Some(imported_path)
        );
        assert_eq!(clean_inspections, 1);
        assert_eq!(importing_inspections, 1);
    }

    #[test]
    fn combined_cargo_inputs_fold_secondary_cache_material() {
        let primary = cargo_inputs_fixture("user-package", Some("user-source"));
        let secondary = cargo_inputs_fixture("sifr-sysroot-stdlib", Some("sysroot-source"));

        let combined = combined_cargo_inputs(primary.clone(), secondary.clone());

        assert_eq!(combined.package_id, "user-package+sifr-sysroot-stdlib");
        assert_ne!(
            combined.cargo_metadata_digest,
            primary.cargo_metadata_digest
        );
        assert_ne!(
            combined.sqlx_offline_metadata_digest,
            primary.sqlx_offline_metadata_digest
        );
        assert_ne!(
            combined.sqlx_offline_metadata_digest,
            secondary.sqlx_offline_metadata_digest
        );
        assert_ne!(
            combined.package_source_map_digest,
            primary.package_source_map_digest
        );
        assert_ne!(
            combined.package_source_map_digest,
            secondary.package_source_map_digest
        );
    }

    #[test]
    fn sysroot_metadata_identity_binds_authority_and_tree_payloads() {
        use super::super::sysroot_interop::SysrootRustInteropTrust;
        use std::fs;
        let root = tempfile::tempdir().unwrap();
        let private = root.path().join("private");
        let stdlib = root.path().join("stdlib");
        let runtime = root.path().join("runtime");
        let lock = root.path().join("Cargo.lock");
        for path in [&private, &stdlib, &runtime] {
            fs::create_dir(path).unwrap();
            fs::write(path.join("lib.rs"), b"original").unwrap();
        }
        fs::write(&lock, b"lock").unwrap();
        let trust = SysrootRustInteropTrust {
            package_id: sifr_package::SifrPackageId("sysroot".into()),
            sysroot_root: root.path().to_path_buf(),
            stdlib_private_sources: private.clone(),
            stdlib_crate: stdlib.clone(),
            runtime_crate: runtime.clone(),
            cargo_lock: lock.clone(),
            vendor_dir: root.path().join("vendor"),
            toolchain_id: "toolchain".into(),
            sysroot_content_sha256: "content".into(),
        };
        let baseline = super::sysroot_metadata_digest(&trust).unwrap();
        let mut changed = trust.clone();
        changed.package_id.0.push('x');
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        changed = trust.clone();
        changed.vendor_dir.push("other");
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        let other_lock = root.path().join("other.lock");
        fs::copy(&lock, &other_lock).unwrap();
        changed = trust.clone();
        changed.cargo_lock = other_lock;
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        let other_stdlib = root.path().join("other-stdlib");
        fs::create_dir(&other_stdlib).unwrap();
        fs::write(other_stdlib.join("lib.rs"), b"original").unwrap();
        changed = trust.clone();
        changed.stdlib_crate = other_stdlib;
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        let other_runtime = root.path().join("other-runtime");
        fs::create_dir(&other_runtime).unwrap();
        fs::write(other_runtime.join("lib.rs"), b"original").unwrap();
        changed = trust.clone();
        changed.runtime_crate = other_runtime;
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        let other_private = root.path().join("other-private");
        fs::create_dir(&other_private).unwrap();
        fs::write(other_private.join("lib.rs"), b"original").unwrap();
        changed = trust.clone();
        changed.stdlib_private_sources = other_private;
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        let mut changed = trust.clone();
        changed.toolchain_id.push('x');
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        changed = trust.clone();
        changed.sysroot_content_sha256.push('x');
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        changed = trust.clone();
        changed.sysroot_root = root.path().join("other-root");
        assert_ne!(baseline, super::sysroot_metadata_digest(&changed).unwrap());
        for path in [&private, &stdlib, &runtime] {
            fs::write(path.join("lib.rs"), b"changed").unwrap();
            assert_ne!(baseline, super::sysroot_metadata_digest(&trust).unwrap());
            fs::write(path.join("lib.rs"), b"original").unwrap();
        }
        fs::write(&lock, b"").unwrap();
        assert_ne!(baseline, super::sysroot_metadata_digest(&trust).unwrap());
        fs::remove_file(&lock).unwrap();
        assert!(super::sysroot_metadata_digest(&trust).is_err());
        fs::remove_dir_all(&private).unwrap();
        assert!(super::sysroot_metadata_digest(&trust).is_err());
    }

    #[test]
    fn combined_cargo_identity_binds_every_field_and_option_state() {
        let primary = cargo_inputs_fixture("primary", Some("source"));
        let secondary = cargo_inputs_fixture("secondary", Some("source"));
        let original = super::combined_cargo_inputs_digest(&primary, &secondary);
        let mut cases = Vec::new();
        macro_rules! mutate {
            ($field:ident, $value:expr) => {{
                let mut changed = secondary.clone();
                changed.$field = $value;
                cases.push((stringify!($field), changed));
            }};
        }
        mutate!(package_id, "other".into());
        mutate!(cargo_metadata_digest, Some("other".into()));
        mutate!(sqlx_offline_metadata_digest, Some("other".into()));
        mutate!(package_graph_digest, Some("other".into()));
        mutate!(package_source_map_digest, Some("other".into()));
        mutate!(cargo_lock_digest, Some("other".into()));
        mutate!(target_triple, Some("other".into()));
        mutate!(target_features, vec!["feature".into()]);
        mutate!(cargo_profile, "debug".into());
        mutate!(panic_strategy, Some("abort".into()));
        mutate!(profile_codegen_settings, vec![("opt".into(), "3".into())]);
        mutate!(cargo_version, Some("other".into()));
        mutate!(rustc_version, Some("other".into()));
        mutate!(trust_policy_digest, "other".into());
        mutate!(declared_build_env, vec!["FLAG".into()]);
        for (name, changed) in cases {
            assert_ne!(
                original,
                super::combined_cargo_inputs_digest(&primary, &changed),
                "{name}"
            );
        }
        let mut absent = secondary.clone();
        absent.package_source_map_digest = None;
        let absent_digest = super::combined_cargo_inputs_digest(&primary, &absent);
        absent.package_source_map_digest = Some(String::new());
        assert_ne!(
            absent_digest,
            super::combined_cargo_inputs_digest(&primary, &absent)
        );
    }

    #[test]
    fn trust_policy_identity_binds_all_declared_fields() {
        let original = sifr_package::TrustPolicy::default();
        let baseline = super::trust_policy_digest(&original);
        macro_rules! changed {
            ($field:ident) => {{
                let mut policy = original.clone();
                policy.$field.push("value".into());
                assert_ne!(
                    baseline,
                    super::trust_policy_digest(&policy),
                    stringify!($field)
                );
            }};
        }
        changed!(security_capabilities);
        changed!(native);
        changed!(build_scripts);
        changed!(proc_macros);
        changed!(python);
        changed!(python_native);
        changed!(rust_build_scripts);
        changed!(rust_proc_macros);
        changed!(native_links);
        changed!(unsafe_rust_bridges);
        changed!(build_env);
        changed!(rust_no_panic);
        changed!(rust_panic_abort);
    }

    fn cargo_inputs_fixture(
        package_id: &str,
        package_source_map_digest: Option<&str>,
    ) -> RustInteropCargoInputs {
        RustInteropCargoInputs {
            package_id: package_id.to_string(),
            cargo_metadata_digest: Some(format!("{package_id}-metadata")),
            sqlx_offline_metadata_digest: Some(format!("{package_id}-sqlx")),
            package_graph_digest: Some(format!("{package_id}-graph")),
            package_source_map_digest: package_source_map_digest.map(str::to_string),
            cargo_lock_digest: Some(format!("{package_id}-lock")),
            target_triple: Some("test-target".to_string()),
            target_features: Vec::new(),
            cargo_profile: "release".to_string(),
            panic_strategy: None,
            profile_codegen_settings: Vec::new(),
            cargo_version: Some("cargo 1.0.0".to_string()),
            rustc_version: Some("rustc 1.0.0".to_string()),
            trust_policy_digest: format!("{package_id}-trust"),
            declared_build_env: Vec::new(),
        }
    }
}
