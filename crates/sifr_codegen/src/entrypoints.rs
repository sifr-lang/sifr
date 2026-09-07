use super::{
    CodegenResult, HashSet, HirModule, ModuleSupportDemand, Renderer, RustEmitter, RustFile,
    RustItem, StdlibCode, SupportEmission, render_import_items,
};
use crate::ir_imports::collect_import_needs_from_items;
use crate::ir_optimize::{
    remove_trivial_clones_in_items, remove_unneeded_mutability_in_items,
    remove_unread_pure_bindings_in_items, simplify_control_flow_in_items,
};
use crate::ir_validate::validate_items;

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    generate_rust_with_metadata(module).rust_source
}

/// Generate Rust source code for a test module (with #[test] attributes).
pub fn generate_rust_test(module: &HirModule, module_name: &str) -> CodegenResult {
    generate_rust_test_with_project_policy(
        module,
        module_name,
        &StdlibCode::default(),
        None,
        None,
        None,
        crate::rust_interop_plan::module_uses_structural_interop(module),
        None,
        None,
        SupportEmission::Inline,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_rust_test_with_project_policy(
    module: &HirModule,
    module_name: &str,
    project_code: &StdlibCode,
    project_union_enums: Option<&HashSet<String>>,
    project_ordinary_union_enums: Option<&HashSet<String>>,
    project_try_error_carrier_enums: Option<&HashSet<String>>,
    structural_interop_enabled: bool,
    project_structural_record_identities: Option<&HashSet<String>>,
    project_structural_identity_expressions: Option<&super::HashMap<String, String>>,
    support_emission: SupportEmission,
) -> CodegenResult {
    let mut emitter = RustEmitter::new();
    emitter.structural_interop_enabled = structural_interop_enabled;
    emitter.project_structural_record_identities = project_structural_record_identities.cloned();
    emitter.project_structural_identity_expressions =
        project_structural_identity_expressions.cloned();
    emitter.structural_identity_module_name = Some(module_name.to_string());
    crate::project_constants::register_imported_constants(&mut emitter, module, project_code);

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);
    crate::lib_project_codegen::register_imported_union_types(&mut emitter, module, project_code);
    if let Some(project_ordinary_union_enums) = project_ordinary_union_enums {
        emitter
            .ordinary_union_enums
            .extend(project_ordinary_union_enums.iter().cloned());
    }
    if let Some(project_try_error_carrier_enums) = project_try_error_carrier_enums {
        emitter
            .try_error_carrier_enums
            .extend(project_try_error_carrier_enums.iter().cloned());
    }
    if let Some(project_union_enums) = project_union_enums {
        emitter.suppressed_union_enum_definitions = emitter
            .union_enums
            .keys()
            .filter(|name| project_union_enums.contains(*name))
            .cloned()
            .collect();
    }

    // Detect recursive (self-referential) class fields that need Box<T>
    emitter.detect_recursive_fields(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();
    if project_union_enums.is_none() {
        emitter.generate_structural_record_definitions();
    }

    // Second pass: emit the actual code
    emitter.emit_named_module(module, false, true, Some(module_name));
    // Expression lowering can introduce canonical intermediate error unions.
    emitter.generate_enum_definitions();
    let support_demand = ModuleSupportDemand::from_emitter(module, &emitter, Some(module_name));

    let mut module_import_items: Vec<RustItem> = Vec::new();
    for import in &module.imports {
        // Stdlib/intrinsic imports are lowered through registry/preamble paths.
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        let mut module_path = vec!["crate".to_string()];
        module_path.extend(import.module.split('.').map(str::to_string));
        for name in &import.names {
            if project_code
                .module_constants
                .get(&import.module)
                .is_some_and(|constants| constants.contains_key(name))
            {
                continue;
            }
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

    let original_module_import_items = module_import_items.clone();
    if support_emission == SupportEmission::Deferred {
        return deferred_test_codegen_result(
            module,
            emitter,
            support_demand,
            original_module_import_items,
        );
    }
    let mut emitted_items: Vec<RustItem> = Vec::new();
    if !emitter.enum_items.is_empty() {
        emitted_items.extend(emitter.enum_items.clone());
    }
    let runtime_demand = &support_demand.runtime;
    let uses_task_scope = runtime_demand.task_scope;
    let uses_join_set = runtime_demand.join_set;
    let uses_async_python = runtime_demand.async_python;
    let uses_native_async_cleanup = runtime_demand.native_async_cleanup;
    let uses_join_set_spawn_cpu = runtime_demand.join_set_spawn_cpu;
    let uses_task_scope_offload = runtime_demand.task_scope_offload;
    let uses_task_scope_spawn_cpu = runtime_demand.task_scope_spawn_cpu;
    let uses_spawn_cpu = runtime_demand.spawn_cpu;
    if uses_task_scope || uses_join_set || runtime_demand.failure_type {
        emitted_items.extend(super::build_failure_type_items());
    }
    if uses_task_scope || uses_join_set || runtime_demand.cancellation_error_type {
        emitted_items.extend(super::build_cancellation_error_type_items());
    }
    if runtime_demand.async_exit_cause_type {
        emitted_items.extend(super::build_async_exit_cause_type_items());
    }
    if uses_task_scope || uses_join_set || uses_async_python || uses_native_async_cleanup {
        emitted_items.extend(super::build_task_cancellation_items(
            uses_async_python || uses_native_async_cleanup,
            uses_task_scope || uses_join_set || uses_native_async_cleanup,
        ));
    }
    if uses_task_scope || uses_join_set {
        emitted_items.extend(super::build_task_scope_items());
        emitted_items.extend(super::build_task_supervisor_items());
        emitted_items.extend(super::build_task_context_scope_extension_items(true));
    }
    if uses_task_scope_offload {
        emitted_items.extend(super::build_task_scope_offload_items());
    }
    if uses_task_scope_spawn_cpu {
        emitted_items.extend(super::build_task_scope_cpu_offload_items());
    }
    if uses_join_set {
        emitted_items.extend(super::build_join_set_items());
    }
    if uses_join_set_spawn_cpu || uses_spawn_cpu || uses_task_scope_spawn_cpu {
        emitted_items.extend(super::build_worker_panic_hook_items());
    }
    if uses_join_set_spawn_cpu {
        emitted_items.extend(super::build_join_set_cpu_items());
    }
    if uses_spawn_cpu {
        emitted_items.extend(super::build_cpu_offload_items());
    }
    if runtime_demand.timeout_result_type && !uses_task_scope && !uses_join_set {
        emitted_items.extend(super::build_timeout_result_type_items());
    }
    if !emitter.body_items.is_empty() {
        emitted_items.extend(emitter.body_items.clone());
    }
    let import_needs = collect_import_needs_from_items(&emitted_items);
    module_import_items.retain(|item| match item {
        RustItem::Use(path) => path
            .last()
            .is_some_and(|name| import_needs.referenced_symbols.contains(name)),
        RustItem::UseAlias { alias, .. } => {
            alias == "_" || import_needs.referenced_symbols.contains(alias)
        }
        _ => true,
    });

    let mut import_items = Vec::new();
    if import_needs.collections.needs_hashmap {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "collections".to_string(),
            "HashMap".to_string(),
        ]));
    }
    if import_needs.collections.needs_hashset {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "collections".to_string(),
            "HashSet".to_string(),
        ]));
    }
    if import_needs.collections.needs_vecdeque {
        import_items.push(RustItem::Use(vec![
            "std".to_string(),
            "collections".to_string(),
            "VecDeque".to_string(),
        ]));
    }
    if import_needs.runtime.numeric.needs_bigint {
        import_items.push(RustItem::Use(vec![
            "num_bigint".to_string(),
            "BigInt".to_string(),
        ]));
    }
    if import_needs.runtime.numeric.needs_decimal {
        import_items.push(RustItem::Use(vec![
            "rust_decimal".to_string(),
            "Decimal".to_string(),
        ]));
    }
    if import_needs.runtime.numeric.needs_bigdecimal {
        import_items.push(RustItem::Use(vec![
            "bigdecimal".to_string(),
            "BigDecimal".to_string(),
        ]));
    }
    if import_needs.runtime.needs_sifr_int {
        import_items.push(RustItem::Use(vec![
            String::new(),
            "sifr_runtime".to_string(),
            "SifrInt".to_string(),
        ]));
    }
    if import_needs.runtime.needs_sifr_range {
        import_items.push(RustItem::Use(vec![
            String::new(),
            "sifr_runtime".to_string(),
            "SifrRange".to_string(),
        ]));
    }

    let mut file_items: Vec<RustItem> = Vec::new();
    file_items.extend(import_items);
    file_items.extend(module_import_items);
    file_items.extend(emitted_items);
    remove_trivial_clones_in_items(&mut file_items);
    simplify_control_flow_in_items(&mut file_items);
    remove_unread_pure_bindings_in_items(&mut file_items);
    remove_unneeded_mutability_in_items(&mut file_items, &emitter.protected_mutable_place_roots);
    let file_issues = validate_items(&file_items);
    assert!(
        file_issues.is_empty(),
        "codegen IR validation failed (test file): {}",
        file_issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let rust_file = RustFile { items: file_items };
    let rust_source = Renderer::new().render_file(&rust_file);
    let mut module_body_items = Vec::new();
    module_body_items.extend(emitter.enum_items.clone());
    module_body_items.extend(emitter.body_items.clone());
    remove_trivial_clones_in_items(&mut module_body_items);
    simplify_control_flow_in_items(&mut module_body_items);
    remove_unread_pure_bindings_in_items(&mut module_body_items);
    remove_unneeded_mutability_in_items(
        &mut module_body_items,
        &emitter.protected_mutable_place_roots,
    );
    let module_body_import_needs = collect_import_needs_from_items(&module_body_items);
    let mut module_body_imports = original_module_import_items;
    module_body_imports.retain(|item| match item {
        RustItem::Use(path) => path
            .last()
            .is_some_and(|name| module_body_import_needs.referenced_symbols.contains(name)),
        RustItem::UseAlias { alias, .. } => {
            alias == "_" || module_body_import_needs.referenced_symbols.contains(alias)
        }
        _ => true,
    });
    module_body_imports.extend(render_import_items(&module_body_import_needs));
    module_body_imports.extend(module_body_items);
    let module_body_source = Renderer::new().render_file(&RustFile {
        items: module_body_imports,
    });
    let uses_task_sleep = runtime_demand.task_sleep;
    let needs_python_runtime =
        super::python_interop_common::rust_source_uses_python_runtime(&rust_source);

    CodegenResult {
        module_body_source,
        rust_source,
        static_programs: Vec::new(),
        static_program_structural_owners: std::collections::BTreeSet::new(),
        used_stdlib_modules: emitter.used_stdlib_modules.clone(),
        used_intrinsic_modules: emitter.used_stdlib_modules,
        required_features: {
            let mut features = emitter.intrinsic_registry_features;
            if import_needs.runtime.numeric.needs_bigint {
                features.insert(sifr_stdlib_manifest::StdlibFeature::NumBigint);
                features.insert(sifr_stdlib_manifest::StdlibFeature::NumTraits);
            }
            if import_needs.runtime.numeric.needs_decimal {
                features.insert(sifr_stdlib_manifest::StdlibFeature::RustDecimal);
            }
            if import_needs.runtime.numeric.needs_bigdecimal {
                features.insert(sifr_stdlib_manifest::StdlibFeature::BigDecimal);
            }
            if import_needs.runtime.needs_sifr_runtime {
                features.insert(sifr_stdlib_manifest::StdlibFeature::SifrRuntime);
            }
            if uses_task_sleep {
                features.insert(sifr_stdlib_manifest::StdlibFeature::Tokio);
            }
            if needs_python_runtime {
                features.insert(sifr_stdlib_manifest::StdlibFeature::PythonRuntime);
            }
            features
        },
        interop: crate::rust_interop_plan::interop_build_plan_for_module(module),
        constant_mappings: emitter.module_constants,
        lowering_stats: emitter.lowering_stats,
        support_demand,
    }
}

