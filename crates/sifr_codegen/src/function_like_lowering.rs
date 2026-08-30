use crate::helpers::{collect_mutated_vars_with_sigs, collect_reassigned_vars};
use crate::stmt_support_emitter::performance_lowering_gate::stmt_needs_performance_lowering;
use crate::{
    ClassScope, RustEmitter, RustStmt, ScopeContext, is_simple_stmt_candidate,
    try_lower_simple_stmt_with_scope_result_and_bindings,
};
use sifr_ir::{HirFunction, HirStmt};

impl RustEmitter {
    pub(crate) fn lower_function_like_body<F>(
        &mut self,
        func: &HirFunction,
        missing_panic_message: &str,
        failed_panic_message: &str,
        mut fallback: F,
    ) -> Vec<RustStmt>
    where
        F: FnMut(&mut Self, &HirStmt) -> Option<Vec<RustStmt>>,
    {
        let saved_return_type = self.current_return_type.clone();
        let saved_mutated_vars = self.mutated_vars.clone();
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_string_char_cache_vars = self.string_char_cache_vars.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();
        let saved_checked_place_read_witnesses =
            std::mem::take(&mut self.checked_place_read_witnesses);
        let saved_nonempty_list_bindings = std::mem::take(&mut self.nonempty_list_bindings);
        let saved_option_unwrapped_vars = std::mem::take(&mut self.option_unwrapped_vars);

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.local_binding_types.clear();
        self.string_char_cache_vars.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        for param in &func.params {
            if param.convention.is_shared_borrow()
                && !crate::helpers::is_copy_type_for_codegen(&param.ty)
            {
                self.borrowed_params.insert(param.name.clone());
            }
            if param.convention.is_mut_borrow()
                && !crate::helpers::is_copy_type_for_codegen(&param.ty)
            {
                self.mut_borrowed_params.insert(param.name.clone());
            }
            self.local_binding_types
                .insert(param.name.clone(), param.ty.clone());
        }
        self.register_local_body_binding_types(&func.body);

        let scope_ctx = ScopeContext {
            function_return_type: self.current_return_type.clone(),
            in_generator_closure: false,
            in_display_impl: false,
            in_loop_with_else: false,
            class_scope: ClassScope::Inside,
        };

        let reassigned_vars = collect_reassigned_vars(&func.body);
        let mut lowered_body = self.prepare_string_char_cache_stmts(func, &reassigned_vars);
        for stmt in &func.body {
            self.lowering_stats.stmt_total += 1;
            if is_simple_stmt_candidate(stmt) {
                self.lowering_stats.stmt_candidate_total += 1;
            }
            let simple_lowered = if stmt_needs_performance_lowering(stmt)
                || Self::stmt_defines_nonempty_list(stmt)
            {
                Ok(None)
            } else {
                try_lower_simple_stmt_with_scope_result_and_bindings(
                    stmt,
                    &self.mutated_vars,
                    &self.borrowed_params,
                    &self.mut_borrowed_params,
                    &self.local_binding_types,
                    &self.recursive_fields,
                    &scope_ctx,
                )
            };
            match simple_lowered {
                Ok(Some(lowered)) => {
                    self.lowering_stats.expr_candidate_total += 1;
                    self.lowering_stats.expr_candidate_structured += 1;
                    self.lowering_stats.stmt_structured += 1;
                    self.lowering_stats.stmt_candidate_structured += 1;
                    lowered_body.extend(
                        lowered
                            .into_iter()
                            .map(|stmt| self.rewrite_stdlib_constant_idents_in_stmt(stmt)),
                    );
                }
                Ok(None) => {
                    let checked_place_lowered = self
                        .lower_checked_place_mutation_stmt_for_ir(stmt)
                        .unwrap_or_else(|error| {
                            panic!("{failed_panic_message}: {stmt:?}; error={error}")
                        });
                    if let Some(lowered) = checked_place_lowered {
                        self.lowering_stats.expr_candidate_total += 1;
                        self.lowering_stats.expr_candidate_structured += 1;
                        self.lowering_stats.stmt_structured += 1;
                        self.lowering_stats.stmt_candidate_structured += 1;
                        lowered_body.extend(lowered);
                    } else if let Some(lowered) = fallback(self, stmt) {
                        self.lowering_stats.expr_candidate_total += 1;
                        self.lowering_stats.expr_candidate_structured += 1;
                        self.lowering_stats.stmt_structured += 1;
                        self.lowering_stats.stmt_candidate_structured += 1;
                        lowered_body.extend(lowered);
                    } else {
                        panic!("{missing_panic_message}: {stmt:?}");
                    }
                }
                Err(_) => {
                    self.lowering_stats.stmt_lowering_errors += 1;
                    panic!("{failed_panic_message}: {stmt:?}");
                }
            }
        }

        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated_vars;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.local_binding_types = saved_local_binding_types;
        self.string_char_cache_vars = saved_string_char_cache_vars;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        self.checked_place_read_witnesses = saved_checked_place_read_witnesses;
        self.nonempty_list_bindings = saved_nonempty_list_bindings;
        self.option_unwrapped_vars = saved_option_unwrapped_vars;
        lowered_body
    }
}
