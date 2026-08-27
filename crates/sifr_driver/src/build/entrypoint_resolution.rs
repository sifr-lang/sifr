use super::cargo_resolution::CargoResolutionPolicy;
use super::entrypoint::PackageEntrypoint;
use super::report::BuildCompilationMode;
use sifr_compiler_services::StdlibCompiled;
use sifr_stdlib_manifest::CargoVendorMode;
use std::path::{Path, PathBuf};

pub(super) fn package_cargo_resolution_policy(
    entrypoint: Option<&PackageEntrypoint>,
    stdlib: &StdlibCompiled,
) -> CargoResolutionPolicy {
    let Some(entrypoint) = entrypoint else {
        return CargoResolutionPolicy::normal();
    };
    let mut authoritative_locks = Vec::new();
    let mut trusted_vendor_dirs = Vec::new();
    if let Some(package) = entrypoint.graph.packages.get(&entrypoint.package_id) {
        if let Some(lock) = nearest_ancestor_file(&package.package_root, "Cargo.lock") {
            authoritative_locks.push(lock);
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
        lock_mode: entrypoint.lock_mode,
        cargo_vendor_mode: CargoVendorMode::PackageOwned,
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
