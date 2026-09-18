//! Module-result transport uses the existing bounded indexed codec. This is
//! deliberately not a project store, generation protocol or semantic cache.
use super::{Encode, Encoder, Result, SourceScope, wire};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_frontend::persistence::{CheckedModule, SemanticInterface};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub struct ProjectModuleInput<'a> {
    pub source: &'a sifr_frontend::persistence::CapturedSource,
    pub input_identity: String,
    pub hir: &'a sifr_ir::HirModule,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectInterfacePayload {
    pub semantic: wire::Ref<wire::SemanticExports>,
    /// Full typed body inventory is deliberately conservative: generic/const
    /// bodies, defaults and inferred ownership/effects cannot be omitted.
    pub bodies: wire::Ref<wire::HirModule>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectModuleReferences {
    pub module: wire::Ref<wire::Module>,
    pub interface: SemanticInterface<ProjectInterfacePayload>,
    pub checked: CheckedModule<wire::Ref<wire::HirModule>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectTypedArtifact {
    pub sources: BTreeMap<wire::Ref<wire::SourceFile>, sifr_frontend::persistence::CapturedSource>,
    pub modules: BTreeMap<String, ProjectModuleReferences>,
    pub bytes: Vec<u8>,
}
/// Package identity must include the package/version/source owner from resolution;
/// two same-named modules from different packages cannot share declaration IDs.
pub fn encode_project_results(
    package_identity: &str,
    modules: &BTreeMap<String, ProjectModuleInput<'_>>,
    defs: &sifr_lowering::ExternalDefs,
    compatibility: wire::Compatibility,
) -> Result<ProjectTypedArtifact> {
    let mut records = wire::MetadataEncoder::new(compatibility, wire::Limits::default());
    let name = records.intern(&wire::Text {
        value: package_identity.into(),
    })?;
    let version = records.intern(&wire::Text {
        value: "project-result-v1".into(),
    })?;
    let package = records.intern(&wire::Package {
        name,
        version,
        source_identity: Sha256::digest(package_identity.as_bytes()).into(),
    })?;
    let mut scopes = BTreeMap::new();
    let mut sources = BTreeMap::new();
    for (name, input) in modules {
        let module = wire::Ref::anchor(&[&package.id(), name.as_bytes()]);
        let source = records.intern(&wire::SourceFile {
            // The indexed container owns artifact-relative source IDs, while
            // the explicit table below preserves the resolver's logical paths.
            relative_path: format!("project/{}.sifr", input.source.identity()),
            content_digest: Sha256::digest(input.source.text.as_bytes()).into(),
            byte_length: u32::try_from(input.source.text.len())
                .map_err(|_| wire::MetadataError("source exceeds transport limit".into()))?,
        })?;
        sources.insert(source, input.source.clone());
        let locations = super::locations::declarations(&input.source.text)?;
        let mut kinds = BTreeMap::new();
        if let Some(classes) = defs.classes.get(name) {
            for (name, ty) in classes {
                kinds.insert(name.clone(), super::type_kind(ty));
            }
        }
        for function in &input.hir.functions {
            kinds.insert(function.name.clone(), wire::DeclarationKind::Function);
        }
        for class in &input.hir.classes {
            kinds.insert(class.name.clone(), super::class_kind(class));
            for method in &class.methods {
                kinds.insert(
                    format!("{}.{}", class.name, method.name),
                    wire::DeclarationKind::Function,
                );
            }
        }
        for (name, _, _) in &input.hir.constants {
            kinds.insert(name.clone(), wire::DeclarationKind::Constant);
        }
        scopes.insert(
            name.clone(),
            SourceScope {
                module,
                source,
                kinds,
                locations: locations.locations,
                parameters: locations.parameters,
            },
        );
    }
    let scopes = Arc::new(scopes);
    let mut references = BTreeMap::new();
    for (name, input) in modules {
        let scope = &scopes[name];
        let module = scope.module;
        let source = scope.source;
        let binder = wire::Ref::anchor(&[&module.id(), b"module-binder"]);
        let mut cx = Encoder {
            compatibility,
            scopes: scopes.clone(),
            module_name: name.clone(),
            records,
            package,
            source,
            owner: String::new(),
            binder,
            parameters: scope
                .parameters
                .iter()
                .enumerate()
                .map(|(slot, name)| {
                    Ok((
                        name.clone(),
                        binder,
                        u32::try_from(slot)
                            .map_err(|_| wire::MetadataError("generic parameter limit".into()))?,
                    ))
                })
                .collect::<Result<_>>()?,
            origins: BTreeMap::new(),
        };
        let declaration = cx.declaration("<module>", wire::DeclarationKind::Metadata)?;
        let parameters = scope
            .parameters
            .iter()
            .map(|name| Ok((name.encode(&mut cx)?, Vec::new())))
            .collect::<Result<_>>()?;
        cx.records.insert(
            binder,
            &wire::Binder {
                declaration,
                nested_path: Vec::new(),
                parameters,
            },
        )?;
        let hir_inventory = input.hir.encode(&mut cx)?;
        let semantic = super::semantic::project(defs, name, &mut cx)?;
        let mut declarations = BTreeMap::new();
        for (name, kind) in &scope.kinds {
            declarations.insert(name.encode(&mut cx)?, cx.declaration(name, kind.clone())?);
        }
        let dependencies: BTreeSet<_> = input
            .hir
            .imports
            .iter()
            .filter_map(|import| scopes.get(&import.module).map(|scope| scope.module))
            .collect();
        let interop = cx.records.intern(&wire::InteropSummary {
            module,
            semantic_dependencies: dependencies.clone(),
            codegen_dependencies: dependencies.clone(),
            body_dependencies: declarations.values().copied().collect(),
            intrinsics: BTreeSet::new(),
            rust_declarations: Vec::new(),
            python_declarations: Vec::new(),
            required_features: BTreeSet::new(),
            required_support: BTreeSet::new(),
        })?;
        let module_name = name.encode(&mut cx)?;
        cx.records.insert(
            module,
            &wire::Module {
                package,
                name: module_name,
                private: false,
                declarations,
                exports: BTreeMap::new(),
                hir_inventory,
                source,
                dependencies,
                semantic,
                rust: None,
                templates: Vec::new(),
                interop,
            },
        )?;
        references.insert(
            name.clone(),
            ProjectModuleReferences {
                module,
                interface: SemanticInterface {
                    module_identity: sifr_frontend::persistence::identity(
                        "resolved-module-v1",
                        &(package_identity, name),
                    )
                    .map_err(|error| wire::MetadataError(error.to_string()))?,
                    input_identity: input.input_identity.clone(),
                    exports: ProjectInterfacePayload {
                        semantic,
                        bodies: hir_inventory,
                    },
                },
                checked: CheckedModule {
                    module_identity: sifr_frontend::persistence::identity(
                        "resolved-module-v1",
                        &(package_identity, name),
                    )
                    .map_err(|error| wire::MetadataError(error.to_string()))?,
                    input_identity: input.input_identity.clone(),
                    hir: hir_inventory,
                },
            },
        );
        records = cx.records;
    }
    Ok(ProjectTypedArtifact {
        sources,
        modules: references,
        bytes: records.finish()?,
    })
}

#[cfg(test)]
mod tests;
