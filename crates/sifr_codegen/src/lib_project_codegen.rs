use super::{
    HashMap, HashSet, HirModule, ModuleSupportDemand, MultiModuleCodegenResult, Renderer, RustFile,
    RustItem, StdlibCode, SupportEmission, crate_visible_generated_support_source, generate_rust,
    generate_rust_with_stdlib_for_module_with_project_policy, publicize_generated_module_source,
    render_import_items, render_support,
};
use crate::lib_project_signatures::{project_class_fields, project_func_signatures};
use crate::project_stdlib_nominals::{
    extract_project_stdlib_nominal_prelude, project_module_binding_names,
    project_stdlib_nominal_plan, relocate_project_stdlib_nominals,
};
use crate::project_union_prelude::render_project_union_prelude;
use crate::render_project_structural_record_prelude;
use crate::stdlib_filter::rust_source_defined_item_names;
use sifr_stdlib_manifest::{StdlibFeature, try_generated_cargo_dependencies};
use sifr_type_system::source_class_rust_name;

mod imported_unions;
pub(crate) use imported_unions::register_imported_union_types;

pub(crate) struct ProjectUnionUsage {
    pub(crate) unions: HashMap<String, Vec<sifr_type_system::Type>>,
    pub(crate) module_unions: HashMap<String, HashSet<String>>,
    pub(crate) ordinary_unions: HashSet<String>,
    pub(crate) try_error_unions: HashSet<String>,
    pub(crate) structural_unions: HashSet<String>,
}

pub(crate) fn project_union_usage(
    modules: &[(&str, &HirModule)],
    project_code: &crate::StdlibEmissionCode,
    structural_interop_enabled: bool,
) -> ProjectUnionUsage {
    let mut unions = HashMap::new();
    let mut module_unions = HashMap::new();
    let mut ordinary_unions = HashSet::new();
    let mut try_error_unions = HashSet::new();
    let mut structural_unions = HashSet::new();
    for (module_name, module) in modules {
        let mut emitter = super::RustEmitter::new();
        emitter.collect_union_types(module);
        register_imported_union_types(&mut emitter, module, project_code);
        ordinary_unions.extend(emitter.ordinary_union_enums.iter().cloned());
        try_error_unions.extend(emitter.try_error_carrier_enums.iter().cloned());
        let names = emitter.union_enums.keys().cloned().collect::<HashSet<_>>();
        for (name, members) in emitter.union_enums {
            unions.entry(name).or_insert(members);
        }
        module_unions.insert((*module_name).to_string(), names);
    }
    if structural_interop_enabled {
        structural_unions =
            crate::structural_impl_codegen::structural_union_names_for_project(&unions, modules);
    }
    ProjectUnionUsage {
        unions,
        module_unions,
        ordinary_unions,
        try_error_unions,
        structural_unions,
    }
}

pub(crate) fn render_project_union_imports(
    module_name: &str,
    module_unions: &HashSet<String>,
    crate_root_modules: &HashSet<&str>,
) -> String {
    if crate_root_modules.contains(module_name) {
        return String::new();
    }
    let mut names = module_unions.iter().collect::<Vec<_>>();
    names.sort();
    let items = names
        .into_iter()
        .map(|name| RustItem::Use(vec!["crate".to_string(), name.clone()]))
        .collect::<Vec<_>>();
    Renderer::new().render_file(&RustFile { items })
}

