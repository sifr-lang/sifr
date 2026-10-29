use super::*;

#[test]
fn builtin_open_text_roots_text_handle_support() {
    let mut emitter = RustEmitter::new();
    let lowered = intrinsics::LoweredIntrinsic {
        expr: RustExpr::Ident("__value".to_string()),
        required_feature: None,
        additional_required_features: &[],
    };

    emitter.apply_intrinsic_registry_side_effects(CompilerIntrinsicId::OpenText, &lowered);

    assert!(!emitter.runtime_needs.file_handles());
    assert!(emitter.used_stdlib_modules.contains("sifr.io"));
    let io_roots = emitter
        .imported_stdlib_names
        .get("sifr.io")
        .expect("sifr.io roots");
    assert!(io_roots.contains("BinaryFileHandle"));
    assert!(io_roots.contains("TextFileHandle"));
}
