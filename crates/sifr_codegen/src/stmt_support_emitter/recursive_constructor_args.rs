use super::{HirExpr, RustEmitter, RustExpr, Type};
use sifr_type_system::ParamConvention;

pub(crate) struct RecursiveOptionConstructorArgContext<'a> {
    pub(crate) ctor_class_name: Option<&'a str>,
    pub(crate) index: usize,
    pub(crate) param_ty: &'a Type,
    pub(crate) arg: &'a HirExpr,
    pub(crate) effective_arg_ty: &'a Type,
    pub(crate) convention: ParamConvention,
    pub(crate) borrowed_name_arg: bool,
}

impl RustEmitter {
    pub(crate) fn try_adapt_recursive_option_constructor_arg_for_ir(
        &self,
        context: &RecursiveOptionConstructorArgContext<'_>,
        mut lowered_arg: RustExpr,
    ) -> Option<RustExpr> {
        if !crate::helpers::is_option_type(crate::resolve_alias_type_for_plain_call(
            context.param_ty,
        )) {
            return None;
        }
        let is_recursive_ctor_param = context
            .ctor_class_name
            .and_then(|class_name| {
                self.class_field_order
                    .get(class_name)
                    .and_then(|fields| fields.get(context.index))
                    .map(|field_name| {
                        self.recursive_fields
                            .contains(&(class_name.to_owned(), field_name.clone()))
                    })
            })
            .unwrap_or(false);
        if !context.param_ty.rust_type().starts_with("Option<Box<") && !is_recursive_ctor_param {
            return None;
        }
        if matches!(context.arg, HirExpr::NoneLiteral) {
            return Some(lowered_arg);
        }

        let arg_is_option = crate::helpers::is_option_type(context.effective_arg_ty);
        let arg_is_non_copy = !crate::helpers::is_copy_type_for_codegen(context.effective_arg_ty);
        let clone_before_adaptation = context.borrowed_name_arg
            && arg_is_non_copy
            && (arg_is_option
                || !context.convention.is_owned()
                || context.param_ty.rust_type().starts_with('&'));
        if clone_before_adaptation {
            lowered_arg = RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered_arg))),
                method: "clone".to_string(),
                args: Vec::new(),
            };
        }

        let expr = if arg_is_option {
            Self::ensure_option_box_inner_for_ir(lowered_arg)
        } else {
            let param_is_owned_rust_value =
                context.convention.is_owned() && !context.param_ty.rust_type().starts_with('&');
            let inner = if !clone_before_adaptation
                && (!param_is_owned_rust_value || context.borrowed_name_arg)
                && arg_is_non_copy
            {
                RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Paren(Box::new(lowered_arg))),
                    method: "clone".to_string(),
                    args: Vec::new(),
                }
            } else {
                lowered_arg
            };
            Self::ensure_some_box_inner_for_ir(inner)
        };

        Some(expr)
    }
}
