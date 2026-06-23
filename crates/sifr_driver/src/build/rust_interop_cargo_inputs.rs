use super::rust_interop::PackageRustInteropContext;
use super::rust_interop_digest::{
    digest_file, digest_path, fnv1a64_hex, normalized_path_string, push_cache_bytes,
};
use super::sysroot_interop::SysrootRustInteropTrust;
use sifr_codegen::{RustBridgeSourceDigest, RustInteropCargoInputs};
use sifr_package::{digest_package_graph, digest_package_source_map, TrustPolicy};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn bridge_source_digests(
    context: &PackageRustInteropContext,
    package: &sifr_package::SifrPackageMetadata,
) -> Vec<RustBridgeSourceDigest> {
    let mut digests = package
        .manifest
        .rust
        .bridges
        .iter()
        .map(|bridge_root| RustBridgeSourceDigest {
            package_id: context.package_id.0.clone(),
            bridge_root: normalized_path_string(bridge_root),
            digest: digest_path(&package.package_root.join(bridge_root)),
        })
        .collect::<Vec<_>>();
    digests.sort_by(|left, right| {
        (&left.package_id, &left.bridge_root).cmp(&(&right.package_id, &right.bridge_root))
    });
    digests
}

pub(super) fn cargo_inputs(
    context: &PackageRustInteropContext,
    package: &sifr_package::SifrPackageMetadata,
) -> RustInteropCargoInputs {
    if let Some(trust) = context
        .sysroot_trust
        .as_ref()
        .filter(|trust| trust.package_id == package.package_id)
    {
        return sysroot_cargo_inputs(trust, &package.manifest.trust);
    }
    let graph_digest = digest_package_graph(&context.graph);
    let source_map_digest = digest_package_source_map(&context.source_map);
    let trust_policy_digest = trust_policy_digest(&package.manifest.trust);
    let mut declared_build_env = package.manifest.trust.build_env.clone();
    declared_build_env.sort();
    RustInteropCargoInputs {
        package_id: context.package_id.0.clone(),
        cargo_metadata_digest: None,
        package_graph_digest: Some(graph_digest.hex),
        package_source_map_digest: Some(source_map_digest.hex),
        cargo_lock_digest: cargo_lock_digest(&package.package_root),
        target_triple: target_triple(),
        target_features: target_features(),
        cargo_profile: "release".to_string(),
        panic_strategy: std::env::var("SIFR_RUST_PANIC_STRATEGY").ok(),
        profile_codegen_settings: profile_codegen_settings(&package.package_root, "release"),
        cargo_version: tool_version("cargo"),
        rustc_version: tool_version("rustc"),
        bridge_version: package.manifest.rust.bridge_version,
        trust_policy_digest,
        declared_build_env,
    }
}

