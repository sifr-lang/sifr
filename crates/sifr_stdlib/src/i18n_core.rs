use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.i18n — ICU-backed locale, formatting, plural, and collation intrinsics.
pub(super) fn intrinsic_i18n() -> IntrinsicModule {
    let mut functions = HashMap::new();

    for name in [
        "i18n_locale_canonicalize",
        "i18n_locale_maximize",
        "i18n_locale_minimize",
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![("locale".to_string(), Type::Str)],
                result_ty(Type::Str, "LocaleIdError"),
            ),
        );
    }

    functions.insert(
        "i18n_host_locale".to_string(),
        FunctionType::all_borrow(vec![], Type::Union(vec![Type::Str, Type::None])),
    );
    functions.insert(
        "i18n_format_number".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("value".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "FormatError"),
        ),
    );
    functions.insert(
        "i18n_format_datetime".to_string(),
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
            result_ty(Type::Str, "FormatError"),
        ),
    );
    functions.insert(
        "i18n_plural_category".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("rule_type".to_string(), Type::Str),
                ("value".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "PluralRulesError"),
        ),
    );
    functions.insert(
        "i18n_collate".to_string(),
        FunctionType::all_borrow(
            vec![
                ("locale".to_string(), Type::Str),
                ("strength".to_string(), Type::Str),
                ("left".to_string(), Type::Str),
                ("right".to_string(), Type::Str),
            ],
            result_ty(Type::Int, "FormatError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
