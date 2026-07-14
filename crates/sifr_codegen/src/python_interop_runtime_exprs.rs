//! Shared Rust-IR helpers for generated Python runtime calls and error mapping.

use crate::rust_interop_error_mapping::bridge_error_expr;
use crate::{RustExpr, RustParam, RustStmt, RustType};
use sifr_type_system::Type;

pub(crate) fn runtime_call(function: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "python".to_string(),
            function.to_string(),
        ])),
        args,
    }
}

pub(crate) fn mapped_let(name: &str, value: RustExpr, error_type: &Type) -> RustStmt {
    RustStmt::Let {
        mutable: false,
        name: name.to_string(),
        ty: None,
        value: mapped_try(value, error_type),
    }
}

pub(crate) fn mapped_try(value: RustExpr, error_type: &Type) -> RustExpr {
    let error_name = "__sifr_python_error";
    RustExpr::Try(Box::new(RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: error_name.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(bridge_error_expr(
                RustExpr::Ident(error_name.to_string()),
                error_type,
            )),
            is_move: false,
        }],
    }))
}
