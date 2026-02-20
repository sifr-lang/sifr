//! Intrinsic registry and dispatch for incremental migration.

mod math;
mod json;

use crate::RustExpr;

pub(crate) struct LoweredIntrinsic {
    pub(crate) expr: RustExpr,
    pub(crate) required_crate: Option<&'static str>,
}

pub(crate) fn lower_intrinsic(name: &str, rendered_args: &[String]) -> Option<LoweredIntrinsic> {
    let (expr, required_crate) = match name {
        "sqrt" => (math::lower_sqrt(rendered_args), None),
        "floor" => (math::lower_floor(rendered_args), None),
        "ceil" => (math::lower_ceil(rendered_args), None),
        "abs_val" => (math::lower_abs_val(rendered_args), None),
        "log" => (math::lower_log(rendered_args), None),
        "cbrt" => (math::lower_cbrt(rendered_args), None),
        "exp2" => (math::lower_exp2(rendered_args), None),
        "sin" => (math::lower_sin(rendered_args), None),
        "cos" => (math::lower_cos(rendered_args), None),
        "tan" => (math::lower_tan(rendered_args), None),
        "pow_val" => (math::lower_pow_val(rendered_args), None),
        "json_loads" => (json::lower_json_loads(rendered_args), Some("serde_json")),
        "json_dumps" => (json::lower_json_dumps(rendered_args), Some("serde_json")),
        _ => return None,
    };

    Some(LoweredIntrinsic {
        expr: expr?,
        required_crate,
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

    #[test]
    fn lowers_json_intrinsics_with_dependency_metadata() {
        let loads = lower_intrinsic("json_loads", &["payload".to_string()])
            .expect("json_loads should lower");
        assert_eq!(loads.required_crate, Some("serde_json"));
        assert!(render_expr(&loads.expr).contains("serde_json::from_str"));

        let dumps = lower_intrinsic("json_dumps", &["value".to_string()])
            .expect("json_dumps should lower");
        assert_eq!(dumps.required_crate, Some("serde_json"));
        assert_eq!(
            render_expr(&dumps.expr),
            "serde_json::to_string(&value).unwrap_or_default()"
        );
    }
}
