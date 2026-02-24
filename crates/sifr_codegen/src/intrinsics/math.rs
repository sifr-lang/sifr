//! Math intrinsic lowerers for registry migration.

use crate::{RustExpr, RustLiteral, RustStmt, RustType};

fn unary_method(args: &[String], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    // Wrap the argument in parentheses to ensure proper precedence
    let arg = format!("({})", args[0]);
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(arg)),
        method: method.to_string(),
        args: vec![],
    })
}

fn unary_method_as_i64(args: &[String], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let arg = format!("({})", args[0]);
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(arg)),
            method: method.to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

fn binary_method(args: &[String], method: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    // Wrap the first arg in parentheses, but pass second arg as-is for clean output
    let arg0 = format!("({})", args[0]);
    let arg1 = args[1].clone();
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(arg0)),
        method: method.to_string(),
        args: vec![RustExpr::Ident(arg1)],
    })
}

pub(super) fn lower_sqrt(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "sqrt")
}

pub(super) fn lower_abs_val(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "abs")
}

pub(super) fn lower_log(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "ln")
}

pub(super) fn lower_cbrt(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "cbrt")
}

pub(super) fn lower_exp2(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "exp2")
}

pub(super) fn lower_sin(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "sin")
}

pub(super) fn lower_cos(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "cos")
}

pub(super) fn lower_tan(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "tan")
}

pub(super) fn lower_asin(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "asin")
}

pub(super) fn lower_acos(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "acos")
}

pub(super) fn lower_atan(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "atan")
}

pub(super) fn lower_sinh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "sinh")
}

pub(super) fn lower_cosh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "cosh")
}

pub(super) fn lower_tanh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "tanh")
}

pub(super) fn lower_asinh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "asinh")
}

pub(super) fn lower_acosh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "acosh")
}

pub(super) fn lower_atanh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "atanh")
}

pub(super) fn lower_floor(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "floor")
}

pub(super) fn lower_ceil(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "ceil")
}

pub(super) fn lower_round(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "round")
}

pub(super) fn lower_trunc(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "trunc")
}

pub(super) fn lower_fract(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "fract")
}

pub(super) fn lower_exp(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "exp")
}

pub(super) fn lower_ln(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "ln")
}

pub(super) fn lower_log10(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "log10")
}

pub(super) fn lower_log2(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "log2")
}

pub(super) fn lower_degrees(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "to_degrees")
}

pub(super) fn lower_radians(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "to_radians")
}

pub(super) fn lower_isnan(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "is_nan")
}

pub(super) fn lower_isinf(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "is_infinite")
}

pub(super) fn lower_copysign(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "copysign")
}

pub(super) fn lower_signbit(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "is_sign_negative")
}

pub(super) fn lower_fmod(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "rem_euclid")
}

pub(super) fn lower_hypot(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "hypot")
}

pub(super) fn lower_fma(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    // fused_multiply_add(a, b, c) => (a * b) + c
    let a = format!("({})", args[0]);
    let b = format!("({})", args[1]);
    let c = format!("({})", args[2]);
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident(a)),
            op: "*".to_string(),
            right: Box::new(RustExpr::Ident(b)),
        }),
        op: "+".to_string(),
        right: Box::new(RustExpr::Ident(c)),
    })
}

pub(super) fn lower_pow_val(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    // Wrap the base in parentheses, but pass exponent as-is for clean output
    let base = format!("({})", args[0]);
    let exp = args[1].clone();
    // pow takes (base, exponent) - we need to use powf for floats
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(base)),
        method: "powf".to_string(),
        args: vec![RustExpr::Ident(exp)],
    })
}

pub(super) fn lower_min_val(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "min")
}

pub(super) fn lower_max_val(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "max")
}

pub(super) fn lower_fmax(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "max")
}

pub(super) fn lower_fmin(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "min")
}

pub(super) fn lower_expm1(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "exp_m1")
}

pub(super) fn lower_round_val(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "round")
}

pub(super) fn lower_atan2(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "atan2")
}

pub(super) fn lower_log1p(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "ln_1p")
}

pub(super) fn lower_fabs(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "abs")
}

pub(super) fn lower_isfinite(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "is_finite")
}

pub(super) fn lower_isnormal(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "is_normal")
}

