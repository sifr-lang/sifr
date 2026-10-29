use super::{HirExpr, RustEmitter, RustExpr, Type};

impl RustEmitter {
    /// Mapping defaults cross an owned value boundary, including nested calls
    /// dispatched directly through the builtin method registry.
    pub(crate) fn adapt_owned_mapping_default(
        &mut self,
        object_ty: &Type,
        method: &str,
        args: &[HirExpr],
        lowered_args: &mut [crate::RustExpr],
    ) {
        if let Type::Dict(_, value_ty) = object_ty.resolve_alias()
            && matches!(method, "get" | "pop" | "remove")
            && let ([_, argument], [_, lowered]) = (args, lowered_args)
        {
            *lowered =
                self.coerce_collection_element_for_registry(value_ty, argument, lowered.clone());
            *lowered = self.materialize_reusable_value_for_ir(argument, lowered.clone());
        }
    }

    pub(crate) fn coerce_collection_element_for_registry(
        &self,
        target_ty: &Type,
        argument: &HirExpr,
        lowered: RustExpr,
    ) -> RustExpr {
        let source_ty = self.effective_registry_expr_ty(argument);
        self.consuming_value_conversion_for_ir(target_ty, &source_ty, lowered)
    }

    pub(crate) fn materialize_borrowed_parameter_for_owned_boundary(
        &self,
        argument: &HirExpr,
        lowered: RustExpr,
    ) -> RustExpr {
        if matches!(argument, HirExpr::Name { name, .. }
            if self.borrowed_params.contains(name) || self.mut_borrowed_params.contains(name))
        {
            crate::ownership_plan::materialize_owned_value(argument.ty(), lowered)
        } else {
            lowered
        }
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
#[path = "../../tests/unit/intrinsic_method_emitters/narrowing_helpers/tests.rs"]
mod tests;
