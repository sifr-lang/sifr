#[test]
fn test_generate_rust_with_stdlib_assembles_single_rust_file() {
    let lib_src = include_str!("../lib_modules_and_codegen.rs");
    let assembly_src = include_str!("../lib_modules_and_codegen/deferred_codegen.rs");
    let start = lib_src
        .find("pub fn generate_rust_with_stdlib")
        .expect("generate_rust_with_stdlib should exist");
    let generate_block = &lib_src[start..];

    assert!(generate_block.contains("inline_codegen_result("));
    assert!(assembly_src.contains("let rendered_support = render_support("));
    assert!(assembly_src.contains("split_leading_imports("));
    assert!(assembly_src.contains("session.validate(&assembled, support)"));
    assert!(assembly_src.contains("syn::parse_file(&assembled).map(|_| ())"));
    assert!(assembly_src.contains("if let Err(error) = validation"));
    assert!(
        assembly_src.contains("failed to parse inline support assembled by the canonical renderer")
    );
    assert!(!generate_block.contains("assert_output_drained("));
    assert!(!generate_block.contains("emitter.output"));
}

#[test]
fn test_generate_rust_multi_assembles_single_rust_file() {
    let lib_src = include_str!("../lib_project_codegen.rs");
    let start = lib_src
        .find("pub fn generate_rust_multi_with_metadata")
        .expect("generate_rust_multi_with_metadata should exist");
    let end = lib_src
        .find("/// Generate a complete Rust project (Cargo.toml + main.rs content).")
        .expect("generate_project docs should exist");
    let generate_block = &lib_src[start..end];

    assert!(generate_block.contains("generate_rust_with_stdlib_for_module_with_project_policy("));
    assert!(generate_block.contains("structural_interop_enabled,"));
    assert!(generate_block.contains("register_imported_generic_classes("));
    assert!(
        generate_block.contains(
            "render_local_module_imports(module, &project_modules, &project_codegen_code)"
        )
    );
    assert!(generate_block.contains("publicize_generated_module_source(&rust_source)"));
    assert!(generate_block.contains("required_features.extend(codegen_result.required_features)"));
    assert!(!generate_block.contains("assert_output_drained("));
    assert!(!generate_block.contains("emitter.output"));
    assert!(!generate_block.contains("module_import_prelude"));
}