pub(super) fn lower_issubnormal(args: &[String]) -> Option<RustExpr> {
    // is_subnormal is not directly available, check using classification
    // A number is subnormal if it is finite but not normal
    if args.len() != 1 {
        return None;
    }
    let arg = format!("({})", args[0]);
    // Use is_finite && !is_normal
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(arg.clone())),
            method: "is_finite".to_string(),
            args: vec![],
        }),
        op: "&&".to_string(),
        right: Box::new(RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(arg)),
                method: "is_normal".to_string(),
                args: vec![],
            }),
        }),
    })
}

pub(super) fn lower_isqrt(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    let arg = format!("({})", args[0]);
    // Integer square root: (n as f64).sqrt() as i64
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Ident(arg)),
                ty: RustType::F64,
            }),
            method: "sqrt".to_string(),
            args: vec![],
        }),
        ty: RustType::I64,
    })
}

pub(super) fn lower_remainder(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __x: f64 = ({}); let __y: f64 = ({}); if __x.is_nan() || __y.is_nan() {{ f64::NAN }} else if __y == 0.0 || __x.is_infinite() {{ f64::NAN }} else if __y.is_infinite() {{ __x }} else {{ let __q = __x / __y; let __n0 = __q.trunc(); let __frac = __q - __n0; let __abs_frac = __frac.abs(); let __n = if __abs_frac < 0.5 {{ __n0 }} else if __abs_frac > 0.5 {{ __n0 + __q.signum() }} else if (__n0 as i64) % 2 == 0 {{ __n0 }} else {{ __n0 + __q.signum() }}; let __r = __x - __n * __y; if __r == 0.0 {{ 0.0f64.copysign(__x) }} else {{ __r }} }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_dist(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __p = &({}); let __q = &({}); if __p.len() != __q.len() {{ f64::NAN }} else if __p.is_empty() {{ 0.0 }} else {{ let mut __scale = 0.0f64; let mut __ssq = 1.0f64; for __i in 0..__p.len() {{ let __d = (__p[__i] - __q[__i]).abs(); if __d != 0.0 {{ if __scale < __d {{ let __r = __scale / __d; __ssq = 1.0 + __ssq * __r * __r; __scale = __d; }} else {{ let __r = __d / __scale; __ssq += __r * __r; }} }} }} if __scale == 0.0 {{ 0.0 }} else {{ __scale * __ssq.sqrt() }} }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_fsum(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __data = &({}); let mut __sum = 0.0f64; let mut __comp = 0.0f64; let mut __pos_inf = false; let mut __neg_inf = false; let mut __has_nan = false; for __x in __data.iter() {{ let __v = *__x; if __v.is_nan() {{ __has_nan = true; continue; }} if __v.is_infinite() {{ if __v.is_sign_positive() {{ __pos_inf = true; }} else {{ __neg_inf = true; }} continue; }} let __t = __sum + __v; if __sum.abs() >= __v.abs() {{ __comp += (__sum - __t) + __v; }} else {{ __comp += (__v - __t) + __sum; }} __sum = __t; }} if __has_nan || (__pos_inf && __neg_inf) {{ f64::NAN }} else if __pos_inf {{ f64::INFINITY }} else if __neg_inf {{ f64::NEG_INFINITY }} else {{ __sum + __comp }} }}",
        args[0]
    )))
}

pub(super) fn lower_sumprod(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__p".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[0].clone()),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__q".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(args[1].clone()),
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__len".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__p".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }),
                    method: "min".to_string(),
                    args: vec![RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("__q".to_string())),
                        method: "len".to_string(),
                        args: vec![],
                    }],
                },
            },
            RustStmt::Let {
                mutable: true,
                name: "__sum".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Literal(RustLiteral::Float(0.0)),
            },
            RustStmt::For {
                var: "__i".to_string(),
                iter: RustExpr::Range {
                    start: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    end: Box::new(RustExpr::Ident("__len".to_string())),
                },
                body: vec![RustStmt::AugAssign {
                    target: RustExpr::Ident("__sum".to_string()),
                    op: "+".to_string(),
                    value: RustExpr::BinOp {
                        left: Box::new(RustExpr::Index {
                            expr: Box::new(RustExpr::Ident("__p".to_string())),
                            index: Box::new(RustExpr::Ident("__i".to_string())),
                        }),
                        op: "*".to_string(),
                        right: Box::new(RustExpr::Index {
                            expr: Box::new(RustExpr::Ident("__q".to_string())),
                            index: Box::new(RustExpr::Ident("__i".to_string())),
                        }),
                    },
                }],
            },
        ],
        expr: Some(Box::new(RustExpr::Ident("__sum".to_string()))),
    })
}

