use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.crypto — Combined crypto intrinsics (random + hash + encoding)
pub(super) fn intrinsic_crypto() -> IntrinsicModule {
    let mut functions = HashMap::new();

    // random_int(min: int, max: int) -> int
    functions.insert(
        "random_int".to_string(),
        FunctionType::all_borrow(
            vec![
                ("min".to_string(), Type::Int),
                ("max".to_string(), Type::Int),
            ],
            Type::Int,
        ),
    );

    // random_float() -> float
    functions.insert(
        "random_float".to_string(),
        FunctionType::all_borrow(vec![], Type::Float),
    );

    // random_choice(items: list[Any]) -> Any
    functions.insert(
        "random_choice".to_string(),
        FunctionType::all_borrow(
            vec![("items".to_string(), Type::List(Box::new(Type::Any)))],
            Type::Any,
        ),
    );

    // sha256(s: str) -> str (hex digest)
    functions.insert(
        "sha256".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // sha256_bytes(data: bytes) -> bytes
    functions.insert(
        "sha256_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // md5(s: str) -> str (hex digest)
    functions.insert(
        "md5".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // md5_bytes(data: bytes) -> bytes
    functions.insert(
        "md5_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // base64_encode(s: str) -> str
    functions.insert(
        "base64_encode".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // base64_encode_bytes(data: bytes) -> bytes
    functions.insert(
        "base64_encode_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // base64_decode(s: str) -> Result[str, ParseError]
    functions.insert(
        "base64_decode".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    // base64_decode_bytes(data: bytes) -> Result[bytes, ParseError]
    functions.insert(
        "base64_decode_bytes".to_string(),
        FunctionType::all_borrow(
            vec![("data".to_string(), Type::Bytes)],
            result_ty(Type::Bytes, "ParseError"),
        ),
    );

    // base64_encode_opts(s: str, altchars: str, wrapcol: int) -> Result[str, ParseError]
    functions.insert(
        "base64_encode_opts".to_string(),
        FunctionType::all_borrow(
            vec![
                ("s".to_string(), Type::Str),
                ("altchars".to_string(), Type::Str),
                ("wrapcol".to_string(), Type::Int),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );

    // base64_decode_opts(s: str, altchars: str, validate: bool, ignorechars: str) -> Result[str, ParseError]
    functions.insert(
        "base64_decode_opts".to_string(),
        FunctionType::all_borrow(
            vec![
                ("s".to_string(), Type::Str),
                ("altchars".to_string(), Type::Str),
                ("validate".to_string(), Type::Bool),
                ("ignorechars".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );

    // random_uniform(min: float, max: float) -> float
    functions.insert(
        "random_uniform".to_string(),
        FunctionType::all_borrow(
            vec![
                ("min".to_string(), Type::Float),
                ("max".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // sha1(s: str) -> str (hex digest)
    functions.insert(
        "sha1".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // sha1_bytes(data: bytes) -> bytes
    functions.insert(
        "sha1_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // sha512(s: str) -> str (hex digest)
    functions.insert(
        "sha512".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // sha512_bytes(data: bytes) -> bytes
    functions.insert(
        "sha512_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // urlsafe_b64encode(s: str) -> str
    functions.insert(
        "urlsafe_b64encode".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // urlsafe_b64encode_bytes(data: bytes) -> bytes
    functions.insert(
        "urlsafe_b64encode_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // urlsafe_b64decode(s: str) -> Result[str, ParseError]
    functions.insert(
        "urlsafe_b64decode".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    // urlsafe_b64decode_bytes(data: bytes) -> Result[bytes, ParseError]
    functions.insert(
        "urlsafe_b64decode_bytes".to_string(),
        FunctionType::all_borrow(
            vec![("data".to_string(), Type::Bytes)],
            result_ty(Type::Bytes, "ParseError"),
        ),
    );

    // random_shuffle(items: list[Any]) -> list[Any]
    functions.insert(
        "random_shuffle".to_string(),
        FunctionType::all_borrow(
            vec![("items".to_string(), Type::List(Box::new(Type::Any)))],
            Type::List(Box::new(Type::Any)),
        ),
    );

    // random_sample(items: list[Any], k: int) -> Result[list[Any], ValueError]
    functions.insert(
        "random_sample".to_string(),
        FunctionType::all_borrow(
            vec![
                ("items".to_string(), Type::List(Box::new(Type::Any))),
                ("k".to_string(), Type::Int),
            ],
            result_ty(Type::List(Box::new(Type::Any)), "ValueError"),
        ),
    );

    // random_randrange(start: int, stop: int, step: int) -> Result[int, ValueError]
    functions.insert(
        "random_randrange".to_string(),
        FunctionType::all_borrow(
            vec![
                ("start".to_string(), Type::Int),
                ("stop".to_string(), Type::Int),
                ("step".to_string(), Type::Int),
            ],
            result_ty(Type::Int, "ValueError"),
        ),
    );

    // random_gauss(mu: float, sigma: float) -> float
    functions.insert(
        "random_gauss".to_string(),
        FunctionType::all_borrow(
            vec![
                ("mu".to_string(), Type::Float),
                ("sigma".to_string(), Type::Float),
            ],
            Type::Float,
        ),
    );

    // random_module_state_words() -> list[int]
    functions.insert(
        "random_module_state_words".to_string(),
        FunctionType::all_borrow(vec![], Type::List(Box::new(Type::Int))),
    );

    // random_module_state_index() -> int
    functions.insert(
        "random_module_state_index".to_string(),
        FunctionType::all_borrow(vec![], Type::Int),
    );

    // random_module_state_gauss_next() -> float | None
    functions.insert(
        "random_module_state_gauss_next".to_string(),
        FunctionType::all_borrow(vec![], Type::Union(vec![Type::Float, Type::None])),
    );

    // random_module_set_state(words: list[int], index: int, gauss_next: float | None)
    //   -> Result[None, ValueError]
    functions.insert(
        "random_module_set_state".to_string(),
        FunctionType::all_borrow(
            vec![
                ("words".to_string(), Type::List(Box::new(Type::Int))),
                ("index".to_string(), Type::Int),
                (
                    "gauss_next".to_string(),
                    Type::Union(vec![Type::Float, Type::None]),
                ),
            ],
            result_ty(Type::None, "ValueError"),
        ),
    );

    // sha224(s: str) -> str (hex digest)
    functions.insert(
        "sha224".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // sha224_bytes(data: bytes) -> bytes
    functions.insert(
        "sha224_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // sha384(s: str) -> str (hex digest)
    functions.insert(
        "sha384".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // sha384_bytes(data: bytes) -> bytes
    functions.insert(
        "sha384_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // blake2b(s: str) -> str (hex digest)
    functions.insert(
        "blake2b".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // blake2b_bytes(data: bytes) -> bytes
    functions.insert(
        "blake2b_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // blake2s(s: str) -> str (hex digest)
    functions.insert(
        "blake2s".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );
    // blake2s_bytes(data: bytes) -> bytes
    functions.insert(
        "blake2s_bytes".to_string(),
        FunctionType::all_borrow(vec![("data".to_string(), Type::Bytes)], Type::Bytes),
    );

    // b32encode(s: str) -> str
    functions.insert(
        "b32encode".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );

    // b32decode(s: str) -> Result[str, ParseError]
    functions.insert(
        "b32decode".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            result_ty(Type::Str, "ParseError"),
        ),
    );

    // b32hexencode(s: str) -> str
    functions.insert(
        "b32hexencode".to_string(),
        FunctionType::all_borrow(vec![("s".to_string(), Type::Str)], Type::Str),
    );

    // b32hexdecode(s: str) -> Result[str, ParseError]
    functions.insert(
        "b32hexdecode".to_string(),
        FunctionType::all_borrow(
            vec![("s".to_string(), Type::Str)],
            result_ty(Type::Str, "ParseError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}

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
