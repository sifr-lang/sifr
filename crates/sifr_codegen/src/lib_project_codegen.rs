use super::{
    generate_rust, generate_rust_with_stdlib, module_class_fields,
    publicize_generated_module_source, HashMap, HashSet, HirModule, MultiModuleCodegenResult,
    Renderer, RustFile, RustItem, StdlibCode,
};
use crate::lib_project_signatures::project_func_signatures;
use sifr_stdlib_manifest::{try_generated_cargo_dependencies, StdlibFeature};
use sifr_type_system::source_class_rust_name;

fn resolve_exported_rust_opaque_class<'a>(
    module_name: &str,
    export_name: &str,
    project_modules: &HashMap<&str, &'a HirModule>,
    visiting: &mut HashSet<(String, String)>,
) -> Option<(String, &'a sifr_ir::HirClass)> {
    if !visiting.insert((module_name.to_string(), export_name.to_string())) {
        return None;
    }
    let module = project_modules.get(module_name)?;
    if let Some(class) = module.classes.iter().find(|class| {
        class.name == export_name
            && class
                .rust_interop
                .iter()
                .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque)
            && class
                .methods
                .iter()
                .any(|method| !method.rust_interop.is_empty())
    }) {
        return Some((module_name.to_string(), class));
    }
    for import in &module.imports {
        for source_name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(source, _)| source == source_name)
                .map_or(source_name.as_str(), |(_, local)| local.as_str());
            if local_name == export_name {
                if let Some(resolved) = resolve_exported_rust_opaque_class(
                    &import.module,
                    source_name,
                    project_modules,
                    visiting,
                ) {
                    return Some(resolved);
                }
            }
        }
    }
    None
}

pub(super) fn render_local_module_imports(
    module: &HirModule,
    project_modules: &HashMap<&str, &HirModule>,
) -> String {
    let mut module_import_items: Vec<RustItem> = Vec::new();
    let mut imported_opaque_traits = HashSet::new();
    for import in &module.imports {
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        let mut module_path = vec!["crate".to_string()];
        module_path.extend(import.module.split('.').map(str::to_string));
        for name in &import.names {
            if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                let mut alias_path = module_path.clone();
                alias_path.push(name.clone());
                module_import_items.push(RustItem::UseAlias {
                    path: alias_path,
                    alias: alias.clone(),
                });
            } else {
                let mut import_path = module_path.clone();
                import_path.push(name.clone());
                module_import_items.push(RustItem::Use(import_path));
            }
            if let Some((owner_module, class)) = resolve_exported_rust_opaque_class(
                &import.module,
                name,
                project_modules,
                &mut HashSet::new(),
            ) {
                let trait_name =
                    format!("__SifrOpaque{}Methods", source_class_rust_name(&class.name));
                if imported_opaque_traits.insert((owner_module.clone(), trait_name.clone())) {
                    let mut trait_path = vec!["crate".to_string()];
                    trait_path.extend(owner_module.split('.').map(str::to_string));
                    trait_path.push(trait_name);
                    module_import_items.push(RustItem::UseAlias {
                        path: trait_path,
                        alias: "_".to_string(),
                    });
                }
            }
        }
    }

    if module_import_items.is_empty() {
        String::new()
    } else {
        Renderer::new().render_file(&RustFile {
            items: module_import_items,
        })
    }
}

fn resolve_exported_generic_class<'a>(
    module_name: &str,
    export_name: &str,
    project_modules: &HashMap<&str, &'a HirModule>,
    visiting: &mut HashSet<(String, String)>,
) -> Option<&'a sifr_ir::HirClass> {
    if !visiting.insert((module_name.to_string(), export_name.to_string())) {
        return None;
    }
    let module = project_modules.get(module_name)?;
    if let Some(class) = module
        .classes
        .iter()
        .find(|class| class.name == export_name && !class.type_params.is_empty())
    {
        return Some(class);
    }
    for import in &module.imports {
        for source_name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(source, _)| source == source_name)
                .map_or(source_name.as_str(), |(_, local)| local.as_str());
            if local_name == export_name {
                if let Some(class) = resolve_exported_generic_class(
                    &import.module,
                    source_name,
                    project_modules,
                    visiting,
                ) {
                    return Some(class);
                }
            }
        }
    }
    None
}

fn register_imported_generic_classes(
    code: &mut StdlibCode,
    module: &HirModule,
    project_modules: &HashMap<&str, &HirModule>,
) {
    for import in &module.imports {
        for source_name in &import.names {
            let Some(source_class) = resolve_exported_generic_class(
                &import.module,
                source_name,
                project_modules,
                &mut HashSet::new(),
            ) else {
                continue;
            };
            let local_name = import
                .aliases
                .iter()
                .find(|(original, _)| original == source_name)
                .map_or(source_name.as_str(), |(_, alias)| alias.as_str());
            let mut template = source_class.clone();
            template.name = local_name.to_string();
            code.generic_classes.insert(local_name.to_string());
            code.generic_class_params
                .insert(local_name.to_string(), source_class.type_params.clone());
            code.generic_class_templates
                .insert(local_name.to_string(), template);
        }
    }
}