fn deferred_test_codegen_result(
    module: &HirModule,
    mut emitter: RustEmitter,
    support_demand: ModuleSupportDemand,
    mut module_import_items: Vec<RustItem>,
) -> CodegenResult {
    let mut body_items = emitter.enum_items.clone();
    body_items.extend(emitter.body_items.clone());
    remove_trivial_clones_in_items(&mut body_items);
    simplify_control_flow_in_items(&mut body_items);
    remove_unread_pure_bindings_in_items(&mut body_items);
    remove_unneeded_mutability_in_items(&mut body_items, &emitter.protected_mutable_place_roots);
    let import_needs = collect_import_needs_from_items(&body_items);
    module_import_items.retain(|item| match item {
        RustItem::Use(path) => path
            .last()
            .is_some_and(|name| import_needs.referenced_symbols.contains(name)),
        RustItem::UseAlias { alias, .. } => {
            alias == "_" || import_needs.referenced_symbols.contains(alias)
        }
        _ => true,
    });
    let mut file_items = render_import_items(&import_needs);
    file_items.extend(module_import_items);
    file_items.extend(body_items);
    let issues = validate_items(&file_items);
    assert!(
        issues.is_empty(),
        "codegen IR validation failed (deferred test file): {}",
        issues
            .iter()
            .map(|issue| issue.message.as_str())
            .collect::<Vec<_>>()
            .join(" | ")
    );
    let module_body_source = Renderer::new().render_file(&RustFile { items: file_items });
    let mut required_features = support_demand.base_required_features();
    crate::add_import_features(&import_needs, &mut required_features);

    CodegenResult {
        rust_source: module_body_source.clone(),
        module_body_source,
        static_programs: Vec::new(),
        static_program_structural_owners: std::collections::BTreeSet::new(),
        used_stdlib_modules: support_demand.directly_used_stdlib_modules(),
        used_intrinsic_modules: std::mem::take(&mut emitter.used_stdlib_modules),
        required_features,
        interop: crate::rust_interop_plan::interop_build_plan_for_module(module),
        constant_mappings: std::mem::take(&mut emitter.module_constants),
        lowering_stats: emitter.lowering_stats,
        support_demand,
    }
}

/// Generate Rust source code from a HIR module, returning metadata about stdlib usage.
pub fn generate_rust_with_metadata(module: &HirModule) -> CodegenResult {
    super::generate_rust_with_stdlib(module, &StdlibCode::default())
}
