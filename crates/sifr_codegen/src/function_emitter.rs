use crate::{
    body_contains_yield, collect_mutated_vars_with_sigs, type_contains_typevar, RustEmitter,
};
use sifr_hir::{HirFunction, HirStmt};
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    fn returns_result_none(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Result(ok_ty, _) => matches!(
                crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                Type::None
            ),
            _ => false,
        }
    }

    pub(super) fn emit_function(
        &mut self,
        func: &HirFunction,
        module_public: bool,
        test_mode: bool,
    ) {
        // In test mode, skip the main function
        if test_mode && func.name == "main" {
            return;
        }

        // Track the current function's return type for Option wrapping
        self.current_return_type = Some(func.return_type.clone());

        // Pre-scan: collect mutated variables so we know which need `mut`.
        // Use func_signatures to detect variables passed to mut params (need `let mut`).
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);

        // Track borrowed parameters for dereference in comparisons
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        // Track Callable-typed params/locals so we can emit correct borrow prefixes when calling them
        self.callable_var_conventions.clear();
        for param in &func.params {
            if param.convention == ParamConvention::Borrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.borrowed_params.insert(param.name.clone());
            }
            if param.convention == ParamConvention::MutBorrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.mut_borrowed_params.insert(param.name.clone());
            }
            // Register Callable-typed params for convention-aware call emission
            if let Type::Callable(ref param_types, ref conventions, _) = param.ty {
                let conv_list: Vec<(Type, ParamConvention)> = param_types
                    .iter()
                    .zip(conventions.iter())
                    .map(|(t, c)| (t.clone(), *c))
                    .collect();
                self.callable_var_conventions
                    .insert(param.name.clone(), conv_list);
            }
        }

        // Emit decorator comments before the function
        for decorator in &func.decorators {
            self.write_indent();
            self.write(&format!("// @{decorator}\n"));
        }

        // In test mode, add #[test] attribute for test_* functions
        if test_mode && func.name.starts_with("test_") {
            self.write_indent();
            self.write("#[test]\n");
        }

        // Function signature -- only emit params without defaults, or all params
        // Since Rust doesn't have default params, we emit all params and handle
        // defaults at call site
        self.write_indent();
        if module_public && func.name != "main" {
            self.write("pub fn ");
        } else {
            self.write("fn ");
        }
        self.write(&func.name);
        // Emit generic type parameters if this is a generic function
        if !func.type_params.is_empty() {
            let needs_hash_eq = Self::func_needs_hash_eq(func);
            self.write("<");
            for (i, tp) in func.type_params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                let extra = Self::extra_bounds_for_type_param(tp, &func.body);
                let base = if needs_hash_eq {
                    "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq"
                } else {
                    "Clone + std::fmt::Display + PartialOrd"
                };
                self.write(&format!("{tp}: {base}{extra}"));
            }
            self.write(">");
        }
        self.write("(");

        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            // Emit `mut` for parameters that are mutated in the body
            // (only for Own params; borrowed params use &mut convention instead)
            if param.convention == ParamConvention::Own && self.mutated_vars.contains(&param.name) {
                self.write("mut ");
            }
            self.write(&param.name);
            self.write(": ");
            // Emit parameter type based on convention
            let rust_ty = param.ty.rust_type();
            match param.convention {
                ParamConvention::Borrow => {
                    if param.ty.ownership() == sifr_type_system::OwnershipKind::Copy {
                        // Copy types are always passed by value
                        self.write(&rust_ty);
                    } else {
                        self.write(&format!("&{rust_ty}"));
                    }
                }
                ParamConvention::MutBorrow => {
                    self.write(&format!("&mut {rust_ty}"));
                }
                ParamConvention::Own => {
                    self.write(&rust_ty);
                }
            }
        }

        self.write(")");

        // Detect if this is a generator function (contains yield statements)
        let is_generator = body_contains_yield(&func.body);
        if is_generator {
            self.generator_functions.insert(func.name.clone());
        }

        // Return type (omit for main and for None return)
        if func.return_type != Type::None || func.name != "main" {
            if func.return_type != Type::None {
                self.write(" -> ");
                if is_generator {
                    // Generator functions return impl Iterator<Item = T>
                    let yield_ty = if let Type::List(ref elem) = func.return_type {
                        elem.rust_type()
                    } else {
                        "i64".to_string()
                    };
                    if matches!(func.return_type, Type::List(_)) {
                        self.write(&format!("Vec<{yield_ty}>"));
                    } else {
                        self.write(&format!("impl Iterator<Item = {yield_ty}>"));
                    }
                } else {
                    // If return type is a generic class and this function has type params,
                    // include the type params in the return type
                    let ret_type = if let Type::Class {
                        name: ref ret_name, ..
                    } = func.return_type
                    {
                        if self.generic_classes.contains(ret_name) && !func.type_params.is_empty() {
                            let type_params_in_ret: Vec<&String> = func
                                .type_params
                                .iter()
                                .filter(|tp| type_contains_typevar(&func.return_type, tp))
                                .collect();
                            if type_params_in_ret.is_empty() {
                                func.return_type.rust_type()
                            } else {
                                format!(
                                    "{}<{}>",
                                    ret_name,
                                    type_params_in_ret
                                        .iter()
                                        .map(|s| s.as_str())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            }
                        } else {
                            func.return_type.rust_type()
                        }
                    } else {
                        func.return_type.rust_type()
                    };
                    self.write(&ret_type);
                }
            }
        }

        self.write(" {\n");
        self.indent += 1;

        if is_generator {
            // Lazy generator using std::iter::from_fn.
            // Pattern: init stmts; while cond: [pre_yield]; yield val; [post_yield]
            // Becomes: init stmts; from_fn(move || { if cond { pre_yield; let v = val; post_yield; Some(v) } else { None } })
            let yield_ty = if let Type::List(ref elem) = func.return_type {
                elem.rust_type()
            } else {
                "i64".to_string()
            };

            // Separate body into init statements and the while loop
            let mut init_stmts = Vec::new();
            let mut while_stmt = None;
            for stmt in &func.body {
                if while_stmt.is_none() {
                    if let HirStmt::While { .. } = stmt {
                        while_stmt = Some(stmt);
                    } else {
                        init_stmts.push(stmt);
                    }
                }
            }

            // Emit init statements (local variable declarations, always mutable)
            for stmt in &init_stmts {
                self.emit_generator_init_stmt(stmt);
            }

            // Emit the lazy iterator
            self.write_indent();
            self.write(&format!(
                "std::iter::from_fn(move || -> Option<{yield_ty}> {{\n"
            ));
            self.indent += 1;

            if let Some(HirStmt::While {
                condition, body, ..
            }) = while_stmt
            {
                // Check if yield is directly in the while body or nested in an if
                let has_conditional_yield =
                    !body.iter().any(|s| matches!(s, HirStmt::Yield { .. }))
                        && body.iter().any(|s| {
                            if let HirStmt::If { then_body, .. } = s {
                                body_contains_yield(then_body)
                            } else {
                                false
                            }
                        });

                if has_conditional_yield {
                    // Conditional yield: while cond: if test: yield val; post_stmts
                    // Emit as: while cond { let mut __yielded = None; if test { __yielded = Some(val); } post_stmts; if let Some(v) = __yielded { return Some(v); } }; None
                    self.write_indent();
                    self.write("while ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;

                    // Emit __yielded variable
                    self.write_indent();
                    self.write(&format!("let mut __yielded: Option<{yield_ty}> = None;\n"));

                    // Emit body with yield replaced by __yielded = Some(val)
                    for s in body {
                        if let HirStmt::If {
                            condition: if_cond,
                            then_body,
                            ..
                        } = s
                        {
                            if body_contains_yield(then_body) {
                                // Emit the if with yield -> __yielded = Some(val)
                                self.write_indent();
                                self.write("if ");
                                self.emit_expr(if_cond);
                                self.write(" {\n");
                                self.indent += 1;
                                for ts in then_body {
                                    if let HirStmt::Yield { value } = ts {
                                        self.write_indent();
                                        self.write("__yielded = Some(");
                                        self.emit_expr(value);
                                        self.write(");\n");
                                    } else {
                                        self.emit_stmt(ts);
                                    }
                                }
                                self.indent -= 1;
                                self.write_indent();
                                self.write("}\n");
                            } else {
                                self.emit_stmt(s);
                            }
                        } else {
                            self.emit_stmt(s);
                        }
                    }

                    // Check if a value was yielded
                    self.write_indent();
                    self.write("if let Some(__v) = __yielded {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write("return Some(__v);\n");
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");

                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                    // After while loop exits, return None
                    self.write_indent();
                    self.write("None\n");
                } else {
                    // Simple yield: while cond: pre_yield; yield val; post_yield
                    // Separate into pre-yield, yield expr, post-yield
                    let mut pre_yield = Vec::new();
                    let mut yield_expr = None;
                    let mut post_yield = Vec::new();
                    let mut found_yield = false;
                    for s in body {
                        if found_yield {
                            post_yield.push(s);
                        } else if let HirStmt::Yield { value } = s {
                            yield_expr = Some(value);
                            found_yield = true;
                        } else {
                            pre_yield.push(s);
                        }
                    }

                    // Emit: if cond { pre_yield; let v = yield_val; post_yield; Some(v) } else { None }
                    self.write_indent();
                    self.write("if ");
                    self.emit_expr(condition);
                    self.write(" {\n");
                    self.indent += 1;

                    for s in &pre_yield {
                        self.emit_stmt(s);
                    }

                    if let Some(yexpr) = yield_expr {
                        self.write_indent();
                        self.write("let __yield_val = ");
                        self.emit_expr(yexpr);
                        self.write(";\n");
                    }

                    for s in &post_yield {
                        self.emit_stmt(s);
                    }

                    self.write_indent();
                    self.write("Some(__yield_val)\n");

                    self.indent -= 1;
                    self.write_indent();
                    self.write("} else {\n");
                    self.indent += 1;
                    self.write_indent();
                    self.write("None\n");
                    self.indent -= 1;
                    self.write_indent();
                    self.write("}\n");
                }
            } else {
                self.write_indent();
                self.write("None\n");
            }

            self.indent -= 1;
            self.write_indent();
            if matches!(func.return_type, Type::List(_)) {
                self.write("}).collect::<Vec<_>>()\n");
            } else {
                self.write("})\n");
            }
        } else {
            // Non-generator: emit body normally
            for stmt in &func.body {
                self.emit_stmt(stmt);
            }
            if Self::returns_result_none(&func.return_type)
                && !matches!(
                    func.body.last(),
                    Some(HirStmt::Return { .. } | HirStmt::Raise { .. })
                )
            {
                self.write_indent();
                self.write("return Ok(());\n");
            }
        }

        self.indent -= 1;
        self.write_indent();
        self.write("}\n");

        self.current_return_type = None;
        self.mutated_vars.clear();
    }
}
