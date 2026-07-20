use super::{
    generate_rust, generate_rust_with_stdlib, module_class_fields,
    publicize_generated_module_source, HashMap, HashSet, HirModule, MultiModuleCodegenResult,
    Renderer, RustFile, RustItem, StdlibCode,
};
use crate::lib_project_signatures::project_func_signatures;
use sifr_stdlib_manifest::{try_generated_cargo_dependencies, StdlibFeature};
pub(super) fn render_local_module_imports(module: &HirModule) -> String {
    let mut module_import_items: Vec<RustItem> = Vec::new();
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
        let local_imports = render_local_module_imports(module);
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
