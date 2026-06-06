use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.unicode — Unicode normalization and property intrinsics.
pub(super) fn intrinsic_unicode() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "unicode_data_version".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "unicode_normalize".to_string(),
        FunctionType::all_borrow(
            vec![
                ("form".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "UnicodeDataError"),
        ),
    );
    functions.insert(
        "unicode_is_normalized".to_string(),
        FunctionType::all_borrow(
            vec![
                ("form".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Bool, "UnicodeDataError"),
        ),
    );

    for name in [
        "unicode_name",
        "unicode_lookup",
        "unicode_category",
        "unicode_bidirectional",
        "unicode_east_asian_width",
        "unicode_decomposition",
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![("text".to_string(), Type::Str)],
                result_ty(Type::Str, "UnicodeDataError"),
            ),
        );
    }

    functions.insert(
        "unicode_combining".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Int, "UnicodeDataError"),
        ),
    );
    functions.insert(
        "unicode_mirrored".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Bool, "UnicodeDataError"),
        ),
    );
    functions.insert(
        "unicode_decimal".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Int, "UnicodeDataError"),
        ),
    );
    functions.insert(
        "unicode_digit".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Int, "UnicodeDataError"),
        ),
    );
    functions.insert(
        "unicode_numeric_value".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Float, "UnicodeDataError"),
        ),
    );
    functions.insert(
        "unicode_case_fold".to_string(),
        FunctionType::all_borrow(vec![("text".to_string(), Type::Str)], Type::Str),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
