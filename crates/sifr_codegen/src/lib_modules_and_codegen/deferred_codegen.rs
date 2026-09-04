use super::{
    CodegenResult, ModuleSupportDemand, ProjectStructuralLayoutLocation, RustEmitter, StdlibCode,
    project_imports,
};
use crate::ir_imports::collect_import_needs_from_items;
use crate::ir_optimize::{
    remove_trivial_clones_in_items, remove_unneeded_mutability_in_items,
    remove_unread_pure_bindings_in_items, simplify_control_flow_in_items,
};
use crate::ir_validate::validate_items;
use crate::{
    Renderer, RustFile, add_import_features, annotate_async_main_entrypoint, render_import_items,
    render_support, scope_async_main_cancellation,
};
use sifr_ir::HirModule;
use sifr_stdlib_manifest::StdlibFeature;

pub(super) fn deferred_codegen_result(
    module: &HirModule,
    stdlib_code: &StdlibCode,
    mut emitter: RustEmitter,
    support_demand: ModuleSupportDemand,
    structural_layout_location: ProjectStructuralLayoutLocation,
    has_project_structural_layout: bool,
) -> CodegenResult {
    let mut body_items = emitter.enum_items.clone();
    body_items.extend(emitter.body_items.clone());
    if support_demand.runtime.async_python || support_demand.runtime.native_async_cleanup {
        scope_async_main_cancellation(&mut body_items);
    }
    remove_trivial_clones_in_items(&mut body_items);
    simplify_control_flow_in_items(&mut body_items);
    remove_unread_pure_bindings_in_items(&mut body_items);
    remove_unneeded_mutability_in_items(&mut body_items, &emitter.protected_mutable_place_roots);
    let has_async_main = annotate_async_main_entrypoint(&mut body_items);
    let body_import_needs = collect_import_needs_from_items(&body_items);
    let mut file_items = if has_project_structural_layout
        && structural_layout_location == ProjectStructuralLayoutLocation::CrateRoot
    {
        project_imports::structural_layout_import_items(&emitter.structural_record_types)
    } else {
        Vec::new()
    };
    file_items.extend(render_import_items(&body_import_needs));
    file_items.extend(body_items);
    let issues = validate_items(&file_items);
    assert!(
        issues.is_empty(),
        "codegen IR validation failed (deferred module): {}",
        issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let module_body_source = Renderer::new().render_file(&RustFile { items: file_items });

    let mut used_stdlib_modules = support_demand.directly_used_stdlib_modules();
    for module_name in support_demand.directly_used_stdlib_modules() {
        if let Some(dependencies) = stdlib_code.transitive_deps.get(&module_name) {
            used_stdlib_modules.extend(dependencies.iter().cloned());
        }
    }
    let mut required_features = support_demand.base_required_features();
    add_import_features(&body_import_needs, &mut required_features);
    if has_async_main {
        required_features.insert(StdlibFeature::Tokio);
    }
    if crate::python_interop_common::rust_source_uses_python_runtime(&module_body_source) {
        required_features.insert(StdlibFeature::PythonRuntime);
    }

    CodegenResult {
        rust_source: module_body_source.clone(),
        module_body_source,
        static_programs: Vec::new(),
        static_program_structural_owners: std::collections::BTreeSet::new(),
        used_stdlib_modules,
        used_intrinsic_modules: std::mem::take(&mut emitter.used_stdlib_modules),
        required_features,
        interop: crate::rust_interop_plan::interop_build_plan_for_module(module),
        constant_mappings: std::mem::take(&mut emitter.module_constants),
        lowering_stats: emitter.lowering_stats,
        support_demand,
    }
}

pub(super) fn inline_codegen_result(
    module: &HirModule,
    stdlib_code: &StdlibCode,
    emitter: RustEmitter,
    support_demand: ModuleSupportDemand,
    structural_layout_location: ProjectStructuralLayoutLocation,
    has_project_structural_layout: bool,
) -> CodegenResult {
    let mut generated = deferred_codegen_result(
        module,
        stdlib_code,
        emitter,
        support_demand,
        structural_layout_location,
        has_project_structural_layout,
    );
    let rendered_support = render_support(&generated.support_demand, stdlib_code);
    let support_imports = Renderer::new().render_file(&RustFile {
        items: render_import_items(&rendered_support.import_needs),
    });
    let (body_imports, body_source) = split_leading_imports(&generated.module_body_source);
    let mut imports = Vec::new();
    for import in support_imports.lines().chain(body_imports.iter().copied()) {
        let import = import.trim();
        if !import.is_empty() && !imports.contains(&import) {
            imports.push(import);
        }
    }
    let import_source = imports.join("\n");
    let rendered_support_source = if rendered_support.source.contains("// --- stdlib:") {
        format!(
            "{}\n// --- end stdlib ---",
            rendered_support.source.trim_end()
        )
    } else {
        rendered_support.source.trim_end().to_string()
    };
    let assembled = [
        import_source.trim(),
        rendered_support_source.trim(),
        body_source.trim(),
    ]
    .into_iter()
    .filter(|source| !source.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n");
    if let Err(error) = syn::parse_file(&assembled) {
        panic!("failed to parse inline support assembled by the canonical renderer: {error}");
    }
    generated.rust_source = format!("{}\n", assembled.trim_end());
    generated
        .used_stdlib_modules
        .extend(rendered_support.used_stdlib_modules);
    generated
        .required_features
        .extend(rendered_support.required_features);
    generated
}

fn split_leading_imports(source: &str) -> (Vec<&str>, &str) {
    let mut imports = Vec::new();
    let mut offset = 0;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") && trimmed.ends_with(';') {
            imports.push(trimmed);
            offset += line.len();
            continue;
        }
        if trimmed.is_empty() {
            offset += line.len();
            continue;
        }
        break;
    }
    (imports, &source[offset..])
}
