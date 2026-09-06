use super::cargo_manifest::generate_portable_dependency_cargo_toml_with_interop;
use super::cargo_resolution::CargoResolutionPolicy;
use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code};
use sifr_codegen::{InteropBuildPlan, RustInteropResolvedRoot};
use sifr_diagnostics::DiagnosticCode;
use sifr_stdlib_manifest::{SysrootCrate, SysrootDependencyPlan};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static LOCAL_RESOLUTION_NONCE: AtomicU64 = AtomicU64::new(0);

pub(super) fn local_resolution_project_path(output_dir: &Path, project_name: &str) -> PathBuf {
    let nonce = LOCAL_RESOLUTION_NONCE.fetch_add(1, Ordering::Relaxed);
    output_dir.join(format!(
        ".{project_name}.sifr-local-resolution-{}-{nonce}",
        std::process::id()
    ))
}

pub(super) fn publish_portable_project(
    local_project: &Path,
    project_path: &Path,
) -> Result<(), Vec<RenderedDiagnostic>> {
    std::fs::create_dir_all(project_path).map_err(|error| {
        vec![portable_error(format!(
            "failed to create portable project directory: {error}"
        ))]
    })?;
    let destination_src = project_path.join("src");
    if destination_src.exists() {
        std::fs::remove_dir_all(&destination_src).map_err(|error| {
            vec![portable_error(format!(
                "failed to reset portable generated source directory: {error}"
            ))]
        })?;
    }
    copy_tree(&local_project.join("src"), &destination_src)?;
    for file_name in ["Cargo.toml", "Cargo.lock"] {
        let destination = project_path.join(file_name);
        if destination.exists() {
            std::fs::remove_file(&destination).map_err(|error| {
                vec![portable_error(format!(
                    "failed to replace portable {file_name}: {error}"
                ))]
            })?;
        }
        std::fs::copy(local_project.join(file_name), &destination).map_err(|error| {
            vec![portable_error(format!(
                "failed to publish portable {file_name}: {error}"
            ))]
        })?;
    }
    Ok(())
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), Vec<RenderedDiagnostic>> {
    std::fs::create_dir_all(destination).map_err(|error| {
        vec![portable_error(format!(
            "failed to create portable source directory: {error}"
        ))]
    })?;
    let entries = std::fs::read_dir(source).map_err(|error| {
        vec![portable_error(format!(
            "failed to read generated source directory: {error}"
        ))]
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            vec![portable_error(format!(
                "failed to read generated source entry: {error}"
            ))]
        })?;
        let destination = destination.join(entry.file_name());
        if entry
            .file_type()
            .map_err(|error| vec![portable_error(error.to_string())])?
            .is_dir()
        {
            copy_tree(&entry.path(), &destination)?;
        } else {
            std::fs::copy(entry.path(), destination).map_err(|error| {
                vec![portable_error(format!(
                    "failed to publish generated source file: {error}"
                ))]
            })?;
        }
    }
    Ok(())
}

pub(super) fn prepare_portable_project_metadata(
    project_path: &Path,
    project_name: &str,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
    cargo_resolution: &CargoResolutionPolicy,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let revision = exact_sysroot_revision(dependency_plan)?;
    let manifest = generate_portable_dependency_cargo_toml_with_interop(
        project_name,
        dependency_plan,
        interop,
        &revision,
    )
    .map_err(|message| vec![portable_error(message)])?;

    // Publish the portable manifest before rewriting the lock. If lock
    // finalization fails, no retained artifact exposes the temporary local
    // dependency paths used to resolve the compiler-owned build.
    std::fs::write(project_path.join("Cargo.toml"), manifest).map_err(|error| {
        vec![portable_error(format!(
            "failed to write portable generated Cargo manifest: {error}"
        ))]
    })?;
    rewrite_lock_sources(
        &project_path.join("Cargo.lock"),
        dependency_plan,
        interop,
        cargo_resolution,
        &revision,
    )
}

fn exact_sysroot_revision(
    dependency_plan: &SysrootDependencyPlan,
) -> Result<String, Vec<RenderedDiagnostic>> {
    let manifest_path = dependency_plan.sysroot_root.join("sysroot.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|source| sifr_sysroot::parse_sysroot_manifest(&source).ok());
    if let Some(revision) = manifest
        .as_ref()
        .map(|manifest| manifest.built_by_compiler_commit.as_str())
        .filter(|revision| exact_revision(revision))
    {
        return Ok(revision.to_ascii_lowercase());
    }

    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD^{commit}"])
        .current_dir(&dependency_plan.sysroot_root)
        .output()
        .map_err(|error| {
            vec![portable_error(format!(
                "failed to resolve the development sysroot revision: {error}"
            ))]
        })?;
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !output.status.success() || !exact_revision(&revision) {
        return Err(vec![portable_error(
            "portable generated projects require a sysroot built from one exact Git commit",
        )]);
    }
    Ok(revision.to_ascii_lowercase())
}

fn exact_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone)]
struct PortableSource {
    name: String,
    version: String,
    source: String,
}

