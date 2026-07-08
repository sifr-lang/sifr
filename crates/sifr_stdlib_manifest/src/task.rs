use super::IntrinsicModule;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashMap;

/// _sifr.task - Structured task context intrinsics.
pub(super) fn intrinsic_task() -> IntrinsicModule {
    let mut functions = HashMap::new();
    functions.insert(
        "task_current_context".to_string(),
        FunctionType::all_borrow(
            vec![],
            Type::Class {
                name: "Context".to_string(),
                fields: vec![("name".to_string(), Type::Str)],
                methods: vec![],
                parent_class: None,
            },
        ),
    );
    IntrinsicModule {
        functions,
        constants: HashMap::new(),
    }
}
