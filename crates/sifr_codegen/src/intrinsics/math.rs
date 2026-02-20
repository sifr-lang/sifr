//! Math intrinsic lowerers for registry migration.

use crate::RustExpr;

fn unary_method(args: &[String], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!("({}).{}()", args[0], method)))
}

fn unary_method_as_i64(args: &[String], method: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!("({}).{}() as i64", args[0], method)))
}

fn binary_method(args: &[String], method: &str) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "({}).{}({})",
        args[0], method, args[1]
    )))
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

pub(super) fn lower_floor(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "floor")
}

pub(super) fn lower_ceil(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "ceil")
}

pub(super) fn lower_pow_val(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "powf")
}

pub(super) fn lower_min_val(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __a = {}; let __b = {}; if __a < __b {{ __a }} else {{ __b }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_max_val(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __a = {}; let __b = {}; if __a > __b {{ __a }} else {{ __b }} }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_round_val(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "round")
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

pub(super) fn lower_atan2(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "atan2")
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

pub(super) fn lower_trunc(args: &[String]) -> Option<RustExpr> {
    unary_method_as_i64(args, "trunc")
}

pub(super) fn lower_copysign(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "copysign")
}

pub(super) fn lower_signbit(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "is_sign_negative")
}

pub(super) fn lower_fmod(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!("({}) % ({})", args[0], args[1])))
}

pub(super) fn lower_hypot(args: &[String]) -> Option<RustExpr> {
    binary_method(args, "hypot")
}

pub(super) fn lower_fma(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "({}).mul_add({}, {})",
        args[0], args[1], args[2]
    )))
}

pub(super) fn lower_fmax(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __a: f64 = {}; let __b: f64 = {}; __a.max(__b) }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_fmin(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __a: f64 = {}; let __b: f64 = {}; __a.min(__b) }}",
        args[0], args[1]
    )))
}

pub(super) fn lower_exp(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "exp")
}

pub(super) fn lower_expm1(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "exp_m1")
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
    unary_method(args, "is_subnormal")
}

pub(super) fn lower_acosh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "acosh")
}

pub(super) fn lower_asinh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "asinh")
}

pub(super) fn lower_atanh(args: &[String]) -> Option<RustExpr> {
    unary_method(args, "atanh")
}

pub(super) fn lower_isqrt(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __n = {} as f64; __n.sqrt() as i64 }}",
        args[0]
    )))
}