/// Generate Rust source code for a multi-module project, returning aggregate dependency metadata.
pub fn generate_rust_multi_with_metadata(
    modules: &[(&str, &HirModule)],
    stdlib_code: &StdlibCode,
) -> MultiModuleCodegenResult {
    let mut files = HashMap::new();
    let mut used_stdlib_modules = HashSet::new();
    let mut required_features = HashSet::new();
    let mut project_codegen_code = stdlib_code.clone();
    let project_modules = modules.iter().copied().collect::<HashMap<_, _>>();

    project_codegen_code
        .func_signatures
        .extend(project_func_signatures(modules));
    for (module_name, module) in modules {
        project_codegen_code
            .module_class_fields
            .insert((*module_name).to_string(), module_class_fields(module));
    }

    for (module_name, module) in modules {
        let module_public = *module_name != "main";
        let mut module_codegen_code = project_codegen_code.clone();
        register_imported_generic_classes(&mut module_codegen_code, module, &project_modules);
        let codegen_result = generate_rust_with_stdlib(module, &module_codegen_code);
        let local_imports = render_local_module_imports(module, &project_modules);
        let mut rust_source = codegen_result.rust_source;
        if !local_imports.trim().is_empty() {
            rust_source = format!("{}\n\n{}", local_imports.trim_end(), rust_source);
        }
        if module_public {
            rust_source = publicize_generated_module_source(&rust_source);
        }
        if rust_source.contains("::sifr_stdlib::fs::") {
            required_features.insert(StdlibFeature::Fs);
        }

        files.insert((*module_name).to_string(), rust_source);
        used_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        required_features.extend(codegen_result.required_features);
    }

    MultiModuleCodegenResult {
        rust_files: files,
        used_stdlib_modules,
        required_features,
        interop: crate::rust_interop_plan::interop_build_plan_for_named_modules(
            modules.iter().map(|(name, module)| (Some(*name), *module)),
        ),
    }
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    generate_rust_multi_with_metadata(modules, &StdlibCode::default())
        .rust_files
        .into_iter()
        .collect()
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    generate_project_with_deps(module, project_name, &HashSet::new())
}

/// Generate a complete Rust project with stdlib dependencies.
pub fn generate_project_with_deps(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
) -> (String, String) {
    generate_project_with_deps_and_crates(module, project_name, stdlib_modules, &HashSet::new())
}

/// Generate a complete Rust project with stdlib and explicit crate dependencies.
#[allow(
    clippy::expect_used,
    reason = "legacy codegen project helper has a tuple return type; driver build paths use fallible sysroot planning"
)]
pub fn generate_project_with_deps_and_crates(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_features: &HashSet<StdlibFeature>,
) -> (String, String) {
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[workspace]
"#
    );

    let deps = try_generated_cargo_dependencies(stdlib_modules, required_features)
        .expect("legacy project generation should resolve the Sifr sysroot");

    if !deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    let main_rs = generate_rust(module);
    (cargo_toml, main_rs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruff_text_size::TextRange;
    use sifr_ir::{
        HirClass, HirClassKind, HirFunction, HirImport, MethodKind, RustInteropAbiRequirements,
        RustInteropDeclaration, RustInteropDecoratorKind, RustInteropEffect,
    };

    #[test]
    fn local_imports_bring_opaque_extension_traits_into_scope() {
        let declaration = |kind| RustInteropDeclaration {
            kind,
            target: None,
            arguments: Vec::new(),
            span: TextRange::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        };
        let provider = HirModule {
            functions: Vec::new(),
            classes: vec![HirClass {
                name: "Resource".to_string(),
                identity: None,
                fields: Vec::new(),
                methods: vec![HirFunction {
                    name: "close".to_string(),
                    params: Vec::new(),
                    return_type: sifr_type_system::Type::None,
                    body: Vec::new(),
                    is_async: false,
                    method_kind: MethodKind::Regular,
                    receiver: None,
                    decorators: Vec::new(),
                    rust_interop: vec![declaration(RustInteropDecoratorKind::Function)],
                    python_interop: Vec::new(),
                    compiler_intrinsic: None,
                    type_params: Vec::new(),
                }],
                is_hashable: false,
                is_error_type: false,
                kind: HirClassKind::Regular,
                operator_impls: Vec::new(),
                newtype_inner: None,
                implements_protocols: Vec::new(),
                parent_class: None,
                parent_type: None,
                type_params: Vec::new(),
                enum_variants: Vec::new(),
                rust_interop: vec![declaration(RustInteropDecoratorKind::Opaque)],
            }],
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let consumer = HirModule {
            functions: Vec::new(),
            classes: Vec::new(),
            imports: vec![HirImport {
                module: "resources".to_string(),
                names: vec!["Resource".to_string()],
                aliases: Vec::new(),
            }],
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        };
        let modules = HashMap::from([("resources", &provider), ("main", &consumer)]);

        let imports = render_local_module_imports(&consumer, &modules);

        assert!(
            imports.contains("use crate::resources::Resource;"),
            "{imports}"
        );
        assert!(
            imports.contains("use crate::resources::__SifrOpaqueResourceMethods as _;"),
            "{imports}"
        );
    }
}
