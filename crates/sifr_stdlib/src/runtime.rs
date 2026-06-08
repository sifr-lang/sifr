use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.runtime - Structured runtime diagnostic intrinsics.
pub(super) fn intrinsic_runtime() -> IntrinsicModule {
    let mut functions = HashMap::new();
    functions.insert(
        "runtime_emit_diagnostic".to_string(),
        FunctionType::all_borrow(
            vec![
                ("level".to_string(), Type::Str),
                ("target".to_string(), Type::Str),
                ("name".to_string(), Type::Str),
                ("message".to_string(), Type::Str),
            ],
            result_ty(Type::None, "DiagnosticError"),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