fn rewrite_lock_sources(
    lock_path: &Path,
    dependency_plan: &SysrootDependencyPlan,
    interop: &InteropBuildPlan,
    cargo_resolution: &CargoResolutionPolicy,
    sifr_revision: &str,
) -> Result<(), Vec<RenderedDiagnostic>> {
    let source = std::fs::read_to_string(lock_path).map_err(|error| {
        vec![portable_error(format!(
            "failed to read resolved generated Cargo lockfile: {error}"
        ))]
    })?;
    let mut lock = source.parse::<toml::Table>().map_err(|error| {
        vec![portable_error(format!(
            "failed to parse resolved generated Cargo lockfile: {error}"
        ))]
    })?;
    let mut requirements = dependency_plan
        .crates
        .iter()
        .map(|dependency| PortableSource {
            name: dependency.krate.package_name().to_string(),
            version: "0.0.0".to_string(),
            source: format!(
                "git+https://github.com/sifr-lang/sifr.git?rev={sifr_revision}#{sifr_revision}"
            ),
        })
        .collect::<Vec<_>>();
    if dependency_plan
        .crates
        .iter()
        .any(|dependency| dependency.krate == SysrootCrate::SifrStdlib)
        && !requirements
            .iter()
            .any(|requirement| requirement.name == "sifr_runtime")
    {
        requirements.push(PortableSource {
            name: "sifr_runtime".to_string(),
            version: "0.0.0".to_string(),
            source: format!(
                "git+https://github.com/sifr-lang/sifr.git?rev={sifr_revision}#{sifr_revision}"
            ),
        });
    }
    requirements.extend(interop_sources(interop)?);

    let authority = authority_packages(&cargo_resolution.authoritative_locks)?;
    let packages = lock
        .get_mut("package")
        .and_then(toml::Value::as_array_mut)
        .ok_or_else(|| {
            vec![portable_error(
                "generated Cargo lockfile has no package array",
            )]
        })?;
    for requirement in requirements {
        let package = packages
            .iter_mut()
            .find_map(|package| {
                let table = package.as_table_mut()?;
                (table.get("name")?.as_str()? == requirement.name
                    && table.get("version")?.as_str()? == requirement.version
                    && table.get("source").is_none())
                .then_some(table)
            })
            .ok_or_else(|| {
                vec![portable_error(format!(
                    "resolved generated Cargo lockfile is missing local package {} {}",
                    requirement.name, requirement.version
                ))]
            })?;
        package.insert(
            "source".to_string(),
            toml::Value::String(requirement.source.clone()),
        );
        package.remove("checksum");
        if requirement.source.starts_with("registry+") {
            let key = (
                requirement.name.clone(),
                requirement.version.clone(),
                requirement.source.clone(),
            );
            let checksum = authority.get(&key).ok_or_else(|| {
                vec![portable_error(format!(
                    "portable dependency {} {} is absent from every authoritative lockfile",
                    requirement.name, requirement.version
                ))]
            })?;
            package.insert(
                "checksum".to_string(),
                toml::Value::String(checksum.clone()),
            );
        }
    }
    let rendered = toml::to_string(&lock).map_err(|error| {
        vec![portable_error(format!(
            "failed to serialize portable generated Cargo lockfile: {error}"
        ))]
    })?;
    std::fs::write(lock_path, rendered).map_err(|error| {
        vec![portable_error(format!(
            "failed to write portable generated Cargo lockfile: {error}"
        ))]
    })
}

fn interop_sources(
    interop: &InteropBuildPlan,
) -> Result<Vec<PortableSource>, Vec<RenderedDiagnostic>> {
    let mut sources = BTreeMap::new();
    for target in &interop.rust.resolved_targets {
        let values = match &target.root {
            RustInteropResolvedRoot::DirectCargoDependency {
                cargo_package_name,
                cargo_version,
                cargo_source: Some(cargo_source),
                ..
            }
            | RustInteropResolvedRoot::PackageBridge {
                cargo_package_name,
                cargo_version,
                cargo_source: Some(cargo_source),
                ..
            } => Some((cargo_package_name, cargo_version, cargo_source)),
            _ => None,
        };
        if let Some((name, version, source)) = values {
            let source = portable_lock_source(name, source)?;
            sources.insert(
                (name.clone(), version.clone(), source.clone()),
                PortableSource {
                    name: name.clone(),
                    version: version.clone(),
                    source,
                },
            );
        }
    }
    Ok(sources.into_values().collect())
}

