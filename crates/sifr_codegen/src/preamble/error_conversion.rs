use crate::{RustExpr, RustItem, RustParam, RustStmt, RustType, Visibility};

pub fn build_error_into_error_impl(source_name: &str) -> RustItem {
    RustItem::Impl {
        target: "Error".to_string(),
        type_params: vec![],
        trait_: Some(format!("From<{source_name}>")),
        items: vec![RustItem::Fn {
            name: "from".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "err".to_string(),
                ty: RustType::Named(source_name.to_string()),
            }],
            ret: Some(RustType::Named("Self".to_string())),
            body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Self".to_string(), "new".to_string()])),
                args: vec![RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("err".to_string())),
                    field: "message".to_string(),
                }],
            }))],
            is_async: false,
        }],
    }
}
