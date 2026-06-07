use super::{error_class, IntrinsicModule};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

pub(super) fn intrinsic_process() -> IntrinsicModule {
    let mut functions = HashMap::new();
    let output_tuple = Type::Tuple(vec![Type::Bytes, Type::Bytes, Type::Int]);
    let process_error = error_class("ProcessError");

    functions.insert(
        "process_output".to_string(),
        FunctionType::all_borrow(
            vec![
                ("program".to_string(), Type::Str),
                ("args".to_string(), Type::List(Box::new(Type::Str))),
                ("cwd".to_string(), Type::Union(vec![Type::Str, Type::None])),
                ("shell".to_string(), Type::Bool),
            ],
            Type::Result(Box::new(output_tuple), Box::new(process_error)),
        ),
    );

    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
