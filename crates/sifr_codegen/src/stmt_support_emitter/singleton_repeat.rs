use crate::{RustEmitter, RustExpr, RustParam, RustStmt, RustType};
use sifr_ir::HirExpr;

impl RustEmitter {
    /// Retain exact integer counts without a host-sized cast or a temporary list.
    pub(crate) fn lower_singleton_repeat_for_ir(
        &self,
        element: RustExpr,
        count_source: &HirExpr,
        count: RustExpr,
        count_first: bool,
    ) -> RustExpr {
        let mut operands = vec![
            RustStmt::Let {
                mutable: false,
                name: "__sifr_repeat_values".to_string(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "iter".to_string(),
                        "repeat".to_string(),
                    ])),
                    args: vec![element],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__sifr_repeat_n".to_string(),
                ty: None,
                value: self.materialize_reusable_value_for_ir(count_source, count),
            },
        ];
        if count_first {
            operands.swap(0, 1);
        }
        let range = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "SifrRange".to_string(),
                "new_known_nonzero".to_string(),
            ])),
            args: vec![
                Self::int_sifr_literal_expr(0),
                RustExpr::Ident("__sifr_repeat_n".to_string()),
                Self::int_sifr_literal_expr(1),
            ],
        };
        let zipped = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__sifr_repeat_values".to_string())),
            method: "zip".to_string(),
            args: vec![range],
        };
        let values = RustExpr::MethodCall {
            receiver: Box::new(zipped),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_repeat_pair".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("__sifr_repeat_pair".to_string())),
                    field: "0".to_string(),
                }),
                is_move: false,
            }],
        };
        RustExpr::Block {
            stmts: operands,
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(values),
                method: "collect::<Vec<_>>".to_string(),
                args: vec![],
            })),
        }
    }
}
