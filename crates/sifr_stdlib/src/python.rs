use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

fn python_error_result(ok: Type) -> Type {
    result_ty(ok, "PythonError")
}

fn object_class() -> Type {
    Type::Class {
        name: "Object".to_string(),
        fields: vec![
            ("_handle".to_string(), Type::Int),
            ("_token".to_string(), Type::Int),
        ],
        methods: vec![],
        parent_class: None,
    }
}

fn object_handle() -> Type {
    Type::Tuple(vec![Type::Int, Type::Int])
}

/// _sifr.python - Embedded CPython opaque object intrinsics.
pub(super) fn intrinsic_python() -> IntrinsicModule {
    let mut functions = HashMap::new();
    functions.insert(
        "py_import_module".to_string(),
        FunctionType::all_borrow(
            vec![("name".to_string(), Type::Str)],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_get_attr".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("name".to_string(), Type::Str),
            ],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_get_item_str".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("key".to_string(), Type::Str),
            ],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_call".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("args".to_string(), Type::List(Box::new(object_class()))),
                (
                    "kwargs".to_string(),
                    Type::List(Box::new(Type::Tuple(vec![Type::Str, object_class()]))),
                ),
            ],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_call_attr".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("name".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(object_class()))),
                (
                    "kwargs".to_string(),
                    Type::List(Box::new(Type::Tuple(vec![Type::Str, object_class()]))),
                ),
            ],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_close".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::None),
        ),
    );
    functions.insert(
        "py_enter_context".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_exit_context".to_string(),
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
