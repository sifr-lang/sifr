use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn signal_class() -> Type {
    Type::Class {
        name: "Signal".to_string(),
        fields: vec![
            ("number".to_string(), Type::Int),
            ("name".to_string(), Type::Str),
            ("supported".to_string(), Type::Bool),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn signal_result() -> Type {
    result_ty(signal_class(), "SignalError")
}

/// _sifr.signal - Structured shutdown signal intrinsics.
pub(super) fn intrinsic_signal() -> IntrinsicModule {
    let mut functions = HashMap::new();
    functions.insert(
        "signal_ctrl_c".to_string(),
        FunctionType::all_borrow(vec![], Type::Awaitable(Box::new(signal_result()))),
    );
    functions.insert(
        "signal_sigterm_supported".to_string(),
        FunctionType::all_borrow(vec![], Type::Bool),
    );
    functions.insert(
        "signal_terminate".to_string(),
        FunctionType::all_borrow(vec![], Type::Awaitable(Box::new(signal_result()))),
    );
    functions.insert(
        "signal_shutdown".to_string(),
        FunctionType::all_borrow(vec![], Type::Awaitable(Box::new(signal_result()))),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
