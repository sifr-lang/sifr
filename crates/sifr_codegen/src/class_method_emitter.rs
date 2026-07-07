use crate::{
    function_emitter::{is_result_int_type, result_int_return_type_to_sifr_int, result_method_key},
    helpers::{body_contains_field_assign_codegen, collect_mutated_vars_with_sigs},
    hir_analysis::traversal::{self, TraversalConfig},
    rust_interop_direct::rust_interop_method_body,
    RustEmitter, RustExpr, RustItem, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use sifr_ir::{HirClass, HirExpr, HirFunction, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashSet;

impl RustEmitter {
    pub(crate) fn class_method_requires_mutable_self(
        &self,
        class: &HirClass,
        method: &HirFunction,
    ) -> bool {
        if method.method_kind != MethodKind::Regular || method.name == "new" {
            return false;
        }
        let mut visiting = HashSet::new();
        self.class_method_requires_mutable_self_recursive(class, &method.name, &mut visiting)
    }

    pub(crate) fn class_method_requires_mutable_self_recursive(
        &self,
        class: &HirClass,
        method_name: &str,
        visiting: &mut HashSet<String>,
    ) -> bool {
        let Some(method) = class
            .methods
            .iter()
            .find(|candidate| candidate.name == method_name)
        else {
            return false;
        };

        if body_contains_field_assign_codegen(&method.body) {
            return true;
        }
        if self.body_contains_mut_borrowed_self_field_call(&method.body) {
            return true;
        }

        if !visiting.insert(method_name.to_string()) {
            return false;
        }

        let delegated_calls = Self::collect_direct_self_method_calls(&method.body);
        for delegated_method in delegated_calls {
            if self.class_method_requires_mutable_self_recursive(class, &delegated_method, visiting)
            {
                visiting.remove(method_name);
                return true;
            }
        }

        visiting.remove(method_name);
        false
    }

    fn body_contains_mut_borrowed_self_field_call(&self, stmts: &[HirStmt]) -> bool {
        let found = std::cell::Cell::new(false);
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if found.get() {
                return;
            }
            let HirExpr::Call { func, args, .. } = expr else {
                return;
            };
            let Some(params) = self.resolve_plain_call_param_info(func, args.len()) else {
                return;
            };
            if args
                .iter()
                .zip(params.iter())
                .any(|(arg, (_, convention))| {
                    convention.is_mut_borrow() && Self::expr_is_self_field_path(arg)
                })
            {
                found.set(true);
            }
        };
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        found.get()
    }

    fn expr_is_self_field_path(expr: &HirExpr) -> bool {
        match expr {
            HirExpr::FieldAccess { object, .. } => {
                matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self")
                    || Self::expr_is_self_field_path(object)
            }
            HirExpr::Index { object, .. } | HirExpr::Slice { object, .. } => {
                Self::expr_is_self_field_path(object)
            }
            _ => false,
        }
    }

    pub(crate) fn collect_direct_self_method_calls(stmts: &[HirStmt]) -> HashSet<String> {
        let mut calls = HashSet::new();
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if let HirExpr::MethodCall { object, method, .. } = expr {
                if matches!(object.as_ref(), HirExpr::Name { name, .. } if name == "self") {
                    calls.insert(method.clone());
                }
            }
        };
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        calls
    }

    pub(crate) fn is_some_call_expr(expr: &RustExpr) -> bool {
        matches!(
            expr,
            RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    || matches!(func.as_ref(), RustExpr::Ident(name) if name == "Some")
        )
    }

    pub(crate) fn is_box_new_call_expr(expr: &RustExpr) -> bool {
        matches!(
            expr,
            RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path.len() == 2 && path[0] == "Box" && path[1] == "new")
                    || matches!(func.as_ref(), RustExpr::Ident(name) if name == "Box::new")
        )
    }

    pub(crate) fn ensure_some_box_inner(expr: RustExpr) -> RustExpr {
        match expr {
            RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), RustExpr::Path(path) if path.len() == 1 && path[0] == "Some")
                    && args.len() == 1 =>
            {
                let mut args_iter = args.into_iter();
                let Some(inner) = args_iter.next() else {
                    unreachable!("Some(_) call must have exactly one argument");
                };
                if Self::is_box_new_call_expr(&inner) {
                    RustExpr::FnCall {
                        func,
                        args: vec![inner],
                    }
                } else {
                    RustExpr::FnCall {
                        func,
                        args: vec![RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Box".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![inner],
                        }],
                    }
                }
            }
            other => RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![other],
                }],
            },
        }
    }

    pub(crate) fn wrap_recursive_constructor_field_value(
        &self,
        class: &HirClass,
        method: &HirFunction,
        field_name: &str,
        field_ty: &Type,
        value_expr: &HirExpr,
        lowered_value: RustExpr,
    ) -> RustExpr {
        let is_recursive = self
            .recursive_fields
            .contains(&(class.name.clone(), field_name.to_string()));
        if !is_recursive {
            return lowered_value;
        }

        let is_boxed_constructor_param = matches!(
            value_expr,
            HirExpr::Name { name, .. }
                if method.name == "new"
                    && name == field_name
                    && method.params.iter().any(|param| param.name == *name)
        );
        if is_boxed_constructor_param {
            return lowered_value;
        }

        if crate::helpers::is_option_type(field_ty) {
            if matches!(value_expr, HirExpr::NoneLiteral) {
                return lowered_value;
            }
            if Self::is_some_call_expr(&lowered_value) {
                return Self::ensure_some_box_inner(lowered_value);
            }
            return Self::ensure_some_box_inner(lowered_value);
        }

        if Self::is_box_new_call_expr(&lowered_value) {
            return lowered_value;
        }

        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
            args: vec![lowered_value],
        }
    }

    pub(crate) fn lower_class_stmt_strict(
        &mut self,
        stmt: &HirStmt,
        _context: &str,
    ) -> Vec<RustStmt> {
        self.capture_structured_stmts(|inner| inner.emit_stmt(stmt))
    }

    pub(crate) fn lower_class_expr_strict(&mut self, expr: &HirExpr, context: &str) -> RustExpr {
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
            Ok(None) => {
                if let Some(lowered) = self.try_lower_registry_expr_strict(expr) {
                    return self.rewrite_stdlib_constant_idents_in_expr(lowered);
                }
                panic!(
                    "structured expression lowering missing for class method IR emission ({context}): {expr:?}"
                )
            }
            Err(err) => {
                self.lowering_stats.expr_lowering_errors += 1;
                panic!(
                    "structured expression lowering failed for class method IR emission ({context}): {}; expr={expr:?}",
                    err.message
                );
            }
        }
    }

    pub(crate) fn lower_class_method_param_type(
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
                return RustType::Named(
                    self.recursive_field_rust_types
                        .get(&(class.name.clone(), param_name.to_string()))
                        .cloned()
                        .unwrap_or_else(|| self.rust_type_with_generics(param_ty)),
                );
            }
            if matches!(param_ty, Type::Callable(..)) {
                return RustType::Named(format!(
                    "{} + 'static",
                    self.rust_type_with_generics(param_ty)
                ));
            }
            return self.rust_ir_type_with_generics(param_ty);
        }

        let param_idx = method
            .params
            .iter()
            .position(|param| param.name == param_name);
        if param_idx.is_some_and(|idx| {
            self.method_param_lowers_to_sifr_int_result(&class.name, &method.name, idx)
        }) && is_result_int_type(param_ty)
        {
            return result_int_return_type_to_sifr_int(param_ty);
        }

        let rust_ty = self.rust_type_with_generics(param_ty);
        if param_ty.ownership() != sifr_type_system::OwnershipKind::Copy && convention.is_borrowed()
        {
            RustType::Ref {
                mutable: convention.is_mut_borrow(),
                inner: Box::new(RustType::Named(rust_ty)),
            }
        } else {
            RustType::Named(rust_ty)
        }
    }

    pub(crate) fn lower_class_method_return_type(
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
        if is_result_int_type(&method.return_type)
            && self
                .sifr_int_result_method_returns
                .borrow()
                .contains(&result_method_key(&class.name, &method.name))
        {
            return Some(result_int_return_type_to_sifr_int(&method.return_type));
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
        Some(self.rust_ir_type_with_generics(&method.return_type))
    }

    pub(crate) fn lower_constructor_body(
        &mut self,
        method: &HirFunction,
        class: &HirClass,
    ) -> Vec<RustStmt> {
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
                let field_ty = class
                    .fields
                    .iter()
                    .find(|(name, _)| name == field_name)
                    .map(|(_, ty)| ty);
                let lowered_value = self.lower_class_expr_strict(
                    value,
                    "class constructor field assignment value lowering",
                );
                fields.push((
                    (*field_name).to_string(),
                    field_ty.map_or(lowered_value.clone(), |ty| {
                        self.wrap_recursive_constructor_field_value(
                            class,
                            method,
                            field_name,
                            ty,
                            value,
                            lowered_value,
                        )
                    }),
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
            let field_ty = class
                .fields
                .iter()
                .find(|(name, _)| name == field_name)
                .map(|(_, ty)| ty);
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
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "VecDeque".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![self.lower_class_expr_strict(
                                value,
                                "deque constructor _data field value lowering",
                            )],
                        }
                    }
                } else {
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![self.lower_class_expr_strict(
                            value,
                            "deque constructor _data field value lowering",
                        )],
                    }
                }
            } else {
                self.lower_class_expr_strict(value, "class constructor field value lowering")
            };
            let lowered_value = field_ty.map_or(lowered_value.clone(), |ty| {
                self.wrap_recursive_constructor_field_value(
                    class,
                    method,
                    field_name,
                    ty,
                    value,
                    lowered_value,
                )
            });
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
                self.wrap_recursive_constructor_field_value(
                    class,
                    method,
                    field_name,
                    field_ty,
                    &HirExpr::Name {
                        name: field_name.clone(),
                        ty: field_ty.clone(),
                    },
                    RustExpr::Ident(field_name.clone()),
                )
            };
            fields.push((field_name.clone(), value));
        }

        body.push(RustStmt::Return(Some(RustExpr::StructInit {
            name: "Self".to_string(),
            fields,
        })));
        body
    }

    pub(crate) fn lower_class_method_item(
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
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();
        let saved_sifr_int_result_local_bindings =
            self.sifr_int_result_local_bindings.borrow().clone();
        let saved_current_sifr_int_result_return = self.current_sifr_int_result_return.get();

        self.current_return_type = Some(method.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&method.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.callable_var_conventions.clear();
        self.local_binding_types.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        self.sifr_int_result_local_bindings.borrow_mut().clear();
        self.current_sifr_int_result_return.set(
            self.sifr_int_result_method_returns
                .borrow()
                .contains(&result_method_key(&class.name, &method.name)),
        );

        for (param_idx, param) in method.params.iter().enumerate() {
            let effective_convention = if method.name == "new" {
                ParamConvention::own()
            } else {
                param.convention
            };

            if effective_convention.is_shared_borrow()
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.borrowed_params.insert(param.name.clone());
            }
            if effective_convention.is_mut_borrow()
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
            self.local_binding_types
                .insert(param.name.clone(), param.ty.clone());
            if self.method_param_lowers_to_sifr_int_result(&class.name, &method.name, param_idx) {
                self.sifr_int_result_local_bindings
                    .borrow_mut()
                    .insert(param.name.clone());
            }
        }
        self.register_local_body_binding_types(&method.body);

        let visibility = if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        };

        let mut params = Vec::new();
        match method.method_kind {
            MethodKind::Regular if method.name != "new" => {
                params.push(RustParam::SelfParam {
                    mutable: self.class_method_requires_mutable_self(class, method),
                });
            }
            _ => {}
        }
        for param in &method.params {
            let rust_ty = self.lower_class_method_param_type(
                class,
                method,
                &param.name,
                &param.ty,
                param.convention,
            );
            params.push(
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
                },
            );
        }

        let mut body = if let Some(interop_body) = rust_interop_method_body(method) {
            interop_body
        } else if method.method_kind == MethodKind::Regular && method.name == "new" {
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
        self.local_binding_types = saved_local_binding_types;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        *self.sifr_int_result_local_bindings.borrow_mut() = saved_sifr_int_result_local_bindings;
        self.current_sifr_int_result_return
            .set(saved_current_sifr_int_result_return);

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
            is_async: method.is_async,
        }
    }
}
