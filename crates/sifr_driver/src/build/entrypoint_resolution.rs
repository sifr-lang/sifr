use super::cargo_resolution::CargoResolutionPolicy;
use super::entrypoint::PackageEntrypoint;
use super::report::BuildCompilationMode;
use crate::stdlib::StdlibCompiled;
use sifr_stdlib_manifest::CargoVendorMode;
use std::path::{Path, PathBuf};

pub(crate) fn package_cargo_resolution_policy(
    entrypoint: Option<&PackageEntrypoint>,
    stdlib: &StdlibCompiled,
    prepare_native: bool,
) -> CargoResolutionPolicy {
    let mut authoritative_locks = Vec::new();
    let mut trusted_vendor_dirs = Vec::new();
    if let Some(entrypoint) = entrypoint {
        if let Some(package) = entrypoint.graph.packages.get(&entrypoint.package_id) {
            if let Some(lock) = nearest_ancestor_file(&package.package_root, "Cargo.lock") {
                authoritative_locks.push(lock);
            }
        }
    }
    if let Some(lock) = stdlib
        .interop
        .sysroot
        .as_ref()
        .map(|sysroot| sysroot.paths.cargo_lock.clone())
    {
        if !authoritative_locks.contains(&lock) {
            authoritative_locks.push(lock);
        }
    }
    if let Some(vendor_dir) = stdlib
        .interop
        .sysroot
        .as_ref()
        .map(|sysroot| sysroot.paths.vendor.clone())
    {
        trusted_vendor_dirs.push(vendor_dir);
    }
    CargoResolutionPolicy {
        application_profile: crate::ApplicationProfile::Development,
        native_toolchain: if prepare_native {
            entrypoint
                .and_then(|entrypoint| entrypoint.graph.packages.get(&entrypoint.package_id))
                .map_or_else(CargoResolutionPolicy::resolve_native_toolchain, |package| {
                    sifr_sysroot::NativeToolchain::resolve_at(&package.package_root).map(|tools| {
                        tools.with_declared_environment(package.manifest.trust.build_env.clone())
                    })
                })
        } else {
            Err("native policy was not requested by this frontend-only check".to_owned())
        },
        lock_mode: entrypoint.map_or(sifr_package::CargoLockMode::Locked, |entrypoint| {
            entrypoint.lock_mode
        }),
        cargo_vendor_mode: entrypoint.map_or(CargoVendorMode::SysrootOnly, |_| {
            CargoVendorMode::PackageOwned
        }),
        authoritative_locks,
        trusted_vendor_dirs,
    }
}

pub(super) const fn requested_vendor_mode_for_build(mode: BuildCompilationMode) -> CargoVendorMode {
    match mode {
        BuildCompilationMode::SingleFile | BuildCompilationMode::Project => {
            CargoVendorMode::SysrootOnly
        }
        BuildCompilationMode::PackageProject => CargoVendorMode::PackageOwned,
    }
}

fn nearest_ancestor_file(start: &Path, file_name: &str) -> Option<PathBuf> {
    start
        .ancestors()
        .map(|ancestor| ancestor.join(file_name))
        .find(|candidate| candidate.is_file())
}
