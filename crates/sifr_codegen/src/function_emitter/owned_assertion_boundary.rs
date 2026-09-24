//! Consuming an owned optional assertion argument also owns its destruction.
use crate::{RustEmitter, RustExpr, RustStmt};
use sifr_ir::{HirExpr, HirFunction, HirStmt};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn owned_assertion_drop(func: &HirFunction) -> Option<RustStmt> {
        let [parameter] = func.params.as_slice() else {
            return None;
        };
        if !parameter.convention.is_owned()
            || !matches!(func.return_type.resolve_alias(), Type::None)
        {
            return None;
        }
        let Type::Union(parts) = parameter.ty.resolve_alias() else {
            return None;
        };
        if parts.len() != 2
            || !parts
                .iter()
                .any(|part| matches!(part.resolve_alias(), Type::None))
            || !parts
                .iter()
                .any(|part| matches!(part.resolve_alias(), Type::TypeVar(_)))
        {
            return None;
        }
        let [
            HirStmt::Assert {
                test:
                    HirExpr::Compare {
                        left,
                        ops,
                        comparators,
                        ..
                    },
                msg: None,
            },
        ] = func.body.as_slice()
        else {
            return None;
        };
        if !matches!(left.as_ref(), HirExpr::Name { name, .. } if name == &parameter.name)
            || !matches!(ops.as_slice(), [op] if op == "is" || op == "is not")
            || !matches!(comparators.as_slice(), [HirExpr::NoneLiteral])
        {
            return None;
        }
        // Keep this after the assertion: a failing assertion must unwind before
        // destroying the payload, exactly as the implicit owned-parameter drop.
        Some(RustStmt::Expr(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "mem".to_string(),
                "drop".to_string(),
            ])),
            args: vec![RustExpr::Ident(parameter.name.clone())],
        }))
    }
}
