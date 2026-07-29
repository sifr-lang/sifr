use crate::{
    RustEmitter, RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, RustTypeParam,
    Visibility,
};
use sifr_ir::{HirClass, HirFunction, MethodKind};
use sifr_type_system::{source_class_rust_name, Type};

impl RustEmitter {
    pub(crate) fn emit_protocol_trait(&mut self, class: &HirClass, _module_public: bool) {
        // Protocol traits are part of the generated structural interface even
        // when only their impls are referenced directly in a binary crate.
        let visibility = Visibility::Pub;
        let methods = class
            .methods
            .iter()
            .map(|method| RustItem::TraitMethodSig {
                name: method.name.clone(),
                params: {
                    let mut params = Vec::with_capacity(method.params.len() + 1);
                    params.push(Self::rust_receiver_param(method));
                    for param in &method.params {
                        params.push(RustParam::Named {
                            name: param.name.clone(),
                            ty: crate::sifr_type_to_rust_type(&param.ty),
                        });
                    }
                    params
                },
                ret: if method.return_type == Type::None {
                    None
                } else {
                    Some(crate::sifr_type_to_rust_type(&method.return_type))
                },
                is_async: method.is_async,
            })
            .collect::<Vec<_>>();
        self.body_items.push(RustItem::Trait {
            name: source_class_rust_name(&class.name),
            visibility,
            supertraits: vec![],
            methods,
        });
    }

    pub(crate) fn emit_enum_class(&mut self, class: &HirClass, module_public: bool) {
        let visibility = if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let mut auto_value = 1_i64;
        let variants = class
            .enum_variants
            .iter()
            .map(|(name, value)| {
                let resolved = value.unwrap_or(auto_value);
                auto_value = resolved + 1;
                crate::RustEnumVariant {
                    name: name.clone(),
                    tuple_fields: Vec::new(),
                    fields: Vec::new(),
                    value: Some(RustExpr::Literal(RustLiteral::Int(resolved))),
                }
            })
            .collect::<Vec<_>>();
        self.body_items.push(RustItem::Enum {
            name: source_class_rust_name(&class.name),
            visibility,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "Copy".to_string(),
                "PartialEq".to_string(),
                "Eq".to_string(),
                "Hash".to_string(),
            ],
            repr: Some("i64".to_string()),
            variants,
        });

        self.body_items.push(RustItem::Impl {
            target: source_class_rust_name(&class.name),
            type_params: Vec::new(),
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
                        RustExpr::Literal(RustLiteral::Str("{:?}".to_string())),
                        RustExpr::Ident("self".to_string()),
                    ],
                }))],
                is_async: false,
            }],
        });

        let mut impl_items = vec![
            RustItem::Fn {
                name: "name".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![RustParam::SelfParam { mutable: false }],
                ret: Some(RustType::String_),
                body: vec![RustStmt::Return(Some(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{:?}".to_string(),
                    args: vec![RustExpr::Ident("self".to_string())],
                }))],
                is_async: false,
            },
            RustItem::Fn {
                name: "value".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![RustParam::SelfParam { mutable: false }],
                ret: Some(RustType::I64),
                body: vec![RustStmt::Return(Some(RustExpr::Cast {
                    expr: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                        "self".to_string(),
                    )))),
                    ty: RustType::I64,
                }))],
                is_async: false,
            },
        ];
        let saved_class_name = self.current_class_name.clone();
        self.current_class_name = Some(class.name.clone());
        for method in &class.methods {
            impl_items.push(self.lower_type_emitter_method_item(method, module_public));
        }
        self.current_class_name = saved_class_name;

        self.body_items.push(RustItem::Impl {
            target: source_class_rust_name(&class.name),
            type_params: Vec::new(),
            trait_: None,
            items: impl_items,
        });
    }

    pub(crate) fn emit_newtype(&mut self, class: &HirClass, inner: &Type, module_public: bool) {
        let visibility = if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let mut derives = Vec::new();
        if inner.supports_debug_formatting() {
            derives.push("Debug".to_string());
        }
        if inner.supports_derived_clone() {
            derives.push("Clone".to_string());
        }
        if inner.supports_structural_equality() {
            derives.push("PartialEq".to_string());
        }
        if inner.supports_hash_key() {
            derives.push("Eq".to_string());
            derives.push("Hash".to_string());
        }
        self.body_items.push(RustItem::TupleStruct {
            name: source_class_rust_name(&class.name),
            visibility,
            derives,
            inner: crate::sifr_type_to_rust_type(inner),
        });

        let method_visibility = if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let mut impl_items = vec![
            RustItem::Fn {
                name: "new".to_string(),
                visibility: method_visibility.clone(),
                type_params: Vec::new(),
                params: vec![RustParam::Named {
                    name: "value".to_string(),
                    ty: crate::sifr_type_to_rust_type(inner),
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Self".to_string()])),
                    args: vec![RustExpr::Ident("value".to_string())],
                }))],
                is_async: false,
            },
            RustItem::Fn {
                name: "value".to_string(),
                visibility: method_visibility.clone(),
                type_params: Vec::new(),
                params: vec![RustParam::SelfParam { mutable: false }],
                ret: Some(crate::sifr_type_to_rust_type(inner)),
                body: vec![RustStmt::Return(Some(
                    if inner.ownership() == sifr_type_system::OwnershipKind::Copy {
                        RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: "0".to_string(),
                        }
                    } else {
                        RustExpr::Clone(Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: "0".to_string(),
                        }))
                    },
                ))],
                is_async: false,
            },
        ];
        let saved_class_name = self.current_class_name.clone();
        self.current_class_name = Some(class.name.clone());
        for method in &class.methods {
            impl_items.push(self.lower_type_emitter_method_item(method, module_public));
        }
        self.current_class_name = saved_class_name;

        self.body_items.push(RustItem::Impl {
            target: source_class_rust_name(&class.name),
            type_params: Vec::new(),
            trait_: None,
            items: impl_items,
        });
        self.body_items.push(RustItem::Impl {
            target: source_class_rust_name(&class.name),
            type_params: Vec::new(),
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
                        RustExpr::Literal(RustLiteral::Str("{}".to_string())),
                        RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: "0".to_string(),
                        },
                    ],
                }))],
                is_async: false,
            }],
        });
    }

    pub(crate) fn lower_type_emitter_method_item(
        &mut self,
        method: &HirFunction,
        module_public: bool,
    ) -> RustItem {
        let visibility = if module_public {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let mut params = Vec::new();
        if method.method_kind == MethodKind::Regular && method.name != "new" {
            params.push(Self::rust_receiver_param(method));
        }
        for param in &method.params {
            let mut ty = crate::sifr_type_to_rust_type(&param.ty);
            if param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
                && param.convention.is_borrowed()
            {
                ty = RustType::Ref {
                    mutable: param.convention.is_mut_borrow(),
                    inner: Box::new(ty),
                };
            }
            params.push(RustParam::Named {
                name: param.name.clone(),
                ty,
            });
        }

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
            ret: if method.return_type == Type::None {
                None
            } else {
                Some(crate::sifr_type_to_rust_type(&method.return_type))
            },
            body: self.lower_type_emitter_method_body(method),
            is_async: false,
        }
    }

    pub(crate) fn lower_type_emitter_method_body(&mut self, method: &HirFunction) -> Vec<RustStmt> {
        self.lower_function_like_body(
            method,
            "structured method statement lowering missing for IR-first type emission",
            "structured method statement lowering failed for IR-first type emission",
            |_, _| Option::<Vec<RustStmt>>::None,
        )
    }
}
