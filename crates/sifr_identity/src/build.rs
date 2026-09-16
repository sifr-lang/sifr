//! Build-only inventory and Cargo change tracking, shared without embedded volatile state.
#![allow(clippy::print_stdout)]
use crate::IdentityEncoder;
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

fn visit(root: &Path, path: &Path, hash: &mut IdentityEncoder) -> io::Result<()> {
    println!("cargo:rerun-if-changed={}", path.display());
    if path.is_dir() {
        // Watching directories detects inventory additions/removals, including source archives.
        println!("cargo:rerun-if-changed={}", path.display());
        let mut children = fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<io::Result<Vec<_>>>()?;
        children.sort();
        for child in children {
            let name = child.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if matches!(name, ".git" | "target" | "__pycache__") {
                continue;
            }
            visit(root, &child, hash)?;
        }
    } else if path.is_file() {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if matches!(extension, "md" | "mdx") {
            return Ok(());
        }
        let relative = path.strip_prefix(root).map_err(io::Error::other)?;
        let name = relative
            .to_str()
            .ok_or_else(|| io::Error::other("non-UTF8 compiler input"))?;
        println!("cargo:rerun-if-changed={}", path.display());
        hash.field(name, &fs::read(path)?);
    }
    Ok(())
}

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

/// Emit a package-local token. Its caller owns additional generated/parser inputs.
pub fn emit_local_token(extra_roots: &[&str]) -> io::Result<()> {
    let package = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| io::Error::other("missing package root"))?,
    );
    let root = package
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| io::Error::other("missing workspace root"))?;
    let mut hash = IdentityEncoder::new("compiled-package-inputs-v1");
    visit(root, &package, &mut hash)?;
    for input in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo",
        "crates/sifr_identity",
    ] {
        visit(root, &root.join(input), &mut hash)?;
    }
    for input in extra_roots {
        visit(root, &root.join(input), &mut hash)?;
    }
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
    let mut hash = IdentityEncoder::new("compiler-build-v1");
    for input in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo",
        "crates",
        "third_party",
    ] {
        visit(root, &root.join(input), &mut hash)?;
    }
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
