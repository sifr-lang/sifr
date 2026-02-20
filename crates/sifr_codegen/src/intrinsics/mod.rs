//! Intrinsic registry and dispatch for incremental migration.

mod math;

use crate::RustExpr;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_crate: Option<&'static str>,
}

pub(crate) fn lower_intrinsic(name: &str, rendered_args: &[String]) -> Option<LoweredIntrinsic> {
    let expr = match name {
        "sqrt" => math::lower_sqrt(rendered_args),
        "floor" => math::lower_floor(rendered_args),
        "ceil" => math::lower_ceil(rendered_args),
        "abs_val" => math::lower_abs_val(rendered_args),
        "log" => math::lower_log(rendered_args),
        "cbrt" => math::lower_cbrt(rendered_args),
        "exp2" => math::lower_exp2(rendered_args),
        "sin" => math::lower_sin(rendered_args),
        "cos" => math::lower_cos(rendered_args),
        "tan" => math::lower_tan(rendered_args),
        "pow_val" => math::lower_pow_val(rendered_args),
        _ => None,
    }?;

    Some(LoweredIntrinsic {
        expr,
        required_crate: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_expr;

    #[test]
    fn lowers_math_intrinsics_via_registry() {
        let lowered = lower_intrinsic("sqrt", &["x".to_string()]).expect("sqrt should lower");
        assert_eq!(render_expr(&lowered.expr), "(x).sqrt()");

        let lowered = lower_intrinsic("pow_val", &["a".to_string(), "b".to_string()])
            .expect("pow_val should lower");
        assert_eq!(render_expr(&lowered.expr), "(a).powf(b)");
    }
}
