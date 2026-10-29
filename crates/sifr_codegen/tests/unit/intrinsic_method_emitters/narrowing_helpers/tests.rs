#[test]
fn intrinsic_emit_wrapper_layer_is_absent() {
    let src = include_str!("../../../../src/intrinsic_method_emitters.rs");
    let prod_src = src.split("\n#[cfg(test)]").next().unwrap_or(src);
    assert!(!prod_src.contains("pub(crate) fn emit_intrinsic_call("));
    assert!(!prod_src.contains("pub(crate) fn try_emit_intrinsic_via_registry("));
}

#[test]
fn registry_arg_lowering_avoids_inline_rawcode_paths() {
    let collection_methods_src =
        include_str!("../../../../src/intrinsic_method_emitters/collection_methods.rs");
    let recursive_exprs_src =
        include_str!("../../../../src/intrinsic_method_emitters/recursive_exprs.rs");
    let field_rewrites_src =
        include_str!("../../../../src/expr_render_helpers/field_and_stdlib_rewrites.rs");
    let prod_src = [
        collection_methods_src,
        recursive_exprs_src,
        field_rewrites_src,
    ]
    .join("\n");

    assert!(collection_methods_src.contains("pub(crate) fn try_lower_registry_expr_strict("));
    assert!(collection_methods_src.contains("pub(crate) fn try_lower_registry_exprs_strict("));
    assert!(recursive_exprs_src.contains("pub(crate) fn try_lower_registry_expr_recursive("));
    assert!(field_rewrites_src.contains("pub(crate) fn try_lower_registry_expr_result("));
    let helper_defs = prod_src
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("fn try_lower_registry_expr")
                || trimmed.starts_with("pub(crate) fn try_lower_registry_expr")
        })
        .count();
    assert_eq!(helper_defs, 4, "unexpected registry expr helper set");
    assert!(!prod_src.contains("lower_registry_expr_with_string_path"));
    assert!(!prod_src.contains("render_expr_via_string_only("));
}
