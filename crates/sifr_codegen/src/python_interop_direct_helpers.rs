use crate::{RustExpr, RustLiteral, RustStmt};
use sifr_ir::PythonParameterKind;

pub(crate) fn reference(name: &str) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(value_place(name)),
    }
}

pub(crate) fn value_place(name: &str) -> RustExpr {
    let mut segments = name.split('.');
    let Some(root) = segments.next() else {
        return RustExpr::Ident(String::new());
    };
    segments.fold(RustExpr::Ident(root.to_string()), |expr, field| {
        RustExpr::Field {
            expr: Box::new(expr),
            field: field.to_string(),
        }
    })
}

pub(crate) fn vector_let(name: &str) -> RustStmt {
    RustStmt::Let {
        mutable: true,
        name: name.to_string(),
        ty: None,
        value: RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
            args: Vec::new(),
        },
    }
}

pub(crate) fn ok_return(value: RustExpr) -> RustStmt {
    RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![value],
    }))
}

pub(crate) fn push_for_shape(
    kind: PythonParameterKind,
    name: &str,
    handle: &str,
) -> Option<RustStmt> {
    match kind {
        PythonParameterKind::Positional => Some(push_positional(handle)),
        PythonParameterKind::KeywordOnly => Some(push_named_keyword(name, handle)),
        PythonParameterKind::PositionalVariadic | PythonParameterKind::KeywordVariadic => None,
    }
}

pub(crate) fn push_named_keyword(name: &str, handle: &str) -> RustStmt {
    push_keyword_expr(
        RustExpr::Literal(RustLiteral::Str(name.to_string())),
        handle,
    )
}

pub(crate) fn push_positional(handle: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__sifr_python_args".to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::Ident(handle.to_string())],
    })
}

pub(crate) fn drop_value(name: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "std".to_string(),
            "mem".to_string(),
            "drop".to_string(),
        ])),
        args: vec![RustExpr::Ident(name.to_string())],
    })
}

pub(crate) fn push_to(vector: &str, value: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(vector.to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::Ident(value.to_string())],
    })
}

pub(crate) fn push_keyword_expr(key: RustExpr, handle: &str) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("__sifr_python_kwargs".to_string())),
        method: "push".to_string(),
        args: vec![RustExpr::Tuple(vec![
            key,
            RustExpr::Ident(handle.to_string()),
        ])],
    })
}
