use super::{HirExpr, ParamConvention, RustEmitter, Type};

impl RustEmitter {
    pub(crate) fn apply_registry_method_arg_convention(
        &self,
        arg: &HirExpr,
        param_ty: &Type,
        convention: ParamConvention,
        mut lowered_arg: crate::RustExpr,
    ) -> crate::RustExpr {
        let resolved_param = crate::resolve_alias_type_for_plain_call(param_ty);
        if let Type::AsyncCallable(params, _, _) = resolved_param {
            lowered_arg = Self::send_async_callable_adapter(lowered_arg, params.len());
        }
        let effective_arg_ty = self.effective_registry_expr_ty(arg);
        let arg_is_option = crate::helpers::is_option_type(&effective_arg_ty);
        let borrowed_name_arg = matches!(arg, HirExpr::Name { name, .. }
            if self.borrowed_params.contains(name)
                || self.mut_borrowed_params.contains(name));
        let unadapted_option_arg = lowered_arg.clone();
        if convention.is_owned() {
            (lowered_arg, _) = self.adapt_consuming_call_argument_for_ir(
                param_ty,
                &effective_arg_ty,
                lowered_arg,
                borrowed_name_arg,
            );
        } else {
            lowered_arg = Self::flatten_option_argument_for_ir(
                arg,
                param_ty,
                &effective_arg_ty,
                convention,
                lowered_arg,
            );
        }
        let option_value_adapted = lowered_arg != unadapted_option_arg;
        if crate::helpers::is_option_type(param_ty)
            && !arg_is_option
            && !matches!(arg, HirExpr::NoneLiteral)
        {
            if !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty) {
                lowered_arg = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
            lowered_arg = crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_arg],
            };
        } else if arg_is_option
            && !crate::helpers::is_option_type(param_ty)
            && !option_value_adapted
        {
            if !crate::helpers::is_copy_type_for_codegen(&effective_arg_ty) {
                lowered_arg = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_arg))),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
        }

        let requires_shared_borrow = convention.is_shared_borrow()
            && (!crate::helpers::is_copy_type_for_codegen(param_ty)
                || matches!(resolved_param, Type::TypeVar(_)));
        let requires_mut_borrow = convention.is_mut_borrow()
            && (!crate::helpers::is_copy_type_for_codegen(param_ty)
                || matches!(resolved_param, Type::TypeVar(_)));

        if requires_shared_borrow
            && !self.arg_is_already_borrowed_for_registry_call(arg, &lowered_arg)
        {
            lowered_arg = crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_arg),
            };
        } else if requires_mut_borrow
            && !self.arg_is_already_mut_borrowed_for_registry_call(arg, &lowered_arg)
        {
            lowered_arg = crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(lowered_arg),
            };
        }
        lowered_arg
    }
}
