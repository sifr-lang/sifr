use crate::{RustEmitter, RustExpr};
use sifr_ir::{HirClass, HirExpr, HirFunction};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn is_some_call_expr(expr: &RustExpr) -> bool {
        matches!(
            expr,
            RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    || matches!(func.as_ref(), RustExpr::Ident(name) if name == "Some")
        )
    }

    pub(crate) fn is_box_new_call_expr(expr: &RustExpr) -> bool {
        matches!(
            expr,
            RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
                    || matches!(func.as_ref(), RustExpr::Ident(name) if name == "Box::new")
        )
    }

    pub(crate) fn ensure_some_box_inner(expr: RustExpr) -> RustExpr {
        match expr {
            RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    && args.len() == 1 =>
            {
                let mut args_iter = args.into_iter();
                let Some(inner) = args_iter.next() else {
                    unreachable!("Some(_) call must have exactly one argument");
                };
                if Self::is_box_new_call_expr(&inner) {
                    RustExpr::FnCall {
                        func,
                        args: vec![inner],
                    }
                } else {
                    RustExpr::FnCall {
                        func,
                        args: vec![RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Box".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![inner],
                        }],
                    }
                }
            }
            other => RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![other],
                }],
            },
        }
    }

    pub(crate) fn wrap_recursive_constructor_field_value(
        &self,
        class: &HirClass,
        method: &HirFunction,
        field_name: &str,
        field_ty: &Type,
        value_expr: &HirExpr,
        lowered_value: RustExpr,
    ) -> RustExpr {
        let is_recursive = self
            .recursive_fields
            .contains(&(class.name.clone(), field_name.to_string()));
        if !is_recursive {
            return lowered_value;
        }

        let is_boxed_constructor_param = matches!(
            value_expr,
            HirExpr::Name { name, .. }
                if method.name == "new"
                    && name == field_name
                    && method.params.iter().any(|param| param.name == *name)
        );
        if is_boxed_constructor_param {
            return lowered_value;
        }

        if crate::helpers::is_option_type(field_ty) {
            if matches!(value_expr, HirExpr::NoneLiteral) {
                return lowered_value;
            }
            return Self::ensure_some_box_inner(lowered_value);
        }

        Self::box_recursive_value_for_ir(lowered_value)
    }
}