pub(super) fn lower_erf(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __x: f64 = ({}); let __t = 1.0 / (1.0 + 0.3275911 * __x.abs()); let __poly = __t * (0.254829592 + __t * (-0.284496736 + __t * (1.421413741 + __t * (-1.453152027 + __t * 1.061405429)))); let __r = 1.0 - __poly * (-__x * __x).exp(); if __x >= 0.0 {{ __r }} else {{ -__r }} }}",
        args[0]
    )))
}

pub(super) fn lower_erfc(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __x: f64 = ({}); let __t = 1.0 / (1.0 + 0.3275911 * __x.abs()); let __poly = __t * (0.254829592 + __t * (-0.284496736 + __t * (1.421413741 + __t * (-1.453152027 + __t * 1.061405429)))); let __r = __poly * (-__x * __x).exp(); if __x >= 0.0 {{ __r }} else {{ 2.0 - __r }} }}",
        args[0]
    )))
}

pub(super) fn lower_gamma(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __x: f64 = ({}); if __x <= 0.0 && __x == __x.floor() {{ f64::INFINITY }} else {{ let __g = 7usize; let __c = [0.99999999999980993f64, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]; let __z = if __x < 0.5 {{ let __y = std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * {{ let __xn = 1.0 - __x; let mut __s = __c[0]; for __i in 1..=__g+1 {{ __s += __c[__i] / (__xn + __i as f64 - 1.0); }} let __t2 = __xn + __g as f64 - 0.5; (2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xn - 0.5) * (-__t2).exp() * __s }}); __y }} else {{ let __xm = __x - 1.0; let mut __s = __c[0]; for __i in 1..=__g+1 {{ __s += __c[__i] / (__xm + __i as f64); }} let __t2 = __xm + __g as f64 + 0.5; (2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xm + 0.5) * (-__t2).exp() * __s }}; __z }} }}",
        args[0]
    )))
}

pub(super) fn lower_lgamma(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __x: f64 = ({}); if __x <= 0.0 && __x == __x.floor() {{ f64::INFINITY }} else {{ let __g = 7usize; let __c = [0.99999999999980993f64, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]; let __xm = if __x < 0.5 {{ 1.0 - __x }} else {{ __x - 1.0 }}; let mut __s = __c[0]; for __i in 1..=__g+1 {{ __s += __c[__i] / (__xm + __i as f64); }} let __t2 = __xm + __g as f64 + 0.5; let __r = (2.0 * std::f64::consts::PI).sqrt().ln() + (__xm + 0.5) * __t2.ln() - __t2 + __s.ln(); if __x < 0.5 {{ (std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * __r.exp())).abs().ln() }} else {{ __r }} }} }}",
        args[0]
    )))
}

pub(super) fn lower_frexp(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __x: f64 = ({}); if __x == 0.0 {{ vec![__x, 0.0] }} else if !__x.is_finite() {{ vec![__x, 0.0] }} else {{ let __bits = __x.to_bits(); let __sign = __bits & 0x8000000000000000; let __exp = ((__bits >> 52) & 0x7ff) as i32; let __frac = __bits & 0x000fffffffffffff; if __exp == 0 {{ let __scaled = __x * (2.0f64).powi(54); let __sbits = __scaled.to_bits(); let __sexp = ((__sbits >> 52) & 0x7ff) as i32; let __sfrac = __sbits & 0x000fffffffffffff; let __mant = f64::from_bits(__sign | (0x3feu64 << 52) | __sfrac); let __e = __sexp - 1022 - 54; vec![__mant, __e as f64] }} else {{ let __mant = f64::from_bits(__sign | (0x3feu64 << 52) | __frac); let __e = __exp - 1022; vec![__mant, __e as f64] }} }} }}",
        args[0]
    )))
}

pub(super) fn lower_ldexp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::BinOp {
        left: Box::new(RustExpr::Cast {
            expr: Box::new(args[0].clone()),
            ty: RustType::F64,
        }),
        op: "*".to_string(),
        right: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Float(2.0))),
                ty: RustType::F64,
            }),
            method: "powi".to_string(),
            args: vec![RustExpr::Cast {
                expr: Box::new(args[1].clone()),
                ty: RustType::Named("i32".to_string()),
            }],
        }),
    })
}

