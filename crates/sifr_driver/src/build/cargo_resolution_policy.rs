//! Frozen native selection and Cargo resolution policy, separate from materialization.
use crate::diagnostics::RenderedDiagnostic;
use sifr_package::CargoLockMode;
use sifr_stdlib_manifest::CargoVendorMode;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub(crate) struct CargoResolutionPolicy {
    pub(crate) child_leases: super::native_child_leases::NativeChildLeases,
    pub(crate) application_profile: crate::ApplicationProfile,
    pub(crate) native_toolchain: Result<sifr_sysroot::NativeToolchain, String>,
    pub(crate) lock_mode: CargoLockMode,
    pub(crate) cargo_vendor_mode: CargoVendorMode,
    pub(crate) authoritative_locks: Vec<PathBuf>,
    pub(crate) trusted_vendor_dirs: Vec<PathBuf>,
}

impl CargoResolutionPolicy {
    pub(crate) fn resolve_native_toolchain() -> Result<sifr_sysroot::NativeToolchain, String> {
        let cwd =
            std::env::current_dir().map_err(|_| "cannot resolve native invocation directory")?;
        sifr_sysroot::NativeToolchain::resolve_at(&cwd)
    }
    pub(crate) fn cargo_command(&self) -> Result<Command, Vec<RenderedDiagnostic>> {
        let mut command = self
            .native_toolchain
            .as_ref()
            .map_err(Clone::clone)
            .and_then(sifr_sysroot::NativeToolchain::cargo_command)
            .map_err(|error| {
                vec![super::cargo_resolution::cargo_resolution_error(
                    error.clone(),
                )]
            })?;
        self.child_leases.bind(&mut command).map_err(|error| {
            vec![super::cargo_resolution::cargo_resolution_error(
                error.to_string(),
            )]
        })?;
        Ok(command)
    }

    pub(crate) fn with_lease(
        &self,
        lease: &std::fs::File,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let mut next = self.clone();
        next.child_leases = self.child_leases.with(lease).map_err(|error| {
            vec![super::cargo_resolution::cargo_resolution_error(
                error.to_string(),
            )]
        })?;
        Ok(next)
    }

    pub(crate) fn normal() -> Self {
        Self::normal_with_native(Self::resolve_native_toolchain())
    }

    pub(crate) fn normal_with_native(
        native_toolchain: Result<sifr_sysroot::NativeToolchain, String>,
    ) -> Self {
        Self {
            child_leases: Default::default(),
            application_profile: crate::ApplicationProfile::Release,
            native_toolchain,
            lock_mode: CargoLockMode::Normal,
            cargo_vendor_mode: CargoVendorMode::SysrootOnly,
            authoritative_locks: Vec::new(),
            trusted_vendor_dirs: Vec::new(),
        }
    }

    pub(super) const fn uses_sysroot_vendor(&self) -> bool {
        matches!(self.cargo_vendor_mode, CargoVendorMode::SysrootOnly)
    }
}
