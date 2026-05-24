//! Calendar intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn int(v: i64) -> RustExpr {
    RustExpr::Literal(RustLiteral::Int(v))
}

fn month_index(month_ident: &str) -> RustExpr {
    RustExpr::Cast {
        expr: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(month_ident.to_string())),
            op: "-".to_string(),
            right: Box::new(int(1)),
        }),
        ty: RustType::Named("usize".to_string()),
    }
}

fn leap_year_expr(year_ident: &str) -> RustExpr {
    RustExpr::BinOp {
        left: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident(year_ident.to_string())),
                    op: "%".to_string(),
                    right: Box::new(int(4)),
                }),
                op: "==".to_string(),
                right: Box::new(int(0)),
            }),
            op: "&&".to_string(),
            right: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident(year_ident.to_string())),
                    op: "%".to_string(),
                    right: Box::new(int(100)),
                }),
                op: "!=".to_string(),
                right: Box::new(int(0)),
            }),
        }),
        op: "||".to_string(),
        right: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::Ident(year_ident.to_string())),
                op: "%".to_string(),
                right: Box::new(int(400)),
            }),
            op: "==".to_string(),
            right: Box::new(int(0)),
        }),
    }
}

fn month_eq(month_ident: &str, val: i64) -> RustExpr {
    RustExpr::BinOp {
        left: Box::new(RustExpr::Ident(month_ident.to_string())),
        op: "==".to_string(),
        right: Box::new(int(val)),
    }
}

fn month_in(month_ident: &str, vals: &[i64]) -> RustExpr {
    let mut iter = vals.iter();
    let Some(first) = iter.next() else {
        return RustExpr::Literal(RustLiteral::Bool(false));
    };
    let mut acc = month_eq(month_ident, *first);
    for v in iter {
        acc = RustExpr::BinOp {
            left: Box::new(acc),
            op: "||".to_string(),
            right: Box::new(month_eq(month_ident, *v)),
        };
    }
    acc
}

fn weekday_expr(year_ident: &str, month_ident: &str, day_expr: RustExpr) -> RustExpr {
    RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__t".to_string(),
                ty: None,
                value: RustExpr::Vec(vec![
                    int(0),
                    int(3),
                    int(2),
                    int(5),
                    int(0),
                    int(3),
                    int(5),
                    int(1),
                    int(4),
                    int(6),
                    int(2),
                    int(4),
                ]),
            },
            RustStmt::Let {
                mutable: false,
                name: "__y".to_string(),
                ty: None,
                value: RustExpr::If {
                    cond: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident(month_ident.to_string())),
                        op: "<".to_string(),
                        right: Box::new(int(3)),
                    }),
                    then_expr: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident(year_ident.to_string())),
                        op: "-".to_string(),
                        right: Box::new(int(1)),
                    }),
                    else_expr: Some(Box::new(RustExpr::Ident(year_ident.to_string()))),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__wd_raw".to_string(),
                ty: None,
                value: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__y".to_string())),
                                            op: "+".to_string(),
                                            right: Box::new(RustExpr::BinOp {
                                                left: Box::new(RustExpr::Ident("__y".to_string())),
                                                op: "/".to_string(),
                                                right: Box::new(int(4)),
                                            }),
                                        }),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::BinOp {
                                            left: Box::new(RustExpr::Ident("__y".to_string())),
                                            op: "/".to_string(),
                                            right: Box::new(int(100)),
                                        }),
                                    }),
                                    op: "+".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__y".to_string())),
                                        op: "/".to_string(),
                                        right: Box::new(int(400)),
                                    }),
                                }),
                                op: "+".to_string(),
                                right: Box::new(RustExpr::Index {
                                    expr: Box::new(RustExpr::Ident("__t".to_string())),
                                    index: Box::new(month_index(month_ident)),
                                }),
                            }),
                            op: "+".to_string(),
                            right: Box::new(day_expr),
                        }),
                        op: "%".to_string(),
                        right: Box::new(int(7)),
                    }),
                    op: "+".to_string(),
                    right: Box::new(int(6)),
                },
            },
        ],
        expr: Some(Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__wd_raw".to_string())),
            op: "%".to_string(),
            right: Box::new(int(7)),
        })),
    }
}

pub(crate) fn lower_calendar_isleap(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__y".to_string(),
            ty: None,
            value: arg_expr(args, 0),
        }],
        expr: Some(Box::new(leap_year_expr("__y"))),
    })
}

pub(crate) fn lower_calendar_weekday(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__y0".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__m0".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__d0".to_string(),
                ty: None,
                value: arg_expr(args, 2),
            },
        ],
        expr: Some(Box::new(weekday_expr(
            "__y0",
            "__m0",
            RustExpr::Ident("__d0".to_string()),
        ))),
    })
}

pub(crate) fn lower_calendar_monthrange(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__y".to_string(),
                ty: None,
                value: arg_expr(args, 0),
            },
            RustStmt::Let {
                mutable: false,
                name: "__m".to_string(),
                ty: None,
                value: arg_expr(args, 1),
            },
            RustStmt::Let {
                mutable: false,
                name: "__days".to_string(),
                ty: None,
                value: RustExpr::If {
                    cond: Box::new(month_in("__m", &[1, 3, 5, 7, 8, 10, 12])),
                    then_expr: Box::new(int(31)),
                    else_expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(month_in("__m", &[4, 6, 9, 11])),
                        then_expr: Box::new(int(30)),
                        else_expr: Some(Box::new(RustExpr::If {
                            cond: Box::new(month_eq("__m", 2)),
                            then_expr: Box::new(RustExpr::If {
                                cond: Box::new(leap_year_expr("__y")),
                                then_expr: Box::new(int(29)),
                                else_expr: Some(Box::new(int(28))),
                            }),
                            else_expr: Some(Box::new(int(30))),
                        })),
                    })),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__wd".to_string(),
                ty: None,
                value: weekday_expr("__y", "__m", int(1)),
            },
        ],
        expr: Some(Box::new(RustExpr::Vec(vec![
            RustExpr::Ident("__wd".to_string()),
            RustExpr::Ident("__days".to_string()),
        ]))),
    })
}