pub(super) fn lower_modf(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__x".to_string(),
            ty: Some(RustType::F64),
            value: RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::F64,
            },
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                method: "is_nan".to_string(),
                args: vec![],
            }),
            then_expr: Box::new(RustExpr::Vec(vec![
                RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
                RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()]),
            ])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                    method: "is_infinite".to_string(),
                    args: vec![],
                }),
                then_expr: Box::new(RustExpr::Vec(vec![
                    RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                            ty: RustType::F64,
                        }),
                        method: "copysign".to_string(),
                        args: vec![RustExpr::Ident("__x".to_string())],
                    },
                    RustExpr::Ident("__x".to_string()),
                ])),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: false,
                            name: "__int".to_string(),
                            ty: None,
                            value: RustExpr::MethodCall {
                                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                method: "trunc".to_string(),
                                args: vec![],
                            },
                        },
                        RustStmt::Let {
                            mutable: true,
                            name: "__frac".to_string(),
                            ty: None,
                            value: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__x".to_string())),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__int".to_string())),
                            },
                        },
                        RustStmt::If {
                            cond: RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__frac".to_string())),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                            },
                            then_body: vec![RustStmt::Assign {
                                target: RustExpr::Ident("__frac".to_string()),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                        ty: RustType::F64,
                                    }),
                                    method: "copysign".to_string(),
                                    args: vec![RustExpr::Ident("__x".to_string())],
                                },
                            }],
                            else_body: None,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::Vec(vec![
                        RustExpr::Ident("__frac".to_string()),
                        RustExpr::Ident("__int".to_string()),
                    ]))),
                })),
            })),
        })),
    })
}

pub(super) fn lower_nextafter(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: false,
                name: "__x".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Cast {
                    expr: Box::new(args[0].clone()),
                    ty: RustType::F64,
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "__y".to_string(),
                ty: Some(RustType::F64),
                value: RustExpr::Cast {
                    expr: Box::new(args[1].clone()),
                    ty: RustType::F64,
                },
            },
        ],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::BinOp {
                left: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                    method: "is_nan".to_string(),
                    args: vec![],
                }),
                op: "||".to_string(),
                right: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__y".to_string())),
                    method: "is_nan".to_string(),
                    args: vec![],
                }),
            }),
            then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("__x".to_string())),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::Ident("__y".to_string())),
                }),
                then_expr: Box::new(RustExpr::Ident("__y".to_string())),
                else_expr: Some(Box::new(RustExpr::If {
                    cond: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__x".to_string())),
                        op: "==".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                    }),
                    then_expr: Box::new(RustExpr::Block {
                        stmts: vec![RustStmt::Let {
                            mutable: false,
                            name: "__sign".to_string(),
                            ty: Some(RustType::Named("u64".to_string())),
                            value: RustExpr::If {
                                cond: Box::new(RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__y".to_string())),
                                    method: "is_sign_negative".to_string(),
                                    args: vec![],
                                }),
                                then_expr: Box::new(RustExpr::BinOp {
                                    left: Box::new(RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    }),
                                    op: "<<".to_string(),
                                    right: Box::new(RustExpr::Literal(RustLiteral::Int(63))),
                                }),
                                else_expr: Some(Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                                    ty: RustType::Named("u64".to_string()),
                                })),
                            },
                        }],
                        expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "f64".to_string(),
                                "from_bits".to_string(),
                            ])),
                            args: vec![RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__sign".to_string())),
                                op: "|".to_string(),
                                right: Box::new(RustExpr::Cast {
                                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                    ty: RustType::Named("u64".to_string()),
                                }),
                            }],
                        })),
                    }),
                    else_expr: Some(Box::new(RustExpr::Block {
                        stmts: vec![
                            RustStmt::Let {
                                mutable: true,
                                name: "__bits".to_string(),
                                ty: Some(RustType::Named("u64".to_string())),
                                value: RustExpr::MethodCall {
                                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                                    method: "to_bits".to_string(),
                                    args: vec![],
                                },
                            },
                            RustStmt::If {
                                cond: RustExpr::BinOp {
                                    left: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__x".to_string())),
                                        op: "<".to_string(),
                                        right: Box::new(RustExpr::Ident("__y".to_string())),
                                    }),
                                    op: "==".to_string(),
                                    right: Box::new(RustExpr::BinOp {
                                        left: Box::new(RustExpr::Ident("__x".to_string())),
                                        op: ">".to_string(),
                                        right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                                    }),
                                },
                                then_body: vec![RustStmt::AugAssign {
                                    target: RustExpr::Ident("__bits".to_string()),
                                    op: "+".to_string(),
                                    value: RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    },
                                }],
                                else_body: Some(vec![RustStmt::AugAssign {
                                    target: RustExpr::Ident("__bits".to_string()),
                                    op: "-".to_string(),
                                    value: RustExpr::Cast {
                                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                        ty: RustType::Named("u64".to_string()),
                                    },
                                }]),
                            },
                        ],
                        expr: Some(Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "f64".to_string(),
                                "from_bits".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("__bits".to_string())],
                        })),
                    })),
                })),
            })),
        })),
    })
}

