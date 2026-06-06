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
        "encoding_is_supported".to_string(),
        FunctionType::all_borrow(vec![("label".to_string(), Type::Str)], Type::Bool),
    );
    functions.insert(
        "encoding_canonical_label".to_string(),
        FunctionType::all_borrow(
            vec![("label".to_string(), Type::Str)],
            result_ty(Type::Str, "DecodeError"),
        ),
    );
    functions.insert(
        "encoding_decode_text".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::Str, "DecodeError"),
        ),
    );
    functions.insert(
        "encoding_decode_recoveries".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "DecodeError"),
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
            result_ty(decode_outcome_ty(), "DecodeError"),
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
            result_ty(decode_outcome_ty(), "DecodeError"),
        ),
    );
    functions.insert(
        "encoding_decode_incremental_pending".to_string(),
        FunctionType::all_borrow(
            vec![
                ("data".to_string(), Type::Bytes),
                ("pending".to_string(), Type::Bytes),
                ("encoding".to_string(), Type::Str),
                ("final".to_string(), Type::Bool),
            ],
            result_ty(Type::Bytes, "DecodeError"),
        ),
    );
    functions.insert(
        "encoding_encode_bytes".to_string(),
        FunctionType::all_borrow(
            vec![
                ("text".to_string(), Type::Str),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::Bytes, "EncodeError"),
        ),
    );
    functions.insert(
        "encoding_encode_recoveries".to_string(),
        FunctionType::all_borrow(
            vec![
                ("text".to_string(), Type::Str),
                ("encoding".to_string(), Type::Str),
                ("errors".to_string(), Type::Str),
            ],
            result_ty(Type::List(Box::new(Type::Str)), "EncodeError"),
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
            result_ty(encode_outcome_ty(), "EncodeError"),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
