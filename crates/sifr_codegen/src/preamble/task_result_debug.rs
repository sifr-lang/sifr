use crate::{
    RustExpr, RustItem, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType, RustTypeParam,
    Visibility,
};

fn debug_tuple_expression(name: &str, value: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("f".to_string())),
                method: "debug_tuple".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::StaticStr(name.to_string()))],
            }),
            method: "field".to_string(),
            args: vec![RustExpr::Ident(value.to_string())],
        }),
        method: "finish".to_string(),
        args: vec![],
    }
}

pub(super) fn build_task_result_debug_impl() -> RustItem {
    let arm = |variant: &str, binding: &str| RustMatchArm {
        pattern: format!("Self::{variant}({binding})"),
        bindings: vec![binding.to_string()],
        guard: None,
        body: vec![RustStmt::Return(Some(debug_tuple_expression(
            variant, binding,
        )))],
    };
    RustItem::Impl {
        target: "__SifrTaskResult<T, E>".to_string(),
        type_params: vec![
            RustTypeParam {
                name: "T".to_string(),
                bounds: vec!["std::fmt::Debug".to_string()],
            },
            RustTypeParam {
                name: "E".to_string(),
                bounds: vec!["std::fmt::Debug".to_string()],
            },
        ],
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
            body: vec![RustStmt::Match {
                expr: RustExpr::Ident("self".to_string()),
                arms: vec![
                    arm("Ok", "value"),
                    arm("Err", "failure"),
                    arm("Cancelled", "failure"),
                ],
            }],
            is_async: false,
        }],
    }
}
