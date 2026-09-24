use super::{Decode, Decoder, Provider, Result, wire};
use crate::stdlib::{SourceStdlibCompiled, StdlibRustInterop};
use sha2::{Digest, Sha256};
use sifr_codegen::{StdlibCode, StdlibRustSource};
use sifr_sysroot::ResolvedSysroot;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    io::Read,
    sync::Arc,
};
impl Provider {
    pub fn materialize(
        &self,
        requested: &[String],
        sysroot: &ResolvedSysroot,
    ) -> Result<SourceStdlibCompiled> {
        let mut code = StdlibCode::default();
        let mut defs = sifr_lowering::ExternalDefs::default();
        let mut metadata_features = HashMap::new();
        let mut hir_modules = BTreeMap::new();
        let mut module_sources = HashMap::new();
        let closure = self.closure(requested)?;
        for name in &closure {
            let module = self.metadata.store.get(self.modules[name])?;
            let mut cx = Decoder::with_nominals(&self.metadata.store, &self.nominals);
            let hir = Arc::<sifr_ir::HirModule>::decode(&module.hir_inventory, &mut cx)?;
            super::templates::validate(&self.metadata.store, &module)?;
            let semantics = self.semantic(name)?;
            defs.extend_baseline(&semantics);
            let interop = self.metadata.store.get(module.interop)?;
            let features: HashSet<String> = Decode::decode(&interop.required_features, &mut cx)?;
            let features = features
                .into_iter()
                .map(|name| {
                    sifr_stdlib_manifest::StdlibFeature::ALL
                        .iter()
                        .copied()
                        .find(|feature| feature.id() == name)
                        .ok_or_else(|| {
                            wire::MetadataError(format!("unknown metadata feature {name}"))
                        })
                })
                .collect::<Result<_>>()?;
            metadata_features.insert(name.clone(), features);
            let mut deps = self.closure(std::slice::from_ref(name))?;
            deps.remove(name);
            if !deps.is_empty() {
                code.transitive_deps
                    .insert(name.clone(), deps.into_iter().collect());
            }
            if let Some(reference) = module.rust {
                let payload = self.metadata.store.get(reference)?;
                let validation = self.metadata.store.get(payload.validation)?;
                let rust = String::decode(&payload.source, &mut cx)?;
                let expected = self.metadata.compatibility;
                if validation.bytes_digest != <[u8; 32]>::from(Sha256::digest(rust.as_bytes()))
                    || validation.compiler_identity != expected.compiler
                    || validation.target_identity != expected.semantic_target
                    || validation.grammar_identity
                        != <[u8; 32]>::from(Sha256::digest(b"syn-stdlib-module-v1"))
                    || *self.metadata.store.get(validation.boundary)?
                        != wire::FragmentBoundary::StdlibModule
                {
                    return Err(wire::MetadataError(format!(
                        "invalid reusable Rust validation identity for {name}"
                    )));
                }
                let source = self.metadata.store.get(module.source)?;
                code.module_rust_code.insert(
                    name.clone(),
                    StdlibRustSource {
                        module: name.clone(),
                        source_path: format!("stdlib/{}", source.relative_path),
                        source_sha256: super::qualification::hex(&source.content_digest),
                        nominal_types: hir
                            .classes
                            .iter()
                            .filter(|class| {
                                !class
                                    .rust_interop
                                    .iter()
                                    .any(|d| d.abi_requirements.opaque_handle)
                            })
                            .map(|c| c.name.clone())
                            .collect(),
                        rust,
                    },
                );
                let mut declared = BTreeMap::new();
                for (symbol, decl) in module.declarations.iter().chain(module.exports.iter()) {
                    declared.insert(String::decode(symbol, &mut cx)?, *decl);
                }
                let mut constants = HashMap::new();
                let mut types = semantics.constants.get(name).cloned().unwrap_or_default();
                types.extend(hir.constants.iter().map(|(n, t, _)| (n.clone(), t.clone())));
                for (symbol, ty) in types {
                    let declaration = declared.get(&symbol).ok_or_else(|| {
                        wire::MetadataError(format!(
                            "constant declaration missing: {name}.{symbol}"
                        ))
                    })?;
                    let rendered = payload.names.get(declaration).ok_or_else(|| {
                        wire::MetadataError(format!(
                            "constant Rust mapping missing: {name}.{symbol}"
                        ))
                    })?;
                    constants.insert(symbol, (ty, String::decode(rendered, &mut cx)?));
                }
                // Codegen also records imported constants that are private to this
                // module. They are absent from its public semantic exports, but
                // their canonical declaration mappings remain in the Rust payload.
                for import in &hir.imports {
                    let Some(reference) = self.modules.get(&import.module) else {
                        continue;
                    };
                    let imported_module = self.metadata.store.get(*reference)?;
                    let imported_semantics = self.semantic(&import.module)?;
                    let Some(imported_constants) = imported_semantics.constants.get(&import.module)
                    else {
                        continue;
                    };
                    for symbol in &import.names {
                        let Some(ty) = imported_constants.get(symbol) else {
                            continue;
                        };
                        let mut declaration = None;
                        for (key, value) in &imported_module.exports {
                            if String::decode(key, &mut cx)? == *symbol {
                                declaration = Some(value);
                                break;
                            }
                        }
                        let Some(rendered) = declaration.and_then(|d| payload.names.get(d)) else {
                            continue;
                        };
                        let local = import
                            .aliases
                            .iter()
                            .find(|(original, _)| original == symbol)
                            .map(|(_, alias)| alias)
                            .unwrap_or(symbol);
                        constants.insert(
                            local.clone(),
                            (ty.clone(), String::decode(rendered, &mut cx)?),
                        );
                    }
                }
                if !constants.is_empty() {
                    code.module_constants.insert(name.clone(), constants);
                }
                let generators: HashSet<String> = Decode::decode(&payload.generators, &mut cx)?;
                if !generators.is_empty() {
                    code.generator_functions.insert(name.clone(), generators);
                }
                super::support_projection::project_signatures(
                    name,
                    &hir,
                    &mut code,
                    module.private,
                );
            }
            if module.private && !interop.rust_declarations.is_empty() {
                let source = self.metadata.store.get(module.source)?;
                let path = sysroot.paths.stdlib_root.join(&source.relative_path);
                let file = std::fs::File::open(&path).map_err(|e| {
                    wire::MetadataError(format!("required interop source {}: {e}", path.display()))
                })?;
                let mut bytes = Vec::new();
                file.take(u64::from(source.byte_length) + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| wire::MetadataError(e.to_string()))?;
                if bytes.len() != source.byte_length as usize
                    || <[u8; 32]>::from(Sha256::digest(&bytes)) != source.content_digest
                {
                    return Err(wire::MetadataError(format!(
                        "installed interop source identity differs: {}",
                        path.display()
                    )));
                }
                let source =
                    String::from_utf8(bytes).map_err(|e| wire::MetadataError(e.to_string()))?;
                module_sources.insert(
                    name.clone(),
                    crate::stdlib::StdlibRustInteropModuleSource {
                        source,
                        display_path: path.display().to_string(),
                    },
                );
            }
            hir_modules.insert(name.clone(), hir);
        }
        let plan = sifr_codegen::interop_build_plan_for_named_modules(
            hir_modules
                .iter()
                .filter(|(name, _)| name.starts_with("_sifr."))
                .map(|(name, hir)| (Some(name.as_str()), hir.as_ref())),
        );
        code.hir_modules = Arc::new(hir_modules);
        Ok(SourceStdlibCompiled {
            defs,
            code,
            metadata_features,
            interop: StdlibRustInterop {
                plan,
                module_sources,
                sysroot: Some(sysroot.clone()),
            },
        })
    }
}
