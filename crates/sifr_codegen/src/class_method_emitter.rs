use crate::{
    helpers::{
        body_contains_field_assign_codegen, collect_mutated_vars_with_sigs,
        recursive_field_rust_type,
    },
    RustEmitter, RustExpr, RustItem, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use sifr_hir::{HirClass, HirExpr, HirFunction, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    fn lower_class_stmt_strict(&mut self, stmt: &HirStmt, _context: &str) -> Vec<RustStmt> {
        let lowered = self.capture_structured_stmts(|inner| inner.emit_stmt(stmt));
        lowered
    }

    fn lower_class_expr_strict(&mut self, expr: &HirExpr, context: &str) -> RustExpr {
        match self.try_lower_stmt_expr_statement_only(expr) {
            Ok(Some(lowered)) => return self.rewrite_stdlib_constant_idents_in_expr(lowered),
            Ok(None) => {}
            Err(err) => {
                self.lowering_stats.expr_lowering_errors += 1;
                panic!(
                    "statement-only expression lowering failed for class method IR emission ({context}): {}; expr={expr:?}",
                    err.message
                );
            }
        }
        match self.lower_stmt_expr_for_ir(expr) {
            Ok(Some(lowered)) => self.rewrite_stdlib_constant_idents_in_expr(lowered),
            Ok(None) => panic!(
                "structured expression lowering missing for class method IR emission ({context}): {expr:?}"
            ),
            Err(err) => {
                self.lowering_stats.expr_lowering_errors += 1;
                panic!(
                    "structured expression lowering failed for class method IR emission ({context}): {}; expr={expr:?}",
                    err.message
                );
            }
        }
    }

    fn lower_class_method_param_type(
        &self,
        class: &HirClass,
        method: &HirFunction,
        param_name: &str,
        param_ty: &Type,
        convention: ParamConvention,
    ) -> RustType {
        if method.name == "new" {
            let is_recursive = self
                .recursive_fields
                .contains(&(class.name.clone(), param_name.to_string()));
            if is_recursive {
                return RustType::Named(recursive_field_rust_type(param_ty, &class.name));
            }
            if matches!(param_ty, Type::Callable(..)) {
                return RustType::Named(format!("{} + 'static", param_ty.rust_type()));
            }
            return crate::sifr_type_to_rust_type(param_ty);
        }

        let rust_ty = self.rust_type_with_generics(param_ty);
        match convention {
            ParamConvention::Borrow
                if param_ty.ownership() != sifr_type_system::OwnershipKind::Copy =>
            {
                RustType::Ref {
                    mutable: false,
                    inner: Box::new(RustType::Named(rust_ty)),
                }
            }
            ParamConvention::MutBorrow
                if param_ty.ownership() != sifr_type_system::OwnershipKind::Copy =>
            {
                RustType::Ref {
                    mutable: true,
                    inner: Box::new(RustType::Named(rust_ty)),
                }
            }
            _ => RustType::Named(rust_ty),
        }
    }

    fn lower_class_method_return_type(
        &self,
        method: &HirFunction,
        class: &HirClass,
    ) -> Option<RustType> {
        if method.name == "new" {
            return Some(RustType::Named("Self".to_string()));
        }
        if method.return_type == Type::None {
            return None;
        }
        if let Type::Class { name: ret_name, .. } = &method.return_type {
            if !class.type_params.is_empty() && ret_name == &class.name {
                return Some(RustType::Named(format!(
                    "{}<{}>",
                    ret_name,
                    class.type_params.join(", ")
                )));
            }
        }
        Some(crate::sifr_type_to_rust_type(&method.return_type))
    }

    fn lower_constructor_body(&mut self, method: &HirFunction, class: &HirClass) -> Vec<RustStmt> {
        let has_super = method.body.iter().any(|stmt| {
            if let HirStmt::Expr { expr } = stmt {
                matches!(expr, HirExpr::SuperCall { .. })
            } else {
                false
            }
        });

        let mut body = Vec::new();
        let inheritance_parent = if has_super {
            class.parent_class.as_ref()
        } else {
            None
        };

        if let Some(parent_name) = inheritance_parent {
            let mut super_args: Option<&Vec<HirExpr>> = None;
            let mut field_inits: Vec<(&str, &HirExpr)> = Vec::new();
            let mut other_stmts: Vec<&HirStmt> = Vec::new();

            for stmt in &method.body {
                if let HirStmt::Expr {
                    expr: HirExpr::SuperCall { args, .. },
                } = stmt
                {
                    super_args = Some(args);
                } else if let HirStmt::FieldAssign { field, value, .. } = stmt {
                    field_inits.push((field, value));
                } else {
                    other_stmts.push(stmt);
                }
            }

            for stmt in &other_stmts {
                body.extend(self.lower_class_stmt_strict(
                    stmt,
                    "class constructor non-field statement lowering",
                ));
            }

            let mut fields = Vec::new();
            let mut parent_args = Vec::new();
            if let Some(args) = super_args {
                parent_args.extend(args.iter().map(|arg| {
                    self.lower_class_expr_strict(arg, "class constructor super-call arg lowering")
                }));
            }
            fields.push((
                parent_name.to_lowercase(),
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![parent_name.clone(), "new".to_string()])),
                    args: parent_args,
                },
            ));

            for (field_name, value) in &field_inits {
                fields.push((
                    (*field_name).to_string(),
                    self.lower_class_expr_strict(
                        value,
                        "class constructor field assignment value lowering",
                    ),
                ));
            }

            body.push(RustStmt::Return(Some(RustExpr::StructInit {
                name: "Self".to_string(),
                fields,
            })));
            return body;
        }

        let mut field_inits: Vec<(&str, &HirExpr)> = Vec::new();
        let mut other_stmts: Vec<&HirStmt> = Vec::new();
        for stmt in &method.body {
            if let HirStmt::FieldAssign { field, value, .. } = stmt {
                field_inits.push((field, value));
            } else {
                other_stmts.push(stmt);
            }
        }

        for stmt in &other_stmts {
            body.extend(
                self.lower_class_stmt_strict(
                    stmt,
                    "class constructor non-field statement lowering",
                ),
            );
        }

        let mut fields = Vec::new();
        for (field_name, value) in &field_inits {
            let lowered_value = if class.name == "deque" && *field_name == "_data" {
                if let HirExpr::ListLiteral { elements, .. } = value {
                    if elements.is_empty() {
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "VecDeque".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![],
                        }
                    } else {
                        self.lower_class_expr_strict(
                            value,
                            "deque constructor _data field value lowering",
                        )
                    }
                } else {
                    self.lower_class_expr_strict(
                        value,
                        "deque constructor _data field value lowering",
                    )
                }
            } else {
                self.lower_class_expr_strict(value, "class constructor field value lowering")
            };

            let field_ty = class
                .fields
                .iter()
                .find(|(name, _)| name == field_name)
                .map(|(_, ty)| ty);
            let final_value = if field_ty.is_some_and(|ty| matches!(ty, Type::Callable(..))) {
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![lowered_value],
                }
            } else {
                lowered_value
            };
            fields.push(((*field_name).to_string(), final_value));
        }

        for (field_name, field_ty) in &class.fields {
            if field_inits.iter().any(|(name, _)| name == field_name) {
                continue;
            }
            if !method.params.iter().any(|param| &param.name == field_name) {
                continue;
            }
            let value = if matches!(field_ty, Type::Callable(..)) {
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![RustExpr::Ident(field_name.clone())],
                }
            } else {
                RustExpr::Ident(field_name.clone())
            };
            fields.push((field_name.clone(), value));
        }

        body.push(RustStmt::Return(Some(RustExpr::StructInit {
            name: "Self".to_string(),
            fields,
        })));
        body
    }

    pub(super) fn lower_class_method_item(
        &mut self,
        method: &HirFunction,
        class: &HirClass,
        module_public: bool,
    ) -> RustItem {
        let saved_return_type = self.current_return_type.clone();
        let saved_mutated_vars = self.mutated_vars.clone();
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_callable_var_conventions = self.callable_var_conventions.clone();

        self.current_return_type = Some(method.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&method.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.callable_var_conventions.clear();

        for param in &method.params {
            let effective_convention = if method.name == "new" {
                ParamConvention::Own
            } else {
                param.convention
            };

            if effective_convention == ParamConvention::Borrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.borrowed_params.insert(param.name.clone());
            }
            if effective_convention == ParamConvention::MutBorrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.mut_borrowed_params.insert(param.name.clone());
            }
            if let Type::Callable(ref param_types, ref conventions, _) = param.ty {
                let conv_list = param_types
                    .iter()
                    .zip(conventions.iter())
                    .map(|(ty, conv)| (ty.clone(), *conv))
                    .collect::<Vec<_>>();
                self.callable_var_conventions
                    .insert(param.name.clone(), conv_list);
            }
        }

        let visibility = if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        };

        let mut params = Vec::new();
        match method.method_kind {
            MethodKind::Regular if method.name != "new" => {
                params.push(RustParam::SelfParam {
                    mutable: body_contains_field_assign_codegen(&method.body),
                });
            }
            _ => {}
        }
        for param in &method.params {
            params.push(RustParam::Named {
                name: param.name.clone(),
                ty: self.lower_class_method_param_type(
                    class,
                    method,
                    &param.name,
                    &param.ty,
                    param.convention,
                ),
            });
        }

        let mut body = if method.method_kind == MethodKind::Regular && method.name == "new" {
            self.lower_constructor_body(method, class)
        } else {
            let mut lowered = Vec::new();
            for stmt in &method.body {
                lowered
                    .extend(self.lower_class_stmt_strict(stmt, "class method statement lowering"));
            }
            lowered
        };

        if body.is_empty() {
            if self.lower_class_method_return_type(method, class).is_none() {
                body.push(RustStmt::Return(None));
            } else {
                panic!(
                    "class method IR lowering produced empty body for non-unit return: {}::{}",
                    class.name, method.name
                );
            }
        }

        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated_vars;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.callable_var_conventions = saved_callable_var_conventions;

        RustItem::Fn {
            name: method.name.clone(),
            visibility,
            type_params: method
                .type_params
                .iter()
                .map(|name| RustTypeParam {
                    name: name.clone(),
                    bounds: Vec::new(),
                })
                .collect(),
            params,
            ret: self.lower_class_method_return_type(method, class),
            body,
            is_async: false,
        }
    }
}
