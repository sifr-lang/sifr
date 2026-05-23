#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::MethodKind;

    fn int_binop_name(name: &str) -> HirExpr {
        HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: name.to_string(),
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
            decorators: vec![],
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
            decorators: vec![],
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
                },
                HirStmt::Return {
                    value: Some(HirExpr::Call {
                        func: "helper".to_string(),
                        args: vec![],
                        ty: Type::Int,
                    }),
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
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
                },
                HirStmt::Return {
                    value: Some(HirExpr::Call {
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
                },
                HirStmt::Return {
                    value: Some(HirExpr::Call {
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
}
