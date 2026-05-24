use super::*;
#[test]
pub(super) fn test_structured_expr_path_handles_intrinsic_call_expression() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("(9.0 as f64).sqrt()"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "intrinsic call should be emitted through structured expr path"
    );
    assert!(
        generated.lowering_stats.stmt_structured > 0,
        "expression statement should be emitted through structured stmt path"
    );
}

#[test]
pub(super) fn test_structured_expr_path_handles_nested_intrinsic_call_argument() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "set_len".to_string(),
                    args: vec![HirExpr::Call {
                        func: "set_from_list".to_string(),
                        args: vec![HirExpr::ListLiteral {
                            elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
                            ty: list_ty.clone(),
                        }],
                        ty: list_ty,
                    }],
                    ty: Type::Int,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.collections".to_string(),
            names: vec!["set_len".to_string(), "set_from_list".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains(".len() as i64"),
        "nested intrinsic call argument should lower through registry"
    );
    assert!(
        !generated.rust_source.contains("set_len("),
        "set_len should not be emitted as unresolved function call"
    );
}

#[test]
pub(super) fn test_structured_expr_path_handles_intrinsic_arg_with_typed_method_call() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "isnan".to_string(),
                    args: vec![HirExpr::MethodCall {
                        object: Box::new(HirExpr::FloatLiteral(1.0)),
                        method: "max".to_string(),
                        args: vec![HirExpr::FloatLiteral(2.0)],
                        ty: Type::Float,
                    }],
                    ty: Type::Bool,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["isnan".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains(".is_nan()"),
        "intrinsic arg with typed method call should lower through registry"
    );
    assert!(
        !generated.rust_source.contains("isnan("),
        "isnan should not be emitted as unresolved function call"
    );
}

#[test]
pub(super) fn test_structured_expr_path_handles_plain_signature_call_expression() {
    let module = HirModule {
        functions: vec![
            HirFunction {
                name: "helper".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "print".to_string(),
                        args: vec![HirExpr::StringLiteral("inner".to_string())],
                        ty: Type::None,
                    },
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
            HirFunction {
                name: "main".to_string(),
                params: vec![],
                return_type: Type::None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "helper".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                }],
                is_async: false,
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
        ],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("helper();"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "plain calls with by-value signatures should use structured expr path"
    );
}

#[test]
pub(super) fn test_structured_expr_path_handles_registry_method_call_expression() {
    let list_ty = Type::List(Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "items".to_string(),
                    ty: list_ty.clone(),
                    value: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: list_ty.clone(),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "items".to_string(),
                            ty: list_ty,
                        }),
                        method: "clear".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("items.clear();"));
    assert!(
        generated.lowering_stats.expr_structured > 0,
        "registry-backed method call should be emitted through structured expr path"
    );
}

#[test]
pub(super) fn test_registry_dict_update_with_typed_literal_arg_lowers_to_extend() {
    let dict_ty = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "d2".to_string(),
                    ty: dict_ty.clone(),
                    value: HirExpr::DictLiteral {
                        keys: vec![HirExpr::StringLiteral("a".to_string())],
                        values: vec![HirExpr::IntLiteral(1)],
                        ty: dict_ty.clone(),
                    },
                    is_mutable: true,
                },
                HirStmt::Expr {
                    expr: HirExpr::MethodCall {
                        object: Box::new(HirExpr::Name {
                            name: "d2".to_string(),
                            ty: dict_ty.clone(),
                        }),
                        method: "update".to_string(),
                        args: vec![HirExpr::DictLiteral {
                            keys: vec![HirExpr::StringLiteral("c".to_string())],
                            values: vec![HirExpr::IntLiteral(3)],
                            ty: dict_ty.clone(),
                        }],
                        ty: Type::None,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(
        generated.rust_source.contains("d2.extend("),
        "dict update should lower via registry to HashMap::extend"
    );
    assert!(
        !generated.rust_source.contains("d2.update("),
        "dict update method call should not be emitted"
    );
}

#[test]
pub(super) fn test_structured_stmt_path_handles_copy_typed_assign_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![
                HirStmt::Let {
                    name: "x".to_string(),
                    ty: Type::Float,
                    value: HirExpr::FloatLiteral(0.0),
                    is_mutable: true,
                },
                HirStmt::Assign {
                    name: "x".to_string(),
                    value: HirExpr::Call {
                        func: "sqrt".to_string(),
                        args: vec![HirExpr::FloatLiteral(9.0)],
                        ty: Type::Float,
                    },
                },
            ],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("x = (9.0 as f64).sqrt();"));
    assert!(
        generated.lowering_stats.stmt_structured >= 2,
        "let + assign should be emitted through structured stmt path"
    );
}

