use crate::RustEmitter;
use sifr_ir::HirExpr;
#[cfg(test)]
use sifr_type_system::Type;

impl RustEmitter {
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
