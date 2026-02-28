use crate::helpers::MUTATING_METHODS;
use crate::RustEmitter;
use sifr_hir::HirExpr;
use sifr_type_system::{FunctionType, ParamConvention, Type};

impl RustEmitter {
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
        let obj_ty = object.ty();
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
            self.emit_class_method_call_with_conventions(object, class_name, method, args);
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

        self.write("(");
        self.emit_expr(object);
        self.write(&format!(".{method})("));
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.emit_expr(arg);
        }
        self.write(")");
        true
    }

    fn emit_class_method_call_with_conventions(
        &mut self,
        object: &HirExpr,
        class_name: &str,
        method: &str,
        args: &[HirExpr],
    ) {
        self.emit_expr(object);
        self.write(&format!(".{method}("));

        let method_key = format!("{class_name}::{method}");
        let method_info = self.func_signatures.get(&method_key).cloned();
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            if let Some((ref params, _)) = method_info {
                if let Some((param_ty, convention)) = params.get(i) {
                    if *convention == ParamConvention::Borrow
                        && matches!(param_ty, Type::TypeVar(_))
                    {
                        self.write("&(");
                        self.emit_expr(arg);
                        self.write(")");
                        continue;
                    }
                    self.emit_borrow_prefix(*convention, arg.ty(), Some(param_ty));
                    self.emit_expr(arg);
                    continue;
                }
            }
            self.emit_expr(arg);
        }
        self.write(")");
    }

    fn emit_generic_method_call(&mut self, object: &HirExpr, method: &str, args: &[HirExpr]) {
        self.emit_expr(object);
        self.write(&format!(".{method}("));
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.emit_expr(arg);
        }
        self.write(")");
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
        // Own convention: pass by value (move), no prefix needed
        if convention == ParamConvention::Own {
            return;
        }
        // If the parameter type is a TypeVar, always emit the borrow prefix
        // because the generated Rust signature uses &T for borrowed TypeVar params
        let is_generic_param = param_ty.is_some_and(|t| matches!(t, Type::TypeVar(_)));
        // Copy types are passed by value regardless of convention unless the
        // parameter is generic (TypeVar). Base this on the parameter type when
        // available (not the argument expression type), because constructor
        // expressions can carry payload types that differ from the parameter.
        let borrow_decision_ty = param_ty.unwrap_or(arg_ty);
        if !is_generic_param
            && borrow_decision_ty.ownership() == sifr_type_system::OwnershipKind::Copy
        {
            return;
        }
        // If the argument is already a borrowed parameter (&T), don't add another borrow.
        // This handles the case where a Callable call passes a borrowed param:
        //   fn apply(f: Callable[[list[int]], int], items: &Vec<i64>) { f(items) }
        // Here items is already &Vec<i64>, so we pass it as-is (no extra &).
        //
        // Similarly, if the argument is already a mutably borrowed parameter (&mut T),
        // don't add another &mut. E.g.:
        //   fn heapify(data: &mut Vec<i64>) { _sift_down(data, 0, n); }
        // Here data is already &mut Vec<i64>; passing &mut data would be &&mut Vec<i64> error.
        if let Some(name) = arg_name {
            if self.borrowed_params.contains(name) && convention == ParamConvention::Borrow {
                return; // already &T, no additional borrow needed
            }
            if self.mut_borrowed_params.contains(name) {
                if convention == ParamConvention::MutBorrow {
                    return; // already &mut T, no additional &mut needed
                }
                if convention == ParamConvention::Borrow {
                    return; // &mut T -> &T is implicit reborrow in Rust; no extra & needed
                }
            }
        }
        match convention {
            ParamConvention::Borrow => self.write("&"),
            ParamConvention::MutBorrow => self.write("&mut "),
            ParamConvention::Own => {} // no prefix -- pass by value (move)
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