pub(crate) fn project_nominal_type_paths(
    modules: &[(&str, &HirModule)],
    crate_root_modules: &HashSet<&str>,
) -> HashMap<String, String> {
    let mut paths = HashMap::new();
    let mut basename_counts = HashMap::new();
    for (_, module) in modules {
        for class in &module.classes {
            *basename_counts
                .entry(class.name.as_str())
                .or_insert(0_usize) += 1;
        }
    }
    for (module_name, module) in modules {
        for class in &module.classes {
            let rust_name = source_class_rust_name(&class.name);
            let path = if crate_root_modules.contains(module_name) {
                format!("crate::{rust_name}")
            } else {
                format!("crate::{}::{rust_name}", module_name.replace('.', "::"))
            };
            let canonical = class
                .identity
                .clone()
                .unwrap_or_else(|| format!("{module_name}.{}", class.name));
            paths.insert(canonical, path.clone());
            paths.insert(format!("{module_name}.{}", class.name), path.clone());
            if basename_counts.get(class.name.as_str()) == Some(&1) {
                paths.insert(class.name.clone(), path);
            }
        }
    }
    paths
}

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
    project_code: &crate::StdlibEmissionCode,
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
            let is_constant = project_code
                .module_constants
                .get(&import.module)
                .is_some_and(|constants| constants.contains_key(name));
            if !is_constant {
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

pub(crate) fn register_imported_generic_classes(
    code: &mut crate::StdlibEmissionCode,
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
                .insert(local_name.to_string(), std::sync::Arc::new(template));
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
    let structural_interop_enabled = modules
        .iter()
        .any(|(_, module)| crate::rust_interop_plan::module_uses_structural_interop(module));
    project_codegen_code
        .func_signatures
        .extend(project_func_signatures(modules));
    project_codegen_code
        .module_class_fields
        .extend(project_class_fields(modules));
    let crate_root_modules = HashSet::from(["main"]);
    crate::project_constants::extend_project_constant_mappings(
        &mut project_codegen_code,
        modules,
        &crate_root_modules,
    );
    let union_usage =
        project_union_usage(modules, &project_codegen_code, structural_interop_enabled);
    let structural_record_identities = if structural_interop_enabled {
        crate::structural_impl_codegen::structural_record_identities_for_project(modules)
    } else {
        HashSet::new()
    };
    let mut stdlib_nominal_plan = project_stdlib_nominal_plan(&union_usage.unions, modules);
    let mut nominal_type_paths = project_nominal_type_paths(modules, &crate_root_modules);
    let structural_identity_expressions = if structural_interop_enabled {
        crate::structural_identity_codegen::class_identity_expressions_for_project(
            modules,
            &structural_record_identities,
            &nominal_type_paths,
        )
    } else {
        HashMap::new()
    };
    nominal_type_paths.extend(stdlib_nominal_plan.registry.rust_paths.clone());
    let mut project_support_demand = ModuleSupportDemand::default();
    let mut module_support_demands = HashMap::new();

    for (module_name, module) in modules {
        let module_public = *module_name != "main";
        let mut module_codegen_code = project_codegen_code.clone();
        register_imported_generic_classes(&mut module_codegen_code, module, &project_modules);
        let used_unions = union_usage
            .module_unions
            .get(*module_name)
            .cloned()
            .unwrap_or_default();
        let owned_unions = HashSet::new();
        let structural_identity_module_name = Some(*module_name);
        let structural_layout_location = if crate_root_modules.contains(module_name) {
            super::ProjectStructuralLayoutLocation::Local
        } else {
            super::ProjectStructuralLayoutLocation::CrateRoot
        };
        let codegen_result = generate_rust_with_stdlib_for_module_with_project_policy(
            module,
            &module_codegen_code.emission_view(),
            Some(module_name),
            structural_identity_module_name,
            structural_interop_enabled,
            Some(&owned_unions),
            Some(&union_usage.ordinary_unions),
            Some(&union_usage.try_error_unions),
            Some(&structural_record_identities),
            structural_layout_location,
            Some(&structural_identity_expressions),
            SupportEmission::Deferred,
        );
        let local_imports =
            render_local_module_imports(module, &project_modules, &project_codegen_code);
        let union_imports =
            render_project_union_imports(module_name, &used_unions, &crate_root_modules);
        let module_support_demand = codegen_result.support_demand.clone();
        project_support_demand.merge_project_module(&module_support_demand);
        module_support_demands.insert((*module_name).to_string(), module_support_demand);
        let mut rust_source = relocate_project_stdlib_nominals(
            &codegen_result.module_body_source,
            module_name,
            &stdlib_nominal_plan,
            &crate_root_modules,
            &project_module_binding_names(module),
        );
        let imports = [local_imports, union_imports]
            .into_iter()
            .filter(|source| !source.trim().is_empty())
            .map(|source| source.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n");
        if !imports.is_empty() {
            rust_source = format!("{imports}\n\n{rust_source}");
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

    project_support_demand.set_error_conversion_paths(&nominal_type_paths);
    let rendered_support = render_support(&project_support_demand, &stdlib_code.emission_view());
    used_stdlib_modules.extend(rendered_support.used_stdlib_modules.iter().cloned());
    required_features.extend(rendered_support.required_features.iter().copied());
    let (nominal_prelude, remaining_support) = extract_project_stdlib_nominal_prelude(
        &rendered_support.source,
        &union_usage.unions,
        stdlib_code,
        &mut stdlib_nominal_plan,
    );
    nominal_type_paths.extend(stdlib_nominal_plan.registry.rust_paths.clone());
    let union_prelude = render_project_union_prelude(&union_usage, &nominal_type_paths);
    let record_prelude = render_project_structural_record_prelude(modules, &project_codegen_code);
    let unpruned_project_prelude = [
        nominal_prelude.as_str(),
        union_prelude.as_str(),
        record_prelude.as_str(),
    ]
    .into_iter()
    .filter(|source| !source.trim().is_empty())
    .map(str::trim_end)
    .collect::<Vec<_>>()
    .join("\n\n");
    // Extraction discovers transitive nominal owners only after all modules'
    // demand is merged. Import those finalized root bindings in every physical
    // consumer, including modules with no union of their own.
    for (module_name, source) in &mut files {
        if !crate_root_modules.contains(module_name.as_str()) {
            *source = crate::import_project_prelude_bindings(&unpruned_project_prelude, source)
                .unwrap_or_else(|error| {
                    panic!("failed to import finalized project owners: {error}")
                });
        }
    }
    let support_imports = Renderer::new().render_file(&RustFile {
        items: render_import_items(&rendered_support.import_needs),
    });
    let support_source = [support_imports.trim(), remaining_support.trim()]
        .into_iter()
        .filter(|source| !source.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let body_consumers = files.values().map(String::as_str).collect::<Vec<_>>();
    let (mut project_union_prelude, support_source) = crate::prune_generated_project_owners(
        &unpruned_project_prelude,
        &support_source,
        &body_consumers,
    )
    .unwrap_or_else(|error| panic!("failed to prune generated project owners: {error}"));
    if !support_source.trim().is_empty() {
        let consumers = std::iter::once(project_union_prelude.as_str())
            .chain(body_consumers.iter().copied())
            .collect::<Vec<_>>();
        let visible_support = crate_visible_generated_support_source(&support_source, &consumers);
        let visible_support =
            crate::import_project_prelude_bindings(&project_union_prelude, &visible_support)
                .unwrap_or_else(|error| {
                    panic!("failed to import project prelude bindings into support: {error}")
                });
        let support_names = rust_source_defined_item_names(&visible_support);
        let prelude_support_refs = crate::stdlib_filter::rust_source_referenced_item_names(
            &project_union_prelude,
            &support_names,
        );
        let prelude_support_traits = crate::stdlib_filter::rust_source_required_trait_names(
            &project_union_prelude,
            &visible_support,
        )
        .unwrap_or_else(|error| panic!("invalid generated project support trait layout: {error}"));
        if !prelude_support_refs.is_empty() || !prelude_support_traits.is_empty() {
            project_union_prelude = crate::import_generated_support_in_project_nominals(
                &project_union_prelude,
                &visible_support,
            )
            .unwrap_or_else(|error| {
                panic!("failed to import generated project support into nominals: {error}")
            });
        }
        for (module_name, source) in &mut files {
            let module_needs_support = module_support_demands
                .get(module_name)
                .is_some_and(ModuleSupportDemand::needs_support);
            let body_support_refs =
                crate::stdlib_filter::rust_source_referenced_item_names(source, &support_names);
            let body_support_traits =
                crate::stdlib_filter::rust_source_required_trait_names(source, &visible_support)
                    .unwrap_or_else(|error| {
                        panic!("invalid generated project support trait layout: {error}")
                    });
            if module_needs_support
                && (!body_support_refs.is_empty() || !body_support_traits.is_empty())
            {
                *source = format!(
                    "{}\n\n{}",
                    crate::generated_visibility::generated_support_import(source, &visible_support),
                    source.trim_start()
                );
            }
        }
        let support_module = format!(
            "mod __sifr_generated_support {{\n{}}}\n",
            visible_support.trim_end()
        );
        project_union_prelude = [support_module.trim(), project_union_prelude.trim()]
            .into_iter()
            .filter(|source| !source.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
    }

    crate::retain_generated_dependency_metadata(
        std::iter::once(project_union_prelude.as_str()).chain(files.values().map(String::as_str)),
        &mut used_stdlib_modules,
        &mut required_features,
    )
    .unwrap_or_else(|error| panic!("failed to finalize generated project dependencies: {error}"));

    MultiModuleCodegenResult {
        rust_files: files,
        project_union_prelude,
        used_stdlib_modules,
        required_features,
        interop: crate::stdlib_interop_demand::application_plan(
            stdlib_code,
            &modules
                .iter()
                .map(|(name, module)| (Some(*name), *module))
                .collect::<Vec<_>>(),
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
    reason = "infallible codegen project helper has a tuple return type; driver build paths use fallible sysroot planning"
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
edition = "2024"

[workspace]
"#
    );

    let deps = try_generated_cargo_dependencies(stdlib_modules, required_features)
        .expect("infallible project generation should resolve the Sifr sysroot");

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
#[path = "lib_project_codegen_opaque_import_tests.rs"]
mod opaque_import_tests;

#[cfg(test)]
mod tests;
