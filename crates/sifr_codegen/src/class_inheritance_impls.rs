use crate::{RustEmitter, RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};
use sifr_ir::HirClass;
use sifr_type_system::{source_class_rust_name, Type};

impl RustEmitter {
    pub(crate) fn class_parent_deref_impls(class: &HirClass) -> Vec<RustItem> {
        let Some(parent) = class
            .parent_class
            .as_deref()
            .filter(|parent| *parent != "NonSend")
        else {
            return Vec::new();
        };
        let parent_rust_type = class
            .parent_type
            .as_ref()
            .map_or_else(|| source_class_rust_name(parent), Type::rust_type);
        let field = parent.to_lowercase();
        let deref = RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_impl_type_params(class),
            trait_: Some("std::ops::Deref".to_string()),
            items: vec![
                RustItem::TypeAlias {
                    name: "Target".to_string(),
                    ty: RustType::Named(parent_rust_type.clone()),
                },
                RustItem::Fn {
                    name: "deref".to_string(),
                    visibility: Visibility::Private,
                    type_params: Vec::new(),
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::Named("&Self::Target".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: field.clone(),
                        }),
                    }))],
                    is_async: false,
                },
            ],
        };
        let deref_mut = RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_impl_type_params(class),
            trait_: Some("std::ops::DerefMut".to_string()),
            items: vec![RustItem::Fn {
                name: "deref_mut".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![RustParam::SelfParam { mutable: true }],
                ret: Some(RustType::Named("&mut Self::Target".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::Ref {
                    mutable: true,
                    expr: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident("self".to_string())),
                        field: field.clone(),
                    }),
                }))],
                is_async: false,
            }],
        };
        let from_child = RustItem::Impl {
            target: parent_rust_type,
            type_params: Self::class_impl_type_params(class),
            trait_: Some(format!(
                "std::convert::From<{}>",
                Self::class_impl_target(class)
            )),
            items: vec![RustItem::Fn {
                name: "from".to_string(),
                visibility: Visibility::Private,
                type_params: Vec::new(),
                params: vec![RustParam::Named {
                    name: "value".to_string(),
                    ty: RustType::Named(Self::class_impl_target(class)),
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("value".to_string())),
                    field,
                }))],
                is_async: false,
            }],
        };
        vec![deref, deref_mut, from_child]
    }
}
