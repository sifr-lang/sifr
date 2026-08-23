use crate::{
    RustEmitter, RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, RustTypeParam,
    Visibility,
    function_emitter::{is_result_int_type, result_int_return_type_to_sifr_int, result_method_key},
    helpers::collect_mutated_vars_with_sigs,
    python_interop_direct::python_interop_method_body_with_retained_errors,
    rust_interop_direct::rust_interop_method_body,
};
use sifr_ir::{HirClass, HirExpr, HirFunction, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};

impl RustEmitter {
    pub(crate) fn rust_receiver_param(method: &HirFunction) -> RustParam {
        let Some(receiver) = method.receiver else {
            panic!("regular instance method is missing receiver convention");
        };
        match receiver {
            ReceiverConvention::SharedBorrow => RustParam::SelfParam { mutable: false },
            ReceiverConvention::MutableBorrow => RustParam::SelfParam { mutable: true },
            ReceiverConvention::Owned => RustParam::SelfValue,
            ReceiverConvention::OwnedMutable => RustParam::MutableSelfValue,
        }
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

        Self::box_recursive_value_for_ir(lowered_value)
    }

    pub(crate) fn lower_class_stmt_strict(
        &mut self,
        stmt: &HirStmt,
        context: &str,
    ) -> Vec<RustStmt> {
        self.lower_class_stmt_strict_with_following(stmt, None, context)
    }

