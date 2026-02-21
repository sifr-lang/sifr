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
        (Type::Str, "startswith") => string::lower_startswith(rendered_object, rendered_args),
        (Type::Str, "endswith") => string::lower_endswith(rendered_object, rendered_args),
        (Type::Str, "split") => string::lower_split(rendered_object, rendered_args),
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

        let starts =
            lower_method(&Type::Str, "startswith", "s", &["prefix".to_string()])
                .expect("startswith lowers");
        assert_eq!(render_expr(&starts.expr), "s.starts_with(&(prefix))");

        let ends =
            lower_method(&Type::Str, "endswith", "s", &["suffix".to_string()])
                .expect("endswith lowers");
        assert_eq!(render_expr(&ends.expr), "s.ends_with(&(suffix))");

        let split_default = lower_method(&Type::Str, "split", "s", &[]).expect("split default");
        assert!(render_expr(&split_default.expr).contains("split_whitespace"));

        let split_sep =
            lower_method(&Type::Str, "split", "s", &["sep".to_string()]).expect("split sep");
        assert_eq!(
            render_expr(&split_sep.expr),
            "s.split(&(sep)).map(|s| s.to_string()).collect::<Vec<String>>()"
        );
    }
}
