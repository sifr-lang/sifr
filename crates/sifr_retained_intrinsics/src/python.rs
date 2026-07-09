use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn python_error_result(ok: Type) -> Type {
    result_ty(ok, "PythonError")
}

/// `_sifr.python` - Embedded `CPython` opaque object intrinsics.
pub(super) fn intrinsic_python() -> IntrinsicModule {
    let mut functions = HashMap::new();
    for name in ["py_local_callback_echo", "py_threadsafe_callback_echo"] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![],
                python_error_result(Type::Tuple(vec![
                    Type::Int,
                    Type::Int,
                    Type::Int,
                    Type::Int,
                    Type::Str,
                ])),
            ),
        );
    }
    functions.insert(
        "py_close_callback".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::None),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
