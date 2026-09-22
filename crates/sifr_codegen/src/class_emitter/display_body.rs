use crate::helpers::collect_mutated_vars_with_sigs;
use crate::{RustEmitter, RustExpr, RustLiteral, RustStmt};
use sifr_ir::HirFunction;

impl RustEmitter {
    pub(crate) fn lower_display_body_for_custom_str(
        &mut self,
        str_func: &HirFunction,
    ) -> Vec<RustStmt> {
        let saved_body_analysis = std::mem::take(&mut self.body_analysis);
        let saved_last_use_moves = std::mem::take(&mut self.last_use_move_exprs);
        let saved_string_caches = std::mem::take(&mut self.string_char_cache_vars);
        let saved_required_caches = std::mem::take(&mut self.string_char_cache_required_names);
        let saved_loop_caches = std::mem::take(&mut self.string_char_cache_loop_local_names);
        let saved_borrowed_params = std::mem::take(&mut self.borrowed_params);
        let saved_mut_borrowed_params = std::mem::take(&mut self.mut_borrowed_params);
        let saved_recursive_views = std::mem::take(&mut self.recursive_option_borrowed_views);
        let saved_display_ctx = self.emission_ctx.in_display_impl;
        let saved_return_type = self.current_return_type.clone();
        let saved_mutated = self.mutated_vars.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();
        let saved_checked_place_read_witnesses =
            std::mem::take(&mut self.checked_place_read_witnesses);
        let saved_nonempty_list_bindings = std::mem::take(&mut self.nonempty_list_bindings);
        let saved_option_unwrapped_vars = std::mem::take(&mut self.option_unwrapped_vars);

        self.emission_ctx.in_display_impl = true;
        self.current_return_type = Some(str_func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&str_func.body, &self.func_signatures);
        self.local_binding_types.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        for param in &str_func.params {
            self.local_binding_types
                .insert(param.name.clone(), param.ty.clone());
            if param.convention.is_shared_borrow() {
                self.borrowed_params.insert(param.name.clone());
            }
            if param.convention.is_mut_borrow() {
                self.mut_borrowed_params.insert(param.name.clone());
            }
        }
        self.register_local_body_binding_types(&str_func.body);
        (self.body_analysis, self.last_use_move_exprs) =
            crate::body_analysis::BodyAnalysis::build(str_func, &self.func_signatures);
        let mut body = self.prepare_string_char_cache_stmts(
            str_func,
            &crate::helpers::collect_reassigned_vars(&str_func.body),
        );
        for (stmt_index, stmt) in str_func.body.iter().enumerate() {
            let lowered = self.capture_structured_stmts(|inner| {
                inner.emit_stmt_with_following(stmt, Some(&str_func.body[stmt_index + 1..]));
            });
            body.extend(lowered);
        }

        if !matches!(body.last(), Some(RustStmt::Return(_))) {
            body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(RustLiteral::Unit)],
            })));
        }

        self.body_analysis = saved_body_analysis;
        self.last_use_move_exprs = saved_last_use_moves;
        self.string_char_cache_vars = saved_string_caches;
        self.string_char_cache_required_names = saved_required_caches;
        self.string_char_cache_loop_local_names = saved_loop_caches;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.recursive_option_borrowed_views = saved_recursive_views;
        self.emission_ctx.in_display_impl = saved_display_ctx;
        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated;
        self.local_binding_types = saved_local_binding_types;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        self.checked_place_read_witnesses = saved_checked_place_read_witnesses;
        self.nonempty_list_bindings = saved_nonempty_list_bindings;
        self.option_unwrapped_vars = saved_option_unwrapped_vars;
        body
    }
}
