use crate::{
    helpers::{collect_mutated_vars_with_sigs, is_auto_display_type},
    RustEmitter, RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, RustTypeParam,
    Visibility,
};
use sifr_ir::{HirClass, HirFunction, HirModule, RustInteropDecoratorKind, RustInteropValue};
use sifr_type_system::Type;

impl RustEmitter {
    fn is_current_process_resource_class(&self, class_name: &str) -> bool {
        self.current_module_name.as_deref() == Some("sifr.process")
            && matches!(
                class_name,
                "Child"
                    | "AsyncChild"
                    | "ProcessHandle"
                    | "PipeReader"
                    | "PipeWriter"
                    | "AsyncPipeReader"
                    | "AsyncPipeWriter"
            )
    }

    fn process_child_drop_impl() -> RustItem {
        RustItem::Impl {
            target: "Child".to_string(),
            type_params: vec![],
            trait_: Some("Drop".to_string()),
            items: vec![RustItem::Fn {
                name: "drop".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![RustParam::SelfParam { mutable: true }],
                ret: None,
                body: vec![RustStmt::If {
                    cond: RustExpr::UnaryOp {
                        op: "!".to_string(),
                        operand: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: "_waited".to_string(),
                        }),
                    },
                    then_body: vec![RustStmt::Let {
                        mutable: false,
                        name: "_".to_string(),
                        ty: None,
                        value: RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "sifr_stdlib".to_string(),
                                "process".to_string(),
                                "process_child_close".to_string(),
                            ])),
                            args: vec![RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "sifr_runtime".to_string(),
                                    "interop".to_string(),
                                    "SifrIntBridge".to_string(),
                                    "from".to_string(),
                                ])),
                                args: vec![RustExpr::Field {
                                    expr: Box::new(RustExpr::Ident("self".to_string())),
                                    field: "_handle".to_string(),
                                }],
                            }],
                        },
                    }],
                    else_body: None,
                }],
                is_async: false,
            }],
        }
    }

    pub(crate) fn auto_display_format_spec_for_field(&self, field_ty: &Type) -> &'static str {
        match field_ty.resolve_alias() {
            Type::Int
            | Type::Float
            | Type::Bool
            | Type::Str
            | Type::None
            | Type::LiteralInt(_)
            | Type::LiteralBool(_)
            | Type::LiteralStr(_)
            | Type::Newtype { .. } => "{}",
            Type::Class { name, .. } if self.display_classes.contains(name) => "{}",
            _ => "{:?}",
        }
    }

    pub(crate) fn class_visibility(module_public: bool) -> Visibility {
        if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        }
    }

    pub(crate) fn class_impl_target(class: &HirClass) -> String {
        if class.type_params.is_empty() {
            return class.name.clone();
        }
        format!("{}<{}>", class.name, class.type_params.join(", "))
    }

    pub(crate) fn class_impl_type_params(class: &HirClass) -> Vec<RustTypeParam> {
        if class.type_params.is_empty() {
            return Vec::new();
        }
        let bounds = Self::generic_bounds_for_class(class);
        class
            .type_params
            .iter()
            .map(|tp| RustTypeParam {
                name: tp.clone(),
                bounds: vec![bounds.clone()],
            })
            .collect()
    }

    pub(crate) fn class_struct_decl_name(class: &HirClass) -> String {
        if class.type_params.is_empty() {
            return class.name.clone();
        }
        let bounds = Self::generic_bounds_for_class(class);
        let params = class
            .type_params
            .iter()
            .map(|tp| format!("{tp}: {bounds}"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}<{params}>", class.name)
    }

    pub(crate) fn class_struct_fields(
        &mut self,
        class: &HirClass,
        module_public: bool,
    ) -> Vec<(String, RustType)> {
        if class.python_opaque_declaration().is_some() {
            let mut fields = vec![
                (
                    "__sifr_python_object".to_string(),
                    RustType::Named("sifr_runtime::python::ObjectHandle".to_string()),
                ),
                (
                    "__sifr_python_callbacks".to_string(),
                    RustType::Named("sifr_runtime::python::CallbackOwnerSlot".to_string()),
                ),
                (
                    "__sifr_python_not_send_sync".to_string(),
                    RustType::Named("std::marker::PhantomData<std::rc::Rc<()>>".to_string()),
                ),
            ];
            if let Some(errors) = self.python_retained_callback_errors.get(&class.name) {
                fields.extend(errors.iter().enumerate().map(|(index, error)| {
                    (
                        format!("__sifr_python_callback_failure_{index}"),
                        RustType::Named(format!(
                            "sifr_runtime::python::CallbackFailureSlot<{}>",
                            self.rust_type_with_generics(error)
                        )),
                    )
                }));
            }
            return fields;
        }
        let mut fields = Vec::new();
        if let Some(parent) = &class.parent_class {
            if parent != "NonSend" {
                let field_name = if module_public {
                    format!("pub {}", parent.to_lowercase())
                } else {
                    parent.to_lowercase()
                };
                fields.push((field_name, RustType::Named(parent.clone())));
            }
        }

        for (field_name, field_ty) in &class.fields {
            let name = if module_public {
                format!("pub {field_name}")
            } else {
                field_name.clone()
            };
            let ty = if self
                .recursive_fields
                .contains(&(class.name.clone(), field_name.clone()))
            {
                RustType::Named(
                    self.recursive_field_rust_types
                        .get(&(class.name.clone(), field_name.clone()))
                        .cloned()
                        .unwrap_or_else(|| field_ty.rust_type()),
                )
            } else if class.name == "deque" && field_name == "_data" {
                self.collection_needs.needs_vecdeque = true;
                if let Type::List(elem) = field_ty {
                    RustType::Named(format!("VecDeque<{}>", self.rust_type_with_generics(elem)))
                } else {
                    RustType::Named(self.rust_struct_field_type_with_generics(field_ty))
                }
            } else {
                RustType::Named(self.rust_struct_field_type_with_generics(field_ty))
            };
            fields.push((name, ty));
        }
        if class.name == "PythonError" {
            let name = if module_public {
                "pub __sifr_python_error".to_string()
            } else {
                "__sifr_python_error".to_string()
            };
            fields.push((
                name,
                RustType::Option(Box::new(RustType::Named(
                    "sifr_runtime::python::PythonError".to_string(),
                ))),
            ));
        }
        fields
    }

    fn python_opaque_constructor_item(&self, class: &HirClass) -> RustItem {
        let mut fields = vec![
            (
                "__sifr_python_object".to_string(),
                RustExpr::Ident("__sifr_python_object".to_string()),
            ),
            (
                "__sifr_python_callbacks".to_string(),
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "sifr_runtime".to_string(),
                        "python".to_string(),
                        "CallbackOwnerSlot".to_string(),
                        "empty".to_string(),
                    ])),
                    args: Vec::new(),
                },
            ),
            (
                "__sifr_python_not_send_sync".to_string(),
                RustExpr::Path(vec![
                    "std".to_string(),
                    "marker".to_string(),
                    "PhantomData".to_string(),
                ]),
            ),
        ];
        if let Some(errors) = self.python_retained_callback_errors.get(&class.name) {
            fields.extend(errors.iter().enumerate().map(|(index, _)| {
                (
                    format!("__sifr_python_callback_failure_{index}"),
                    RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec![
                            "sifr_runtime".to_string(),
                            "python".to_string(),
                            "CallbackFailureSlot".to_string(),
                            "new".to_string(),
                        ])),
                        args: Vec::new(),
                    },
                )
            }));
        }
        RustItem::Fn {
            name: "__sifr_from_python_object".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: vec![RustParam::Named {
                name: "__sifr_python_object".to_string(),
                ty: RustType::Named("sifr_runtime::python::ObjectHandle".to_string()),
            }],
            ret: Some(RustType::Named("Self".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                name: "Self".to_string(),
                fields,
            }))],
            is_async: false,
        }
    }

    pub(crate) fn lower_default_constructor_item(
        &self,
        class: &HirClass,
        module_public: bool,
    ) -> RustItem {
        let params = class
            .fields
            .iter()
            .map(|(field_name, field_ty)| {
                let ty = if self
                    .recursive_fields
                    .contains(&(class.name.clone(), field_name.clone()))
                {
                    RustType::Named(
                        self.recursive_field_rust_types
                            .get(&(class.name.clone(), field_name.clone()))
                            .cloned()
                            .unwrap_or_else(|| self.rust_type_with_generics(field_ty)),
                    )
                } else {
                    self.rust_ir_type_with_generics(field_ty)
                };
                RustParam::Named {
                    name: field_name.clone(),
                    ty,
                }
            })
            .collect::<Vec<_>>();
        let mut fields = class
            .fields
            .iter()
            .map(|(field_name, _)| (field_name.clone(), RustExpr::Ident(field_name.clone())))
            .collect::<Vec<_>>();
        if class.name == "PythonError" {
            fields.push((
                "__sifr_python_error".to_string(),
                RustExpr::Literal(RustLiteral::None),
            ));
        }
        RustItem::Fn {
            name: "new".to_string(),
            visibility: Self::class_visibility(module_public),
            type_params: Vec::new(),
            params,
            ret: Some(RustType::Named("Self".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                name: "Self".to_string(),
                fields,
            }))],
            is_async: false,
        }
    }

    pub(crate) fn lower_display_body_for_custom_str(
        &mut self,
        str_func: &HirFunction,
    ) -> Vec<RustStmt> {
        let saved_display_ctx = self.emission_ctx.in_display_impl;
        let saved_return_type = self.current_return_type.clone();
        let saved_mutated = self.mutated_vars.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();

        self.emission_ctx.in_display_impl = true;
        self.current_return_type = Some(str_func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&str_func.body, &self.func_signatures);
        self.local_binding_types.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        self.register_local_body_binding_types(&str_func.body);

        let mut body = Vec::new();
        for stmt in &str_func.body {
            let lowered = self.capture_structured_stmts(|inner| inner.emit_stmt(stmt));
            body.extend(lowered);
        }

        if !matches!(body.last(), Some(RustStmt::Return(_))) {
            body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(RustLiteral::Unit)],
            })));
        }

        self.emission_ctx.in_display_impl = saved_display_ctx;
        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated;
        self.local_binding_types = saved_local_binding_types;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        body
    }

    pub(crate) fn build_display_impl_for_error(class: &HirClass) -> RustItem {
        let display_expr = if class.fields.iter().any(|(name, _)| name == "message") {
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident("self".to_string())),
                field: "message".to_string(),
            }
        } else {
            RustExpr::Ident("self".to_string())
        };
        let format_spec = if class.fields.iter().any(|(name, _)| name == "message") {
            "{}"
        } else {
            "{:?}"
        };

        RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_impl_type_params(class),
            trait_: Some("std::fmt::Display".to_string()),
            items: vec![RustItem::Fn {
                name: "fmt".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![
                    RustParam::SelfParam { mutable: false },
                    RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::Ref {
                            mutable: true,
                            inner: Box::new(RustType::Named("std::fmt::Formatter<'_>".to_string())),
                        },
                    },
                ],
                ret: Some(RustType::Named("std::fmt::Result".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::MacroCall {
                    name: "write".to_string(),
                    args: vec![
                        RustExpr::Ident("f".to_string()),
                        RustExpr::Literal(RustLiteral::Str(format_spec.to_string())),
                        display_expr,
                    ],
                }))],
                is_async: false,
            }],
        }
    }

    pub(crate) fn build_display_impl_for_auto_fields(&self, class: &HirClass) -> RustItem {
        let format_str = format!(
            "{}({})",
            class.name,
            class
                .fields
                .iter()
                .map(|(name, ty)| {
                    format!("{name}={}", self.auto_display_format_spec_for_field(ty))
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
        let mut args = vec![
            RustExpr::Ident("f".to_string()),
            RustExpr::Literal(RustLiteral::Str(format_str)),
        ];
        args.extend(class.fields.iter().map(|(field_name, _)| RustExpr::Field {
            expr: Box::new(RustExpr::Ident("self".to_string())),
            field: field_name.clone(),
        }));

        RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_impl_type_params(class),
            trait_: Some("std::fmt::Display".to_string()),
            items: vec![RustItem::Fn {
                name: "fmt".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![
                    RustParam::SelfParam { mutable: false },
                    RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::Ref {
                            mutable: true,
                            inner: Box::new(RustType::Named("std::fmt::Formatter<'_>".to_string())),
                        },
                    },
                ],
                ret: Some(RustType::Named("std::fmt::Result".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::MacroCall {
                    name: "write".to_string(),
                    args,
                }))],
                is_async: false,
            }],
        }
    }

    pub(crate) fn emit_class(&mut self, class: &HirClass, module: &HirModule, module_public: bool) {
        if class.is_protocol() {
            self.emit_protocol_trait(class, module_public);
            return;
        }
        if class.is_enum() {
            self.emit_enum_class(class, module_public);
            return;
        }
        if let Some(inner) = &class.newtype_inner {
            self.emit_newtype(class, inner, module_public);
            return;
        }
        if let Some(target) = opaque_rust_type_path(class) {
            self.body_items.push(RustItem::TypeAlias {
                name: class.name.clone(),
                ty: RustType::Named(format!(
                    "sifr_runtime::interop::Handle<{}>",
                    target.replace('.', "::")
                )),
            });
            return;
        }

        let is_python_opaque = class.python_opaque_declaration().is_some();
        let has_custom_eq = class
            .operator_impls
            .iter()
            .any(|(name, _)| name == "__eq__");
        let has_custom_str = class
            .operator_impls
            .iter()
            .any(|(name, _)| name == "__str__");
        let has_callable_field = class
            .fields
            .iter()
            .any(|(_, ty)| matches!(ty, Type::Callable(..) | Type::AsyncCallable(..)));
        let has_affine_field = class.fields.iter().any(|(_, ty)| {
            matches!(
                ty.resolve_alias(),
                Type::Class { parent_class, .. }
                    if parent_class.as_deref() == Some("NonSend")
            )
        });
        let has_auto_display =
            !class.fields.is_empty() && class.fields.iter().all(|(_, ty)| is_auto_display_type(ty));
        let derives = if is_python_opaque || self.is_current_process_resource_class(&class.name) {
            vec!["Debug".to_string()]
        } else if has_callable_field || has_affine_field {
            Vec::new()
        } else if has_custom_eq {
            vec!["Debug".to_string(), "Clone".to_string()]
        } else if class.is_hashable {
            vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
                "Eq".to_string(),
                "Hash".to_string(),
            ]
        } else {
            vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
            ]
        };

        let struct_fields = self.class_struct_fields(class, module_public);
        self.body_items.push(RustItem::Struct {
            name: Self::class_struct_decl_name(class),
            visibility: Self::class_visibility(module_public),
            derives,
            fields: struct_fields,
        });

        let saved_class_name = self.current_class_name.clone();
        self.current_class_name = Some(class.name.clone());
        let mut impl_items = Vec::new();
        if is_python_opaque {
            impl_items.push(self.python_opaque_constructor_item(class));
        }
        let has_constructor = class.methods.iter().any(|method| method.name == "new");
        if !is_python_opaque
            && !has_constructor
            && class
                .parent_class
                .as_deref()
                .is_none_or(|parent| parent == "NonSend")
        {
            impl_items.push(self.lower_default_constructor_item(class, module_public));
        }
        for method in &class.methods {
            if method.python_interop.first().is_some_and(|declaration| {
                matches!(
                    declaration.kind,
                    sifr_ir::PythonInteropDecoratorKind::ContextEnter
                        | sifr_ir::PythonInteropDecoratorKind::ContextExit
                        | sifr_ir::PythonInteropDecoratorKind::ContextAsyncEnter
                        | sifr_ir::PythonInteropDecoratorKind::ContextAsyncExit
                )
            }) {
                continue;
            }
            impl_items.push(self.lower_class_method_item(method, class, module_public));
        }
        self.current_class_name = saved_class_name;
        self.body_items.push(RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_impl_type_params(class),
            trait_: None,
            items: impl_items,
        });

        if self.current_module_name.as_deref() == Some("sifr.process") && class.name == "Child" {
            self.body_items.push(Self::process_child_drop_impl());
        }

        self.emit_operator_impls(class);

        if class.is_error_type {
            self.body_items
                .push(Self::build_display_impl_for_error(class));
            self.body_items.push(RustItem::Impl {
                target: Self::class_impl_target(class),
                type_params: Self::class_impl_type_params(class),
                trait_: Some("std::error::Error".to_string()),
                items: Vec::new(),
            });
        } else if has_custom_str {
            if let Some((_, str_func)) = class
                .operator_impls
                .iter()
                .find(|(name, _)| name == "__str__")
            {
                let saved_class_name = self.current_class_name.clone();
                self.current_class_name = Some(class.name.clone());
                let body = self.lower_display_body_for_custom_str(str_func);
                self.current_class_name = saved_class_name;
                self.body_items.push(RustItem::Impl {
                    target: Self::class_impl_target(class),
                    type_params: Self::class_impl_type_params(class),
                    trait_: Some("std::fmt::Display".to_string()),
                    items: vec![RustItem::Fn {
                        name: "fmt".to_string(),
                        visibility: Visibility::Private,
                        type_params: Vec::new(),
                        params: vec![
                            RustParam::SelfParam { mutable: false },
                            RustParam::Named {
                                name: "f".to_string(),
                                ty: RustType::Ref {
                                    mutable: true,
                                    inner: Box::new(RustType::Named(
                                        "std::fmt::Formatter<'_>".to_string(),
                                    )),
                                },
                            },
                        ],
                        ret: Some(RustType::Named("std::fmt::Result".to_string())),
                        body,
                        is_async: false,
                    }],
                });
            }
        } else if has_auto_display {
            self.body_items
                .push(self.build_display_impl_for_auto_fields(class));
        }

        self.emit_protocol_impls(class, module);
    }
}

fn opaque_rust_type_path(class: &HirClass) -> Option<String> {
    class
        .rust_interop
        .iter()
        .find(|declaration| declaration.kind == RustInteropDecoratorKind::Opaque)?
        .arguments
        .iter()
        .find(|argument| argument.name.as_deref() == Some("type"))
        .and_then(|argument| match &argument.value {
            RustInteropValue::TargetPath(path) => Some(path.dotted()),
            _ => None,
        })
}
