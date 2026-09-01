use crate::{
    RustExpr, RustItem, RustParam, RustStmt, RustTrait, RustType, Type, Visibility,
    homogeneous_large_tuple_backing_array,
};
use sifr_type_system::{class_rust_name, source_class_rust_name};

pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    match ty {
        Type::Int | Type::LiteralInt(_) => RustType::Named("SifrInt".to_string()),
        Type::FixedInt(fixed) => RustType::Named(fixed.rust_name().to_string()),
        Type::Float => RustType::F64,
        Type::Bool | Type::LiteralBool(_) => RustType::Bool,
        Type::Str | Type::LiteralStr(_) => RustType::String_,
        Type::Bytes => RustType::Vec(Box::new(RustType::Named("u8".to_string()))),
        Type::None => RustType::Unit,
        Type::List(inner) => RustType::Vec(Box::new(sifr_type_to_rust_type(inner))),
        Type::Dict(key, value) => RustType::HashMap(
            Box::new(sifr_type_to_rust_type(key)),
            Box::new(sifr_type_to_rust_type(value)),
        ),
        Type::Set(inner) => RustType::HashSet(Box::new(sifr_type_to_rust_type(inner))),
        Type::Tuple(items) => {
            if let Some((elem, len)) = homogeneous_large_tuple_backing_array(ty) {
                RustType::Array {
                    element: Box::new(sifr_type_to_rust_type(elem)),
                    len,
                }
            } else {
                RustType::Tuple(items.iter().map(sifr_type_to_rust_type).collect())
            }
        }
        Type::Template(_) => RustType::Named("__SifrTemplate".to_string()),
        Type::StructuralRecord(record) => crate::structural_record_rust_type(record),
        Type::Range => RustType::Named("SifrRange".to_string()),
        Type::Iterable(inner) => RustType::Vec(Box::new(sifr_type_to_rust_type(inner))),
        Type::Iterator(inner) => RustType::Boxed(Box::new(RustType::DynTrait {
            trait_: RustTrait::Named {
                name: "Iterator".to_string(),
                params: Vec::new(),
                associated_types: vec![("Item".to_string(), sifr_type_to_rust_type(inner))],
            },
            auto_traits: Vec::new(),
        })),
        Type::Any | Type::Unknown | Type::Intersection(_) => {
            RustType::Boxed(Box::new(RustType::DynTrait {
                trait_: RustTrait::Named {
                    name: "std::any::Any".to_string(),
                    params: Vec::new(),
                    associated_types: Vec::new(),
                },
                auto_traits: Vec::new(),
            }))
        }
        Type::Never => RustType::Never,
        Type::Function(function) | Type::AsyncFunction(function) => RustType::Fn {
            params: function
                .params
                .iter()
                .map(|(_, ty, _)| sifr_type_to_rust_type(ty))
                .collect(),
            ret: Box::new(sifr_type_to_rust_type(&function.return_type)),
        },
        Type::Result(ok, err) => RustType::Result(
            Box::new(sifr_type_to_rust_type(ok)),
            Box::new(sifr_type_to_rust_type(err)),
        ),
        Type::Task(ok, err) => RustType::Generic {
            base: "__SifrTask".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::BlockingTask(ok, err) => RustType::Generic {
            base: "__SifrBlockingTask".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::JoinSet(ok, err) => RustType::Generic {
            base: "__SifrJoinSet".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::TaskResult(ok, err) => RustType::Generic {
            base: "__SifrTaskResult".to_string(),
            params: vec![
                sifr_type_to_rust_type(ok),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::Failure(err) => RustType::Generic {
            base: "__SifrFailure".to_string(),
            params: vec![task_error_type_to_rust_type(err)],
        },
        Type::TimeoutResult(err) => RustType::Generic {
            base: "__SifrTimeoutResult".to_string(),
            params: vec![task_error_type_to_rust_type(err)],
        },
        Type::Select2(first, second) => RustType::Generic {
            base: "__SifrSelect2".to_string(),
            params: vec![
                sifr_type_to_rust_type(first),
                sifr_type_to_rust_type(second),
            ],
        },
        Type::AsyncGenerator(item, err) => RustType::Generic {
            base: "AsyncGenerator".to_string(),
            params: vec![
                sifr_type_to_rust_type(item),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::Coroutine(ok, err) => future_type(RustType::Result(
            Box::new(sifr_type_to_rust_type(ok)),
            Box::new(sifr_type_to_rust_type(err)),
        )),
        Type::Awaitable(result) => future_type(sifr_type_to_rust_type(result)),
        Type::AsyncIterator(item, err) => RustType::Generic {
            base: "AsyncIterator".to_string(),
            params: vec![
                sifr_type_to_rust_type(item),
                task_error_type_to_rust_type(err),
            ],
        },
        Type::PythonBuffer(element) => RustType::Generic {
            base: "::sifr_stdlib::python::PythonBuffer".to_string(),
            params: vec![sifr_type_to_rust_type(element)],
        },
        Type::PythonArrow(kind) => {
            RustType::Named(format!("::sifr_stdlib::python::{}", kind.rust_name()))
        }
        Type::PythonDlpackTensor(element) => RustType::Generic {
            base: "::sifr_stdlib::python::PythonDlpackTensor".to_string(),
            params: vec![sifr_type_to_rust_type(element)],
        },
        Type::PythonDlpackStream => {
            RustType::Named("::sifr_stdlib::python::PythonDlpackStream".to_string())
        }
        Type::Union(_) => {
            if let Some(member) = ty.optional_member_type() {
                RustType::Option(Box::new(sifr_type_to_rust_type(&member)))
            } else {
                RustType::Named(ty.union_enum_name())
            }
        }
        Type::Alias { body, .. } => sifr_type_to_rust_type(body),
        class @ Type::Class {
            identity,
            type_args,
            name,
            ..
        } => {
            if identity.as_deref() == Some("sifr.meta.NoContext") {
                return RustType::Named(
                    "::sifr_runtime::interop::structural::NoContext".to_string(),
                );
            } else if class.is_python_object_contract() {
                return RustType::Generic {
                    base: "::sifr_runtime::interop::Handle".to_string(),
                    params: vec![RustType::Named(
                        "::sifr_runtime::python::ForeignObject".to_string(),
                    )],
                };
            } else if class.is_python_resource_identity_contract() {
                return RustType::Generic {
                    base: "::sifr_runtime::interop::Handle".to_string(),
                    params: vec![RustType::Named(
                        "::sifr_runtime::python::PythonResourceIdentity".to_string(),
                    )],
                };
            }
            let base = class_rust_name(identity.as_deref(), name);
            if type_args.is_empty() {
                RustType::Named(base)
            } else {
                RustType::Generic {
                    base,
                    params: type_args.iter().map(sifr_type_to_rust_type).collect(),
                }
            }
        }
        Type::Protocol { name, .. } => RustType::Boxed(Box::new(RustType::DynTrait {
            trait_: RustTrait::Named {
                name: source_class_rust_name(name),
                params: Vec::new(),
                associated_types: Vec::new(),
            },
            auto_traits: Vec::new(),
        })),
        Type::Newtype { identity, name, .. } | Type::Enum { identity, name, .. } => {
            RustType::Named(class_rust_name(identity.as_deref(), name))
        }
        Type::TypeVar(name) => RustType::Named(name.clone()),
        Type::Callable(params, conventions, ret) => callable_type(params, conventions, ret, false),
        Type::AsyncCallable(params, conventions, ret) => {
            callable_type(params, conventions, ret, true)
        }
        Type::Decimal => RustType::Named("Decimal".to_string()),
        Type::BigDecimal => RustType::Named("BigDecimal".to_string()),
    }
}

pub(crate) fn rust_type_base_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Alias { body, .. } => rust_type_base_name(body),
        Type::Class { identity, name, .. }
        | Type::Protocol { identity, name, .. }
        | Type::Newtype { identity, name, .. }
        | Type::Enum { identity, name, .. } => Some(class_rust_name(identity.as_deref(), name)),
        Type::TypeVar(name) => Some(name.clone()),
        _ => None,
    }
}

pub fn sifr_type_to_rust_field_type(ty: &Type) -> RustType {
    match ty {
        Type::Callable(params, conventions, ret) => RustType::Boxed(Box::new(callable_trait_type(
            params,
            conventions,
            ret,
            false,
        ))),
        Type::AsyncCallable(params, conventions, ret) => {
            let future = future_type(sifr_type_to_rust_type(ret));
            RustType::Boxed(Box::new(RustType::DynTrait {
                trait_: RustTrait::Callable {
                    name: "Fn".to_string(),
                    params: callable_param_types(params, conventions),
                    ret: Some(Box::new(future)),
                },
                auto_traits: vec!["Send".to_string(), "Sync".to_string()],
            }))
        }
        _ => sifr_type_to_rust_type(ty),
    }
}

fn callable_type(
    params: &[Type],
    conventions: &[sifr_type_system::ParamConvention],
    ret: &Type,
    is_async: bool,
) -> RustType {
    if is_async {
        RustType::ImplTrait {
            trait_: RustTrait::Callable {
                name: "Fn".to_string(),
                params: callable_param_types(params, conventions),
                ret: Some(Box::new(future_type(sifr_type_to_rust_type(ret)))),
            },
            auto_traits: vec!["Send".to_string(), "Sync".to_string()],
        }
    } else {
        callable_trait_type(params, conventions, ret, true)
    }
}

fn callable_trait_type(
    params: &[Type],
    conventions: &[sifr_type_system::ParamConvention],
    ret: &Type,
    impl_trait: bool,
) -> RustType {
    let trait_ = RustTrait::Callable {
        name: "Fn".to_string(),
        params: callable_param_types(params, conventions),
        ret: (ret != &Type::None).then(|| Box::new(sifr_type_to_rust_type(ret))),
    };
    if impl_trait {
        RustType::ImplTrait {
            trait_,
            auto_traits: Vec::new(),
        }
    } else {
        RustType::DynTrait {
            trait_,
            auto_traits: Vec::new(),
        }
    }
}

pub(crate) fn callable_dyn_rust_type(
    params: &[Type],
    conventions: &[sifr_type_system::ParamConvention],
    ret: &Type,
    is_async: bool,
) -> RustType {
    if is_async {
        RustType::DynTrait {
            trait_: RustTrait::Callable {
                name: "Fn".to_string(),
                params: callable_param_types(params, conventions),
                ret: Some(Box::new(future_type(sifr_type_to_rust_type(ret)))),
            },
            auto_traits: vec!["Send".to_string(), "Sync".to_string()],
        }
    } else {
        callable_trait_type(params, conventions, ret, false)
    }
}

fn callable_param_types(
    params: &[Type],
    conventions: &[sifr_type_system::ParamConvention],
) -> Vec<RustType> {
    params
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            let converted = sifr_type_to_rust_type(ty);
            match conventions.get(index) {
                Some(convention)
                    if convention.is_shared_borrow()
                        && !crate::helpers::is_copy_type_for_codegen(ty) =>
                {
                    crate::ownership_plan::shared_borrowed_param_type(ty, converted)
                }
                Some(convention)
                    if convention.is_mut_borrow()
                        && !crate::helpers::is_copy_type_for_codegen(ty) =>
                {
                    RustType::Ref {
                        mutable: true,
                        inner: Box::new(converted),
                    }
                }
                _ => converted,
            }
        })
        .collect()
}

fn future_type(output: RustType) -> RustType {
    RustType::Generic {
        base: "std::pin::Pin".to_string(),
        params: vec![RustType::Boxed(Box::new(RustType::DynTrait {
            trait_: RustTrait::Named {
                name: "std::future::Future".to_string(),
                params: Vec::new(),
                associated_types: vec![("Output".to_string(), output)],
            },
            auto_traits: vec!["Send".to_string()],
        }))],
    }
}

pub(crate) fn task_error_type_to_rust_type(ty: &Type) -> RustType {
    if matches!(ty.resolve_alias(), Type::Never) {
        RustType::Named("std::convert::Infallible".to_string())
    } else {
        sifr_type_to_rust_type(ty)
    }
}

pub fn build_error_type_items(
    name: &str,
    extra_fields: &[(String, RustType)],
    constructor_defaults: &[(String, RustExpr)],
) -> Vec<RustItem> {
    if name == "SecondaryError" {
        return super::secondary_error::build_secondary_error_type_items();
    }
    let mut fields = vec![("message".to_string(), RustType::String_)];
    fields.extend(extra_fields.iter().cloned());

    let mut init_fields = vec![(
        "message".to_string(),
        RustExpr::Ident("message".to_string()),
    )];
    init_fields.extend(constructor_defaults.iter().cloned());

    vec![
        RustItem::Struct {
            name: name.to_string(),
            visibility: Visibility::Private,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
                "Eq".to_string(),
                "Hash".to_string(),
            ],
            fields,
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![RustParam::Named {
                    name: "message".to_string(),
                    ty: RustType::String_,
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: init_fields,
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::fmt::Display".to_string()),
            items: vec![RustItem::Fn {
                name: "fmt".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
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
                body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "fmt".to_string(),
                        "Display".to_string(),
                        "fmt".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(RustExpr::Field {
                                expr: Box::new(RustExpr::Ident("self".to_string())),
                                field: "message".to_string(),
                            }),
                        },
                        RustExpr::Ident("f".to_string()),
                    ],
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::error::Error".to_string()),
            items: vec![],
        },
    ]
}

pub fn build_failure_type_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "__SifrFailure<E>".to_string(),
            visibility: Visibility::Private,
            derives: vec![],
            fields: vec![
                ("primary".to_string(), RustType::Named("E".to_string())),
                (
                    "secondary".to_string(),
                    RustType::Vec(Box::new(RustType::Named("SecondaryError".to_string()))),
                ),
            ],
        },
        build_failure_debug_impl(),
        RustItem::Impl {
            target: "__SifrFailure<E>".to_string(),
            type_params: vec![crate::RustTypeParam {
                name: "E".to_string(),
                bounds: vec![],
            }],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::Named {
                        name: "primary".to_string(),
                        ty: RustType::Named("E".to_string()),
                    }],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "primary".to_string(),
                                RustExpr::Ident("primary".to_string()),
                            ),
                            (
                                "secondary".to_string(),
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Vec".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![],
                                },
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "map_primary".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![crate::RustTypeParam {
                        name: "F".to_string(),
                        bounds: vec![],
                    }],
                    params: vec![
                        RustParam::SelfValue,
                        RustParam::Named {
                            name: "f".to_string(),
                            ty: RustType::Named("impl FnOnce(E) -> F".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("__SifrFailure<F>".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "__SifrFailure".to_string(),
                        fields: vec![
                            (
                                "primary".to_string(),
                                RustExpr::Verbatim("f(self.primary)".to_string()),
                            ),
                            (
                                "secondary".to_string(),
                                RustExpr::Verbatim("self.secondary".to_string()),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "with_secondary".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::Named {
                            name: "primary".to_string(),
                            ty: RustType::Named("E".to_string()),
                        },
                        RustParam::Named {
                            name: "secondary".to_string(),
                            ty: RustType::Named("Vec<SecondaryError>".to_string()),
                        },
                    ],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            (
                                "primary".to_string(),
                                RustExpr::Ident("primary".to_string()),
                            ),
                            (
                                "secondary".to_string(),
                                RustExpr::Ident("secondary".to_string()),
                            ),
                        ],
                    }))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "push_secondary_message".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::SelfParam { mutable: true },
                        RustParam::Named {
                            name: "message".to_string(),
                            ty: RustType::String_,
                        },
                    ],
                    ret: Some(RustType::Unit),
                    body: vec![RustStmt::Verbatim(
                        "self.secondary.push(SecondaryError::new(message))".to_string(),
                    )],
                    is_async: false,
                },
            ],
        },
    ]
}

