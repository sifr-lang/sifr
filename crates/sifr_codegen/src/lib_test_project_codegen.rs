use super::{
    HashMap, HashSet, HirModule, StdlibCode,
    generate_rust_with_stdlib_for_module_with_project_policy, publicize_generated_module_source,
};
use crate::entrypoints::generate_rust_test_with_project_policy;
use crate::lib_project_codegen::{
    project_nominal_type_paths, project_union_usage, register_imported_generic_classes,
    render_local_module_imports, render_project_union_imports,
};
use crate::lib_project_signatures::{project_class_fields, project_func_signatures};
use crate::project_stdlib_nominals::{
    project_stdlib_nominal_plan, relocate_project_stdlib_nominals,
};
use crate::project_union_prelude::render_project_union_prelude;
use sifr_stdlib_manifest::StdlibFeature;

/// Generated Rust sources and aggregate dependency metadata for one test crate.
pub struct TestProjectCodegenResult {
    pub support_rust_files: HashMap<String, String>,
    pub test_rust_files: HashMap<String, String>,
    pub project_union_prelude: String,
    pub used_stdlib_modules: HashSet<String>,
    pub required_features: HashSet<StdlibFeature>,
    pub interop: crate::InteropBuildPlan,
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
    let stdlib_nominal_plan = project_stdlib_nominal_plan(
        &union_usage.unions,
        stdlib_code,
        &all_modules,
        structural_interop_enabled,
    );
    let crate_root_modules = test_modules
        .iter()
        .map(|(module_name, _)| *module_name)
        .collect::<HashSet<_>>();
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
    let union_prelude = render_project_union_prelude(&union_usage, &nominal_type_paths);
    let project_union_prelude = [stdlib_nominal_plan.prelude.as_str(), union_prelude.as_str()]
        .into_iter()
        .filter(|source| !source.trim().is_empty())
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n\n");
    let project_modules = all_modules.iter().copied().collect::<HashMap<_, _>>();
    let all_union_names = union_usage.unions.keys().cloned().collect::<HashSet<_>>();

    let mut support_rust_files = HashMap::new();
    let mut test_rust_files = HashMap::new();
    let mut used_stdlib_modules = stdlib_nominal_plan.used_stdlib_modules.clone();
    let mut required_features = stdlib_nominal_plan.required_features.clone();

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
            Some(&structural_identity_expressions),
        );
        let imports = [
            render_local_module_imports(module, &project_modules),
            render_project_union_imports(module_name, &used_unions, &crate_root_modules),
        ]
        .into_iter()
        .filter(|source| !source.trim().is_empty())
        .map(|source| source.trim_end().to_string())
        .collect::<Vec<_>>()
        .join("\n");
        let source = if imports.is_empty() {
            generated.rust_source
        } else {
            format!("{imports}\n\n{}", generated.rust_source)
        };
        let source = relocate_project_stdlib_nominals(
            &source,
            module_name,
            &stdlib_nominal_plan,
            &crate_root_modules,
            &module
                .classes
                .iter()
                .map(|class| sifr_type_system::source_class_rust_name(&class.name))
                .collect(),
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
        );
        test_rust_files.insert((*module_name).to_string(), generated.rust_source);
        used_stdlib_modules.extend(generated.used_stdlib_modules);
        required_features.extend(generated.required_features);
    }

    TestProjectCodegenResult {
        support_rust_files,
        test_rust_files,
        project_union_prelude,
        used_stdlib_modules,
        required_features,
        interop: crate::rust_interop_plan::interop_build_plan_for_named_modules(
            all_modules
                .iter()
                .map(|(module_name, module)| (Some(*module_name), *module)),
        ),
    }
}
