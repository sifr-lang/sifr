use super::{HirExpr, RustEmitter};
use crate::{RustExpr, RustLiteral, RustStmt};

impl RustEmitter {
    pub(crate) fn lower_registry_unit_string_slice(
        &mut self,
        object: &HirExpr,
        object_expr: RustExpr,
        start: Option<&HirExpr>,
        stop: Option<&HirExpr>,
    ) -> Option<RustExpr> {
        let cached_chars = self.string_char_cache_for_expr(object);
        let cache_override = if let HirExpr::Name { name, .. } = object {
            Some((
                name.clone(),
                self.string_char_cache_vars
                    .insert(name.clone(), "__sifr_slice_src".to_string()),
            ))
        } else {
            None
        };
        let lowered_bounds = (|| {
            let start = if let Some(expr) = start {
                Some(self.lower_stmt_expr_for_ir(expr).ok().flatten()?)
            } else {
                None
            };
            let stop = if let Some(expr) = stop {
                Some(self.lower_stmt_expr_for_ir(expr).ok().flatten()?)
            } else {
                None
            };
            Some((start, stop))
        })();
        if let Some((name, previous)) = cache_override {
            if let Some(previous) = previous {
                self.string_char_cache_vars.insert(name, previous);
            } else {
                self.string_char_cache_vars.remove(&name);
            }
        }
        let (start, stop) = lowered_bounds?;

        let slice_src = if let Some(cache_name) = cached_chars {
            RustExpr::Ref {
                mutable: false,
                expr: Box::new(RustExpr::Ident(cache_name)),
            }
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(object_expr),
                    method: "chars".to_string(),
                    args: Vec::new(),
                }),
                method: "collect::<Vec<char>>".to_string(),
                args: Vec::new(),
            }
        };
        let normalize = |bound: RustExpr| RustExpr::MethodCall {
            receiver: Box::new(bound),
            method: "clamp_slice_bound".to_string(),
            args: vec![RustExpr::Ident("__sifr_slice_len".to_string())],
        };
        let start = start.map_or(RustExpr::Literal(RustLiteral::Int(0)), normalize);
        let stop = stop.map_or_else(
            || RustExpr::Ident("__sifr_slice_len".to_string()),
            normalize,
        );
        let take = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__sifr_slice_stop".to_string())),
            method: "saturating_sub".to_string(),
            args: vec![RustExpr::Ident("__sifr_slice_start".to_string())],
        };
        let iter = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_slice_src".to_string())),
                        method: "iter".to_string(),
                        args: Vec::new(),
                    }),
                    method: "skip".to_string(),
                    args: vec![RustExpr::Ident("__sifr_slice_start".to_string())],
                }),
                method: "take".to_string(),
                args: vec![take],
            }),
            method: "copied".to_string(),
            args: Vec::new(),
        };
        Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_slice_src".to_string(),
                    ty: None,
                    value: slice_src,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_slice_len".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__sifr_slice_src".to_string())),
                        method: "len".to_string(),
                        args: Vec::new(),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_slice_start".to_string(),
                    ty: None,
                    value: start,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_slice_stop".to_string(),
                    ty: None,
                    value: stop,
                },
            ],
            expr: Some(Box::new(RustExpr::MethodCall {
                receiver: Box::new(iter),
                method: "collect::<String>".to_string(),
                args: Vec::new(),
            })),
        })
    }
}
