use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn decode_outcome_ty() -> Type {
    Type::Class {
        name: "DecodeOutcome".to_string(),
        fields: vec![
            ("text".to_string(), Type::Str),
            ("recoveries".to_string(), Type::List(Box::new(Type::Str))),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn encode_outcome_ty() -> Type {
    Type::Class {
        name: "EncodeOutcome".to_string(),
        fields: vec![
            ("data".to_string(), Type::Bytes),
            ("recoveries".to_string(), Type::List(Box::new(Type::Str))),
        ],
        methods: vec![],
        parent_class: None,
    }
}

/// _sifr.encoding — explicit text encoding intrinsics.
pub(super) fn intrinsic_encoding() -> IntrinsicModule {
    let mut functions = HashMap::new();

    functions.insert(
        "_encoding_is_supported_impl".to_string(),
        FunctionType::all_borrow(vec![("label".to_string(), Type::Str)], Type::Bool),
    );
    functions.insert(
        "_encoding_canonical_label_impl".to_string(),
        FunctionType::all_borrow(
            vec![("label".to_string(), Type::Str)],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_decode_text_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_decode_recoveries_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "ParseError"),
        ),
    );
    functions.insert(
        "encoding_decode_outcome".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(decode_outcome_ty(), "ParseError"),
        ),
    );
    functions.insert(
        "encoding_decode_incremental_outcome".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("pending".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
                ("final".to_string(), Type::Bool),
            ],
            result_ty(decode_outcome_ty(), "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_decode_incremental_text_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("pending".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
                ("final".to_string(), Type::Bool),
            ],
            result_ty(Type::Str, "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_decode_incremental_recoveries_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("pending".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
                ("final".to_string(), Type::Bool),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_decode_incremental_pending_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("pending".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("final".to_string(), Type::Bool),
            ],
            result_ty(Type::Bytes, "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_encode_bytes_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("text".to_string(), Type::Str),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::Bytes, "ParseError"),
        ),
    );
    functions.insert(
        "_encoding_encode_recoveries_impl".to_string(),
        FunctionType::all_borrow(
            vec![
                ("text".to_string(), Type::Str),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "ParseError"),
        ),
    );
    functions.insert(
        "encoding_encode_outcome".to_string(),
        FunctionType::all_borrow(
            vec![
                ("text".to_string(), Type::Str),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(encode_outcome_ty(), "ParseError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