#[test]
pub(super) fn test_structured_stmt_path_handles_copy_typed_let_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "main".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Float,
                value: HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                },
                is_mutable: false,
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains("let x: f64 = (9.0 as f64).sqrt();"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "copy-typed let should be emitted through structured stmt path"
    );
}

#[test]
pub(super) fn test_structured_stmt_path_handles_copy_typed_return_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "value".to_string(),
            params: vec![],
            return_type: Type::Float,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "sqrt".to_string(),
                    args: vec![HirExpr::FloatLiteral(9.0)],
                    ty: Type::Float,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![HirImport {
            module: "sifr.math".to_string(),
            names: vec!["sqrt".to_string()],
            aliases: vec![],
        }],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated
        .rust_source
        .contains("return (9.0 as f64).sqrt();"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "copy-typed return should be emitted through structured stmt path"
    );
}

#[test]
pub(super) fn test_structured_stmt_path_wraps_non_optional_string_index_into_option_local() {
    let stmt = HirStmt::Let {
        name: "part".to_string(),
        ty: Type::Union(vec![Type::Str, Type::None]),
        value: HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "text".to_string(),
                ty: Type::Str,
            }),
            index: Box::new(HirExpr::Name {
                name: "j".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Str,
        },
        is_mutable: false,
    };
    let mut emitter = RustEmitter::new();

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));
    let Some(RustStmt::Let {
        name, ty, value, ..
    }) = captured.first()
    else {
        panic!("expected structured let for optional local");
    };
    assert_eq!(name, "part");
    assert!(
        matches!(
            ty,
            Some(RustType::Option(inner)) if matches!(inner.as_ref(), RustType::String_)
        ) || matches!(ty, Some(RustType::Named(named)) if named == "Option<String>")
    );
    assert!(matches!(
        value,
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Some".to_string()])
                && matches!(args.as_slice(), [RustExpr::Block { .. }])
    ));
}

#[test]
pub(super) fn test_structured_stmt_path_handles_non_optional_string_index_return_expr() {
    let module = HirModule {
        functions: vec![HirFunction {
            name: "char_at".to_string(),
            params: vec![
                HirParam {
                    name: "text".to_string(),
                    ty: Type::Str,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::borrow(),
                },
                HirParam {
                    name: "j".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: ParamConvention::own(),
                },
            ],
            return_type: Type::Str,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Index {
                    object: Box::new(HirExpr::Name {
                        name: "text".to_string(),
                        ty: Type::Str,
                    }),
                    index: Box::new(HirExpr::Name {
                        name: "j".to_string(),
                        ty: Type::Int,
                    }),
                    ty: Type::Str,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module);
    assert!(generated.rust_source.contains("return {"));
    assert!(generated
        .rust_source
        .contains("let Some(__indexed_char) = text.chars().nth(j as usize) else {"));
    assert!(generated.rust_source.contains("__indexed_char.to_string()"));
    assert!(
        generated.lowering_stats.stmt_structured >= 1,
        "non-optional string index return should stay on the structured stmt path"
    );
}

#[test]
pub(super) fn test_emit_expr_prefers_structured_name_path() {
    let mut emitter = RustEmitter::new();
    emitter.intrinsic_functions.insert("clock".to_string());
    let expr = HirExpr::Name {
        name: "clock".to_string(),
        ty: Type::Int,
    };

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("name expression should lower through structured registry path");

    assert_eq!(crate::render_expr(&lowered), "clock");
}

#[test]
pub(super) fn test_emit_expr_borrowed_compare_is_structured() {
    let mut emitter = RustEmitter::new();
    emitter.borrowed_params.insert("lhs".to_string());
    let expr = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            ty: Type::Str,
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::StringLiteral("ok".to_string())],
        ty: Type::Bool,
    };

    let lowered = emitter
        .try_lower_registry_expr_strict(&expr)
        .expect("borrowed compare should lower through structured registry path");
    let rendered = crate::render_expr(&lowered);

    assert!(rendered.contains("lhs"));
    assert!(rendered.contains("=="));
}

#[test]
pub(super) fn test_lib_decomposition_guards_keep_stmt_expr_logic_out_of_lib_rs() {
    let lib_src = include_str!("../lib.rs");

    assert!(!lib_src.contains("mod stmt_emitter;"));
    assert!(!lib_src.contains("mod expr_emitter;"));
    assert!(!lib_src.contains("CodegenLoweringMode"));
    assert!(!lib_src.contains("StructuredPreferred"));
    assert!(!lib_src.contains("should_force_stmt_string_path"));
    assert!(!lib_src.contains("should_force_expr_string_path"));
    assert!(!lib_src.contains("fn emit_expr(&mut self, expr: &HirExpr) {"));
    assert!(!lib_src.contains("fn try_lower_structured_expr("));

    let emit_stmt_start = lib_src
        .find("fn emit_stmt(&mut self, stmt: &HirStmt) {")
        .expect("emit_stmt wrapper should exist");
    let body_contains_yield_start = lib_src
        .find("pub fn body_contains_yield(stmts: &[HirStmt]) -> bool {")
        .expect("body_contains_yield should exist");
    let emit_stmt_wrapper = &lib_src[emit_stmt_start..body_contains_yield_start];
    assert!(emit_stmt_wrapper.contains("structured statement emission missing for production path"));
    assert!(!emit_stmt_wrapper.contains("self.try_emit_stmt_string_"));
    assert!(
        !emit_stmt_wrapper.contains("match stmt"),
        "emit_stmt should stay orchestration-only"
    );

    let lib_lines = lib_src.lines().count();
    assert!(
        lib_lines <= 1450,
        "lib.rs should stay decomposed (current lines: {lib_lines})"
    );
}

#[test]
pub(super) fn test_production_lowering_contract_uses_result_helpers_only() {
    let lib_src = include_str!("../lib.rs");
    let lower_expr_src = include_str!("../lower_expr.rs");
    let module_constants_src = include_str!("../module_constants.rs");
    let expr_render_helpers_src = include_str!("../expr_render_helpers.rs");

    assert!(lib_src.contains("try_lower_simple_stmt_with_scope_result("));
    assert!(lower_expr_src.contains("pub fn try_lower_leaf_expr_result("));
    assert!(module_constants_src.contains("try_lower_simple_module_constant_item_result("));
    assert!(expr_render_helpers_src.contains("try_lower_registry_expr_result("));

    assert!(!lib_src.contains("try_lower_simple_stmt_with_scope("));
    assert!(!lib_src.contains("try_lower_leaf_expr("));
    assert!(!module_constants_src.contains("try_lower_simple_module_constant_item("));
}

#[test]
pub(super) fn test_capture_structured_stmts_collects_ir_without_output_writes() {
    let mut emitter = RustEmitter::new();
    let stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Int,
        value: HirExpr::IntLiteral(1),
        is_mutable: false,
    };

    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert_eq!(captured.len(), 1);
    assert!(matches!(
        captured.first(),
        Some(RustStmt::Let {
            name,
            mutable: false,
            ..
        }) if name == "x"
    ));
}

