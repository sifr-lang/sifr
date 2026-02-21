//! Method registry and dispatch for incremental migration.

mod string;

use crate::RustExpr;
use sifr_type_system::Type;

pub(crate) struct LoweredMethod {
    pub(crate) expr: RustExpr,
}

pub(crate) fn lower_method(
    object_ty: &Type,
    method: &str,
    rendered_object: &str,
    rendered_args: &[String],
) -> Option<LoweredMethod> {
    let expr = match (object_ty, method) {
        (Type::Str, "upper") => string::lower_upper(rendered_object, rendered_args),
        (Type::Str, "lower") => string::lower_lower(rendered_object, rendered_args),
        (Type::Str, "strip") => string::lower_strip(rendered_object, rendered_args),
        _ => return None,
    };

    Some(LoweredMethod { expr: expr? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_expr;

    #[test]
    fn lowers_string_methods_via_registry() {
        let upper = lower_method(&Type::Str, "upper", "s", &[]).expect("upper lowers");
        assert_eq!(render_expr(&upper.expr), "s.to_uppercase()");

        let lower = lower_method(&Type::Str, "lower", "s", &[]).expect("lower lowers");
        assert_eq!(render_expr(&lower.expr), "s.to_lowercase()");

        let strip = lower_method(&Type::Str, "strip", "s", &[]).expect("strip lowers");
        assert_eq!(render_expr(&strip.expr), "s.trim().to_string()");
    }
}
