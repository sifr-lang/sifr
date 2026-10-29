use super::*;

#[test]
fn generic_generator_calls_use_the_canonical_function_name() {
    let mut emitter = RustEmitter::new();
    emitter.generator_functions.insert("produce".to_string());
    let expression = HirExpr::GenericCall {
        func: "produce::<i64>".to_string(),
        type_args: vec![Type::Int],
        args: Vec::new(),
        mutable_arg_places: Vec::new(),
        ty: Type::Iterator(Box::new(Type::Int)),
    };

    assert!(emitter.is_generator_call(&expression));
}
