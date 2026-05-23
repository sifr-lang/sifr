impl RustEmitter {
    fn rewrite_special_ident(&self, name: String) -> crate::RustExpr {
        if self.local_binding_types.contains_key(&name) {
            return crate::RustExpr::Ident(name);
        }

        if self.is_stdlib_constant(&name) {
            return match name.as_str() {
                "pi" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "PI".to_string(),
                ]),
                "e" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "E".to_string(),
                ]),
                "tau" => crate::RustExpr::Path(vec![
                    "std".to_string(),
                    "f64".to_string(),
                    "consts".to_string(),
                    "TAU".to_string(),
                ]),
                "inf" => crate::RustExpr::Path(vec!["f64".to_string(), "INFINITY".to_string()]),
                "nan" => crate::RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
                _ => crate::RustExpr::Ident(name),
            };
        }

        if let Some((_ty, rust_name)) = self.module_constants.get(&name) {
            if let Some(mapped) = parse_module_constant_expr(rust_name) {
                return mapped;
            }
        }

        crate::RustExpr::Ident(name)
    }

    fn coerce_expr_to_sifr_int(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
                crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(crate::RustExpr::Ident(name)),
                }
            }
            crate::RustExpr::Paren(inner) => {
                crate::RustExpr::Paren(Box::new(self.coerce_expr_to_sifr_int(*inner)))
            }
            crate::RustExpr::BinOp { left, op, right }
                if is_sifr_int_arithmetic_op(&op)
                    && (self.is_sifr_int_expr(&left) || self.is_sifr_int_expr(&right)) =>
            {
                crate::RustExpr::BinOp {
                    left: Box::new(self.coerce_expr_to_sifr_int(*left)),
                    op,
                    right: Box::new(self.coerce_expr_to_sifr_int(*right)),
                }
            }
            other if self.is_sifr_int_expr(&other) => other,
            crate::RustExpr::Cast {
                expr,
                ty: crate::RustType::I64,
            } => sifr_int_from_i64_expr(*expr),
            other => sifr_int_from_i64_expr(other),
        }
    }

    fn sifr_int_known_nonzero_floor_expr(
        &self,
        op: &str,
        left: crate::RustExpr,
        right: crate::RustExpr,
    ) -> crate::RustExpr {
        let method = match op {
            "/" => "floor_div_known_nonzero",
            "%" => "floor_mod_known_nonzero",
            _ => unreachable!("SifrInt floor rewrite only handles floor division and modulo"),
        };
        crate::RustExpr::MethodCall {
            receiver: Box::new(self.coerce_expr_to_sifr_int_method_receiver(left)),
            method: method.to_string(),
            args: vec![self.coerce_expr_to_sifr_int_comparison_operand(right)],
        }
    }

    fn coerce_expr_to_sifr_int_method_receiver(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
                crate::RustExpr::Ident(name)
            }
            crate::RustExpr::Paren(inner) => crate::RustExpr::Paren(Box::new(
                self.coerce_expr_to_sifr_int_method_receiver(*inner),
            )),
            crate::RustExpr::UnaryOp { op, operand } if op == "-" => {
                crate::RustExpr::Paren(Box::new(crate::RustExpr::UnaryOp { op, operand }))
            }
            other if self.is_sifr_int_expr(&other) => other,
            crate::RustExpr::Cast {
                expr,
                ty: crate::RustType::I64,
            } => sifr_int_from_i64_expr(*expr),
            other => sifr_int_from_i64_expr(other),
        }
    }

    pub(super) fn coerce_expr_to_sifr_int_value(&self, expr: crate::RustExpr) -> crate::RustExpr {
        match expr {
            crate::RustExpr::Ident(name) if self.is_registered_sifr_int_local(&name) => {
                crate::RustExpr::Clone(Box::new(crate::RustExpr::Ident(name)))
            }
            crate::RustExpr::Paren(inner) => {
                crate::RustExpr::Paren(Box::new(self.coerce_expr_to_sifr_int_value(*inner)))
            }
            crate::RustExpr::BinOp { left, op, right }
                if is_sifr_int_arithmetic_op(&op)
                    && (self.is_sifr_int_expr(&left) || self.is_sifr_int_expr(&right)) =>
            {
                crate::RustExpr::BinOp {
                    left: Box::new(self.coerce_expr_to_sifr_int(*left)),
                    op,
                    right: Box::new(self.coerce_expr_to_sifr_int(*right)),
                }
            }
            other if self.is_sifr_int_expr(&other) => other,
            crate::RustExpr::Cast {
                expr,
                ty: crate::RustType::I64,
            } => sifr_int_from_i64_expr(*expr),
            other => sifr_int_from_i64_expr(other),
        }
    }

    pub(super) fn coerce_result_int_expr_to_sifr_int_value(
        &self,
        expr: crate::RustExpr,
    ) -> crate::RustExpr {
        match expr {
            crate::RustExpr::FnCall { func, args } if is_ok_result_path(&func) => {
                let args = args
                    .into_iter()
                    .map(|arg| self.coerce_expr_to_sifr_int_value(arg))
                    .collect();
                crate::RustExpr::FnCall { func, args }
            }
            crate::RustExpr::Paren(inner) => crate::RustExpr::Paren(Box::new(
                self.coerce_result_int_expr_to_sifr_int_value(*inner),
            )),
            other if self.is_sifr_int_result_expr(&other) => other,
            other => other,
        }
    }

    fn coerce_expr_to_sifr_int_comparison_operand(&self, expr: crate::RustExpr) -> crate::RustExpr {
        let coerced = self.coerce_expr_to_sifr_int(expr);
        if matches!(coerced, crate::RustExpr::Ref { .. }) {
            return coerced;
        }
        crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(coerced),
        }
    }

    pub(super) fn is_registered_sifr_int_local(&self, name: &str) -> bool {
        self.sifr_int_local_bindings.borrow().contains(name)
    }

    pub(super) fn is_forced_sifr_int_local(&self, name: &str) -> bool {
        self.sifr_int_forced_local_bindings.borrow().contains(name)
    }

    pub(super) fn is_sifr_int_expr(&self, expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::FnCall { func, args } => {
                (args.is_empty() && self.is_sifr_int_module_constant_func(func))
                    || self.is_sifr_int_returning_function_call(func)
                    || matches!(
                        func.as_ref(),
                        crate::RustExpr::Path(path)
                            if string_path_matches(path, &["SifrInt", "from_i64"])
                                || string_path_matches(path, &["sifr_runtime", "SifrInt", "from_i64"])
                    )
            }
            crate::RustExpr::Ident(name) => self.is_registered_sifr_int_local(name),
            crate::RustExpr::BinOp { left, op, right } if is_sifr_int_arithmetic_op(op) => {
                self.is_sifr_int_expr(left) || self.is_sifr_int_expr(right)
            }
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } if matches!(
                method.as_str(),
                "floor_div_known_nonzero" | "floor_mod_known_nonzero"
            ) =>
            {
                self.is_sifr_int_expr(receiver)
            }
            crate::RustExpr::UnaryOp { op, operand } if op == "-" => self.is_sifr_int_expr(operand),
            crate::RustExpr::Paren(inner) => self.is_sifr_int_expr(inner),
            crate::RustExpr::Ref { expr, .. } => self.is_sifr_int_expr(expr),
            crate::RustExpr::Clone(expr) => self.is_sifr_int_expr(expr),
            crate::RustExpr::Try(expr) => self.is_sifr_int_result_expr(expr),
            _ => false,
        }
    }

    pub(super) fn is_sifr_int_result_expr(&self, expr: &crate::RustExpr) -> bool {
        match expr {
            crate::RustExpr::Block {
                expr: Some(inner), ..
            } => self.is_sifr_int_result_expr(inner),
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } if method == "ok_or_else" => Self::is_sifr_int_checked_floor_option_expr(receiver),
            crate::RustExpr::MethodCall {
                receiver, method, ..
            } => self.is_sifr_int_result_returning_method_call(receiver, method),
            crate::RustExpr::FnCall { func, .. } => {
                self.is_sifr_int_result_returning_function_call(func)
            }
            crate::RustExpr::Ident(name) => self.is_registered_sifr_int_result_local(name),
            crate::RustExpr::Paren(inner) => self.is_sifr_int_result_expr(inner),
            _ => false,
        }
    }

    fn is_sifr_int_checked_floor_option_expr(expr: &crate::RustExpr) -> bool {
        matches!(
            expr,
            crate::RustExpr::MethodCall {
                method,
                ..
            } if matches!(method.as_str(), "checked_floor_div" | "checked_floor_mod")
        )
    }

    fn is_sifr_int_module_constant_func(&self, func: &crate::RustExpr) -> bool {
        let Some(func_name) = rust_expr_identifier_path(func) else {
            return false;
        };
        self.module_constants.values().any(|(ty, rust_name)| {
            matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
                && rust_name
                    .strip_suffix("()")
                    .is_some_and(|const_func| const_func == func_name)
        })
    }

    fn is_sifr_int_returning_function_call(&self, func: &crate::RustExpr) -> bool {
        rust_expr_identifier_path(func).is_some_and(|name| self.function_returns_sifr_int(&name))
    }

    fn is_sifr_int_result_returning_function_call(&self, func: &crate::RustExpr) -> bool {
        rust_expr_identifier_path(func).is_some_and(|name| {
            self.sifr_int_result_function_returns
                .borrow()
                .contains(&name)
        })
    }

    fn is_sifr_int_result_returning_method_call(
        &self,
        receiver: &crate::RustExpr,
        method: &str,
    ) -> bool {
        self.rust_expr_class_name(receiver)
            .is_some_and(|class_name| {
                self.sifr_int_result_method_returns.borrow().contains(
                    &crate::function_emitter::result_method_key(&class_name, method),
                )
            })
    }

    fn rust_expr_class_name(&self, expr: &crate::RustExpr) -> Option<String> {
        match expr {
            crate::RustExpr::Ident(name) if name == "self" => self.current_class_name.clone(),
            crate::RustExpr::Ident(name) => self.local_binding_types.get(name).and_then(|ty| {
                match crate::resolve_alias_type_for_plain_call(ty) {
                    Type::Class { name, .. } => Some(name.clone()),
                    _ => None,
                }
            }),
            crate::RustExpr::Field { expr, field } => {
                self.rust_expr_class_name(expr).and_then(|owner_class| {
                    self.class_field_types
                        .get(&(owner_class, field.clone()))
                        .and_then(|ty| match crate::resolve_alias_type_for_plain_call(ty) {
                            Type::Class { name, .. } => Some(name.clone()),
                            _ => None,
                        })
                })
            }
            crate::RustExpr::MethodCall {
                receiver,
                method,
                args,
            } if method == "clone" && args.is_empty() => self.rust_expr_class_name(receiver),
            crate::RustExpr::Paren(inner) => self.rust_expr_class_name(inner),
            _ => None,
        }
    }

    pub(super) fn function_returns_sifr_int(&self, name: &str) -> bool {
        self.sifr_int_function_returns.borrow().contains(name)
    }

    pub(super) fn function_param_lowers_to_sifr_int(&self, name: &str, idx: usize) -> bool {
        self.sifr_int_function_params
            .borrow()
            .get(name)
            .is_some_and(|params| params.contains(&idx))
    }

    pub(super) fn function_param_lowers_to_sifr_int_result(&self, name: &str, idx: usize) -> bool {
        self.sifr_int_result_function_params
            .borrow()
            .get(name)
            .is_some_and(|params| params.contains(&idx))
    }

    pub(super) fn method_param_lowers_to_sifr_int_result(
        &self,
        class_name: &str,
        method_name: &str,
        idx: usize,
    ) -> bool {
        self.sifr_int_result_method_params
            .borrow()
            .get(&crate::function_emitter::result_method_key(
                class_name,
                method_name,
            ))
            .is_some_and(|params| params.contains(&idx))
    }

    fn is_registered_sifr_int_result_local(&self, name: &str) -> bool {
        self.sifr_int_result_local_bindings.borrow().contains(name)
    }
}

