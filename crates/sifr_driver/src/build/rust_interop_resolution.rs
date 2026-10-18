use super::cargo_resolution::CargoResolutionPolicy;
use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, apply_package_rust_interop_metadata_with_resolution,
};
use super::rust_interop_probe_policy::DirectProbePolicy;
use crate::diagnostics::RenderedDiagnostic;

pub(super) fn resolve_package_rust_interop_metadata(
    generated: GeneratedBinaryProject,
    context: Option<PackageRustInteropContext>,
    native: Option<&Result<sifr_sysroot::NativeToolchain, String>>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    let policy = native.map_or_else(CargoResolutionPolicy::normal, |native| {
        CargoResolutionPolicy::normal_with_native(native.clone())
    });
    apply_package_rust_interop_metadata_with_resolution(
        generated,
        context,
        &policy,
        DirectProbePolicy::DeferTrustedSysroot,
    )
}

#[cfg(test)]
pub(super) fn apply_package_rust_interop_metadata(
    generated: GeneratedBinaryProject,
    context: Option<PackageRustInteropContext>,
) -> Result<GeneratedBinaryProject, Vec<RenderedDiagnostic>> {
    apply_package_rust_interop_metadata_with_resolution(
        generated,
        context,
        &CargoResolutionPolicy::normal(),
        DirectProbePolicy::ExecuteAll,
    )
}
