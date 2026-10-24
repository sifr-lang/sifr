use super::super::native_storage::write_changed;
use super::super::rust_interop_probe_manifest::bind_probe_package_identity;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Each original probe keeps a crate root; crate-relative types and local stubs
/// cannot bind to another probe. Only identical dependency graphs share a batch.
pub(super) fn write(root: &Path, probes: &[(&str, &str)]) -> Result<Vec<PathBuf>, String> {
    let first = probes.first().ok_or("empty Rust probe workspace")?;
    let mut workspace: toml::Table = first.0.parse().map_err(|e| format!("{e}"))?;
    workspace.remove("package");
    workspace.remove("dependencies");
    let mut names = Vec::new();
    let mut sources = BTreeMap::new();
    let mut results = Vec::new();
    for &(manifest, source_bytes) in probes {
        if manifest != first.0 {
            return Err("Rust probe workspace has incompatible manifests".into());
        }
        let manifest = bind_probe_package_identity(&manifest, source_bytes);
        let mut member: toml::Table = manifest.parse().map_err(|e| format!("{e}"))?;
        let name = member
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(toml::Value::as_str)
            .ok_or("Rust probe package has no name")?
            .to_owned();
        if !name.starts_with("sifr-rust-probe-")
            || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("invalid Rust probe package identity".into());
        }
        let relative = format!("members/{name}");
        let member_root = root.join(&relative);
        let source = member_root.join("src/lib.rs");
        if let Some(existing) = sources.get(&relative) {
            if existing != source_bytes {
                return Err("Rust probe package identity collision".into());
            }
        } else {
            sources.insert(relative.clone(), source_bytes.to_owned());
            names.push(toml::Value::String(relative));
            member.remove("workspace");
            member.remove("profile");
            member.remove("patch");
            std::fs::create_dir_all(member_root.join("src")).map_err(|e| e.to_string())?;
            write_changed(
                &member_root.join("Cargo.toml"),
                toml::to_string(&member)
                    .map_err(|e| e.to_string())?
                    .as_bytes(),
            )
            .map_err(|e| e.to_string())?;
            write_changed(&source, source_bytes.as_bytes()).map_err(|e| e.to_string())?;
        }
        results.push(source);
    }
    workspace.insert(
        "workspace".into(),
        toml::Value::Table(toml::Table::from_iter([
            ("resolver".into(), toml::Value::String("3".into())),
            ("members".into(), toml::Value::Array(names)),
        ])),
    );
    write_changed(
        &root.join("Cargo.toml"),
        toml::to_string(&workspace)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )
    .map_err(|e| e.to_string())?;
    Ok(results)
}
