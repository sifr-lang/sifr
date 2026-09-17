//! Explicit full-inventory qualification; ordinary consumers stay demand loaded.
use super::{Provider, Result, wire};
use sifr_sysroot::ResolvedSysroot;

impl Provider {
    pub(crate) fn qualify(&self, root: &ResolvedSysroot) -> Result<serde_json::Value> {
        self.metadata.store.validate_complete()?;
        let sources = sifr_stdlib_manifest::load_stdlib_tooling_sources_from_sysroot(root)
            .map_err(|error| wire::MetadataError(error.to_string()))?;
        let expected: std::collections::BTreeSet<_> = sources
            .iter()
            .map(|source| source.module.as_str())
            .collect();
        let actual = self.modules.keys().map(String::as_str).collect();
        if expected != actual {
            return Err(wire::MetadataError(
                "metadata module inventory differs from canonical source inventory".into(),
            ));
        }
        let mut modules = Vec::new();
        for source in sources {
            let module = self.metadata.store.get(self.modules[&source.module])?;
            let record = self.metadata.store.get(module.source)?;
            if sifr_sysroot::sha256_hex(source.source.as_bytes()) != hex(&record.content_digest)
                || source.source.len() != record.byte_length as usize
            {
                return Err(wire::MetadataError(format!(
                    "metadata source identity differs for {}",
                    source.module
                )));
            }
            // Projects all semantic/HIR/template/Rust payloads through the real reader.
            let compiled = self.materialize(std::slice::from_ref(&source.module), root)?;
            modules.push(serde_json::json!({
                "module": source.module, "private": module.private,
                "source": record.relative_path, "source_sha256":hex(&record.content_digest),
                "exports":module.exports.len(), "templates":module.templates.len(),
                "rust_payload":module.rust.is_some(),
                "projected_modules":compiled.code.module_rust_code.len()
            }));
        }
        Ok(serde_json::json!({
            "metadata_id":self.metadata.metadata_id, "metadata_path":self.metadata.path,
            "compiler_identity":hex(&self.metadata.compatibility.compiler),
            "semantic_target_id":hex(&self.metadata.compatibility.semantic_target),
            "stdlib_inputs_id":hex(&self.metadata.compatibility.stdlib_inputs),
            "structural_status":"ok", "behavioral_coverage":"separately-qualified",
            "portable_payload_sha256":self.metadata.store.portable_payload_digest()?,
            "portable_scope":"all records; compiler header and FragmentValidation.compiler_identity excluded",
            "modules":modules
        }))
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
impl crate::CompilerContext {
    /// Traverse every canonical module explicitly, including sources and codegen payloads.
    pub fn qualify_metadata(&self) -> std::result::Result<serde_json::Value, String> {
        let errors = |values: Vec<crate::diagnostics::RenderedDiagnostic>| {
            values
                .into_iter()
                .map(|value| value.message)
                .collect::<Vec<_>>()
                .join("\n")
        };
        self.metadata_provider()
            .map_err(errors)?
            .qualify(self.sysroot().map_err(errors)?)
            .map_err(|error| error.to_string())
    }
}

impl crate::CompilerContext {
    /// Inspect selected metadata without repairing or replacing development artifacts.
    pub fn inspect_metadata(
        &self,
        complete: bool,
    ) -> std::result::Result<serde_json::Value, String> {
        let errors = |values: Vec<crate::diagnostics::RenderedDiagnostic>| {
            values
                .into_iter()
                .map(|value| value.message)
                .collect::<Vec<_>>()
                .join("\n")
        };
        let root = self.sysroot().map_err(errors)?;
        let override_path = if root.mode() == sifr_sysroot::SysrootMode::SourceTreeDevelopment {
            Some(
                crate::metadata_producer::development_metadata_path(
                    self.identity(),
                    &root.root,
                    super::selection::target(),
                    &crate::cache_storage::root(),
                )
                .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        let provider =
            super::select(self.identity(), root, override_path.as_deref()).map_err(|error| {
                let remedy = if root.mode() == sifr_sysroot::SysrootMode::SourceTreeDevelopment {
                    format!(
                        "run sifr sysroot build-metadata --source-root {} --output <file> to explicitly ensure the selected development cache",
                        root.root.display()
                    )
                } else {
                    "select or reinstall a matching Sifr toolchain".into()
                };
                format!(
                    "{error}; expected compiler {}; selected sysroot {}; {remedy}",
                    self.identity().as_str(),
                    root.root.display()
                )
            })?;
        if complete {
            return provider.qualify(root).map_err(|error| error.to_string());
        }
        Ok(
            serde_json::json!({"metadata_id":provider.metadata.metadata_id,
            "path":provider.metadata.path,"compiler_identity":hex(&provider.metadata.compatibility.compiler),
            "semantic_target_id":hex(&provider.metadata.compatibility.semantic_target),
            "stdlib_inputs_id":hex(&provider.metadata.compatibility.stdlib_inputs),
            "status":"ready", "integrity":"consumed-records"}),
        )
    }
}

/// Qualify an explicit host-produced artifact for a declared semantic target.
/// This never executes a foreign target binary or changes the selected artifact.
pub fn qualify_development_metadata(
    identity: &sifr_identity::CompilerIdentity,
    source_root: &std::path::Path,
    metadata: &std::path::Path,
    target: &str,
) -> std::result::Result<serde_json::Value, String> {
    let prepared = crate::metadata_producer::validate_development_metadata(
        identity,
        source_root,
        target,
        metadata,
    )
    .map_err(|e| e.to_string())?;
    let root = sifr_sysroot::resolve_sysroot(Some(source_root.to_owned()))
        .map_err(|e| e.boundary_message())?;
    Provider::new(prepared)
        .and_then(|p| p.qualify(&root))
        .map_err(|e| e.to_string())
}
