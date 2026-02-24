use super::{render_items, CodegenResult, HirModule, RustEmitter, RustItem, StdlibCode};
use crate::ir_imports::collect_import_needs_from_items;

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

    let mut emitted_items = Vec::new();
    if !emitter.enum_defs.is_empty() {
        emitted_items.push(RustItem::RawCode(emitter.enum_defs.clone()));
    }
    if !emitter.output.is_empty() {
        emitted_items.push(RustItem::RawCode(emitter.output.clone()));
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
    if import_needs.runtime.needs_bigint {
        import_items.push(RustItem::Use(vec![
            "num_bigint".to_string(),
            "BigInt".to_string(),
        ]));
    }

    let mut result = String::new();
    if !import_items.is_empty() {
        result.push_str(&render_items(&import_items));
        result.push('\n');
    }
    if !emitter.enum_defs.is_empty() {
        result.push_str(&emitter.enum_defs);
        result.push('\n');
    }
    result.push_str(&emitter.output);

    CodegenResult {
        rust_source: result,
        used_stdlib_modules: emitter.used_stdlib_modules.clone(),
        used_intrinsic_modules: emitter.used_stdlib_modules,
        required_crates: {
            let mut crates = emitter.intrinsic_registry_crates;
            if emitter.runtime_needs.needs_bigint || import_needs.runtime.needs_bigint {
                crates.insert("num-bigint".to_string());
                crates.insert("num-traits".to_string());
            }
            crates
        },
        constant_mappings: emitter.module_constants,
        lowering_stats: emitter.lowering_stats,
    }
}

/// Generate Rust source code from a HIR module, returning metadata about stdlib usage.
pub fn generate_rust_with_metadata(module: &HirModule) -> CodegenResult {
    super::generate_rust_with_stdlib(module, &StdlibCode::default())
}