    pub(crate) fn lower_class_stmt_strict_with_following(
        &mut self,
        stmt: &HirStmt,
        following_stmts: Option<&[HirStmt]>,
        _context: &str,
    ) -> Vec<RustStmt> {
        self.capture_structured_stmts(|inner| inner.emit_stmt_with_following(stmt, following_stmts))
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
        if let Some(callback) = method
            .python_interop
            .iter()
            .flat_map(|declaration| &declaration.callbacks)
            .find(|callback| callback.parameter_name == param_name)
        {
            if matches!(
                callback.dispatch,
                sifr_ir::PythonCallbackDispatch::Foreign | sifr_ir::PythonCallbackDispatch::Asyncio
            ) {
                return self.lower_python_callback_param_type(
                    param_ty,
                    convention,
                    callback.lifetime != sifr_ir::PythonCallbackLifetime::Call,
                );
            }
        }
        if method
            .rust_interop
            .iter()
            .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Callback)
            && matches!(param_ty.resolve_alias(), Type::Callable(..))
        {
            return self.lower_python_callback_param_type(param_ty, convention, true);
        }
        if method.name == "new" {
            let is_recursive = self
                .recursive_fields
                .contains(&(class.name.clone(), param_name.to_string()));
            if is_recursive {
                return self
                    .recursive_field_rust_types
                    .get(&(class.name.clone(), param_name.to_string()))
                    .cloned()
                    .unwrap_or_else(|| self.rust_ir_type_with_generics(param_ty));
            }
            if matches!(param_ty, Type::Callable(..) | Type::AsyncCallable(..)) {
                return self.rust_ir_type_with_static_bound(param_ty);
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

        let rust_ty = self.rust_ir_type_with_generics(param_ty);
        if param_ty.ownership() != sifr_type_system::OwnershipKind::Copy && convention.is_borrowed()
        {
            RustType::Ref {
                mutable: convention.is_mut_borrow(),
                inner: Box::new(rust_ty),
            }
        } else {
            rust_ty
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
        if !class.type_params.is_empty() && class.is_self_type(&method.return_type) {
            return Some(RustType::Named(Self::class_impl_target(class)));
        }
        Some(self.rust_ir_type_with_generics(&method.return_type))
    }

    pub(crate) fn lower_constructor_body(
        &mut self,
        method: &HirFunction,
        class: &HirClass,
        uses_python_error_bridge: bool,
    ) -> Vec<RustStmt> {
        let inheritance_parent = method
            .body
            .iter()
            .any(|stmt| {
                matches!(
                    stmt,
                    HirStmt::Expr {
                        expr: HirExpr::SuperCall { .. }
                    }
                )
            })
            .then_some(class.parent_class.as_ref())
            .flatten();
        let mut body = Vec::new();
        let mut field_inits = Vec::<(String, RustExpr)>::new();
        let mut instance_materialized = false;

        for (stmt_index, stmt) in method.body.iter().enumerate() {
            let following_stmts = Some(&method.body[stmt_index + 1..]);
            if instance_materialized {
                let rewritten = Self::rewrite_constructor_self(stmt);
                body.extend(self.lower_class_stmt_strict_with_following(
                    &rewritten,
                    following_stmts,
                    "class constructor post-initialization statement lowering",
                ));
                continue;
            }

            if let HirStmt::Expr {
                expr: HirExpr::SuperCall { args, .. },
            } = stmt
            {
                let Some(parent_name) = inheritance_parent else {
                    continue;
                };
                let parent_rust_type = class
                    .parent_type
                    .as_ref()
                    .and_then(crate::rust_type_base_name)
                    .unwrap_or_else(|| sifr_type_system::source_class_rust_name(parent_name));
                let parent_args = args
                    .iter()
                    .map(|arg| {
                        self.lower_class_expr_strict(
                            arg,
                            "class constructor super-call arg lowering",
                        )
                    })
                    .collect();
                body.push(RustStmt::Let {
                    mutable: false,
                    name: "__sifr_parent".to_string(),
                    ty: None,
                    value: RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![parent_rust_type, "new".to_string()])),
                        args: parent_args,
                    },
                });
                continue;
            }

            if let HirStmt::FieldAssign {
                object,
                field,
                value,
                ..
            } = stmt
            {
                let is_uninitialized_own_field = object == "self"
                    && class.fields.iter().any(|(name, _)| name == field)
                    && !field_inits.iter().any(|(name, _)| name == field)
                    && !Self::constructor_body_references_self(&[HirStmt::Expr {
                        expr: value.clone(),
                    }]);
                if is_uninitialized_own_field {
                    let field_ty = class
                        .fields
                        .iter()
                        .find(|(name, _)| name == field)
                        .map(|(_, ty)| ty);
                    let temp_ty =
                        field_ty.map(|ty| self.class_struct_field_rust_type(class, field, ty));
                    let lowered_value = self.lower_constructor_field_value(class, field, value);
                    let lowered_value = field_ty.map_or(lowered_value.clone(), |ty| {
                        self.wrap_recursive_constructor_field_value(
                            class,
                            method,
                            field,
                            ty,
                            value,
                            lowered_value,
                        )
                    });
                    let final_value = field_ty.map_or(lowered_value.clone(), |ty| {
                        Self::box_constructor_callable_value(ty, lowered_value)
                    });
                    let temp_name = format!("__sifr_field_init_{}", field_inits.len());
                    body.push(RustStmt::Let {
                        mutable: false,
                        name: temp_name.clone(),
                        ty: temp_ty,
                        value: final_value,
                    });
                    field_inits.push((field.clone(), RustExpr::Ident(temp_name)));
                    continue;
                }
            }

            if Self::constructor_body_references_self(std::slice::from_ref(stmt)) {
                let fields = self.constructor_instance_fields(
                    class,
                    method,
                    inheritance_parent,
                    &field_inits,
                    uses_python_error_bridge,
                );
                body.push(RustStmt::Let {
                    mutable: true,
                    name: "__sifr_self".to_string(),
                    ty: None,
                    value: RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields,
                    },
                });
                instance_materialized = true;
                let rewritten = Self::rewrite_constructor_self(stmt);
                body.extend(self.lower_class_stmt_strict_with_following(
                    &rewritten,
                    following_stmts,
                    "class constructor post-initialization statement lowering",
                ));
                continue;
            }

