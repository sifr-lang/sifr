//! ICU i18n intrinsic lowerers for registry lowering.

use crate::{RustExpr, RustParam, RustType};

fn arg_expr(args: &[RustExpr], idx: usize) -> RustExpr {
    args[idx].clone()
}

fn ref_arg(expr: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(expr),
    }
}

fn runtime_call(func: &str, args: Vec<RustExpr>) -> RustExpr {
    RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "sifr_runtime".to_string(),
            "i18n".to_string(),
            func.to_string(),
        ])),
        args,
    }
}

fn error_expr(error_name: &str, message: RustExpr) -> RustExpr {
    RustExpr::StructInit {
        name: error_name.to_string(),
        fields: vec![("message".to_string(), message)],
    }
}

fn map_string_error(expr: RustExpr, error_name: &str) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(expr),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__message".to_string(),
                ty: RustType::Named("String".to_string()),
            }],
            body: Box::new(error_expr(
                error_name,
                RustExpr::Ident("__message".to_string()),
            )),
            is_move: false,
        }],
    }
}

fn lower_locale_result_unary(args: &[RustExpr], func: &str) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(map_string_error(
        runtime_call(func, vec![ref_arg(arg_expr(args, 0))]),
        "LocaleIdError",
    ))
}

pub(crate) fn lower_i18n_locale_canonicalize(args: &[RustExpr]) -> Option<RustExpr> {
    lower_locale_result_unary(args, "canonicalize_locale")
}

pub(crate) fn lower_i18n_locale_maximize(args: &[RustExpr]) -> Option<RustExpr> {
    lower_locale_result_unary(args, "maximize_locale")
}

pub(crate) fn lower_i18n_locale_minimize(args: &[RustExpr]) -> Option<RustExpr> {
    lower_locale_result_unary(args, "minimize_locale")
}

pub(crate) fn lower_i18n_host_locale(args: &[RustExpr]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(runtime_call("host_locale", vec![]))
}

pub(crate) fn lower_i18n_format_number(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "format_number",
            vec![ref_arg(arg_expr(args, 0)), ref_arg(arg_expr(args, 1))],
        ),
        "FormatError",
    ))
}

pub(crate) fn lower_i18n_format_datetime(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 8 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "format_datetime",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                arg_expr(args, 2),
                arg_expr(args, 3),
                arg_expr(args, 4),
                arg_expr(args, 5),
                arg_expr(args, 6),
                arg_expr(args, 7),
            ],
        ),
        "FormatError",
    ))
}

pub(crate) fn lower_i18n_plural_category(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "plural_category",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        ),
        "PluralRulesError",
    ))
}

pub(crate) fn lower_i18n_collate(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "collate",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
                ref_arg(arg_expr(args, 3)),
            ],
        ),
        "FormatError",
    ))
}

pub(crate) fn lower_i18n_mo_validate(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(map_string_error(
        runtime_call("validate_mo_catalog", vec![ref_arg(arg_expr(args, 0))]),
        "CatalogError",
    ))
}

pub(crate) fn lower_i18n_mo_load_file(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(map_string_error(
        runtime_call("read_mo_catalog_file", vec![ref_arg(arg_expr(args, 0))]),
        "CatalogError",
    ))
}

pub(crate) fn lower_i18n_mo_lookup(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "mo_lookup",
            vec![ref_arg(arg_expr(args, 0)), ref_arg(arg_expr(args, 1))],
        ),
        "CatalogError",
    ))
}

pub(crate) fn lower_i18n_mo_lookup_context(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "mo_lookup_context",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
            ],
        ),
        "CatalogError",
    ))
}

pub(crate) fn lower_i18n_mo_lookup_plural(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 4 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "mo_lookup_plural",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
                arg_expr(args, 3),
            ],
        ),
        "CatalogError",
    ))
}

pub(crate) fn lower_i18n_mo_lookup_context_plural(args: &[RustExpr]) -> Option<RustExpr> {
    if args.len() != 5 {
        return None;
    }
    Some(map_string_error(
        runtime_call(
            "mo_lookup_context_plural",
            vec![
                ref_arg(arg_expr(args, 0)),
                ref_arg(arg_expr(args, 1)),
                ref_arg(arg_expr(args, 2)),
                ref_arg(arg_expr(args, 3)),
                arg_expr(args, 4),
            ],
        ),
        "CatalogError",
    ))
}
