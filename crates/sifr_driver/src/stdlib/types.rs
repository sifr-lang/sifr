use sifr_codegen::StdlibCode;
use sifr_lowering::ExternalDefs;
use std::collections::HashMap;

/// Completed bootstrap owner, shared immutably by compilation consumers.
/// Mutable lowering receives its own definitions projection.
pub(crate) struct StdlibCompiled {
    pub(crate) provider: Option<std::sync::Arc<crate::metadata_reader::Provider>>,
    pub(crate) defs: ExternalDefs,
    #[allow(dead_code)] // Retained for test-only source bootstrap until C02a1 moves the reader.
    pub(crate) metadata_features:
        HashMap<String, std::collections::BTreeSet<sifr_stdlib_manifest::StdlibFeature>>,
    pub(crate) code: StdlibCode,
    pub(crate) interop: StdlibRustInterop,
}

pub(crate) use sifr_compiler_services::stdlib::{StdlibRustInterop, StdlibRustInteropModuleSource};

impl StdlibCompiled {
    pub(crate) fn for_codegen<'a>(
        self: &std::sync::Arc<Self>,
        modules: impl IntoIterator<Item = &'a sifr_ir::HirModule>,
    ) -> Result<std::sync::Arc<Self>, Vec<crate::diagnostics::RenderedDiagnostic>> {
        let Some(provider) = &self.provider else {
            return Ok(self.clone());
        };
        let requested = modules
            .into_iter()
            .flat_map(sifr_codegen::stdlib_module_roots)
            .collect::<Vec<_>>();
        let sysroot = self.interop.sysroot.as_ref().ok_or_else(|| {
            vec![crate::diagnostics::diagnostic_with_code(
                "metadata codegen requires a pinned sysroot",
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })?;
        provider
            .materialize(&requested, sysroot)
            .map(std::sync::Arc::new)
            .map_err(|error| {
                vec![crate::diagnostics::diagnostic_with_code(
                    error.to_string(),
                    sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
                )]
            })
    }
}
