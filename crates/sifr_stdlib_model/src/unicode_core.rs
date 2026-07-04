use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.unicode — Unicode normalization and property intrinsics.
pub(super) fn intrinsic_unicode() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "_unicode_data_version_impl".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    functions.insert(
        "_unicode_normalize_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("form".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_unicode_is_normalized_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("form".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Bool, "ParseError"),
        ),
    );

    for name in [
        "_unicode_name_impl",
        "_unicode_lookup_impl",
        "_unicode_category_impl",
        "_unicode_bidirectional_impl",
        "_unicode_east_asian_width_impl",
        "_unicode_decomposition_impl",
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![("text".to_string(), Type::Str)],
                result_ty(Type::Str, "ParseError"),
            ),
        );
    }

    functions.insert(
        "_unicode_combining_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Int, "ParseError"),
        ),
    );
    functions.insert(
        "_unicode_mirrored_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Bool, "ParseError"),
        ),
    );
    functions.insert(
        "_unicode_decimal_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Int, "ParseError"),
        ),
    );
    functions.insert(
        "_unicode_digit_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Int, "ParseError"),
        ),
    );
    functions.insert(
        "_unicode_numeric_value_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            result_ty(Type::Float, "ParseError"),
        ),
    );
    functions.insert(
        "_unicode_case_fold_impl".to_string(),
        FunctionType::all_borrow(vec![("text".to_string(), Type::Str)], Type::Str),
    );
    functions.insert(
        "_unicode_graphemes_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::List(Box::new(Type::Str)),
        ),
    );
    functions.insert(
        "_unicode_grapheme_indices_flat_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::List(Box::new(Type::Str)),
        ),
    );
    functions.insert(
        "_unicode_words_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::List(Box::new(Type::Str)),
        ),
    );
    functions.insert(
        "_unicode_word_boundaries_flat_impl".to_string(),
        FunctionType::all_borrow(
            vec![("text".to_string(), Type::Str)],
            Type::List(Box::new(Type::Str)),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
