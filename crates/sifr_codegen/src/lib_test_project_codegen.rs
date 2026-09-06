use super::{
    HashMap, HashSet, HirModule, ModuleSupportDemand, Renderer, RustFile, StdlibCode,
    SupportEmission, crate_visible_generated_support_source,
    generate_rust_with_stdlib_for_module_with_project_policy, publicize_generated_module_source,
    render_import_items, render_support,
};
use crate::entrypoints::generate_rust_test_with_project_policy;
use crate::lib_project_codegen::{
    project_nominal_type_paths, project_union_usage, register_imported_generic_classes,
    render_local_module_imports, render_project_union_imports,
};
use crate::lib_project_signatures::{project_class_fields, project_func_signatures};
use crate::project_stdlib_nominals::{
    extract_project_stdlib_nominal_prelude, project_stdlib_nominal_plan,
    relocate_project_stdlib_nominals,
};
use crate::project_union_prelude::render_project_union_prelude;
use crate::render_project_structural_record_prelude;
use crate::stdlib_filter::rust_source_defined_item_names;
use sifr_stdlib_manifest::StdlibFeature;

/// Generated Rust sources and aggregate dependency metadata for one test crate.
pub struct TestProjectCodegenResult {
    pub support_rust_files: HashMap<String, String>,
    pub test_rust_files: HashMap<String, String>,
    pub project_union_prelude: String,
    pub used_stdlib_modules: HashSet<String>,
    pub required_features: HashSet<StdlibFeature>,
}

