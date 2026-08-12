use super::{
    generate_rust_with_stdlib_for_module_with_project_policy, publicize_generated_module_source,
    HashMap, HashSet, HirModule, StdlibCode,
};
use crate::entrypoints::generate_rust_test_with_project_policy;
use crate::lib_project_codegen::{
    project_nominal_type_paths, project_union_usage, register_imported_generic_classes,
    render_local_module_imports, render_project_union_imports, render_project_union_prelude,
};
use crate::lib_project_signatures::{project_class_fields, project_func_signatures};
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
    let union_usage = project_union_usage(&all_modules, &project_code);
    let crate_root_modules = test_modules
        .iter()
        .map(|(module_name, _)| *module_name)
        .collect::<HashSet<_>>();
    let nominal_type_paths = project_nominal_type_paths(&all_modules, &crate_root_modules);
    let project_union_prelude = render_project_union_prelude(&union_usage, &nominal_type_paths);
    let project_modules = all_modules.iter().copied().collect::<HashMap<_, _>>();
    let structural_interop_enabled = all_modules
        .iter()
        .any(|(_, module)| crate::rust_interop_plan::module_uses_structural_interop(module));
    let all_union_names = union_usage.unions.keys().cloned().collect::<HashSet<_>>();

    let mut support_rust_files = HashMap::new();
    let mut test_rust_files = HashMap::new();
    let mut used_stdlib_modules = HashSet::new();
    let mut required_features = HashSet::new();

    for (module_name, module) in support_modules {
        let mut module_code = project_code.clone();
        register_imported_generic_classes(&mut module_code, module, &project_modules);
        let used_unions = union_usage
            .module_unions
            .get(*module_name)
            .cloned()
            .unwrap_or_default();
        let generated = generate_rust_with_stdlib_for_module_with_project_policy(
            module,
            &module_code,
            Some(module_name),
            structural_interop_enabled,
            Some(&HashSet::new()),
            Some(&union_usage.ordinary_unions),
            Some(&union_usage.try_error_unions),
        );
        let imports = [
            render_local_module_imports(module, &project_modules),
            render_project_union_imports(module_name, &used_unions),
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
    }
}
