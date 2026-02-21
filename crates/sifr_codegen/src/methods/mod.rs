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
        (Type::Str, "replace") => string::lower_replace(rendered_object, rendered_args),
        (Type::Str, "find") => string::lower_find(rendered_object, rendered_args),
        (Type::Str, "lstrip") => string::lower_lstrip(rendered_object, rendered_args),
        (Type::Str, "rstrip") => string::lower_rstrip(rendered_object, rendered_args),
        (Type::Str, "count") => string::lower_count(rendered_object, rendered_args),
        (Type::Str, "join") => string::lower_join(rendered_object, rendered_args),
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

        let replace = lower_method(
            &Type::Str,
            "replace",
            "s",
            &["old".to_string(), "new".to_string()],
        )
        .expect("replace lowers");
        assert_eq!(render_expr(&replace.expr), "s.replace(&(old), &(new))");

        let find = lower_method(&Type::Str, "find", "s", &["needle".to_string()])
            .expect("find lowers");
        assert_eq!(render_expr(&find.expr), "s.find(&(needle)).map(|i| i as i64)");

        let lstrip = lower_method(&Type::Str, "lstrip", "s", &[]).expect("lstrip lowers");
        assert_eq!(render_expr(&lstrip.expr), "s.trim_start().to_string()");

        let rstrip = lower_method(&Type::Str, "rstrip", "s", &[]).expect("rstrip lowers");
        assert_eq!(render_expr(&rstrip.expr), "s.trim_end().to_string()");

        let count = lower_method(&Type::Str, "count", "s", &["needle".to_string()])
            .expect("count lowers");
        assert_eq!(render_expr(&count.expr), "s.matches(&(needle)).count() as i64");

        let join =
            lower_method(&Type::Str, "join", "sep", &["parts".to_string()]).expect("join lowers");
        assert_eq!(render_expr(&join.expr), "parts.join(&(sep))");
    }
}