/// Generate support modules and root-level test bodies under one project union policy.
pub fn generate_rust_test_project_with_metadata(
    support_modules: &[(&str, &HirModule)],
    test_modules: &[(&str, &HirModule)],
    stdlib_code: &StdlibCode,
) -> TestProjectCodegenResult {
    let mut all_modules = Vec::with_capacity(support_modules.len() + test_modules.len());
    all_modules.extend_from_slice(support_modules);
    all_modules.extend_from_slice(test_modules);

    let mut project_code = stdlib_code.clone();
    project_code
        .func_signatures
        .extend(project_func_signatures(&all_modules));
    project_code
        .module_class_fields
        .extend(project_class_fields(&all_modules));
    let structural_interop_enabled = all_modules
        .iter()
        .any(|(_, module)| crate::rust_interop_plan::module_uses_structural_interop(module));
    let union_usage = project_union_usage(&all_modules, &project_code, structural_interop_enabled);
    let structural_record_identities = if structural_interop_enabled {
        crate::structural_impl_codegen::structural_record_identities_for_project(&all_modules)
    } else {
        HashSet::new()
    };
    let mut stdlib_nominal_plan = project_stdlib_nominal_plan(&union_usage.unions, &all_modules);
    let crate_root_modules = test_modules
        .iter()
        .map(|(module_name, _)| *module_name)
        .collect::<HashSet<_>>();
    crate::project_constants::extend_project_constant_mappings(
        &mut project_code,
        support_modules,
        &crate_root_modules,
    );
    let mut nominal_type_paths = project_nominal_type_paths(&all_modules, &crate_root_modules);
    let structural_identity_expressions = if structural_interop_enabled {
        crate::structural_identity_codegen::class_identity_expressions_for_project(
            &all_modules,
            &structural_record_identities,
            &nominal_type_paths,
        )
    } else {
        HashMap::new()
    };
    nominal_type_paths.extend(stdlib_nominal_plan.registry.rust_paths.clone());
    let project_modules = all_modules.iter().copied().collect::<HashMap<_, _>>();
    let all_union_names = union_usage.unions.keys().cloned().collect::<HashSet<_>>();

    let mut support_rust_files = HashMap::new();
    let mut test_rust_files = HashMap::new();
    let mut used_stdlib_modules = HashSet::new();
    let mut required_features = HashSet::new();
    let mut project_support_demand = ModuleSupportDemand::default();
    let mut support_module_demands = HashMap::new();
    let mut test_module_demands = HashMap::new();

    for (module_name, module) in support_modules {
        let mut module_code = project_code.clone();
        register_imported_generic_classes(&mut module_code, module, &project_modules);
        let used_unions = union_usage
            .module_unions
            .get(*module_name)
            .cloned()
            .unwrap_or_default();
        let structural_identity_module_name = Some(*module_name);
        let generated = generate_rust_with_stdlib_for_module_with_project_policy(
            module,
            &module_code,
            Some(module_name),
            structural_identity_module_name,
            structural_interop_enabled,
            Some(&HashSet::new()),
            Some(&union_usage.ordinary_unions),
            Some(&union_usage.try_error_unions),
            Some(&structural_record_identities),
            super::ProjectStructuralLayoutLocation::CrateRoot,
            Some(&structural_identity_expressions),
            SupportEmission::Deferred,
        );
        let imports = [
            render_local_module_imports(module, &project_modules, &project_code),
            render_project_union_imports(module_name, &used_unions, &crate_root_modules),
        ]
        .into_iter()
        .filter(|source| !source.trim().is_empty())
        .map(|source| source.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let module_demand = generated.support_demand.clone();
        project_support_demand.merge_project_module(&module_demand);
        support_module_demands.insert((*module_name).to_string(), module_demand);
        let source = if imports.is_empty() {
            generated.module_body_source
        } else {
            format!("{imports}\n\n{}", generated.module_body_source)
        };
        let source = relocate_project_stdlib_nominals(
            &source,
            module_name,
            &stdlib_nominal_plan,
            &crate_root_modules,
            &crate::project_stdlib_nominals::project_module_binding_names(module),
        );
        support_rust_files.insert(
            (*module_name).to_string(),
            publicize_generated_module_source(&source),
        );
        used_stdlib_modules.extend(generated.used_stdlib_modules);
        required_features.extend(generated.required_features);
    }

    for (module_name, module) in test_modules {
        let generated = generate_rust_test_with_project_policy(
            module,
            module_name,
            &project_code,
            Some(&all_union_names),
            Some(&union_usage.ordinary_unions),
            Some(&union_usage.try_error_unions),
            structural_interop_enabled,
            Some(&structural_record_identities),
            Some(&structural_identity_expressions),
            SupportEmission::Deferred,
        );
        let module_demand = generated.support_demand.clone();
        project_support_demand.merge_project_module(&module_demand);
        test_module_demands.insert((*module_name).to_string(), module_demand);
        test_rust_files.insert((*module_name).to_string(), generated.module_body_source);
        used_stdlib_modules.extend(generated.used_stdlib_modules);
        required_features.extend(generated.required_features);
    }

    project_support_demand.set_error_conversion_paths(&nominal_type_paths);
    let rendered_support = render_support(&project_support_demand, stdlib_code);
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
    let record_prelude = render_project_structural_record_prelude(&all_modules, &project_code);
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
    let support_imports = Renderer::new().render_file(&RustFile {
        items: render_import_items(&rendered_support.import_needs),
    });
    let support_source = [support_imports.trim(), remaining_support.trim()]
        .into_iter()
        .filter(|source| !source.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let body_consumers = support_rust_files
        .values()
        .chain(test_rust_files.values())
        .map(String::as_str)
        .collect::<Vec<_>>();
    let (mut project_union_prelude, support_source) = crate::prune_generated_project_owners(
        &unpruned_project_prelude,
        &support_source,
        &body_consumers,
    )
    .unwrap_or_else(|error| panic!("failed to prune generated test-project owners: {error}"));
    if !support_source.trim().is_empty() {
        let visible_support = crate_visible_generated_support_source(&support_source);
        let visible_support = crate::import_project_prelude_bindings_in_generated_support(
            &project_union_prelude,
            &visible_support,
        )
        .unwrap_or_else(|error| {
            panic!("failed to import test-project prelude bindings into support: {error}")
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
        .unwrap_or_else(|error| {
            panic!("invalid generated test-project support trait layout: {error}")
        });
        if !prelude_support_refs.is_empty() || !prelude_support_traits.is_empty() {
            project_union_prelude =
                crate::import_generated_support_in_project_nominals(&project_union_prelude)
                    .unwrap_or_else(|error| {
                        panic!(
                            "failed to import generated test-project support into nominals: {error}"
                        )
                    });
        }
        for (module_name, source) in &mut support_rust_files {
            let body_support_refs =
                crate::stdlib_filter::rust_source_referenced_item_names(source, &support_names);
            let body_support_traits =
                crate::stdlib_filter::rust_source_required_trait_names(source, &visible_support)
                    .unwrap_or_else(|error| {
                        panic!("invalid generated test-project support trait layout: {error}")
                    });
            if support_module_demands
                .get(module_name)
                .is_some_and(ModuleSupportDemand::needs_support)
                && (!body_support_refs.is_empty() || !body_support_traits.is_empty())
            {
                *source = format!(
                    "use crate::__sifr_generated_support::*;\n\n{}",
                    source.trim_start()
                );
            }
        }
        let tests_need_support = test_rust_files.iter().any(|(module_name, source)| {
            let body_support_refs =
                crate::stdlib_filter::rust_source_referenced_item_names(source, &support_names);
            let body_support_traits =
                crate::stdlib_filter::rust_source_required_trait_names(source, &visible_support)
                    .unwrap_or_else(|error| {
                        panic!("invalid generated test-project support trait layout: {error}")
                    });
            test_module_demands
                .get(module_name)
                .is_some_and(ModuleSupportDemand::needs_support)
                && (!body_support_refs.is_empty() || !body_support_traits.is_empty())
        });
        let support_module = format!(
            "mod __sifr_generated_support {{\n{}}}\n",
            visible_support.trim_end()
        );
        project_union_prelude = if tests_need_support {
            format!(
                "{}\n\nuse crate::__sifr_generated_support::*;\n\n{}",
                support_module.trim_end(),
                project_union_prelude.trim()
            )
        } else {
            [support_module.trim(), project_union_prelude.trim()]
                .into_iter()
                .filter(|source| !source.is_empty())
                .collect::<Vec<_>>()
                .join("\n\n")
        };
    }

    crate::retain_generated_dependency_metadata(
        std::iter::once(project_union_prelude.as_str())
            .chain(support_rust_files.values().map(String::as_str))
            .chain(test_rust_files.values().map(String::as_str)),
        &mut used_stdlib_modules,
        &mut required_features,
    )
    .unwrap_or_else(|error| {
        panic!("failed to finalize generated test-project dependencies: {error}")
    });

    TestProjectCodegenResult {
        support_rust_files,
        test_rust_files,
        project_union_prelude,
        used_stdlib_modules,
        required_features,
    }
}