pub(super) fn lower_ulp(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Block {
        stmts: vec![RustStmt::Let {
            mutable: false,
            name: "__x".to_string(),
            ty: Some(RustType::F64),
            value: RustExpr::Cast {
                expr: Box::new(args[0].clone()),
                ty: RustType::F64,
            },
        }],
        expr: Some(Box::new(RustExpr::If {
            cond: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__x".to_string())),
                method: "is_nan".to_string(),
                args: vec![],
            }),
            then_expr: Box::new(RustExpr::Path(vec!["f64".to_string(), "NAN".to_string()])),
            else_expr: Some(Box::new(RustExpr::If {
                cond: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__x".to_string())),
                    method: "is_infinite".to_string(),
                    args: vec![],
                }),
                then_expr: Box::new(RustExpr::Path(vec![
                    "f64".to_string(),
                    "INFINITY".to_string(),
                ])),
                else_expr: Some(Box::new(RustExpr::Block {
                    stmts: vec![RustStmt::Let {
                        mutable: false,
                        name: "__a".to_string(),
                        ty: None,
                        value: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__x".to_string())),
                            method: "abs".to_string(),
                            args: vec![],
                        },
                    }],
                    expr: Some(Box::new(RustExpr::If {
                        cond: Box::new(RustExpr::BinOp {
                            left: Box::new(RustExpr::Ident("__a".to_string())),
                            op: "==".to_string(),
                            right: Box::new(RustExpr::Literal(RustLiteral::Float(0.0))),
                        }),
                        then_expr: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "f64".to_string(),
                                "from_bits".to_string(),
                            ])),
                            args: vec![RustExpr::Cast {
                                expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                ty: RustType::Named("u64".to_string()),
                            }],
                        }),
                        else_expr: Some(Box::new(RustExpr::If {
                            cond: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__a".to_string())),
                                op: "==".to_string(),
                                right: Box::new(RustExpr::Path(vec![
                                    "f64".to_string(),
                                    "MAX".to_string(),
                                ])),
                            }),
                            then_expr: Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::Ident("__a".to_string())),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "f64".to_string(),
                                        "from_bits".to_string(),
                                    ])),
                                    args: vec![RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__a".to_string())),
                                            method: "to_bits".to_string(),
                                            args: vec![],
                                        }),
                                        op: "-".to_string(),
                                        right: Box::new(RustExpr::Cast {
                                            expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                            ty: RustType::Named("u64".to_string()),
                                        }),
                                    }],
                                }),
                            }),
                            else_expr: Some(Box::new(RustExpr::BinOp {
                                left: Box::new(RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "f64".to_string(),
                                        "from_bits".to_string(),
                                    ])),
                                    args: vec![RustExpr::BinOp {
                                        left: Box::new(RustExpr::MethodCall {
                                            receiver: Box::new(RustExpr::Ident("__a".to_string())),
                                            method: "to_bits".to_string(),
                                            args: vec![],
                                        }),
                                        op: "+".to_string(),
                                        right: Box::new(RustExpr::Cast {
                                            expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                                            ty: RustType::Named("u64".to_string()),
                                        }),
                                    }],
                                }),
                                op: "-".to_string(),
                                right: Box::new(RustExpr::Ident("__a".to_string())),
                            })),
                        })),
                    })),
                })),
            })),
        })),
    })
}
