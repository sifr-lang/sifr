use super::{stmts_reference_var, stmts_reference_var_including_nested_functions};
use sifr_ir::{HirExpr, HirFunction, HirParam, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};

#[test]
fn nested_function_defaults_use_the_defining_scope_and_parameters_shadow_the_body() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![HirParam {
            name: "value".to_string(),
            ty: Type::Int,
            default: Some(HirExpr::Name {
                name: "default_source".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::Int,
        body: vec![HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "value".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };
    let stmts = vec![HirStmt::NestedFunction {
        func: nested,
        move_captures: false,
        capture_clones: Vec::new(),
    }];

    assert!(stmts_reference_var(&stmts, "default_source"));
    assert!(stmts_reference_var_including_nested_functions(
        &stmts,
        "default_source"
    ));
    assert!(!stmts_reference_var_including_nested_functions(
        &stmts, "value"
    ));
}