pub(super) fn combined_cargo_inputs(
    mut primary: RustInteropCargoInputs,
    secondary: RustInteropCargoInputs,
) -> RustInteropCargoInputs {
    let digest = combined_cargo_inputs_digest(&primary, &secondary);
    primary.package_id = format!("{}+{}", primary.package_id, secondary.package_id);
    primary.cargo_metadata_digest = Some(digest.clone());
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
    trust: &SysrootRustInteropTrust,
    policy: &TrustPolicy,
) -> RustInteropCargoInputs {
    let trust_policy_digest = trust_policy_digest(policy);
    let mut declared_build_env = policy.build_env.clone();
    declared_build_env.sort();
    RustInteropCargoInputs {
        package_id: format!("sifr-sysroot-stdlib@{}", trust.toolchain_id),
        cargo_metadata_digest: Some(sysroot_metadata_digest(trust)),
        package_graph_digest: Some(trust.sysroot_content_sha256.clone()),
        package_source_map_digest: Some(digest_path(&trust.stdlib_private_sources)),
        cargo_lock_digest: digest_file(&trust.cargo_lock),
        target_triple: target_triple(),
        target_features: target_features(),
        cargo_profile: "release".to_string(),
        panic_strategy: std::env::var("SIFR_RUST_PANIC_STRATEGY").ok(),
        profile_codegen_settings: profile_codegen_settings(&trust.sysroot_root, "release"),
        cargo_version: tool_version("cargo"),
        rustc_version: tool_version("rustc"),
        bridge_version: Some(1),
        trust_policy_digest,
        declared_build_env,
    }
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
            if !path.extension().is_some_and(|extension| extension == "rs") {
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
            'r' if matches!(chars.peek(), Some('"') | Some('#')) => {
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
    combined_digest(&[&format!("{primary:?}"), &format!("{secondary:?}")])
}

fn combine_optional_digest(left: Option<String>, right: Option<String>) -> Option<String> {
    match (left, right) {
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(value),
        (Some(left), Some(right)) => Some(combined_digest(&[left.as_str(), right.as_str()])),
    }
}

fn combined_digest(parts: &[&str]) -> String {
    let mut bytes = Vec::new();
    for part in parts {
        push_cache_bytes(&mut bytes, part);
    }
    fnv1a64_hex(&bytes)
}

fn sysroot_metadata_digest(trust: &SysrootRustInteropTrust) -> String {
    let mut bytes = Vec::new();
    push_cache_bytes(&mut bytes, &trust.sysroot_root.display().to_string());
    push_cache_bytes(&mut bytes, &trust.toolchain_id);
    push_cache_bytes(&mut bytes, &trust.sysroot_content_sha256);
    push_cache_bytes(&mut bytes, &digest_path(&trust.stdlib_private_sources));
    push_cache_bytes(&mut bytes, &digest_path(&trust.stdlib_crate));
    push_cache_bytes(&mut bytes, &digest_path(&trust.runtime_crate));
    if let Some(lock_digest) = digest_file(&trust.cargo_lock) {
        push_cache_bytes(&mut bytes, &lock_digest);
    }
    fnv1a64_hex(&bytes)
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

fn target_triple() -> Option<String> {
    std::env::var("SIFR_TARGET").ok().or_else(rustc_host_triple)
}

fn rustc_host_triple() -> Option<String> {
    Command::new("rustc")
        .arg("-vV")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|version| {
            version
                .lines()
                .find_map(|line| line.strip_prefix("host: "))
                .map(str::to_string)
        })
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

fn trust_policy_digest(trust: &TrustPolicy) -> String {
    let mut entries = BTreeMap::new();
    entries.insert("rust-build-scripts", trust.rust_build_scripts.clone());
    entries.insert("rust-proc-macros", trust.rust_proc_macros.clone());
    entries.insert("native-links", trust.native_links.clone());
    entries.insert("unsafe-rust-bridges", trust.unsafe_rust_bridges.clone());
    entries.insert("build-env", trust.build_env.clone());
    entries.insert("rust-no-panic", trust.rust_no_panic.clone());
    entries.insert("rust-panic-abort", trust.rust_panic_abort.clone());
    let mut bytes = Vec::new();
    for (key, mut values) in entries {
        values.sort();
        push_cache_bytes(&mut bytes, key);
        for value in values {
            push_cache_bytes(&mut bytes, &value);
        }
    }
    fnv1a64_hex(&bytes)
}

fn tool_version(tool: &str) -> Option<String> {
    Command::new(tool)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|version| version.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        combined_cargo_inputs, generated_bridge_module_path, imports_generated_bridge_namespace,
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
            combined.package_source_map_digest,
            primary.package_source_map_digest
        );
        assert_ne!(
            combined.package_source_map_digest,
            secondary.package_source_map_digest
        );
    }

    fn cargo_inputs_fixture(
        package_id: &str,
        package_source_map_digest: Option<&str>,
    ) -> RustInteropCargoInputs {
        RustInteropCargoInputs {
            package_id: package_id.to_string(),
            cargo_metadata_digest: Some(format!("{package_id}-metadata")),
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
            bridge_version: Some(1),
            trust_policy_digest: format!("{package_id}-trust"),
            declared_build_env: Vec::new(),
        }
    }
}
