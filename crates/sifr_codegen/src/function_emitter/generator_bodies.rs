use super::{
    HirFunction, HirStmt, RustEmitter, RustExpr, RustItem, RustLiteral, RustParam, RustStmt,
    RustType, Type, Visibility, body_contains_yield, collect_mutated_vars_with_sigs,
    collect_reassigned_vars,
    python_callback_bounds::{
        python_callback_bound_param_names, python_callback_static_param_names,
        rust_callback_bound_param_names,
    },
};
use crate::python_interop_common::python_omit_parameter_indices;
use crate::python_interop_direct::python_interop_function_body_with_retained_errors;
use crate::rust_interop_direct::rust_interop_function_body;
use std::collections::HashSet;
impl RustEmitter {
    pub(crate) fn emit_function(
        &mut self,
        func: &HirFunction,
        module_public: bool,
        test_mode: bool,
    ) {
        // In test mode, skip the main function
        if test_mode && func.name == "main" {
            return;
        }

        let saved_return_type = self.current_return_type.clone();
        let saved_mutated_vars = self.mutated_vars.clone();
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_callable_var_conventions = self.callable_var_conventions.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_string_char_cache_vars = self.string_char_cache_vars.clone();
        let saved_hoistable_static_dict_locals = self.hoistable_static_dict_locals.clone();
        let saved_none_widened_local_bindings = self.none_widened_local_bindings.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();
        let saved_sifr_int_result_local_bindings =
            self.sifr_int_result_local_bindings.borrow().clone();
        let saved_sifr_int_function_returns = self.sifr_int_function_returns.borrow().clone();
        let saved_sifr_int_result_function_returns =
            self.sifr_int_result_function_returns.borrow().clone();
        let saved_current_sifr_int_return = self.current_sifr_int_return.get();
        let saved_current_sifr_int_result_return = self.current_sifr_int_result_return.get();
        let saved_python_context_counter = self.python_context_counter;
        let saved_python_context_envelope_depth = self.python_context_envelope_depth;
        let saved_checked_place_read_witnesses =
            std::mem::take(&mut self.checked_place_read_witnesses);
        let saved_nonempty_list_bindings = std::mem::take(&mut self.nonempty_list_bindings);
        let saved_option_unwrapped_vars = std::mem::take(&mut self.option_unwrapped_vars);

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.callable_var_conventions.clear();
        self.local_binding_types.clear();
        self.string_char_cache_vars.clear();
        self.hoistable_static_dict_locals = self.collect_hoistable_static_dict_locals(func);
        self.none_widened_local_bindings.clear();
        self.python_context_counter = 0;
        self.python_context_envelope_depth = 0;
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        self.sifr_int_result_local_bindings.borrow_mut().clear();
        self.current_sifr_int_return
            .set(self.function_returns_sifr_int(&func.name));
        self.current_sifr_int_result_return.set(
            self.sifr_int_result_function_returns
                .borrow()
                .contains(&func.name),
        );
        self.register_function_scope_params(&func.name, &func.params);
        let active_function_returns = self.function_sifr_int_returns_for_body(&func.body);
        *self.sifr_int_function_returns.borrow_mut() = active_function_returns;
        self.register_local_body_binding_types(&func.body);

        let visibility = if !test_mode && module_public && func.name != "main" {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let is_generator = body_contains_yield(&func.body);
        let is_async_generator =
            is_generator && matches!(func.return_type.resolve_alias(), Type::AsyncGenerator(_, _));
        if is_generator {
            self.generator_functions.insert(func.name.clone());
        }

        let reassigned_vars = collect_reassigned_vars(&func.body);
        let mutable_param_shadows =
            Self::lower_mutable_param_shadows(&func.params, &reassigned_vars);
        self.apply_mutable_param_shadowing(&mutable_param_shadows);

        let mut callback_bound_params = python_callback_bound_param_names(func);
        let mut callback_static_params = python_callback_static_param_names(func);
        if is_generator {
            let generator_callable_params = func.params.iter().filter_map(|param| {
                matches!(
                    param.ty.resolve_alias(),
                    Type::Callable(..) | Type::AsyncCallable(..)
                )
                .then_some(param.name.clone())
            });
            callback_bound_params.extend(generator_callable_params.clone());
            callback_static_params.extend(generator_callable_params);
        }
        let rust_callback_params = rust_callback_bound_param_names(func);
        callback_bound_params.extend(rust_callback_params.iter().cloned());
        callback_static_params.extend(rust_callback_params);
        let python_omit_params = func
            .python_interop
            .first()
            .map(|declaration| python_omit_parameter_indices(declaration).collect::<HashSet<_>>())
            .unwrap_or_default();
        let params = func
            .params
            .iter()
            .enumerate()
            .map(|(param_idx, param)| {
                let rust_ty = if callback_bound_params.contains(&param.name) {
                    self.lower_python_callback_param_type(
                        &param.ty,
                        param.convention,
                        callback_static_params.contains(&param.name),
                    )
                } else {
                    self.lower_module_function_param_type(&func.name, param_idx, param)
                };
                let rust_ty = if python_omit_params.contains(&param_idx) {
                    RustType::Option(Box::new(rust_ty))
                } else {
                    rust_ty
                };
                if param.convention.is_owned() && param.convention.is_mutable() {
                    RustParam::NamedMut {
                        name: param.name.clone(),
                        ty: rust_ty,
                    }
                } else {
                    RustParam::Named {
                        name: param.name.clone(),
                        ty: rust_ty,
                    }
                }
            })
            .collect::<Vec<_>>();

        let interop_body = python_interop_function_body_with_retained_errors(
            func,
            &self.python_opaque_classes,
            &self.python_retained_callback_errors,
        )
        .or_else(|| rust_interop_function_body(func));
        let interop_body_supplied = interop_body.is_some();
        let mut lowered_body = if let Some(interop_body) = interop_body {
            interop_body
        } else if is_async_generator {
            self.lower_resumable_async_generator_function_body(func, &mutable_param_shadows)
        } else if is_generator {
            self.lower_resumable_generator_function_body(func, &mutable_param_shadows)
        } else {
            let mut lowered = Self::emit_mutable_param_shadow_stmts(&mutable_param_shadows);
            lowered.extend(self.prepare_string_char_cache_stmts(func, &reassigned_vars));
            for (stmt_index, stmt) in func.body.iter().enumerate() {
                lowered.extend(self.lower_stmt_strict_for_function_with_following(
                    stmt,
                    Some(&func.body[stmt_index + 1..]),
                    "function body statement lowering",
                ));
            }
            lowered
        };

        if !is_generator
            && !interop_body_supplied
            && Self::returns_result_none(&func.return_type)
            && !matches!(
                func.body.last(),
                Some(HirStmt::Return { .. } | HirStmt::Raise { .. })
            )
        {
            lowered_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(RustLiteral::Unit)],
            })));
        }
        if lowered_body.is_empty() {
            if self
                .lower_function_return_type(func, is_generator)
                .is_none()
            {
                lowered_body.push(RustStmt::Return(None));
            } else {
                panic!(
                    "function IR lowering produced empty body for non-unit return: {}",
                    func.name
                );
            }
        }

        for decorator in &func.decorators {
            self.body_items
                .push(RustItem::Attr(format!("// @{decorator}")));
        }
        if test_mode && func.name.starts_with("test_") {
            self.body_items.push(RustItem::Attr("#[test]".to_string()));
        }

        self.body_items.push(RustItem::Fn {
            name: func.name.clone(),
            visibility,
            type_params: self.lower_function_type_params(func),
            params,
            ret: self.lower_function_return_type(func, is_generator),
            body: lowered_body,
            is_async: func.is_async && !is_async_generator,
        });

        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated_vars;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.callable_var_conventions = saved_callable_var_conventions;
        self.local_binding_types = saved_local_binding_types;
        self.string_char_cache_vars = saved_string_char_cache_vars;
        self.hoistable_static_dict_locals = saved_hoistable_static_dict_locals;
        self.none_widened_local_bindings = saved_none_widened_local_bindings;
        self.python_context_counter = saved_python_context_counter;
        self.python_context_envelope_depth = saved_python_context_envelope_depth;
        self.checked_place_read_witnesses = saved_checked_place_read_witnesses;
        self.nonempty_list_bindings = saved_nonempty_list_bindings;
        self.option_unwrapped_vars = saved_option_unwrapped_vars;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        *self.sifr_int_result_local_bindings.borrow_mut() = saved_sifr_int_result_local_bindings;
        *self.sifr_int_function_returns.borrow_mut() = saved_sifr_int_function_returns;
        *self.sifr_int_result_function_returns.borrow_mut() =
            saved_sifr_int_result_function_returns;
        self.current_sifr_int_return
            .set(saved_current_sifr_int_return);
        self.current_sifr_int_result_return
            .set(saved_current_sifr_int_result_return);
    }
}
