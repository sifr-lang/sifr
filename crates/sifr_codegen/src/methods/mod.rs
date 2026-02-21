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
        (Type::Str, "title") => string::lower_title(rendered_object, rendered_args),
        (Type::Str, "capitalize") => string::lower_capitalize(rendered_object, rendered_args),
        (Type::Str, "swapcase") => string::lower_swapcase(rendered_object, rendered_args),
        (Type::Str, "isdigit") => string::lower_isdigit(rendered_object, rendered_args),
        (Type::Str, "isalpha") => string::lower_isalpha(rendered_object, rendered_args),
        (Type::Str, "isalnum") => string::lower_isalnum(rendered_object, rendered_args),
        (Type::Str, "isspace") => string::lower_isspace(rendered_object, rendered_args),
        (Type::Str, "isupper") => string::lower_isupper(rendered_object, rendered_args),
        (Type::Str, "islower") => string::lower_islower(rendered_object, rendered_args),
        (Type::Str, "center") => string::lower_center(rendered_object, rendered_args),
        (Type::Str, "ljust") => string::lower_ljust(rendered_object, rendered_args),
        (Type::Str, "rjust") => string::lower_rjust(rendered_object, rendered_args),
        (Type::Str, "zfill") => string::lower_zfill(rendered_object, rendered_args),
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

        let title = lower_method(&Type::Str, "title", "s", &[]).expect("title lowers");
        assert!(render_expr(&title.expr).contains("split_whitespace"));

        let cap = lower_method(&Type::Str, "capitalize", "s", &[]).expect("capitalize lowers");
        assert!(render_expr(&cap.expr).contains("let _s = (s).clone()"));

        let swap = lower_method(&Type::Str, "swapcase", "s", &[]).expect("swapcase lowers");
        assert!(render_expr(&swap.expr).contains("is_uppercase"));

        let isdigit = lower_method(&Type::Str, "isdigit", "s", &[]).expect("isdigit lowers");
        assert!(render_expr(&isdigit.expr).contains("is_ascii_digit"));

        let isalpha = lower_method(&Type::Str, "isalpha", "s", &[]).expect("isalpha lowers");
        assert!(render_expr(&isalpha.expr).contains("is_alphabetic"));

        let isalnum = lower_method(&Type::Str, "isalnum", "s", &[]).expect("isalnum lowers");
        assert!(render_expr(&isalnum.expr).contains("is_alphanumeric"));

        let isspace = lower_method(&Type::Str, "isspace", "s", &[]).expect("isspace lowers");
        assert!(render_expr(&isspace.expr).contains("is_whitespace"));

        let isupper = lower_method(&Type::Str, "isupper", "s", &[]).expect("isupper lowers");
        assert!(render_expr(&isupper.expr).contains("is_uppercase"));

        let islower = lower_method(&Type::Str, "islower", "s", &[]).expect("islower lowers");
        assert!(render_expr(&islower.expr).contains("is_lowercase"));

        let center = lower_method(&Type::Str, "center", "s", &["5".to_string()])
            .expect("center lowers");
        assert!(render_expr(&center.expr).contains("let _w = 5 as usize"));

        let ljust = lower_method(&Type::Str, "ljust", "s", &["5".to_string()])
            .expect("ljust lowers");
        assert_eq!(
            render_expr(&ljust.expr),
            "format!(\"{:<width$}\", s, width = 5 as usize)"
        );

        let rjust = lower_method(&Type::Str, "rjust", "s", &["5".to_string()])
            .expect("rjust lowers");
        assert_eq!(
            render_expr(&rjust.expr),
            "format!(\"{:>width$}\", s, width = 5 as usize)"
        );

        let zfill = lower_method(&Type::Str, "zfill", "s", &["5".to_string()])
            .expect("zfill lowers");
        assert_eq!(
            render_expr(&zfill.expr),
            "format!(\"{:0>width$}\", s, width = 5 as usize)"
        );
    }
}
