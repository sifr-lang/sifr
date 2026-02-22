use super::{CodegenResult, HirModule, RustEmitter, StdlibCode};

/// Generate Rust source code from a HIR module.
pub fn generate_rust(module: &HirModule) -> String {
    generate_rust_with_metadata(module).rust_source
}

/// Generate Rust source code for a test module (with #[test] attributes).
pub fn generate_rust_test(module: &HirModule) -> CodegenResult {
    let mut emitter = RustEmitter::new();
    emitter.test_mode = true;

    // First pass: collect all union types used in the module
    emitter.collect_union_types(module);

    // Detect recursive (self-referential) class fields that need Box<T>
    emitter.detect_recursive_fields(module);

    // Generate enum definitions for non-Option union types
    emitter.generate_enum_definitions();

    // Second pass: emit the actual code
    emitter.emit_module(module, false);

    let mut result = String::new();
    if emitter.needs_hashmap {
        result.push_str("use std::collections::HashMap;\n");
    }
    if emitter.needs_hashset {
        result.push_str("use std::collections::HashSet;\n");
    }
    if emitter.needs_vecdeque {
        result.push_str("use std::collections::VecDeque;\n");
    }
    if emitter.needs_bigint {
        result.push_str("use num_bigint::BigInt;\n");
    }
    if emitter.needs_hashmap || emitter.needs_hashset || emitter.needs_vecdeque || emitter.needs_bigint {
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
            if emitter.needs_bigint {
                crates.insert("num-bigint".to_string());
                crates.insert("num-traits".to_string());
            }
            crates
        },
        constant_mappings: emitter.module_constants,
    }
}

/// Generate Rust source code from a HIR module, returning metadata about stdlib usage.
pub fn generate_rust_with_metadata(module: &HirModule) -> CodegenResult {
    super::generate_rust_with_stdlib(module, &StdlibCode::default())
}