fn portable_lock_source(
    dependency_name: &str,
    source: &str,
) -> Result<String, Vec<RenderedDiagnostic>> {
    if source.starts_with("registry+") {
        return Ok(source.to_string());
    }
    let Some(git) = source.strip_prefix("git+") else {
        return Err(vec![portable_error(format!(
            "Rust dependency `{dependency_name}` uses unsupported Cargo source `{source}`"
        ))]);
    };
    let (location, revision) = git.rsplit_once('#').ok_or_else(|| {
        vec![portable_error(format!(
            "Git dependency `{dependency_name}` has no exact locked revision"
        ))]
    })?;
    if !exact_revision(revision) {
        return Err(vec![portable_error(format!(
            "Git dependency `{dependency_name}` has no exact 40-character locked revision"
        ))]);
    }
    let url = location.split('?').next().unwrap_or(location);
    Ok(format!("git+{url}?rev={revision}#{revision}"))
}

// Checksums keyed by the exact package name, version, and Cargo source identity.
type AuthorityPackageChecksums = BTreeMap<(String, String, String), String>;

fn authority_packages(
    paths: &[PathBuf],
) -> Result<AuthorityPackageChecksums, Vec<RenderedDiagnostic>> {
    let mut packages = BTreeMap::new();
    for path in paths {
        let source = std::fs::read_to_string(path).map_err(|error| {
            vec![portable_error(format!(
                "failed to read authoritative Cargo lockfile '{}': {error}",
                path.display()
            ))]
        })?;
        let lock = source.parse::<toml::Table>().map_err(|error| {
            vec![portable_error(format!(
                "failed to parse authoritative Cargo lockfile '{}': {error}",
                path.display()
            ))]
        })?;
        for package in lock
            .get("package")
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_table)
        {
            let Some((name, version, source, checksum)) = package
                .get("name")
                .and_then(toml::Value::as_str)
                .zip(package.get("version").and_then(toml::Value::as_str))
                .zip(package.get("source").and_then(toml::Value::as_str))
                .zip(package.get("checksum").and_then(toml::Value::as_str))
                .map(|(((name, version), source), checksum)| (name, version, source, checksum))
            else {
                continue;
            };
            packages.insert(
                (name.to_string(), version.to_string(), source.to_string()),
                checksum.to_string(),
            );
        }
    }
    Ok(packages)
}

fn portable_error(message: impl Into<String>) -> RenderedDiagnostic {
    diagnostic_with_code(
        message.into(),
        DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_stdlib_manifest::{CargoVendorMode, SysrootCrateDependency};
    use std::collections::BTreeSet;

    #[test]
    fn portable_lock_rewrites_local_sysroot_packages_to_exact_git_sources() {
        let root = std::env::temp_dir().join(format!(
            "sifr_portable_lock_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("test root should be created");
        let lock_path = root.join("Cargo.lock");
        std::fs::write(
            &lock_path,
            "version = 4\n\n[[package]]\nname = \"sifr_runtime\"\nversion = \"0.0.0\"\n",
        )
        .expect("test lock should be written");
        let revision = "0123456789abcdef0123456789abcdef01234567";
        let plan = SysrootDependencyPlan {
            stdlib_modules: BTreeSet::new(),
            required_features: BTreeSet::new(),
            sysroot_root: PathBuf::from("/private/host/sysroot"),
            toolchain_id: "test".to_string(),
            sysroot_content_sha256: "content".to_string(),
            cargo_config: PathBuf::from("/private/host/sysroot/.cargo/config.toml"),
            vendor_dir: PathBuf::from("/private/host/sysroot/vendor"),
            crates: vec![SysrootCrateDependency {
                krate: SysrootCrate::SifrRuntime,
                path: PathBuf::from("/private/host/sysroot/crates/sifr_runtime"),
                features: BTreeSet::new(),
            }],
            retained_direct_dependencies: Vec::new(),
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            cache_fingerprint: "test".to_string(),
        };
        let policy = CargoResolutionPolicy {
            lock_mode: sifr_package::CargoLockMode::Locked,
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            authoritative_locks: Vec::new(),
            trusted_vendor_dirs: Vec::new(),
        };

        rewrite_lock_sources(
            &lock_path,
            &plan,
            &InteropBuildPlan::default(),
            &policy,
            revision,
        )
        .expect("portable lock should render");
        let lock = std::fs::read_to_string(&lock_path).expect("portable lock should be readable");
        assert!(!lock.contains("/private/host"), "{lock}");
        assert!(lock.contains(&format!(
            "source = \"git+https://github.com/sifr-lang/sifr.git?rev={revision}#{revision}\""
        )));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn git_lock_sources_match_the_portable_exact_revision_manifest() {
        let revision = "0123456789abcdef0123456789abcdef01234567";
        let source = format!("git+https://example.com/dependency.git?branch=main#{revision}");

        assert_eq!(
            portable_lock_source("dependency", &source).expect("exact Git source should normalize"),
            format!("git+https://example.com/dependency.git?rev={revision}#{revision}")
        );
    }
}