fn parse_module_constant_expr(rust_name: &str) -> Option<crate::RustExpr> {
    if let Some(func) = rust_name.strip_suffix("()") {
        let func_expr = parse_identifier_path_expr(func)?;
        return Some(crate::RustExpr::FnCall {
            func: Box::new(func_expr),
            args: vec![],
        });
    }
    parse_identifier_path_expr(rust_name)
}

fn parse_identifier_path_expr(name: &str) -> Option<crate::RustExpr> {
    let segments = name
        .split("::")
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() || !segments.iter().all(|segment| is_ident(segment)) {
        return None;
    }
    if segments.len() == 1 {
        return Some(crate::RustExpr::Ident(segments[0].to_string()));
    }
    Some(crate::RustExpr::Path(
        segments.into_iter().map(ToString::to_string).collect(),
    ))
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch == '_' || ch.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn sifr_int_from_i64_expr(expr: crate::RustExpr) -> crate::RustExpr {
    crate::RustExpr::FnCall {
        func: Box::new(crate::RustExpr::Path(vec![
            "SifrInt".to_string(),
            "from_i64".to_string(),
        ])),
        args: vec![expr],
    }
}

fn is_sifr_int_arithmetic_op(op: &str) -> bool {
    matches!(op, "+" | "-" | "*")
}

fn is_sifr_int_checked_floor_op(op: &str) -> bool {
    matches!(op, "/" | "%")
}

fn is_sifr_int_comparison_op(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}

fn is_sifr_int_operand_coercion_op(op: &str) -> bool {
    is_sifr_int_arithmetic_op(op) || is_sifr_int_comparison_op(op)
}

fn is_legacy_i64_type(ty: Option<&crate::RustType>) -> bool {
    matches!(ty, Some(crate::RustType::I64))
        || matches!(ty, Some(crate::RustType::Named(name)) if name == "i64")
}

fn is_result_legacy_i64_type(ty: Option<&crate::RustType>) -> bool {
    matches!(ty, Some(crate::RustType::Result(ok, _)) if is_legacy_i64_rust_type(ok))
        || matches!(ty, Some(crate::RustType::Named(name)) if name.starts_with("Result<i64, "))
}

fn is_legacy_i64_rust_type(ty: &crate::RustType) -> bool {
    matches!(ty, crate::RustType::I64)
        || matches!(ty, crate::RustType::Named(name) if name == "i64")
}

fn result_i64_type_to_sifr_int(ty: crate::RustType) -> crate::RustType {
    match ty {
        crate::RustType::Result(ok, err) if is_legacy_i64_rust_type(&ok) => {
            crate::RustType::Result(Box::new(crate::RustType::Named("SifrInt".to_string())), err)
        }
        crate::RustType::Named(name) if name.starts_with("Result<i64, ") => {
            crate::RustType::Named(name.replacen("Result<i64, ", "Result<SifrInt, ", 1))
        }
        other => other,
    }
}

fn is_proven_nonzero_integer_expr(expr: &crate::RustExpr) -> bool {
    match expr {
        crate::RustExpr::Literal(crate::RustLiteral::Int(value)) => *value != 0,
        crate::RustExpr::Cast {
            expr,
            ty: crate::RustType::I64,
        } => is_proven_nonzero_integer_expr(expr),
        crate::RustExpr::UnaryOp { op, operand } if op == "-" => {
            is_proven_nonzero_integer_expr(operand)
        }
        crate::RustExpr::Paren(inner) => is_proven_nonzero_integer_expr(inner),
        crate::RustExpr::FnCall { func, args }
            if args.len() == 1
                && matches!(
                    func.as_ref(),
                    crate::RustExpr::Path(path)
                        if string_path_matches(path, &["SifrInt", "from_i64"])
                            || string_path_matches(path, &["sifr_runtime", "SifrInt", "from_i64"])
                ) =>
        {
            is_proven_nonzero_integer_expr(&args[0])
        }
        _ => false,
    }
}

fn rust_expr_identifier_path(expr: &crate::RustExpr) -> Option<String> {
    match expr {
        crate::RustExpr::Ident(name) => Some(name.clone()),
        crate::RustExpr::Path(path) => Some(path.join("::")),
        _ => None,
    }
}

fn string_path_matches(path: &[String], expected: &[&str]) -> bool {
    path.len() == expected.len()
        && path
            .iter()
            .zip(expected)
            .all(|(segment, expected_segment)| segment == expected_segment)
}

fn is_ok_result_path(expr: &crate::RustExpr) -> bool {
    match expr {
        crate::RustExpr::Path(path) => string_path_matches(path, &["Ok"]),
        crate::RustExpr::Ident(name) => name == "Ok",
        _ => false,
    }
}