#[test]
pub(super) fn test_structured_stmt_path_handles_nested_subscript_augassign_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::SubscriptAugAssign {
                object: "values".to_string(),
                index: HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "i".to_string(),
                        ty: Type::Int,
                    }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::IntLiteral(1)),
                    ty: Type::Int,
                },
                op: "*=".to_string(),
                value: HirExpr::IntLiteral(2),
                object_ty: Type::List(Box::new(Type::Int)),
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}

#[test]
pub(super) fn test_structured_stmt_path_handles_delete_with_name_key_inside_loop_if() {
    let stmt = HirStmt::For {
        target: "ch".to_string(),
        target_ty: Type::Str,
        iter: HirExpr::Name {
            name: "order".to_string(),
            ty: Type::Str,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::ContainsOp {
                element: Box::new(HirExpr::Name {
                    name: "ch".to_string(),
                    ty: Type::Str,
                }),
                collection: Box::new(HirExpr::Name {
                    name: "counts".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Delete {
                object: HirExpr::Name {
                    name: "counts".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                },
                index: HirExpr::Name {
                    name: "ch".to_string(),
                    ty: Type::Str,
                },
            }],
            elif_clauses: vec![],
            else_body: None,
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::For { .. })));
}

#[test]
pub(super) fn test_structured_stmt_path_handles_chained_compare_condition_inside_loop_if() {
    let stmt = HirStmt::While {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "left".to_string(),
                ty: Type::Int,
            }),
            ops: vec!["<=".to_string()],
            comparators: vec![HirExpr::Name {
                name: "right".to_string(),
                ty: Type::Int,
            }],
            ty: Type::Bool,
        },
        body: vec![HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "left".to_string(),
                    ty: Type::Int,
                }),
                ops: vec!["<=".to_string(), "<".to_string()],
                comparators: vec![
                    HirExpr::Name {
                        name: "target".to_string(),
                        ty: Type::Int,
                    },
                    HirExpr::Name {
                        name: "right".to_string(),
                        ty: Type::Int,
                    },
                ],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Assign {
                name: "left".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::AugAssign {
                name: "left".to_string(),
                op: "+=".to_string(),
                value: HirExpr::IntLiteral(1),
            }]),
        }],
        else_body: None,
    };

    let mut emitter = RustEmitter::new();
    let captured = emitter.capture_structured_stmts(|inner| inner.emit_stmt(&stmt));

    assert!(matches!(captured.first(), Some(RustStmt::While { .. })));
}
