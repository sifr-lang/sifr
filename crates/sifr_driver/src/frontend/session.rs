//! Reusable immutable setup for one single-file compilation invocation.
use crate::diagnostics::{CompileResultFull, RenderedDiagnostic};

/// Owns one metadata generation and native-toolchain selection. Sources may
/// differ, but sysroot files, tool binaries, environment, cwd and configuration
/// must remain unchanged while this session is alive. Construct a new context
/// and session for a changed input snapshot; do not persist this across edits.
/// Mutable parsing, lowering and code generation remain per source. `check`
/// keeps its separate frontend-only path and never constructs this session.
pub struct SingleFileCompilationSession {
    compiler: crate::CompilerContext,
    interop: crate::build::SingleFileInteropSession,
}

impl SingleFileCompilationSession {
    pub fn new(compiler: crate::CompilerContext) -> Result<Self, Vec<RenderedDiagnostic>> {
        // A caller may pass a cloned context from an earlier invocation. Give
        // this session its own provider owner while preserving selected identity.
        let compiler = compiler.without_cached_metadata();
        crate::stdlib::compile_stdlib(&compiler)?;
        let native_toolchain = crate::build::CargoResolutionPolicy::resolve_native_toolchain();
        Ok(Self {
            compiler,
            interop: crate::build::SingleFileInteropSession::new(native_toolchain),
        })
    }

    pub fn compile_with_metadata(&self, source: &str) -> CompileResultFull {
        match crate::build::compile_single_file_with_prepared_native(
            &self.compiler,
            source,
            sifr_lowering::LoweringOptions::default(),
            Some(&self.interop),
        ) {
            Ok(metadata) => super::api::metadata_result(metadata),
            Err(errors) => CompileResultFull::Errors { errors },
        }
    }
}
