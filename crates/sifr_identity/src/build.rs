//! Build-only inventory and Cargo change tracking, shared without embedded volatile state.
#![allow(clippy::print_stdout)]
use crate::IdentityEncoder;
use std::{
    env, io,
    path::{Path, PathBuf},
    process::Command,
};

#[path = "input_tree.rs"]
mod input_tree;
#[path = "production_inputs.rs"]
mod production_inputs;
use input_tree::visit;
use production_inputs::{visit_package, workspace_packages};

fn configuration(hash: &mut IdentityEncoder) -> io::Result<()> {
    let mut settings: Vec<_> = env::vars_os()
        .filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value)))
        .filter(|(name, _)| {
            name.starts_with("CARGO_FEATURE_")
                || name.starts_with("CARGO_CFG_")
                || matches!(
                    name.as_str(),
                    "TARGET"
                        | "HOST"
                        | "PROFILE"
                        | "OPT_LEVEL"
                        | "DEBUG"
                        | "CARGO_ENCODED_RUSTFLAGS"
                        | "RUSTC_WRAPPER"
                        | "RUSTC_WORKSPACE_WRAPPER"
                )
        })
        .collect();
    settings.sort();
    for (name, value) in settings {
        println!("cargo:rerun-if-env-changed={name}");
        hash.field(&name, value.as_encoded_bytes());
    }
    println!("cargo:rerun-if-env-changed=RUSTC");
    let rustc =
        env::var_os("RUSTC").ok_or_else(|| io::Error::other("Cargo did not supply RUSTC"))?;
    let version = Command::new(rustc).arg("-vV").output()?;
    if !version.status.success() {
        return Err(io::Error::other("selected build rustc -vV failed"));
    }
    hash.field("rustc", &version.stdout);
    Ok(())
}

/// Emit a package-local token from manifest-owned production inputs.
pub fn emit_local_token() -> io::Result<()> {
    let package = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| io::Error::other("missing package root"))?,
    );
    let root = package
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("missing workspace root"))?;
    let mut hash = IdentityEncoder::new("compiled-package-inputs-v2");
    visit_package(root, &package, &mut hash)?;
    for input in ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo"] {
        visit(root, &root.join(input), &mut hash)?;
    }
    visit_package(root, &root.join("crates/sifr_identity"), &mut hash)?;
    println!(
        "cargo:rustc-env=SIFR_PORTABLE_SOURCE_TOKEN={}",
        hash.clone().finish()
    );
    configuration(&mut hash)?;
    let token = hash.finish();
    println!("cargo:rustc-env=SIFR_LOCAL_INPUT_TOKEN={token}");
    let mut closure = IdentityEncoder::new("compiled-dependency-closure-v1");
    closure.field("local", token.as_bytes());
    dependency_tokens(&mut closure);
    println!("cargo:compiled_identity={}", closure.finish());
    Ok(())
}

/// Only the outer executable embeds this aggregate; libraries never depend on it.
pub fn emit_product_identity() -> io::Result<()> {
    let package = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| io::Error::other("missing package root"))?,
    );
    let root = package
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("missing workspace root"))?;
    let mut hash = IdentityEncoder::new("compiler-build-v2");
    for input in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo",
        "third_party",
    ] {
        visit(root, &root.join(input), &mut hash)?;
    }
    for package in workspace_packages(root)? {
        visit_package(root, &package, &mut hash)?;
    }
    // Pair canonical compiler sources across native-tool feature/profile selections.
    // This is source identity, not permission to execute a helper or package authentication.
    println!(
        "cargo:rustc-env=SIFR_COMPILER_SOURCE_ID={}",
        hash.clone().finish()
    );
    configuration(&mut hash)?;
    dependency_tokens(&mut hash);
    println!("cargo:rustc-env=SIFR_COMPILER_BUILD_ID={}", hash.finish());
    Ok(())
}

fn dependency_tokens(hash: &mut IdentityEncoder) {
    let mut tokens: Vec<_> = env::vars_os()
        .filter_map(|(name, value)| name.into_string().ok().map(|name| (name, value)))
        .filter(|(key, _)| key.starts_with("DEP_") && key.ends_with("_COMPILED_IDENTITY"))
        .collect();
    tokens.sort();
    for (name, token) in tokens {
        hash.field(&name, token.as_encoded_bytes());
    }
}

#[cfg(test)]
#[path = "../tests/unit/build_tests.rs"]
mod tests;
