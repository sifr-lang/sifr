use crate::helpers::MUTATING_METHODS;
use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::{FunctionType, ParamConvention, Type};

impl RustEmitter {
    fn try_emit_lowered_callable_field_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> bool {
        let Some(object_expr) = self.try_lower_registry_expr_strict(object) else {
            return false;
        };
        let Some(arg_exprs) = self.try_lower_registry_exprs_strict(args) else {
            return false;
        };
        let lowered = crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Field {
                expr: Box::new(object_expr),
                field: method.to_string(),
            }),
            args: arg_exprs,
        };
        self.write_registry_expr(&lowered);
        true
    }

    fn try_emit_lowered_class_method_call_with_conventions(
        &mut self,
        object: &HirExpr,
        class_name: &str,
        method: &str,
        args: &[HirExpr],
        class_methods: &[(String, FunctionType)],
    ) -> bool {
        let Some(object_expr) = self.try_lower_registry_expr_strict(object) else {
            return false;
        };
        let method_key = format!("{class_name}::{method}");
        let method_params: Option<Vec<(Type, ParamConvention)>> = self
            .func_signatures
            .get(&method_key)
            .map(|(params, _)| params.clone())
            .or_else(|| {
                class_methods
                    .iter()
                    .find(|(method_name, _)| method_name == method)
                    .map(|(_, fty)| {
                        let self_offset = usize::from(
                            fty.params
                                .first()
                                .is_some_and(|(param_name, _, _)| param_name == "self"),
                        );
                        fty.params
                            .iter()
                            .skip(self_offset)
                            .map(|(_, ty, conv)| (ty.clone(), *conv))
                            .collect::<Vec<_>>()
                    })
            });
        let mut lowered_args = Vec::with_capacity(args.len());
        for (i, arg) in args.iter().enumerate() {
            let mut lowered = match self.try_lower_registry_expr_strict(arg) {
                Some(expr) => expr,
                None => return false,
            };
            if let Some(params) = method_params.as_ref() {
                if let Some((param_ty, convention)) = params.get(i) {
                    if *convention == ParamConvention::Borrow
                        && matches!(param_ty, Type::TypeVar(_))
                    {
                        lowered = crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(lowered),
                        };
                        lowered_args.push(lowered);
                        continue;
                    }
                    lowered = self.apply_borrow_convention_expr(
                        *convention,
                        arg.ty(),
                        Some(param_ty),
                        Self::name_from_expr(arg),
                        lowered,
                    );
                }
            }
            lowered_args.push(lowered);
        }

        let lowered = crate::RustExpr::MethodCall {
            receiver: Box::new(object_expr),
            method: method.to_string(),
            args: lowered_args,
        };
        self.write_registry_expr(&lowered);
        true
    }

    fn try_emit_lowered_generic_method_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> bool {
        let Some(object_expr) = self.try_lower_registry_expr_strict(object) else {
            return false;
        };
        let Some(arg_exprs) = self.try_lower_registry_exprs_strict(args) else {
            return false;
        };
        let lowered = crate::RustExpr::MethodCall {
            receiver: Box::new(object_expr),
            method: method.to_string(),
            args: arg_exprs,
        };
        self.write_registry_expr(&lowered);
        true
    }

    fn name_from_expr(expr: &HirExpr) -> Option<&str> {
        if let HirExpr::Name { name, .. } = expr {
            return Some(name);
        }
        None
    }

    fn borrow_prefix_for_name(
        &self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
        arg_name: Option<&str>,
    ) -> Option<&'static str> {
        // Own convention: pass by value (move), no prefix needed
        if convention == ParamConvention::Own {
            return None;
        }
        // If the parameter type is a TypeVar, always emit the borrow prefix
        // because the generated Rust signature uses &T for borrowed TypeVar params.
        let is_generic_param = param_ty.is_some_and(|t| matches!(t, Type::TypeVar(_)));
        // Copy types are passed by value regardless of convention unless the
        // parameter is generic (TypeVar).
        let borrow_decision_ty = param_ty.unwrap_or(arg_ty);
        if !is_generic_param
            && borrow_decision_ty.ownership() == sifr_type_system::OwnershipKind::Copy
        {
            return None;
        }
        // Avoid borrowing an already borrowed parameter again.
        if let Some(name) = arg_name {
            if self.borrowed_params.contains(name) && convention == ParamConvention::Borrow {
                return None;
            }
            if self.mut_borrowed_params.contains(name) {
                if convention == ParamConvention::MutBorrow {
                    return None;
                }
                if convention == ParamConvention::Borrow {
                    return None;
                }
            }
        }
        match convention {
            ParamConvention::Borrow => Some("&"),
            ParamConvention::MutBorrow => Some("&mut "),
            ParamConvention::Own => None,
        }
    }

    fn apply_borrow_convention_expr(
        &self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
        arg_name: Option<&str>,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        match self.borrow_prefix_for_name(convention, arg_ty, param_ty, arg_name) {
            Some("&") => crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered),
            },
            Some("&mut ") => crate::RustExpr::Ref {
                mutable: true,
                expr: Box::new(lowered),
            },
            _ => lowered,
        }
    }

    /// Check if an expression is a call to a generator function
    pub(crate) fn is_generator_call(&self, expr: &HirExpr) -> bool {
        if let HirExpr::Call { func, .. } = expr {
            self.generator_functions.contains(func)
        } else {
            false
        }
    }

    pub(crate) fn emit_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        // For mutating methods on self.field, suppress .clone() so mutations are applied
        // to the actual field, not a temporary clone.
        let is_self_field = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
        let needs_self_field_clone_suppression =
            is_self_field && MUTATING_METHODS.contains(&method);
        let obj_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        if self.try_emit_method_via_registry(obj_ty, object, method, args) {
            return;
        }
        if needs_self_field_clone_suppression {
            self.pending_self_field_clone_suppression += 1;
        }
        if let Type::Class {
            name: class_name,
            fields,
            methods,
            ..
        } = obj_ty
        {
            if self.emit_class_callable_field_call(object, method, args, fields, methods) {
                return;
            }
            self.emit_class_method_call_with_conventions(object, class_name, method, args, methods);
            return;
        }

        self.emit_generic_method_call(object, method, args);
    }

    fn emit_class_callable_field_call(
        &mut self,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
        fields: &[(String, Type)],
        methods: &[(String, FunctionType)],
    ) -> bool {
        let is_callable_field = !methods.iter().any(|(name, _)| name == method)
            && fields
                .iter()
                .any(|(name, ty)| name == method && matches!(ty, Type::Callable(..)));
        if !is_callable_field {
            return false;
        }

        if self.try_emit_lowered_callable_field_call(object, method, args) {
            return true;
        }
        panic!(
            "structured callable-field method call lowering missing for production path: object={object:?}, method={method}, args={args:?}"
        );
    }

    fn emit_class_method_call_with_conventions(
        &mut self,
        object: &HirExpr,
        class_name: &str,
        method: &str,
        args: &[HirExpr],
        class_methods: &[(String, FunctionType)],
    ) {
        if self.try_emit_lowered_class_method_call_with_conventions(
            object,
            class_name,
            method,
            args,
            class_methods,
        ) {
            return;
        }
        panic!(
            "structured class-method call lowering missing for production path: class={class_name}, method={method}, object={object:?}, args={args:?}"
        );
    }

    fn emit_generic_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        if self.try_emit_lowered_generic_method_call(object, method, args) {
            return;
        }
        panic!(
            "structured generic method call lowering missing for production path: method={method}, object={object:?}, args={args:?}"
        );
    }

    /// Emit `&` or `&mut` prefix for a function argument based on parameter convention.
    /// Copy types never get a borrow prefix (they're passed by value),
    /// unless the parameter type is a `TypeVar` (generic), in which case we always borrow.
    pub(crate) fn emit_borrow_prefix(
        &mut self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
    ) {
        self.emit_borrow_prefix_for_name(convention, arg_ty, param_ty, None);
    }

    pub(crate) fn emit_borrow_prefix_for_name(
        &mut self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
        arg_name: Option<&str>,
    ) {
        if let Some(prefix) = self.borrow_prefix_for_name(convention, arg_ty, param_ty, arg_name) {
            for ch in prefix.chars() {
                self.output.push(ch);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn emit_method_call_uses_registry_first_without_old_type_match() {
        let src = include_str!("method_call_emitter.rs");
        let start = src
            .find("pub(crate) fn emit_method_call")
            .expect("emit_method_call should exist");
        let end = src
            .find("fn emit_class_callable_field_call")
            .expect("class callable helper should exist");
        let emit_block = &src[start..end];
        assert!(!emit_block.contains("match (obj_ty, method)"));
        assert!(!emit_block.contains("tuple.count() not fully supported"));
    }

}
