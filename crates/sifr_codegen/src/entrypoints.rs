use super::{CodegenResult, HirModule, Renderer, RustEmitter, RustFile, RustItem, StdlibCode};
use crate::ir_imports::collect_import_needs_from_items;
use crate::ir_optimize::{remove_trivial_clones_in_items, remove_unneeded_mutability_in_items};
use crate::ir_validate::validate_items;

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    generate_rust_with_metadata(module).rust_source
}

/// Generate Rust source code for a test module (with #[test] attributes).
pub fn generate_rust_test(module: &HirModule) -> CodegenResult {
    let mut emitter = RustEmitter::new();

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);

    // Detect recursive (self-referential) class fields that need Box<T>
    emitter.detect_recursive_fields(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();

    // Second pass: emit the actual code
    emitter.emit_module(module, false, true);

    let mut module_import_items: Vec<RustItem> = Vec::new();
    for import in &module.imports {
        // Stdlib/intrinsic imports are lowered through registry/preamble paths.
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

    let mut emitted_items: Vec<RustItem> = Vec::new();
    emitted_items.extend(module_import_items);
    if !emitter.enum_items.is_empty() {
        emitted_items.extend(emitter.enum_items.clone());
    }
    let uses_task_scope = super::module_uses_task_scope(module);
    let uses_join_set = super::module_uses_join_set(module);
    let uses_join_set_spawn_cpu = super::module_uses_join_set_spawn_cpu(module);
    let uses_task_scope_offload = super::module_uses_task_scope_offload(module);
    let uses_task_scope_spawn_cpu = super::module_uses_task_scope_spawn_cpu(module);
    let uses_spawn_cpu = super::module_uses_spawn_cpu(module);
    if uses_task_scope || uses_join_set || super::module_uses_failure_type(module) {
        emitted_items.extend(super::build_failure_type_items());
    }
    if uses_task_scope || uses_join_set || super::module_uses_cancellation_error_type(module) {
        emitted_items.extend(super::build_cancellation_error_type_items());
    }
    if super::module_uses_async_exit_cause_type(module) {
        emitted_items.extend(super::build_async_exit_cause_type_items());
    }
    if uses_task_scope || uses_join_set {
        emitted_items.extend(super::build_task_scope_items());
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
    if super::module_uses_timeout_result_type(module) && !uses_task_scope && !uses_join_set {
        emitted_items.extend(super::build_timeout_result_type_items());
    }
    if !emitter.body_items.is_empty() {
        emitted_items.extend(emitter.body_items.clone());
    }
    let import_needs = collect_import_needs_from_items(&emitted_items);

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
            "sifr_runtime".to_string(),
            "SifrInt".to_string(),
        ]));
    }

    let mut file_items: Vec<RustItem> = Vec::new();
    file_items.extend(import_items);
    file_items.extend(emitted_items);
    remove_trivial_clones_in_items(&mut file_items);
    remove_unneeded_mutability_in_items(&mut file_items);
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
    let uses_task_sleep = super::module_uses_task_sleep(module);
    let needs_python_runtime = rust_source.contains("sifr_stdlib::python::");

    CodegenResult {
        rust_source,
        used_stdlib_modules: emitter.used_stdlib_modules.clone(),
        used_intrinsic_modules: emitter.used_stdlib_modules,
        required_features: {
            let mut features = emitter.intrinsic_registry_features;
            if emitter.runtime_needs.bigint() || import_needs.runtime.numeric.needs_bigint {
                features.insert(sifr_stdlib_manifest::StdlibFeature::NumBigint);
                features.insert(sifr_stdlib_manifest::StdlibFeature::NumTraits);
            }
            if import_needs.runtime.numeric.needs_decimal {
                features.insert(sifr_stdlib_manifest::StdlibFeature::RustDecimal);
            }
            if import_needs.runtime.numeric.needs_bigdecimal {
                features.insert(sifr_stdlib_manifest::StdlibFeature::BigDecimal);
            }
            if import_needs.runtime.needs_sifr_int {
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
    }
}

/// Generate Rust source code from a HIR module, returning metadata about stdlib usage.
pub fn generate_rust_with_metadata(module: &HirModule) -> CodegenResult {
    super::generate_rust_with_stdlib(module, &StdlibCode::default())
}
