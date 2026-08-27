use crate::{
    RustEmitter, RustExpr, RustItem, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use sifr_ir::HirClass;

impl RustEmitter {
    pub(super) fn class_debug_type_params(class: &HirClass) -> Vec<RustTypeParam> {
        class
            .type_params
            .iter()
            .map(|name| {
                let mut bounds = Self::class_base_type_param_bounds(class, name);
                if class
                    .fields
                    .iter()
                    .any(|(_, ty)| Self::type_mentions_type_param(ty, name))
                {
                    bounds.push("std::fmt::Debug".to_string());
                }
                RustTypeParam {
                    name: name.clone(),
                    bounds,
                }
            })
            .collect()
    }

    pub(crate) fn build_debug_impl_for_error(class: &HirClass) -> RustItem {
        let mut debug_expr = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("f".to_string())),
            method: "debug_struct".to_string(),
            args: vec![RustExpr::compiler_fragment(format!("{:?}", class.name))],
        };
        for (field_name, _) in &class.fields {
            debug_expr = RustExpr::MethodCall {
                receiver: Box::new(debug_expr),
                method: "field".to_string(),
                args: vec![
                    RustExpr::compiler_fragment(format!("{field_name:?}")),
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(RustExpr::Field {
                            expr: Box::new(RustExpr::Ident("self".to_string())),
                            field: field_name.clone(),
                        }),
                    },
                ],
            };
        }
        debug_expr = RustExpr::MethodCall {
            receiver: Box::new(debug_expr),
            method: "finish".to_string(),
            args: Vec::new(),
        };

        RustItem::Impl {
            target: Self::class_impl_target(class),
            type_params: Self::class_debug_type_params(class),
            trait_: Some("std::fmt::Debug".to_string()),
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
                body: vec![RustStmt::Return(Some(debug_expr))],
                is_async: false,
            }],
        }
    }
}
