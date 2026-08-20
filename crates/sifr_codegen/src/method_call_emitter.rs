use crate::RustEmitter;
use sifr_ir::HirExpr;
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    pub(crate) fn borrow_prefix_for_name(
        &self,
        convention: ParamConvention,
        arg_ty: &Type,
        param_ty: Option<&Type>,
        arg_name: Option<&str>,
    ) -> Option<&'static str> {
        // Own convention: pass by value (move), no prefix needed
        if convention.is_owned() {
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
            if self.borrowed_params.contains(name) && convention.is_shared_borrow() {
                return None;
            }
            if self.mut_borrowed_params.contains(name) {
                if convention.is_mut_borrow() {
                    return None;
                }
                if convention.is_shared_borrow() {
                    return None;
                }
            }
        }
        if convention.is_mut_borrow() {
            Some("&mut ")
        } else if convention.is_shared_borrow() {
            Some("&")
        } else {
            None
        }
    }

    /// Check if an expression is a call to a generator function
    pub(crate) fn is_generator_call(&self, expr: &HirExpr) -> bool {
        matches!(
            expr,
            HirExpr::Call { func, .. } | HirExpr::GenericCall { func, .. }
                if self.generator_functions.contains(
                    crate::stmt_support_emitter::canonical_plain_call_name_for_ir(func)
                )
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_generator_calls_use_the_canonical_function_name() {
        let mut emitter = RustEmitter::new();
        emitter.generator_functions.insert("produce".to_string());
        let expression = HirExpr::GenericCall {
            func: "produce::<i64>".to_string(),
            type_args: vec![Type::Int],
            args: Vec::new(),
            mutable_arg_places: Vec::new(),
            ty: Type::Iterator(Box::new(Type::Int)),
        };

        assert!(emitter.is_generator_call(&expression));
    }
}
