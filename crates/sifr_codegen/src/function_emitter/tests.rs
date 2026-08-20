use super::*;
use sifr_ir::MethodKind;

fn int_binop_name(name: &str) -> HirExpr {
    HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: name.to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::IntLiteral(1)),
        ty: Type::Int,
    }
}

fn regular_int_function(params: Vec<HirParam>, body: Vec<HirStmt>) -> HirFunction {
    HirFunction {
        name: "f".to_string(),
        params,
        return_type: Type::Int,
        body,
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    }
}

fn helper_returning_name(name: &str) -> HirFunction {
    HirFunction {
        name: "helper".to_string(),
        params: vec![],
        return_type: Type::Int,
        body: vec![HirStmt::Return {
            value: Some(int_binop_name(name)),
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    }
}

fn middle_with_inner_returning_name(name: &str) -> HirFunction {
    HirFunction {
        name: "middle".to_string(),
        params: vec![],
        return_type: Type::Int,
        body: vec![
            HirStmt::NestedFunction {
                func: helper_returning_name(name),
                move_captures: false,
                capture_clones: Vec::new(),
            },
            HirStmt::Return {
                value: Some(HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::Int,
                }),
            },
        ],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    }
}

#[test]
fn shadowed_module_const_local_does_not_promote_return_to_sifr_int() {
    let func = regular_int_function(
        vec![],
        vec![
            HirStmt::Let {
                name: "BIG_LIMIT".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(5),
                is_mutable: false,
            },
            HirStmt::Return {
                value: Some(int_binop_name("BIG_LIMIT")),
            },
        ],
    );
    let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

    assert!(!hir_function_returns_sifr_int(
        &func,
        &module_sifr_int_bindings,
        &HashSet::new(),
    ));
}

#[test]
fn shadowed_module_const_param_does_not_promote_return_to_sifr_int() {
    let func = regular_int_function(
        vec![HirParam {
            name: "BIG_LIMIT".to_string(),
            ty: Type::Int,
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        vec![HirStmt::Return {
            value: Some(int_binop_name("BIG_LIMIT")),
        }],
    );
    let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

    assert!(!hir_function_returns_sifr_int(
        &func,
        &module_sifr_int_bindings,
        &HashSet::new(),
    ));
}

#[test]
fn nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int() {
    let func = regular_int_function(
        vec![],
        vec![
            HirStmt::Let {
                name: "BIG_LIMIT".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(5),
                is_mutable: false,
            },
            HirStmt::NestedFunction {
                func: helper_returning_name("BIG_LIMIT"),
                move_captures: false,
                capture_clones: Vec::new(),
            },
            HirStmt::Return {
                value: Some(HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "helper".to_string(),
                    args: vec![],
                    ty: Type::Int,
                }),
            },
        ],
    );
    let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

    assert!(!hir_function_returns_sifr_int(
        &func,
        &module_sifr_int_bindings,
        &HashSet::new(),
    ));
}

#[test]
fn multilevel_nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int() {
    let func = regular_int_function(
        vec![],
        vec![
            HirStmt::Let {
                name: "BIG_LIMIT".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(5),
                is_mutable: false,
            },
            HirStmt::NestedFunction {
                func: middle_with_inner_returning_name("BIG_LIMIT"),
                move_captures: false,
                capture_clones: Vec::new(),
            },
            HirStmt::Return {
                value: Some(HirExpr::Call {
                    mutable_arg_places: Vec::new(),
                    func: "middle".to_string(),
                    args: vec![],
                    ty: Type::Int,
                }),
            },
        ],
    );
    let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

    assert!(!hir_function_returns_sifr_int(
        &func,
        &module_sifr_int_bindings,
        &HashSet::new(),
    ));
}

#[test]
fn multilevel_nested_helper_captures_forced_local_and_promotes_return_to_sifr_int() {
    let func = middle_with_inner_returning_name("big");
    let forced_locals = HashSet::from(["big".to_string()]);

    assert_eq!(
        collect_sifr_int_captured_forced_locals(&func, &forced_locals),
        forced_locals
    );
    assert!(hir_function_returns_sifr_int_with_extra_forced(
        &func,
        &HashSet::new(),
        &HashSet::new(),
        &forced_locals,
    ));
}

#[test]
fn unshadowed_module_const_still_promotes_return_to_sifr_int() {
    let func = regular_int_function(
        vec![],
        vec![HirStmt::Return {
            value: Some(int_binop_name("BIG_LIMIT")),
        }],
    );
    let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

    assert!(hir_function_returns_sifr_int(
        &func,
        &module_sifr_int_bindings,
        &HashSet::new(),
    ));
}

#[test]
fn generic_call_uses_canonical_sifr_int_parameter_metadata() {
    let body = vec![HirStmt::Expr {
        expr: HirExpr::GenericCall {
            func: "consume::<i64>".to_string(),
            type_args: vec![Type::Int],
            args: vec![HirExpr::LargeIntLiteral(
                "100000000000000000000".to_string(),
            )],
            mutable_arg_places: vec![None],
            ty: Type::None,
        },
    }];
    let params = HashMap::from([("consume".to_string(), vec![Type::Int])]);

    let discovered = collect_sifr_int_call_arg_function_params(
        &body,
        &params,
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
    );

    assert_eq!(discovered.get("consume"), Some(&HashSet::from([0])));
}

#[test]
fn generic_call_uses_canonical_sifr_int_return_metadata() {
    let expression = HirExpr::GenericCall {
        func: "produce::<i64>".to_string(),
        type_args: vec![Type::Int],
        args: Vec::new(),
        mutable_arg_places: Vec::new(),
        ty: Type::Int,
    };

    assert!(hir_expr_needs_sifr_int_storage(
        &expression,
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::new(),
        &HashSet::from(["produce".to_string()]),
    ));
}

#[test]
fn generic_result_call_uses_canonical_sifr_int_metadata() {
    let result_ty = Type::Result(Box::new(Type::Int), Box::new(Type::Str));
    let body = vec![HirStmt::Expr {
        expr: HirExpr::GenericCall {
            func: "consume_result::<i64>".to_string(),
            type_args: vec![Type::Int],
            args: vec![HirExpr::GenericCall {
                func: "produce_result::<i64>".to_string(),
                type_args: vec![Type::Int],
                args: Vec::new(),
                mutable_arg_places: Vec::new(),
                ty: result_ty.clone(),
            }],
            mutable_arg_places: vec![None],
            ty: Type::None,
        },
    }];
    let params = HashMap::from([("consume_result".to_string(), vec![result_ty])]);

    let discovered = collect_sifr_int_result_call_arg_function_params_with_initial(
        &body,
        &params,
        &HashSet::from(["produce_result".to_string()]),
        &HashSet::new(),
        HashSet::new(),
    );

    assert_eq!(discovered.get("consume_result"), Some(&HashSet::from([0])));
}
