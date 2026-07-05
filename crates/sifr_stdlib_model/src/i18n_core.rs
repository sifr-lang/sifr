use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.i18n — private ICU-backed locale, formatting, plural, and collation declarations.
pub(super) fn intrinsic_i18n() -> IntrinsicModule {
    let mut functions = HashMap::new();

    for name in [
        "_i18n_locale_canonicalize_impl",
        "_i18n_locale_maximize_impl",
        "_i18n_locale_minimize_impl",
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![("locale".to_string(), Type::Str)],
                result_ty(Type::Str, "ParseError"),
            ),
        );
    }

    functions.insert(
        "_i18n_host_locale_impl".to_string(),
        FunctionType::all_borrow(vec![], Type::Union(vec![Type::Str, Type::None])),
    );
    functions.insert(
        "_i18n_format_number_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("value".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_format_datetime_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("style".to_string(), Type::Str),
                ("year".to_string(), Type::Int),
                ("month".to_string(), Type::Int),
                ("day".to_string(), Type::Int),
                ("hour".to_string(), Type::Int),
                ("minute".to_string(), Type::Int),
                ("second".to_string(), Type::Int),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_plural_category_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("rule_type".to_string(), Type::Str),
                ("value".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_collate_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("strength".to_string(), Type::Str),
                ("left".to_string(), Type::Str),
                ("right".to_string(), Type::Str),
            ],
            result_ty(Type::Int, "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_mo_validate_impl".to_string(),
        FunctionType::all_borrow(
            vec![("catalog".to_string(), Type::Bytes)],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_mo_load_file_impl".to_string(),
        FunctionType::all_borrow(
            vec![("path".to_string(), Type::Str)],
            result_ty(Type::Bytes, "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_mo_lookup_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("catalog".to_string(), Type::Bytes),
                ("message_id".to_string(), Type::Str),
            ],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_mo_lookup_context_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("catalog".to_string(), Type::Bytes),
                ("context".to_string(), Type::Str),
                ("message_id".to_string(), Type::Str),
            ],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_mo_lookup_plural_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("catalog".to_string(), Type::Bytes),
                ("singular".to_string(), Type::Str),
                ("plural".to_string(), Type::Str),
                ("count".to_string(), Type::Int),
            ],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "ParseError"),
        ),
    );
    functions.insert(
        "_i18n_mo_lookup_context_plural_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("catalog".to_string(), Type::Bytes),
                ("context".to_string(), Type::Str),
                ("singular".to_string(), Type::Str),
                ("plural".to_string(), Type::Str),
                ("count".to_string(), Type::Int),
            ],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "ParseError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