fn build_failure_debug_impl() -> RustItem {
    let debug = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("f".to_string())),
                    method: "debug_struct".to_string(),
                    args: vec![RustExpr::Literal(crate::RustLiteral::StaticStr(
                        "SifrGeneratedFailure".to_string(),
                    ))],
                }),
                method: "field".to_string(),
                args: vec![
                    RustExpr::Literal(crate::RustLiteral::StaticStr("primary".to_string())),
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: "primary".to_string(),
                        }),
                    },
                ],
            }),
            method: "field".to_string(),
            args: vec![
                RustExpr::Literal(crate::RustLiteral::StaticStr("secondary".to_string())),
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: "secondary".to_string(),
                    }),
                },
            ],
        }),
        method: "finish".to_string(),
        args: vec![],
    };
    RustItem::Impl {
        target: "__SifrFailure<E>".to_string(),
        type_params: vec![crate::RustTypeParam {
            name: "E".to_string(),
            bounds: vec!["std::fmt::Debug".to_string()],
        }],
        trait_: Some("std::fmt::Debug".to_string()),
        items: vec![RustItem::Fn {
            name: "fmt".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
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
            body: vec![RustStmt::Return(Some(debug))],
            is_async: false,
        }],
    }
}

pub fn build_timeout_result_type_items() -> Vec<RustItem> {
    vec![RustItem::Enum {
        name: "__SifrTimeoutResult<E>".to_string(),
        visibility: Visibility::Private,
        derives: vec!["Debug".to_string()],
        repr: None,
        variants: vec![
            crate::RustEnumVariant {
                name: "Inner".to_string(),
                tuple_fields: vec![RustType::Named("E".to_string())],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Timeout".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
        ],
    }]
}
pub fn build_cancellation_error_type_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "CancellationError".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string()],
            fields: vec![],
        },
        RustItem::Impl {
            target: "CancellationError".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: vec![],
                }))],
                is_async: false,
            }],
        },
    ]
}

pub fn build_async_exit_cause_type_items() -> Vec<RustItem> {
    vec![RustItem::Enum {
        name: "AsyncExitCause".to_string(),
        visibility: Visibility::Private,
        derives: vec!["Clone".to_string(), "Debug".to_string()],
        repr: None,
        variants: vec![
            crate::RustEnumVariant {
                name: "Normal".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Return".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "OrdinaryError".to_string(),
                tuple_fields: vec![RustType::Named("String".to_string())],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Timeout".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "Cancellation".to_string(),
                tuple_fields: vec![],
                fields: vec![],
                value: None,
            },
            crate::RustEnumVariant {
                name: "RuntimeFault".to_string(),
                tuple_fields: vec![RustType::Named("String".to_string())],
                fields: vec![],
                value: None,
            },
        ],
    }]
}
