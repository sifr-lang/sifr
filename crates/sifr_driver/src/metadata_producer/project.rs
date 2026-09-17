use super::{Encode, Encoder, Result, wire};
use sha2::{Digest, Sha256};
use sifr_stdlib_manifest::LoadedStdlibSource;
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn project(
    compiled: &crate::stdlib::StdlibCompiled,
    sources: &[LoadedStdlibSource],
    compatibility: wire::Compatibility,
    root: &std::path::Path,
) -> Result<Vec<u8>> {
    let mut records = wire::MetadataEncoder::new(compatibility, wire::Limits::default());
    let name = records.intern(&wire::Text {
        value: "sifr-stdlib".into(),
    })?;
    let version = records.intern(&wire::Text {
        value: env!("CARGO_PKG_VERSION").into(),
    })?;
    let package = records.intern(&wire::Package {
        name,
        version,
        source_identity: compatibility.stdlib_inputs,
    })?;
    let modules = sources
        .iter()
        .map(|s| {
            (
                s.module.clone(),
                wire::Ref::<wire::Module>::anchor(&[&package.id(), s.module.as_bytes()]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut scopes = BTreeMap::new();
    for source in sources {
        let relative = source
            .path
            .strip_prefix(root)
            .map_err(|error| wire::MetadataError(error.to_string()))?;
        let source_file = records.intern(&wire::SourceFile {
            relative_path: relative.to_string_lossy().into_owned(),
            content_digest: Sha256::digest(source.source.as_bytes()).into(),
            byte_length: u32::try_from(source.source.len())
                .map_err(|_| wire::MetadataError("source exceeds wire range".into()))?,
        })?;
        let hir = &compiled.code.hir_modules[&source.module];
        let mut kinds = BTreeMap::new();
        if let Some(classes) = compiled.defs.classes.get(&source.module) {
            for (name, ty) in classes {
                kinds.insert(name.clone(), super::type_kind(ty));
            }
        }
        for function in &hir.functions {
            kinds
                .entry(function.name.clone())
                .or_insert(wire::DeclarationKind::Function);
        }
        for class in &hir.classes {
            kinds.insert(class.name.clone(), super::class_kind(class));
            for method in &class.methods {
                kinds.insert(
                    format!("{}.{}", class.name, method.name),
                    wire::DeclarationKind::Function,
                );
            }
        }
        for (name, _, _) in &hir.constants {
            kinds.insert(name.clone(), wire::DeclarationKind::Constant);
        }
        let declared = super::locations::declarations(&source.source)?;
        scopes.insert(
            source.module.clone(),
            super::SourceScope {
                parameters: declared.parameters,
                kinds,
                module: modules[&source.module],
                source: source_file,
                locations: declared.locations,
            },
        );
    }
    let scopes = std::sync::Arc::new(scopes);
    for source in sources {
        let module = modules[&source.module];
        let source_file = scopes[&source.module].source;
        let binder = wire::Ref::anchor(&[&module.id(), b"module-binder"]);
        let mut cx = Encoder {
            compatibility,
            scopes: scopes.clone(),
            module_name: source.module.clone(),
            records,
            package,
            source: source_file,
            owner: String::new(),
            binder,
            parameters: scopes[&source.module]
                .parameters
                .iter()
                .enumerate()
                .map(|(slot, name)| {
                    Ok((
                        name.clone(),
                        binder,
                        u32::try_from(slot).map_err(|_| {
                            wire::MetadataError("too many module generic parameters".into())
                        })?,
                    ))
                })
                .collect::<Result<_>>()?,
            origins: BTreeMap::new(),
        };
        let declaration = cx.declaration("<module>", wire::DeclarationKind::Metadata)?;
        let module_parameters = scopes[&source.module]
            .parameters
            .iter()
            .map(|name| Ok((name.encode(&mut cx)?, Vec::new())))
            .collect::<Result<_>>()?;
        cx.records.insert(
            binder,
            &wire::Binder {
                declaration,
                nested_path: Vec::new(),
                parameters: module_parameters,
            },
        )?;
        let hir = &compiled.code.hir_modules[&source.module];
        let hir_inventory = hir.encode(&mut cx)?;
        let semantic = super::semantic::project(&compiled.defs, &source.module, &mut cx)?;
        let mut declarations = BTreeMap::new();
        for function in &hir.functions {
            let name = function.name.encode(&mut cx)?;
            let declaration = cx.declaration(&function.name, wire::DeclarationKind::Function)?;
            declarations.insert(name, declaration);
        }
        for class in &hir.classes {
            let name = class.name.encode(&mut cx)?;
            let kind = if class.newtype_inner.is_some() {
                wire::DeclarationKind::Newtype
            } else if class.is_enum() {
                wire::DeclarationKind::Enum
            } else if class.is_protocol() {
                wire::DeclarationKind::Protocol
            } else {
                wire::DeclarationKind::Class
            };
            let declaration = cx.declaration(&class.name, kind)?;
            declarations.insert(name, declaration);
        }
        for (name, _, _) in &hir.constants {
            let declaration = cx.declaration(name, wire::DeclarationKind::Constant)?;
            declarations.insert(name.encode(&mut cx)?, declaration);
        }
        let exports =
            super::exports::project(compiled, &source.module, &mut declarations, &mut cx)?;
        let rust_names = declarations
            .iter()
            .chain(exports.iter())
            .map(|(name, declaration)| (*name, *declaration))
            .collect();
        let (rust, required_support) =
            super::rust_payload::project(compiled, &source.module, module, &rust_names, &mut cx)?;
        let mut templates = Vec::new();
        for function in &hir.functions {
            if function.type_params.is_empty() {
                continue;
            }
            let owner = cx.declaration(&function.name, wire::DeclarationKind::Function)?;
            let role = cx.records.intern(&wire::TemplateRole::GenericFunction)?;
            let function = Some(function.encode(&mut cx)?);
            templates.push(cx.records.intern(&wire::TemplatePayload {
                owner,
                role,
                function,
                class: None,
            })?);
        }
        for class in &hir.classes {
            let owner = cx.declaration(&class.name, super::class_kind(class))?;
            let role = cx.records.intern(&if class.type_params.is_empty() {
                wire::TemplateRole::ProjectPolicyClass
            } else {
                wire::TemplateRole::GenericClass
            })?;
            let class = Some(class.encode(&mut cx)?);
            templates.push(cx.records.intern(&wire::TemplatePayload {
                owner,
                role,
                function: None,
                class,
            })?);
        }
        let dependencies = hir
            .imports
            .iter()
            .filter_map(|i| modules.get(&i.module).copied())
            .collect::<BTreeSet<_>>();
        let rust_declarations = hir
            .functions
            .iter()
            .flat_map(|f| &f.rust_interop)
            .chain(hir.classes.iter().flat_map(|c| &c.rust_interop))
            .map(|r| r.encode(&mut cx))
            .collect::<Result<_>>()?;
        let python_declarations = hir
            .functions
            .iter()
            .flat_map(|f| &f.python_interop)
            .map(|r| r.encode(&mut cx))
            .collect::<Result<_>>()?;
        let intrinsics = hir
            .functions
            .iter()
            .filter_map(|f| f.compiler_intrinsic)
            .map(|v| v.encode(&mut cx))
            .collect::<Result<_>>()?;
        let required_features = compiled
            .metadata_features
            .get(&source.module)
            .into_iter()
            .flatten()
            .map(|feature| feature.id().encode(&mut cx))
            .collect::<Result<_>>()?;
        let interop = cx.records.intern(&wire::InteropSummary {
            module,
            semantic_dependencies: dependencies.clone(),
            codegen_dependencies: dependencies.clone(),
            body_dependencies: declarations
                .values()
                .chain(exports.values())
                .copied()
                .collect(),
            intrinsics,
            rust_declarations,
            python_declarations,
            required_features,
            required_support,
        })?;
        let name = source.module.encode(&mut cx)?;
        cx.records.insert(
            module,
            &wire::Module {
                package,
                name,
                private: source.module.starts_with("_sifr."),
                exports,
                declarations,
                hir_inventory,
                source: source_file,
                dependencies,
                semantic,
                rust,
                templates,
                interop,
            },
        )?;
        records = cx.records;
    }
    records.finish()
}
