use crate::{intrinsics, methods, RustEmitter};
use sifr_hir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    /// Check if a name is a stdlib constant.
    pub(crate) fn is_stdlib_constant(&self, name: &str) -> bool {
        matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
            && self.intrinsic_functions.contains(name)
    }

    /// Emit a stdlib constant value.
    pub(crate) fn emit_stdlib_constant(&mut self, name: &str) {
        match name {
            "pi" => self.write("std::f64::consts::PI"),
            "e" => self.write("std::f64::consts::E"),
            "tau" => self.write("std::f64::consts::TAU"),
            "inf" => self.write("f64::INFINITY"),
            "nan" => self.write("f64::NAN"),
            _ => self.write(name),
        }
    }

    /// Emit an intrinsic function call with the correct Rust code.
    pub(crate) fn emit_intrinsic_call(&mut self, func: &str, args: &[HirExpr]) {
        if self.try_emit_intrinsic_via_registry(func, args) {
            return;
        }

        // Unknown intrinsic name: emit as regular function call.
        self.write(func);
        self.write("(");
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.emit_expr(arg);
        }
        self.write(")");
    }

    pub(crate) fn try_emit_intrinsic_via_registry(&mut self, func: &str, args: &[HirExpr]) -> bool {
        let ir_args = args
            .iter()
            .map(|arg| self.lower_registry_expr_with_fallback(arg))
            .collect::<Vec<_>>();
        let Some(lowered) = intrinsics::lower_intrinsic(func, &ir_args) else {
            return false;
        };

        if matches!(
            func,
            "builtin_open"
                | "open_file"
                | "file_read"
                | "file_write"
                | "file_readline"
                | "file_readlines"
                | "file_close"
                | "file_read_bytes"
                | "file_write_bytes"
        ) {
            self.runtime_needs.needs_file_handles = true;
        }
        if func == "builtin_open" {
            self.used_stdlib_modules.insert("io".to_string());
        }
        if matches!(func, "set_global_level" | "get_global_level") {
            self.runtime_needs.needs_logging_state = true;
        }

        if let Some(required_crate) = lowered.required_crate {
            self.intrinsic_registry_crates
                .insert(required_crate.to_string());
        }
        for required_crate in lowered.additional_required_crates {
            self.intrinsic_registry_crates
                .insert((*required_crate).to_string());
        }

        self.write(&crate::render_expr(&lowered.expr));
        true
    }

    pub(crate) fn try_emit_method_via_registry(
        &mut self,
        object_ty: &Type,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> bool {
        let is_deque_data_field = self.is_deque_data_field(object);
        let object_expr = self.lower_registry_expr_with_fallback(object);
        let mut arg_exprs = args
            .iter()
            .map(|arg| self.lower_registry_expr_with_fallback(arg))
            .collect::<Vec<_>>();

        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && !args.is_empty()
        {
            // Clone TypeVar list args to avoid move issues.
            if matches!(args[0].ty(), Type::TypeVar(_)) {
                arg_exprs[0] = crate::RustExpr::MethodCall {
                    receiver: Box::new(arg_exprs[0].clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
        }

        if matches!(object_ty, Type::List(_)) && method == "insert" && args.len() >= 2 {
            // Clone borrowed/mut-borrowed move-owned values.
            let needs_clone = if let HirExpr::Name { name, ty } = &args[1] {
                (self.borrowed_params.contains(name.as_str())
                    || self.mut_borrowed_params.contains(name.as_str()))
                    && ty.ownership() != sifr_type_system::OwnershipKind::Copy
            } else {
                false
            };
            if needs_clone {
                arg_exprs[1] = crate::RustExpr::MethodCall {
                    receiver: Box::new(arg_exprs[1].clone()),
                    method: "clone".to_string(),
                    args: vec![],
                };
            }
        }

        let Some(lowered) = methods::lower_method_with_context(
            object_ty,
            method,
            &object_expr,
            &arg_exprs,
            is_deque_data_field,
        ) else {
            return false;
        };
        self.write(&crate::render_expr(&lowered.expr));
        true
    }

    fn lower_registry_expr_with_fallback(&mut self, expr: &HirExpr) -> crate::RustExpr {
        if self.should_force_render_fallback(expr) {
            return crate::RustExpr::RawCode(self.render_expr_with_lowered_fallback(expr));
        }
        match crate::try_lower_leaf_expr_result(expr) {
            Ok(Some(lowered_expr)) => lowered_expr,
            Ok(None) => crate::RustExpr::RawCode(self.render_expr_with_lowered_fallback(expr)),
            Err(_) => {
                self.lowering_stats.expr_lowering_errors += 1;
                crate::RustExpr::RawCode(self.render_expr_with_lowered_fallback(expr))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn emit_intrinsic_call_has_no_legacy_match_dispatch() {
        let src = include_str!("intrinsic_method_emitters.rs");
        let start = src
            .find("pub(crate) fn emit_intrinsic_call")
            .expect("emit_intrinsic_call should exist");
        let end = src
            .find("pub(crate) fn try_emit_intrinsic_via_registry")
            .expect("try_emit_intrinsic_via_registry should exist");
        let emit_block = &src[start..end];
        assert!(!emit_block.contains("match func"));
    }
}
