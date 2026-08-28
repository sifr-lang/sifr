use super::{HirExpr, RustEmitter, RustExpr, Type};

impl RustEmitter {
    pub(crate) fn adapt_collection_method_args_for_ir(
        &self,
        object_ty: &Type,
        method: &str,
        args: &[HirExpr],
        arg_exprs: &mut [RustExpr],
    ) {
        let object_ty = crate::resolve_alias_type_for_plain_call(object_ty);
        let collection_element_target = match (object_ty, method) {
            (Type::List(element_ty), "append" | "appendleft") => Some((0, element_ty.as_ref())),
            (Type::List(element_ty), "insert") => Some((1, element_ty.as_ref())),
            (Type::Set(element_ty), "add") => Some((0, element_ty.as_ref())),
            (Type::Dict(_, value_ty), "setdefault") => Some((1, value_ty.as_ref())),
            _ => None,
        };
        if let Some((index, target_ty)) = collection_element_target {
            if let (Some(argument), Some(lowered_arg)) = (args.get(index), arg_exprs.get_mut(index))
            {
                *lowered_arg = self.coerce_collection_element_for_registry(
                    target_ty,
                    argument,
                    lowered_arg.clone(),
                );
            }
        }

        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && let (Some(argument), Some(lowered_arg)) = (args.first(), arg_exprs.first_mut())
        {
            if matches!(argument.ty(), Type::TypeVar(_)) {
                *lowered_arg = RustExpr::MethodCall {
                    receiver: Box::new(lowered_arg.clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
            *lowered_arg = Self::clone_owned_append_arg_expr_for_ir(argument, lowered_arg.clone());
        }

        if matches!(object_ty, Type::List(_)) && method == "insert" && args.len() >= 2 {
            let Some(HirExpr::Name { name, ty, .. }) = args.get(1) else {
                return;
            };
            let needs_clone = (self.borrowed_params.contains(name.as_str())
                || self.mut_borrowed_params.contains(name.as_str()))
                && ty.ownership() != sifr_type_system::OwnershipKind::Copy;
            if needs_clone && let Some(lowered_arg) = arg_exprs.get_mut(1) {
                *lowered_arg = RustExpr::MethodCall {
                    receiver: Box::new(lowered_arg.clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
        }
    }
}
