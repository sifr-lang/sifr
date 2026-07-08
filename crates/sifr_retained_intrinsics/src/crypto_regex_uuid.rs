use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.regex — Combined regex intrinsics
pub(super) fn intrinsic_regex() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // re_match(pattern: str, text: str) -> Result[bool, RegexError]
    functions.insert(
        "re_match".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Bool, "RegexError"),
        ),
    );

    // re_find(pattern: str, text: str) -> Result[str | None, RegexError]
    functions.insert(
        "re_find".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "RegexError"),
        ),
    );

    // re_replace(pattern: str, replacement: str, text: str) -> Result[str, RegexError]
    functions.insert(
        "re_replace".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("replacement".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "RegexError"),
        ),
    );

    // re_findall(pattern: str, text: str) -> Result[list[str], RegexError]
    functions.insert(
        "re_findall".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "RegexError"),
        ),
    );

    // re_split(pattern: str, text: str) -> Result[list[str], RegexError]
    functions.insert(
        "re_split".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "RegexError"),
        ),
    );

    // re_find_start(pattern: str, text: str) -> Result[int, RegexError]
    // Returns the start index of the first match, or -1 if no match
    functions.insert(
        "re_find_start".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Int, "RegexError"),
        ),
    );

    // re_find_end(pattern: str, text: str) -> Result[int, RegexError]
    // Returns the end index of the first match, or -1 if no match
    functions.insert(
        "re_find_end".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
            ],
            result_ty(Type::Int, "RegexError"),
        ),
    );

    // re_match_flags(pattern: str, text: str, flags: int) -> Result[bool, RegexError]
    functions.insert(
        "re_match_flags".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
                ("flags".to_string(), Type::Int),
            ],
            result_ty(Type::Bool, "RegexError"),
        ),
    );

    // re_find_flags(pattern: str, text: str, flags: int) -> Result[str | None, RegexError]
    functions.insert(
        "re_find_flags".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
                ("flags".to_string(), Type::Int),
            ],
            result_ty(Type::Union(vec![Type::Str, Type::None]), "RegexError"),
        ),
    );

    // re_replace_flags(pattern: str, replacement: str, text: str, flags: int) -> Result[str, RegexError]
    functions.insert(
        "re_replace_flags".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("replacement".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
                ("flags".to_string(), Type::Int),
            ],
            result_ty(Type::Str, "RegexError"),
        ),
    );

    // re_findall_flags(pattern: str, text: str, flags: int) -> Result[list[str], RegexError]
    functions.insert(
        "re_findall_flags".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
                ("flags".to_string(), Type::Int),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "RegexError"),
        ),
    );

    // re_split_flags(pattern: str, text: str, flags: int) -> Result[list[str], RegexError]
    functions.insert(
        "re_split_flags".to_string(),
        FunctionType::all_borrow(
            vec![
                ("pattern".to_string(), Type::Str),
                ("text".to_string(), Type::Str),
                ("flags".to_string(), Type::Int),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "RegexError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

/// _sifr.uuid — UUID generation intrinsics
pub(super) fn intrinsic_uuid() -> IntrinsicModule {
    let mut functions = HashMap::new();
    // uuid4() -> str (random UUID v4)
    functions.insert(
        "uuid4".to_string(),
        FunctionType::all_borrow(vec![], Type::Str),
    );
    // uuid3_text(namespace: str, name: str) -> str (deterministic UUID v3)
    functions.insert(
        "uuid3_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("namespace".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
            ],
            Type::Str,
        ),
    );
    // uuid5_text(namespace: str, name: str) -> str (deterministic UUID v5)
    functions.insert(
        "uuid5_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("namespace".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
            ],
            Type::Str,
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
