use crate::diagnostics::diagnostic_with_code;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_frontend::{SourceOrigin, SourcePath, SourceText, WorkspaceAuxiliarySource};
use sifr_stdlib_model::{load_stdlib_tooling_sources_from_sysroot, LoadedStdlibSourceKind};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolingSysrootStatus {
    pub root: PathBuf,
    pub toolchain_id: String,
}

pub fn sysroot_status() -> Result<ToolingSysrootStatus, Vec<RenderedDiagnostic>> {
    let sysroot = resolve_tooling_sysroot()?;
    let toolchain_id = sysroot.toolchain_id();
    Ok(ToolingSysrootStatus {
        root: sysroot.root,
        toolchain_id,
    })
}

pub fn tooling_sources() -> Result<Vec<WorkspaceAuxiliarySource>, Vec<RenderedDiagnostic>> {
    let sysroot = resolve_tooling_sysroot()?;
    let sources = load_stdlib_tooling_sources_from_sysroot(&sysroot).map_err(|error| {
        vec![diagnostic_with_code(
            format!(
                "Sifr stdlib source inventory is invalid for sysroot {}: {error}",
                sysroot.root.display()
            ),
            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
        )]
    })?;
    Ok(sources
        .into_iter()
        .map(|source| WorkspaceAuxiliarySource {
            module_name: Some(source.module),
            path: SourcePath::new(source.path),
            source: SourceText::new(source.source),
            origin: match source.kind {
                LoadedStdlibSourceKind::Public => SourceOrigin::SysrootPublicStdlib,
                LoadedStdlibSourceKind::PrivateDeclaration => {
                    SourceOrigin::SysrootPrivateDeclaration
                }
            },
        })
        .collect())
}

fn resolve_tooling_sysroot() -> Result<sifr_sysroot::ResolvedSysroot, Vec<RenderedDiagnostic>> {
    sifr_sysroot::resolve_sysroot(None).map_err(|error| {
        vec![diagnostic_with_code(
            error.boundary_message(),
            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
        )]
    })
}
