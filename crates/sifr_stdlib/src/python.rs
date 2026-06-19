use super::{result_ty, IntrinsicModule};
use sifr_type_system::{FixedIntType, FunctionType, Type};
use std::collections::HashMap;

fn python_error_result(ok: Type) -> Type {
    result_ty(ok, "PythonError")
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
                ("args".to_string(), Type::List(Box::new(object_handle()))),
                (
                    "kwargs".to_string(),
                    Type::List(Box::new(Type::Tuple(vec![Type::Str, object_handle()]))),
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
                ("args".to_string(), Type::List(Box::new(object_handle()))),
                (
                    "kwargs".to_string(),
                    Type::List(Box::new(Type::Tuple(vec![Type::Str, object_handle()]))),
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
        "py_buffer_u8".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("require_writable".to_string(), Type::Bool),
            ],
            python_error_result(Type::Tuple(vec![
                Type::Int,
                Type::Int,
                Type::Int,
                Type::Int,
                Type::Bool,
                Type::Int,
                Type::List(Box::new(Type::Int)),
                Type::List(Box::new(Type::Int)),
                Type::List(Box::new(Type::Int)),
                Type::Bool,
                Type::Bool,
                Type::Str,
            ])),
        ),
    );
    functions.insert(
        "py_copy_buffer_u8".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::Bytes),
        ),
    );
    functions.insert(
        "py_release_buffer".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::None),
        ),
    );
    for name in ["py_arrow_array", "py_arrow_stream", "py_arrow_schema"] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![
                    ("handle".to_string(), Type::Int),
                    ("token".to_string(), Type::Int),
                ],
                python_error_result(Type::Tuple(vec![
                    Type::Int,
                    Type::Int,
                    Type::Str,
                    Type::List(Box::new(Type::Str)),
                    Type::Str,
                    Type::Str,
                    Type::Bool,
                ])),
            ),
        );
    }
    functions.insert(
        "py_release_arrow".to_string(),
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
    functions.insert(
        "py_exit_context_with_error".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("kind".to_string(), Type::Str),
                ("exception_type".to_string(), Type::Str),
                ("message".to_string(), Type::Str),
                ("traceback".to_string(), Type::Str),
                ("context".to_string(), Type::Str),
            ],
            python_error_result(Type::None),
        ),
    );
    functions.insert(
        "py_resource_diagnostics".to_string(),
        FunctionType::all_borrow(
            vec![],
            python_error_result(Type::Tuple(vec![Type::Bool, Type::Int, Type::Int])),
        ),
    );
    functions.insert(
        "py_run_coroutine_blocking".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_none".to_string(),
        FunctionType::all_borrow(vec![], python_error_result(object_handle())),
    );
    functions.insert(
        "py_from_bool".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Bool)],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_int".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Int)],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_float".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Float)],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_str".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Str)],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_bytes".to_string(),
        FunctionType::all_borrow(
            vec![("value".to_string(), Type::Bytes)],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_list".to_string(),
        FunctionType::all_borrow(
            vec![("values".to_string(), Type::List(Box::new(object_handle())))],
            python_error_result(object_handle()),
        ),
    );
    functions.insert(
        "py_from_tuple".to_string(),
        FunctionType::all_borrow(
            vec![("values".to_string(), Type::List(Box::new(object_handle())))],
            python_error_result(object_handle()),
        ),
    );
    for name in ["py_from_dict_str", "py_from_record"] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![(
                    "values".to_string(),
                    Type::List(Box::new(Type::Tuple(vec![Type::Str, object_handle()]))),
                )],
                python_error_result(object_handle()),
            ),
        );
    }
    functions.insert(
        "py_to_none".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::None),
        ),
    );
    functions.insert(
        "py_to_bool".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::Bool),
        ),
    );
    functions.insert(
        "py_to_int".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::Int),
        ),
    );
    for (name, fixed) in [
        ("py_to_i8", FixedIntType::I8),
        ("py_to_i16", FixedIntType::I16),
        ("py_to_i32", FixedIntType::I32),
        ("py_to_i64", FixedIntType::I64),
        ("py_to_u8", FixedIntType::U8),
        ("py_to_u16", FixedIntType::U16),
        ("py_to_u32", FixedIntType::U32),
        ("py_to_u64", FixedIntType::U64),
        ("py_to_isize", FixedIntType::ISize),
        ("py_to_usize", FixedIntType::USize),
    ] {
        functions.insert(
            name.to_string(),
            FunctionType::all_borrow(
                vec![
                    ("handle".to_string(), Type::Int),
                    ("token".to_string(), Type::Int),
                ],
                python_error_result(Type::FixedInt(fixed)),
            ),
        );
    }
    functions.insert(
        "py_to_float".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::Float),
        ),
    );
    functions.insert(
        "py_to_str".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::Str),
        ),
    );
    functions.insert(
        "py_to_bytes".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
            ],
            python_error_result(Type::Bytes),
        ),
    );
    for (suffix, value_ty) in [
        ("bool", Type::Bool),
        ("int", Type::Int),
        ("i32", Type::FixedInt(FixedIntType::I32)),
        ("u8", Type::FixedInt(FixedIntType::U8)),
        ("float", Type::Float),
        ("str", Type::Str),
        ("bytes", Type::Bytes),
    ] {
        let handle_params = vec![
            ("handle".to_string(), Type::Int),
            ("token".to_string(), Type::Int),
        ];
        functions.insert(
            format!("py_copy_list_{suffix}"),
            FunctionType::all_borrow(
                handle_params.clone(),
                python_error_result(Type::List(Box::new(value_ty.clone()))),
            ),
        );
        functions.insert(
            format!("py_copy_tuple_{suffix}"),
            FunctionType::all_borrow(
                handle_params.clone(),
                python_error_result(Type::List(Box::new(value_ty.clone()))),
            ),
        );
        functions.insert(
            format!("py_copy_dict_str_{suffix}"),
            FunctionType::all_borrow(
                handle_params,
                python_error_result(Type::Dict(Box::new(Type::Str), Box::new(value_ty))),
            ),
        );
    }
    functions.insert(
        "py_copy_record_fields".to_string(),
        FunctionType::all_borrow(
            vec![
                ("handle".to_string(), Type::Int),
                ("token".to_string(), Type::Int),
                ("fields".to_string(), Type::List(Box::new(Type::Str))),
            ],
            python_error_result(Type::List(Box::new(Type::Tuple(vec![
                Type::Str,
                object_handle(),
            ])))),
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
