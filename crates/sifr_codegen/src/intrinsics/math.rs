//! Math intrinsic lowerers for registry migration.

use crate::{RustExpr, RustType};

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
    unary_method(args, "trunc")
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
