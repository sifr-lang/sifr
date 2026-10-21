//! A pinned semantic session. Source checking has no native metadata side effect.
use crate::{diagnostics::diagnostic_with_code, metadata, stdlib::CheckedStdlib};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_identity::CompilerIdentity;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

enum StdlibOwner {
    Source(CheckedStdlib),
    Metadata(Arc<metadata::reader::Provider>),
}
#[derive(Clone)]
pub struct SemanticContext {
    identity: CompilerIdentity,
    sysroot: Result<sifr_sysroot::ResolvedSysroot, sifr_sysroot::SysrootError>,
    metadata_override: Option<PathBuf>,
    owner: Arc<Mutex<Option<Arc<StdlibOwner>>>>,
}
impl SemanticContext {
    pub fn new(identity: CompilerIdentity) -> Self {
        Self::from_resolved(identity, sifr_sysroot::resolve_sysroot(None))
    }
    pub fn with_sysroot(
        identity: CompilerIdentity,
        sysroot: sifr_sysroot::ResolvedSysroot,
    ) -> Self {
        Self::from_resolved(identity, Ok(sysroot))
    }
    fn from_resolved(
        identity: CompilerIdentity,
        sysroot: Result<sifr_sysroot::ResolvedSysroot, sifr_sysroot::SysrootError>,
    ) -> Self {
        Self {
            identity,
            sysroot,
            metadata_override: None,
            owner: Arc::new(Mutex::new(None)),
        }
    }
    #[must_use]
    pub fn with_metadata_override(mut self, path: PathBuf) -> Self {
        self.metadata_override = Some(path);
        self.owner = Arc::new(Mutex::new(None));
        self
    }
    pub fn for_test_tokens(mut tokens: Vec<(&str, &str)>, configuration: &str) -> Self {
        tokens.extend(crate::compiled_input_tokens());
        Self::new(CompilerIdentity::for_test(tokens, configuration))
    }
    pub fn for_test() -> Self {
        Self::for_test_tokens(Vec::new(), "semantic-consumer")
    }
    pub fn identity(&self) -> &CompilerIdentity {
        &self.identity
    }
    pub fn sysroot(&self) -> Result<&sifr_sysroot::ResolvedSysroot, Vec<RenderedDiagnostic>> {
        self.sysroot
            .as_ref()
            .map_err(|e| failure(e.boundary_message()))
    }
    pub fn external_defs(&self) -> Result<sifr_lowering::ExternalDefs, Vec<RenderedDiagnostic>> {
        let mut owner = self
            .owner
            .lock()
            .map_err(|_| failure("semantic stdlib owner poisoned"))?;
        if owner.is_none() {
            let root = self.sysroot()?;
            let selected = if root.mode() == sifr_sysroot::SysrootMode::InstalledToolchain {
                StdlibOwner::Metadata(
                    metadata::installed::select(
                        &self.identity,
                        root,
                        self.metadata_override.as_deref(),
                    )
                    .map_err(failure)?,
                )
            } else {
                let inputs = metadata::inputs::Inputs::capture(
                    &self.identity,
                    &root.root,
                    metadata::installed::target(),
                )
                .map_err(failure)?;
                if let Some(path) = &self.metadata_override {
                    // An explicit artifact is exact: validate its full structure and
                    // compatibility before consuming it; never replace a failure.
                    let metadata = metadata::open_consumer(path, inputs.compatibility, None)
                        .map_err(failure)?;
                    metadata.store.validate_complete().map_err(failure)?;
                    StdlibOwner::Metadata(
                        metadata::reader::Provider::new(metadata).map_err(failure)?,
                    )
                } else {
                    let checked = crate::stdlib::check_sources(&inputs.sources)?;
                    let current = metadata::inputs::Inputs::capture(
                        &self.identity,
                        &root.root,
                        metadata::installed::target(),
                    )
                    .map_err(failure)?;
                    if current.compatibility != inputs.compatibility {
                        return Err(failure(
                            "stdlib inputs changed during semantic preparation; retry preparation",
                        ));
                    }
                    StdlibOwner::Source(checked)
                }
            };
            *owner = Some(Arc::new(selected));
        }
        match owner.as_deref() {
            Some(StdlibOwner::Source(checked)) => Ok(checked.defs.clone()),
            Some(StdlibOwner::Metadata(provider)) => {
                let mut defs = sifr_lowering::ExternalDefs::default();
                defs.provider = Some(provider.clone());
                Ok(defs)
            }
            None => Err(failure(
                "semantic stdlib preparation did not produce an owner",
            )),
        }
    }
}
fn failure(error: impl std::fmt::Display) -> Vec<RenderedDiagnostic> {
    vec![diagnostic_with_code(
        error.to_string(),
        DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
    )]
}
