use super::{HirExpr, RustEmitter, RustExpr, Type};

impl RustEmitter {
    pub(crate) fn coerce_collection_element_for_registry(
        &self,
        target_ty: &Type,
        argument: &HirExpr,
        lowered: RustExpr,
    ) -> RustExpr {
        let source_ty = self.effective_registry_expr_ty(argument);
        let lowered =
            crate::helpers::flatten_option_value_for_target(target_ty, &source_ty, lowered);
        self.consuming_value_upcast_for_ir(target_ty, &source_ty, lowered)
    }
}

pub(crate) fn supports_nonempty_pop_narrowing_type_for_codegen(object_ty: &Type) -> bool {
    match crate::resolve_alias_type_for_plain_call(object_ty) {
        Type::List(_) => true,
        Type::Class { name, .. } => is_deque_class_name_for_codegen(name),
        _ => false,
    }
}

pub(crate) fn is_deque_class_name_for_codegen(name: &str) -> bool {
    name == "deque"
        || name
            .rsplit_once('.')
            .is_some_and(|(_, tail)| tail == "deque")
}

pub(crate) fn is_narrowable_pop_call_for_codegen(method: &str, args: &[HirExpr]) -> bool {
    match method {
        "pop" => matches!(args, [] | [HirExpr::IntLiteral(0)]),
        "popleft" => args.is_empty(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn intrinsic_emit_wrapper_layer_is_absent() {
        let src = include_str!("../intrinsic_method_emitters.rs");
        let prod_src = src.split("\n#[cfg(test)]").next().unwrap_or(src);
        assert!(!prod_src.contains("pub(crate) fn emit_intrinsic_call("));
        assert!(!prod_src.contains("pub(crate) fn try_emit_intrinsic_via_registry("));
    }

    #[test]
    fn registry_arg_lowering_avoids_inline_rawcode_paths() {
        let collection_methods_src = include_str!("collection_methods.rs");
        let recursive_exprs_src = include_str!("recursive_exprs.rs");
        let field_rewrites_src =
            include_str!("../expr_render_helpers/field_and_stdlib_rewrites.rs");
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
}
