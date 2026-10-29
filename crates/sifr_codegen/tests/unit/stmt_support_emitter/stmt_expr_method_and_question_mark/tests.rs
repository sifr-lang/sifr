use super::*;

#[test]
fn generic_imported_project_calls_use_the_canonical_function_name() {
    let expression = HirExpr::GenericCall {
        func: "load::<i64>".to_string(),
        type_args: vec![Type::Int],
        args: Vec::new(),
        mutable_arg_places: Vec::new(),
        ty: Type::None,
    };
    let imported = std::collections::HashSet::from(["load".to_string()]);

    assert!(is_imported_project_call_for_ir(&expression, &imported));
}