            body.extend(self.lower_class_stmt_strict_with_following(
                stmt,
                following_stmts,
                "class constructor pre-initialization statement lowering",
            ));
        }

        if instance_materialized {
            body.push(RustStmt::Return(Some(RustExpr::Ident(
                "__sifr_self".to_string(),
            ))));
        } else {
            let fields = self.constructor_instance_fields(
                class,
                method,
                inheritance_parent,
                &field_inits,
                uses_python_error_bridge,
            );
            body.push(RustStmt::Return(Some(RustExpr::StructInit {
                name: "Self".to_string(),
                fields,
            })));
        }
        body
    }

    fn constructor_body_references_self(stmts: &[HirStmt]) -> bool {
        let mut probe = stmts.to_vec();
        let mut references_self = false;
        sifr_ir::visit_hir_stmts_exprs_mut(&mut probe, &mut |expr| {
            if matches!(expr, HirExpr::Name { name, .. } if name == "self") {
                references_self = true;
            }
        });
        sifr_ir::visit_hir_stmts_storage_roots_mut(&mut probe, &mut |root| {
            if root == "self" {
                references_self = true;
            }
        });
        references_self
    }

    fn rewrite_constructor_self(stmt: &HirStmt) -> HirStmt {
        let mut rewritten = vec![stmt.clone()];
        sifr_ir::visit_hir_stmts_exprs_mut(&mut rewritten, &mut |expr| {
            if let HirExpr::Name { name, .. } = expr {
                if name == "self" {
                    *name = "__sifr_self".to_string();
                }
            }
        });
        sifr_ir::visit_hir_stmts_storage_roots_mut(&mut rewritten, &mut |root| {
            if root == "self" {
                *root = "__sifr_self".to_string();
            }
        });
        let mut rewritten = rewritten.into_iter();
        match rewritten.next() {
            Some(statement) => statement,
            None => unreachable!("constructor statement rewrite preserves one statement"),
        }
    }

    fn lower_constructor_field_value(
        &mut self,
        class: &HirClass,
        field_name: &str,
        value: &HirExpr,
    ) -> RustExpr {
        if class.name == "deque" && field_name == "_data" {
            let constructor = if matches!(value, HirExpr::ListLiteral { elements, .. } if elements.is_empty())
            {
                "new"
            } else {
                "from"
            };
            let args =
                if constructor == "new" {
                    Vec::new()
                } else {
                    vec![self.lower_class_expr_strict(
                        value,
                        "deque constructor _data field value lowering",
                    )]
                };
            return RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "VecDeque".to_string(),
                    constructor.to_string(),
                ])),
                args,
            };
        }
        self.lower_class_expr_strict(value, "class constructor field value lowering")
    }

    fn constructor_instance_fields(
        &self,
        class: &HirClass,
        method: &HirFunction,
        inheritance_parent: Option<&String>,
        field_inits: &[(String, RustExpr)],
        uses_python_error_bridge: bool,
    ) -> Vec<(String, RustExpr)> {
        let mut fields = Vec::new();
        if let Some(parent_name) = inheritance_parent {
            fields.push((
                parent_name.to_lowercase(),
                RustExpr::Ident("__sifr_parent".to_string()),
            ));
        }
        fields.extend(field_inits.iter().cloned());
        for (field_name, field_ty) in &class.fields {
            if field_inits.iter().any(|(name, _)| name == field_name)
                || !method.params.iter().any(|param| &param.name == field_name)
            {
                continue;
            }
            let value = if matches!(field_ty, Type::Callable(..) | Type::AsyncCallable(..)) {
                Self::box_constructor_callable_value(field_ty, RustExpr::Ident(field_name.clone()))
            } else {
                self.wrap_recursive_constructor_field_value(
                    class,
                    method,
                    field_name,
                    field_ty,
                    &HirExpr::Name {
                        name: field_name.clone(),
                        binding_id: None,
                        ty: field_ty.clone(),
                    },
                    RustExpr::Ident(field_name.clone()),
                )
            };
            fields.push((field_name.clone(), value));
        }
        Self::append_class_phantom_initializer(class, &mut fields);
        if uses_python_error_bridge {
            fields.push((
                "__sifr_python_error".to_string(),
                RustExpr::Literal(RustLiteral::None),
            ));
        }
        fields
    }

    fn box_constructor_callable_value(field_ty: &Type, value: RustExpr) -> RustExpr {
        let boxed_value = match field_ty {
            Type::AsyncCallable(params, _, _) => {
                let closure_params = (0..params.len())
                    .map(|index| RustParam::Named {
                        name: format!("__sifr_async_arg_{index}"),
                        ty: RustType::Named("_".to_string()),
                    })
                    .collect::<Vec<_>>();
                let call_args = (0..params.len())
                    .map(|index| RustExpr::Ident(format!("__sifr_async_arg_{index}")))
                    .collect::<Vec<_>>();
                let async_call = RustExpr::AsyncBlock {
                    body: vec![RustStmt::Return(Some(RustExpr::Await(Box::new(
                        RustExpr::FnCall {
                            func: Box::new(RustExpr::Ident("__sifr_async_callable".to_string())),
                            args: call_args,
                        },
                    ))))],
                    is_move: true,
                };
                RustExpr::Block {
                    stmts: vec![RustStmt::Let {
                        mutable: false,
                        name: "__sifr_async_callable".to_string(),
                        ty: None,
                        value: RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "std".to_string(),
                                "sync".to_string(),
                                "Arc".to_string(),
                                "new".to_string(),
                            ])),
                            args: vec![value],
                        },
                    }],
                    expr: Some(Box::new(RustExpr::ClosureBlock {
                        params: closure_params,
                        body: vec![
                            RustStmt::Let {
                                mutable: false,
                                name: "__sifr_async_callable".to_string(),
                                ty: None,
                                value: RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "std".to_string(),
                                        "sync".to_string(),
                                        "Arc".to_string(),
                                        "clone".to_string(),
                                    ])),
                                    args: vec![RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(RustExpr::Ident(
                                            "__sifr_async_callable".to_string(),
                                        )),
                                    }],
                                },
                            },
                            RustStmt::Return(Some(RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "Box".to_string(),
                                    "pin".to_string(),
                                ])),
                                args: vec![async_call],
                            })),
                        ],
                        is_move: true,
                        is_async: false,
                    })),
                }
            }
            Type::Callable(..) => value,
            _ => return value,
        };
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
            args: vec![boxed_value],
        }
    }

    pub(crate) fn lower_class_method_item(
        &mut self,
        method: &HirFunction,
        class: &HirClass,
        module_public: bool,
        uses_python_error_bridge: bool,
    ) -> RustItem {
        let saved_return_type = self.current_return_type.clone();
        let saved_mutated_vars = self.mutated_vars.clone();
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_callable_var_conventions = self.callable_var_conventions.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_python_context_counter = self.python_context_counter;
        let saved_python_context_envelope_depth = self.python_context_envelope_depth;
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
        self.python_context_counter = 0;
        self.python_context_envelope_depth = 0;
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
            if let Type::Callable(ref param_types, ref conventions, _)
            | Type::AsyncCallable(ref param_types, ref conventions, _) = param.ty
            {
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
                params.push(Self::rust_receiver_param(method));
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

        let mut body = if let Some(interop_body) = python_interop_method_body_with_retained_errors(
            method,
            &self.python_opaque_classes,
            class.python_opaque_declaration(),
            &self.python_retained_callback_errors,
            self.python_retained_callback_errors
                .get(&class.name)
                .map_or(&[], Vec::as_slice),
        ) {
            interop_body
        } else if let Some(interop_body) = rust_interop_method_body(
            method,
            class
                .rust_interop
                .iter()
                .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque),
        ) {
            interop_body
        } else if method.method_kind == MethodKind::Regular && method.name == "new" {
            self.lower_constructor_body(method, class, uses_python_error_bridge)
        } else {
            let mut lowered = Vec::new();
            for (stmt_index, stmt) in method.body.iter().enumerate() {
                lowered.extend(self.lower_class_stmt_strict_with_following(
                    stmt,
                    Some(&method.body[stmt_index + 1..]),
                    "class method statement lowering",
                ));
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
        self.python_context_counter = saved_python_context_counter;
        self.python_context_envelope_depth = saved_python_context_envelope_depth;
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
