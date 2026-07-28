use crate::{RustExpr, RustParam, RustStmt};

pub(crate) fn closure_with_capture_clones(
    params: Vec<RustParam>,
    body: Vec<RustStmt>,
    is_move: bool,
    is_async: bool,
    capture_clones: &[String],
) -> RustExpr {
    let closure = RustExpr::ClosureBlock {
        params,
        body,
        is_move,
        is_async,
    };
    if capture_clones.is_empty() {
        return closure;
    }
    RustExpr::Block {
        stmts: capture_clones
            .iter()
            .map(|name| RustStmt::Let {
                mutable: false,
                name: name.clone(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(name.clone()))),
            })
            .collect(),
        expr: Some(Box::new(closure)),
    }
}
